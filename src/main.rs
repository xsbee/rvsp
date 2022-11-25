use std::f32::consts::PI;
use std::ops::{Add, Mul, Div};
use std::sync::Arc;
use std::sync::Mutex;

use realfft::{RealFftPlanner, RealToComplex};
use realfft::num_complex::{Complex, ComplexFloat};

use sdl2::audio::AudioSpecDesired;
use sdl2::audio::AudioCallback;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::sys::{SDL_FRect, SDL_RenderDrawRectsF};

struct FftCompute<'a> {
    fft: Arc<dyn RealToComplex<f32>>,
    // shared fft buffer
    buf: &'a Mutex<Vec<Complex<f32>>>,
    // sliding window for incoming samples
    sliding: Vec<f32>,
    // buffer for RustFFT to work with
    scratch: Vec<Complex<f32>>,
    // von-Hann window coefficients
    hann: Vec<f32>
}

// https://en.wikipedia.org/wiki/Hann_function
fn hann_window(w: &mut [f32]) {
    #![allow(non_snake_case)]
    let N_ = w.len();

    for n in 0..N {
        w[n] = (n as f32 / N_ as f32 * PI).sin();
    }
}

impl<'a> FftCompute<'a> {
    fn new(
        fft: Arc<dyn RealToComplex<f32>>, 
        buf: &'a Mutex<Vec<Complex<f32>>>, 
        len: usize
    ) -> Self {
        let mut hann = vec![0f32; len];

        hann_window(&mut hann);

        Self {
            buf,
            scratch: fft.make_scratch_vec(),
            sliding: fft.make_input_vec(),
            fft,
            hann
        }
    }
}

impl<'a> AudioCallback for FftCompute<'a> {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        self.sliding.drain(0..samples.len());
        self.sliding.extend(samples.iter());

        for (i, s) in self.sliding.iter_mut().enumerate() {
            *s = self.hann[i] * (*s);
        }

        self.fft.process_with_scratch(
            &mut self.sliding, 
            &mut self.buf.lock().unwrap(), 
            &mut self.scratch).unwrap();
    }
}

const N: usize = 1024;
const W: u32 = 800;
const H: u32 = 600;

use const_format::formatcp;

const WINDOW_TITLE: &'static str = formatcp!(
    "{} (v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_audio = sdl.audio().unwrap();
    let sdl_video = sdl.video().unwrap();
    let mut sdl_events = sdl.event_pump().unwrap();

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(N);
    let fftbuf = fft.make_output_vec();
    let fftlen = fftbuf.len();
    let fftbuf = Mutex::new(fftbuf);

    let device = sdl_audio.open_capture(
        None, 
        &AudioSpecDesired{
            channels: Some(1), 
            freq: None,
            samples: None
        }, 
        |_|
            FftCompute::new(fft, &fftbuf, N))
    .unwrap();

    // TODO: make window resizable
    let window = sdl_video.window(WINDOW_TITLE, W, H)
        .position_centered()
        .build()
        .unwrap();
    
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    device.resume();
    
    // FIXME: use Vec<FRect> once floating points functions are introduced
    let mut points: Vec<SDL_FRect> = vec![SDL_FRect{x: 0.0, y: 0.0, w: 0.0, h: 0.0}; fftlen];
    
    // temporally interpolated FFT output
    // https://webaudio.github.io/web-audio-api/#smoothing-over-time
    let mut fftdata_interp: Vec<f32> = vec![0f32; fftlen];

    // width of each frequency bin, in display
    let xstep = 2.0 * W as f32 / N as f32;

    'running: loop {
        for event in sdl_events.poll_iter() {
            match event {
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        if let Ok(fftbuf) = fftbuf.try_lock() {
            let mut x = 0.0;

            canvas.set_draw_color((0, 0, 0, 255));
            canvas.clear();

            for (i, bin) in fftbuf.iter().enumerate() {
                fftdata_interp[i] = 0.3 * fftdata_interp[i] + 0.7 * (bin.abs() / N as f32); // T = 0.3

                let y = fftdata_interp[i]
                    .log10().mul(20.0) // https://webaudio.github.io/web-audio-api/#conversion-to-db
                    .clamp(-96.0, 0.0)// -96dB .. 0dB
                    .add(96.0).div(96.0) // [-96..0] -> [0..1]
                    .mul(H as f32);

                points[i] = SDL_FRect{x, y: H as f32 - y, w: xstep, h: y};

                x += xstep;
            }
            
            canvas.set_draw_color((255, 255, 255, 255));

            unsafe {
                // FIXME: add support for floating point functions in library
                SDL_RenderDrawRectsF(canvas.raw(), points.as_ptr(), fftlen as i32);
            }

            canvas.present();
        }
    }
}

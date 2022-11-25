use std::f32::consts::PI;
use std::ops::{Add, Mul, Div};
use std::sync::Arc;
use std::sync::Mutex;

use rustfft::Fft;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use rustfft::num_complex::ComplexFloat;

use sdl2::audio::AudioSpecDesired;
use sdl2::audio::AudioCallback;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::sys::{SDL_FRect, SDL_RenderDrawRectsF};

struct FftCompute<'a> {
    fft: Arc<dyn Fft<f32>>,
    // shared fft buffer
    buf: &'a Mutex<Vec<Complex<f32>>>,
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
        planner: &mut FftPlanner<f32>, 
        buf: &'a Mutex<Vec<Complex<f32>>>, 
        len: usize
    ) -> Self {
        let fft = planner.plan_fft_forward(len);
        let mut hann = vec![0f32; len];

        hann_window(&mut hann);

        Self {
            scratch: vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()],
            buf,
            fft,
            hann
        }
    }
}

impl<'a> AudioCallback for FftCompute<'a> {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let mut buf = self.buf.lock().unwrap();

        for (i, s) in samples.into_iter().enumerate() {
            buf[i] = Complex::new(*s * self.hann[i], 0.0);
        }

        // TODO: put samples in a sliding window prior to transform
        self.fft.process_with_scratch(&mut buf, &mut self.scratch);
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

    let mut planner = FftPlanner::new();
    let fftbuf = Mutex::new(vec![Complex::new(0.0, 0.0); N]);

    let device = sdl_audio.open_capture(
        None, 
        &AudioSpecDesired{
            channels: Some(1), 
            freq: None,
            samples: Some(N as u16)
        }, 
        |spec|
            FftCompute::new(&mut planner, &fftbuf, spec.samples as usize))
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

    const FFT_MAXLEN: usize = N / 2 + 1; // N/2 is at nyquist freq
    
    // FIXME: use Vec<FRect> once floating points functions are introduced
    let mut points: Vec<SDL_FRect> = vec![SDL_FRect{x: 0.0, y: 0.0, w: 0.0, h: 0.0}; FFT_MAXLEN];
    
    // temporally interpolated FFT output
    // https://webaudio.github.io/web-audio-api/#smoothing-over-time
    let mut fftdata_interp: Vec<f32> = vec![0f32; FFT_MAXLEN];

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

            for (i, bin) in fftbuf[..FFT_MAXLEN].into_iter().enumerate() {
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
                SDL_RenderDrawRectsF(canvas.raw(), points.as_ptr(), FFT_MAXLEN as i32);
            }

            canvas.present();
        }
    }
}

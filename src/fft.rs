use std::sync::Arc;
use std::f32::consts::{TAU};

use realfft::RealToComplex;
use realfft::num_complex::Complex;

use sdl2::audio::AudioCallback;

use crate::render::Renderer;

pub struct FftCompute {
    fft: Arc<dyn RealToComplex<f32>>,
    // shared fft buffer
    buf: Vec<Complex<f32>>,
    // sliding window for incoming samples
    sliding: Vec<f32>,
    // windowed snapshot of the sliding window
    window: Vec<f32>,
    // buffer for RustFFT to work with
    scratch: Vec<Complex<f32>>,
    // blackman window coefficients
    blackman: Vec<f32>,
    renderer: Renderer,
}

/// https://webaudio.github.io/web-audio-api/#blackman-window
fn blackman_window(w: &mut [f32]) {
    let n = w.len();

    const A0: f32 = 0.42f32;
    const A1: f32 = 0.5f32;
    const A2: f32 = 0.08f32;

    for i in 0..n {
        let x = i as f32 / n as f32 * TAU;

        w[i] = A0 - A1 * x.cos() + A2 * (2.0f32 * x).cos();
    }
}

impl FftCompute {
    pub fn new(
        fft: Arc<dyn RealToComplex<f32>>,
        len: usize,
        renderer: Renderer,
    ) -> Self {
        let mut blackman = vec![0f32; len];

        blackman_window(&mut blackman);

        Self {
            buf: fft.make_output_vec(),
            scratch: fft.make_scratch_vec(),
            sliding: fft.make_input_vec(),
            window: fft.make_input_vec(),
            fft,
            blackman,
            renderer,
        }
    }
}

// TODO: This apparently seems to work but theoratically shouldn't.
//
// Rc isn't Send because Rc::clone exists and Rc is non-atomic ref
// counted. However, since this is the only reference we create so
// the concern of data-races do not quite apply.
//
// Aparently, subsystems can't moved across threads or be used on
// non-main threads, which is a major concern here.
unsafe impl Send for FftCompute {}

impl AudioCallback for FftCompute {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        self.sliding.drain(0..samples.len());
        self.sliding.extend(samples.iter());

        for (i, s) in self.sliding.iter().enumerate() {
            self.window[i] = self.blackman[i] * (*s);
        }

        self.fft.process_with_scratch(
            &mut self.window, 
            &mut self.buf, 
            &mut self.scratch).unwrap();

        self.renderer.render(&mut self.buf);
    }
}
use std::sync::{Arc, Mutex};
use std::f32::consts::{TAU};

use realfft::RealToComplex;
use realfft::num_complex::Complex;

use sdl2::audio::AudioCallback;

pub struct FftCompute<'a> {
    fft: Arc<dyn RealToComplex<f32>>,
    // shared fft buffer
    buf: &'a Mutex<Vec<Complex<f32>>>,
    // sliding window for incoming samples
    sliding: Vec<f32>,
    // windowed snapshot of the sliding window
    window: Vec<f32>,
    // buffer for RustFFT to work with
    scratch: Vec<Complex<f32>>,
    // blackman window coefficients
    blackman: Vec<f32>,
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

impl<'a> FftCompute<'a> {
    pub fn new(
        fft: Arc<dyn RealToComplex<f32>>, 
        buf: &'a Mutex<Vec<Complex<f32>>>, 
        len: usize
    ) -> Self {
        let mut blackman = vec![0f32; len];

        blackman_window(&mut blackman);

        Self {
            buf,
            scratch: fft.make_scratch_vec(),
            sliding: fft.make_input_vec(),
            window: fft.make_input_vec(),
            fft,
            blackman
        }
    }
}

impl<'a> AudioCallback for FftCompute<'a> {
    type Channel = f32;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        if let Ok(mut buf) = self.buf.lock() {
	    if samples.len() > self.sliding.len() {
		self.sliding.clear();
		self.sliding.extend(samples.iter().nth(samples.len() - self.sliding.len()));
	    } else {
		self.sliding.drain(0..samples.len());
		self.sliding.extend(samples.iter());
	    }

            for (i, s) in self.sliding.iter().enumerate() {
            self.window[i] = self.blackman[i] * (*s);
            }

            self.fft.process_with_scratch(
                &mut self.window, 
                &mut buf, 
                &mut self.scratch).unwrap();
        }
    }
}

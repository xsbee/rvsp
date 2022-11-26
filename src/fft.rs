use std::sync::{Arc, Mutex};
use std::f32::consts::PI;

use realfft::RealToComplex;
use realfft::num_complex::Complex;

use sdl2::audio::AudioCallback;

pub struct FftCompute<'a> {
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

    for n in 0..N_ {
        w[n] = (n as f32 / N_ as f32 * PI).sin();
    }
}

impl<'a> FftCompute<'a> {
    pub fn new(
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
        if let Ok(mut buf) = self.buf.try_lock() {
            self.sliding.drain(0..samples.len());
            self.sliding.extend(samples.iter());

            for (i, s) in self.sliding.iter_mut().enumerate() {
                *s = self.hann[i] * (*s);
            }

            self.fft.process_with_scratch(
                &mut self.sliding, 
                &mut buf, 
                &mut self.scratch).unwrap();
        }
    }
}
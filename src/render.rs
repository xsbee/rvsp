use std::ops::{Mul, Sub, Div};

use realfft::num_complex::{Complex, ComplexFloat};

use sdl2::render::Canvas;
use sdl2::sys::{SDL_FRect, SDL_RenderFillRectsF};
use sdl2::video::Window;
use sdl2::pixels::Color;

use crate::utils;

pub struct Renderer {
    canvas: Canvas<Window>,
    rects: Vec<SDL_FRect>,
    interp: Vec<f32>,       // time interp freq data
    stc: f32,               // smoothing time constant
    dbmin: f32,             // minimum decibels
    dbmax: f32,             // maximum decibels
    width: u32,
    height: u32,
    fg: Color,
    bg: Color
}

impl Renderer {
    pub fn new(
        canvas: Canvas<Window>,
        fftlen: usize,
        stc: f32,
        dbmin: f32,
        dbmax: f32,
        width: u32,
        height: u32,
        fg: Color,
        bg: Color
    ) -> Self {
        Self {
            canvas,
            rects: vec![utils::frect_new(); fftlen],
            interp: vec![0f32; fftlen],
            stc,
            dbmin,
            dbmax,
            width,
            height,
            fg,
            bg
        }
    }

    pub fn render(&mut self, fftdata: &[Complex<f32>]) {
        let xstep = self.width as f32 / fftdata.len() as f32;
        let normfac = 1.0 / fftdata.len() as f32;

        let mut x = 0.0;

        self.canvas.set_draw_color(self.bg);
        self.canvas.clear();

        for (i, bin) in fftdata.iter().enumerate() {
            let y = self.stc * self.interp[i] + (1.0 - self.stc) * bin.abs() * normfac;
            
            self.interp[i] = y;

            let y = y
                .log10().mul(20.0)
                .clamp(self.dbmin, self.dbmax)
                .sub(self.dbmin)
                .div(self.dbmax - self.dbmin)
                .mul(self.height as f32);

            self.rects[i] = SDL_FRect {
                x,
                y: self.height as f32 - y,
                w: xstep,
                h: y,
            };

            x += xstep;
        }

        self.canvas.set_draw_color(self.fg);
        
        unsafe {
            SDL_RenderFillRectsF(
                self.canvas.raw(), 
                self.rects.as_ptr(), 
                self.rects.len() as i32);
        }

        self.canvas.present();
    }
}

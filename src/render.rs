use std::ops::{Div, Mul, Sub};

use realfft::num_complex::{Complex, ComplexFloat};

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::sys::{SDL_FRect, SDL_RenderFillRectsF};
use sdl2::video::Window;

use crate::utils;

pub struct Renderer {
    canvas: Canvas<Window>,
    rects: Vec<SDL_FRect>,
    interp: Vec<f32>, // time interp freq data
    stc: f32,         // smoothing time constant
    dbmin: f32,       // minimum decibels
    dbmax: f32,       // maximum decibels
    width: u32,
    height: u32,
    x_max: usize,
    fg: Color,
    bg: Color,
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
        max_freq: f32,
        resolution: f32,
        fg: Color,
        bg: Color,
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
            x_max: (max_freq / resolution) as usize,
            fg,
            bg,
        }
    }

    pub fn render(&mut self, fftdata: &[Complex<f32>]) {
        let xstep = self.width as f32 / self.x_max as f32;
        let normfac = 1.0 / fftdata.len() as f32;
        let height_half = self.height as f32 / 2.0;

        let mut x = 0.0;

        self.canvas.set_draw_color(self.bg);
        self.canvas.clear();

        for (i, bin) in fftdata[..self.x_max].iter().enumerate() {
            let y = self.stc * self.interp[i] + (1.0 - self.stc) * bin.abs() * normfac;

            self.interp[i] = y;

            let y = y
                .log10()
                .mul(20.0)
                .clamp(self.dbmin, self.dbmax)
                .sub(self.dbmin)
                .div(self.dbmax - self.dbmin)
                .mul(height_half);

            self.rects[i] = SDL_FRect {
                x,
                y: height_half - y,
                w: xstep,
                h: 2.0 * y,
            };

            x += xstep;
        }

        self.canvas.set_draw_color(self.fg);
        self.canvas
            .draw_line(
                (0, height_half as i32),
                (self.width as i32, height_half as i32),
            )
            .unwrap();

        unsafe {
            SDL_RenderFillRectsF(
                self.canvas.raw(),
                self.rects.as_ptr(),
                self.rects.len() as i32,
            );
        }

        self.canvas.present();
    }
}

use std::num::ParseIntError;

use sdl2::{sys::SDL_FRect, pixels::Color};

pub fn str_to_color(s: &str) -> Result<Color, ParseIntError> {
    let c = u32::from_str_radix(s, 16)?;

    Ok(Color {
        a: (c >> 24) as u8,
        r: (c >> 16 & 0xFF) as u8,
        g: (c >> 8 & 0xFF) as u8,
        b: (c & 0xFF) as u8, 
    })
} 

pub const fn frect_new() -> SDL_FRect {
    SDL_FRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }
}
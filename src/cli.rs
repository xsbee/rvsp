use clap::Parser;

use std::num::ParseIntError;
use sdl2::pixels::Color;

fn str_to_color(s: &str) -> Result<Color, ParseIntError> {
    let c = u32::from_str_radix(s, 16)?;

    Ok(Color {
        a: (c >> 24) as u8,
        r: (c >> 16 & 0xFF) as u8,
        g: (c >> 8 & 0xFF) as u8,
        b: (c & 0xFF) as u8, 
    })
} 

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Window size of FFT in samples
    #[arg(short = 's', long, default_value = "1024")]
    pub fftsize: usize,

    /// Width of visualizer window
    #[arg(long, default_value = "800")]
    pub width: u32,

    /// Height of visualizer window
    #[arg(long, default_value = "600")]
    pub height: u32,

    /// Smoothing time constant (0..1)
    #[arg(long, default_value = "0.3")]
    pub stc: f32,

    /// Minimum decibels (-Infty...0)
    #[arg(long, default_value = "-80")]
    pub dbmin: f32,

    /// Maximum decibels (-Infty...0)
    #[arg(long, default_value = "-20")]
    pub dbmax: f32,

    /// Color of frequency bin bars (in ARGB format)
    #[arg(long, default_value = "ffffffff", value_parser = str_to_color)]
    pub fgcolor: Color,

    /// Color of background (in ARGB format)
    #[arg(long, default_value = "ff000000", value_parser = str_to_color)]
    pub bgcolor: Color
}

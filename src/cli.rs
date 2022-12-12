use clap::Parser;

use sdl2::pixels::Color;

use crate::utils;

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
    #[arg(long, default_value = "ffffffff", value_parser = utils::str_to_color)]
    pub fgcolor: Color,

    /// Color of background (in ARGB format)
    #[arg(long, default_value = "ff000000", value_parser = utils::str_to_color)]
    pub bgcolor: Color
}

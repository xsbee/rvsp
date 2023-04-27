use clap::Parser;

use sdl2::pixels::Color;

use crate::utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Window size of FFT in samples
    #[arg(short = 's', long, default_value_t = 1024)]
    pub fftsize: usize,

    #[arg(short = 'r', long, default_value_t = 60)]
    pub framerate: u32,

    /// Width of visualizer window
    #[arg(long, default_value_t = 800)]
    pub width: u32,

    /// Height of visualizer window
    #[arg(long, default_value_t = 600)]
    pub height: u32,

    /// Samplerate of audio
    #[arg(long, default_value_t = 48000)]
    pub samplerate: u32,

    #[arg(long, default_value_t = 16000)]
    pub max_frequency: u32,

    /// Smoothing time constant (0..1)
    #[arg(long, default_value_t = 0.8)]
    pub stc: f32,

    /// Minimum decibels (-Infty...0)
    #[arg(long, default_value_t = -80.0)]
    pub dbmin: f32,

    /// Maximum decibels (-Infty...0)
    #[arg(long, default_value_t = -6.0)]
    pub dbmax: f32,

    /// Color of frequency bin bars (in ARGB format)
    #[arg(long, default_value = "FFBBB6A5", value_parser = utils::str_to_color)]
    pub fgcolor: Color,

    /// Color of background (in ARGB format)
    #[arg(long, default_value = "FF2A2F33", value_parser = utils::str_to_color)]
    pub bgcolor: Color,

    #[arg(short, long)]
    pub device: Option<u32>,

    #[arg(short, long, default_value_t = false)]
    pub list_devices: bool,
}

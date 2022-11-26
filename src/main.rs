use std::sync::Mutex;

use clap::Parser;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use realfft::RealFftPlanner;

use const_format::formatcp;

mod fft;
mod render;
mod cli;

const WINDOW_TITLE: &'static str = formatcp!(
    "{} (v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

fn main() {
    let args = cli::Args::parse();

    let sdl = sdl2::init().unwrap();
    let sdl_audio = sdl.audio().unwrap();
    let sdl_video = sdl.video().unwrap();
    let mut sdl_events = sdl.event_pump().unwrap();

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(args.fftsize);
    let fftbuf = fft.make_output_vec();
    let fftlen = fftbuf.len();
    let fftbuf = Mutex::new(fftbuf);

    let device = sdl_audio.open_capture(
        None, 
        &AudioSpecDesired {
            channels: Some(1), 
            freq: None,
            samples: None
        }, 
        |_| fft::FftCompute::new(fft, &fftbuf, args.fftsize)).unwrap();

    let window = sdl_video.window(WINDOW_TITLE, args.width, args.height)
        .position_centered()
        .build()
        .unwrap();
    
    let canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    device.resume();

    let mut renderer = render::Renderer::new(
        canvas, fftlen, args.stc, args.dbmin, args.dbmax, args.fgcolor, args.bgcolor);

    'running: loop {
        for event in sdl_events.poll_iter() {
            match event {
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        if let Ok(fftbuf) = fftbuf.try_lock() {
            renderer.render(&*fftbuf);
        }
    }
}

use std::sync::Mutex;

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use realfft::RealFftPlanner;

use const_format::formatcp;

mod fft;
mod render;

const WINDOW_TITLE: &'static str = formatcp!(
    "{} (v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
const N: usize = 1024;
const W: u32 = 800;
const H: u32 = 600;
const STC: f32 = 0.3;
const DBMIN: f32 = -80.0;
const DBMAX: f32 = -20.0;
const FGCOLOR: Color = Color {r: 255, g: 255, b: 255, a: 255};
const BGCOLOR: Color = Color {r: 0, g: 0, b: 0, a: 255};

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_audio = sdl.audio().unwrap();
    let sdl_video = sdl.video().unwrap();
    let mut sdl_events = sdl.event_pump().unwrap();

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(N);
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
        |_| fft::FftCompute::new(fft, &fftbuf, N)).unwrap();

    // TODO: make window resizable
    let window = sdl_video.window(WINDOW_TITLE, W, H)
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
        canvas, fftlen, STC, DBMIN, DBMAX, FGCOLOR, BGCOLOR);

    'running: loop {
        for event in sdl_events.poll_iter() {
            match event {
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        if let Ok(fftbuf) = fftbuf.lock() {
            renderer.render(&*fftbuf);
        }
    }
}

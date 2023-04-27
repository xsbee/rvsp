use clap::Parser;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use realfft::RealFftPlanner;

use const_format::formatcp;

mod cli;
mod fft;
mod render;
mod utils;

const WINDOW_TITLE: &'static str = formatcp!(
    "{} (v{})",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION")
);

fn main() {
    let args = cli::Args::parse();

    let sdl = sdl2::init().unwrap();
    let sdl_audio = sdl.audio().unwrap();

    if args.list_devices {
        let audio_driver = sdl_audio.current_audio_driver();
        let num_devices = sdl_audio
            .num_audio_capture_devices()
            .expect("Could not enumrate devices");

        println!("List of capture devices");
        println!("-----------------------");
        println!("Driver: {audio_driver}");
        println!("");
        println!("[Index]    Name");

        for i in 0..num_devices {
            let device_name = sdl_audio.audio_capture_device_name(i).unwrap();

            println!("[{i:>5}]    {device_name}");
        }

        return;
    }

    let sdl_video = sdl.video().unwrap();
    let mut sdl_events = sdl.event_pump().unwrap();

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(args.fftsize);
    let fftlen = args.fftsize / 2 + 1;

    let device_name = args
        .device
        .map(|d| sdl_audio.audio_capture_device_name(d).unwrap());

    let device = sdl_audio
        .open_capture(
            device_name.as_ref().map(|d| d.as_str()),
            &AudioSpecDesired {
                channels: Some(1),
                freq: Some(args.samplerate as i32),

                // This is the overlap of the sliding window
                samples: Some((args.samplerate / args.framerate) as u16),
            },
            |_| {
                let window = sdl_video
                    .window(WINDOW_TITLE, args.width, args.height)
                    .position_centered()
                    .build()
                    .unwrap();

                let (width, height) = window.size();
                let canvas = window
                    .into_canvas()
                    .present_vsync()
                    .accelerated()
                    .build()
                    .unwrap();

                let renderer = render::Renderer::new(
                    canvas,
                    fftlen,
                    args.stc,
                    args.dbmin,
                    args.dbmax,
                    width,
                    height,
                    args.max_frequency as f32,
                    args.samplerate as f32 / args.fftsize as f32,
                    args.fgcolor,
                    args.bgcolor,
                );

                fft::FftCompute::new(fft, args.fftsize, renderer)
            },
        )
        .unwrap();

    device.resume();

    'running: loop {
        for event in sdl_events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }
}

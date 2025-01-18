extern crate sdl3_sys;

use std::ffi::c_float;
use config::{Config, FileFormat};
use sdl3_sys::everything::*;
use std::process::exit;
use std::time::Duration;
use crate::sdl::Event;

mod sdl;

fn die(s: &str) -> ! {
    println!("{}", s);
    sdl::quit();
    panic!();
}

fn or_die(result: Result<(), String>) {
    if let Err(msg) = result {
        die(format!("SDL Something weird happened because a function that should not have failed has failed: {}", msg).as_str());
    }
}

// todo copious error checking

pub fn main() {
    // window inits as x11 instead of wayland due to lack of fifo-v1 protocol in gnome.
    // fifo-v1 was added here https://gitlab.gnome.org/GNOME/mutter/-/merge_requests/3355 and will be present in gnome 48.
    // The X11 window is responsible for the window flashing on creation. Wayland does not experience this issue.
    // SDL_VIDEO_DRIVER=wayland can force wayland

    let settings = Config::builder()
        .add_source(config::File::new("audio-sidecar-config", FileFormat::Toml))
        .build()
        .unwrap();

    let interface: String = settings
        .get("Interface")
        .expect("Interface key to exist in config file");
    let window_width: u32 = settings
        .get("WindowWidth")
        .expect("WindowWidth key to exist in config file");
    let window_height: u32 = settings
        .get("WindowHeight")
        .expect("WindowHeight key to exist in config file");

    if let Err(msg) = sdl::init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS) {
        die(format!("SDL initialization failed: {}", msg).as_str());
    }

    let gfx = match sdl::create_window_and_renderer(
        "Record Audio",
        window_width,
        window_height,
        SDL_WINDOW_RESIZABLE,
    ) {
        Ok(gfx) => gfx,
        Err(msg) => die(format!("SDL window creation failed: {}", msg).as_str()),
    };

    let recording_devices = match sdl::get_audio_recording_devices() {
        Ok(a) => a,
        Err(msg) => die(format!("SDL finding audio recording devices failed: {}", msg).as_str()),
    };

    let mut desired_interface_id = SDL_AUDIO_DEVICE_DEFAULT_RECORDING;

    println!("Found {} Audio Devices:", recording_devices.len());
    for device in recording_devices {
        let found = if device.name.to_lowercase().contains(interface.as_str()) {
            desired_interface_id = device.id;
            " <<<< MATCH FOUND <<<<"
        } else {
            ""
        };

        println!("\t{} {}", device.name, found);
    }

    let logical_interface_id = match sdl::open_audio_device(desired_interface_id) {
        Ok(i) => i,
        Err(msg) => die(format!("SDL could not open audio device: {}", msg).as_str()),
    };

    let audio_stream = match sdl::create_audio_stream() {
        Ok(s) => s,
        Err(msg) => die(format!("SDL could not create audio stream: {}", msg).as_str()),
    };

    if let Err(msg) = sdl::bind_audio_stream(logical_interface_id, audio_stream) {
        die(format!("SDL could not bind logical audio device to stream: {}", msg).as_str());
    }

    loop {
        // poll until all events are handled and the queue runs dry
        while let Some(event) = sdl::poll_event() {
            match event {
                // todo New events will have to be added both here and in sdl::poll_event()
                Event::Quit(_) => {
                    if let Err(msg) = sdl::flush_audio_stream(audio_stream) {
                        die(format!("SDL could not flush audio stream: {}", msg).as_str());
                    }
                    sdl::close_audio_device(logical_interface_id);
                    sdl::quit();
                    exit(0);
                }
                _ => continue,
            }
        }

        let samples = match sdl::get_audio_stream_data_i32(audio_stream) {
            Ok(s) => s,
            Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
        };

        let mut max = 0;
        for s in samples {
            if s > max {
                max = s;
            }
        }

        or_die(sdl::set_render_draw_color(
            &gfx,
            SDL_Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        ));
        or_die(sdl::render_clear(&gfx));
        or_die(sdl::set_render_draw_color(
            &gfx,
            SDL_Color {
                r: 255,
                g: 150,
                b: 255,
                a: 255,
            },
        ));

        let rect = SDL_FRect {
            h: (max as f64 * (480.0 / i32::MAX as f64)) as c_float,
            w: 100.0,
            x: 0.0,
            y: 0.0,
        };

        or_die(sdl::render_fill_rect(&gfx, &rect));

        or_die(sdl::render_present(&gfx));

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }
}

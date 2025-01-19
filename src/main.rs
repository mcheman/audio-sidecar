extern crate sdl3_sys;

use std::cmp::{max, min};
use crate::sdl::Event;
use config::{Config, FileFormat};
use sdl3_sys::everything::*;
use std::ffi::c_float;
use std::process::exit;
use std::time::{Duration, Instant};

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
// todo if audio is for an image, load a thumbnail and display it so it's clearer which file the audio will be associated with. Loading thumbnails rather than the image itself should be both faster and have fewer file formats to deal with. We could even try to load _any_ thumbnail that matches the file in question, say for video files, since we'll only care if there _is_ one.
// todo Audio should be saved periodically to some temporary location and always on quit in case the wrong button is pressed. Potentially, the audio could be moved to trash if the X button was clicked, rather than save. Or use hidden files, but what would clean them up?
// todo MVP should ONLY record at end of existing audio.

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

    let mut recorded_audio: Vec<i32> = Vec::new();

    loop {
        // poll until all events are handled and the queue runs dry
        while let Some(event) = sdl::poll_event() {
            match event {
                // todo New events will have to be added both here and in sdl::poll_event()
                // todo check timestamp of event and compare to time to see how much elapsed between when X was clicked and when the event finally got handled to debug the slow close
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

        let begin_audio = Instant::now();

        let mut samples = match sdl::get_audio_stream_data_i32(audio_stream) {
            Ok(s) => s,
            Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
        };

        recorded_audio.append(&mut samples);

        let audio_time = begin_audio.elapsed().as_nanos();



        or_die(sdl::set_render_draw_color(&gfx, 0, 0, 0, 255));
        or_die(sdl::render_clear(&gfx));
        or_die(sdl::set_render_draw_color(&gfx, 255, 150, 255, 255));

        // render waveform for last 10 sec
        // todo put this in its own handler widget thing which only renders the new audio to a buffer which can then be scrolled on the screen, rather than line rendering the whole waveform
        // todo or look at rendering fewer samples and optimizing
        // todo only draw "chunked" sections of the waveform so the running average/max is always the same for a chunk, then scroll the buffer to simulate movement
        // todo OR have a smaller vector which contains the result of chunking the audio down. i.e. each index contains the result of max(1000 samples). Then we can take an average over this for display
        let max_samples_to_render = 44100 * 20;
        let waveform_display_area = 1600; // pixels
        let display_interval = max_samples_to_render as f64 / waveform_display_area as f64; // render sample after every "display_interval" samples


        let samples_to_render = min(max_samples_to_render, recorded_audio.len());
        let start = max(0, recorded_audio.len() as i64 - samples_to_render as i64 - 1) as usize;

        let mut samples_seen = 0f64;
        let mut max_amplitude = 0;
        // let mut average = 0f64;
        let mut x = 0;

        let step_size = display_interval / 10.0;

        let begin_waveform = Instant::now();
        for s in recorded_audio.iter().skip(start).step_by(step_size as usize) {
            samples_seen += step_size;
            if (*s as i64).abs() > max_amplitude {
                max_amplitude = (*s as i64).abs();
            }

            if samples_seen > display_interval {
                samples_seen -= display_interval;

                let rect = SDL_FRect {
                    h: max_amplitude as f32 * (400.0 / i32::MAX as f32),
                    w: 1.0,
                    x: x as f32,
                    y: (400.0 / 2.0) - (max_amplitude as f32 / 2.0 * (400.0 / i32::MAX as f32)),
                };

                or_die(sdl::render_fill_rect(&gfx, &rect));

                x += 1;
                max_amplitude = 0;
            }
        }

        let waveform_time = begin_waveform.elapsed().as_nanos();


        or_die(sdl::set_render_draw_color(&gfx, 255, 255, 255, 255));
        or_die(sdl::render_debug_text(&gfx, "AudioSidecar", 10.0, 10.0));
        or_die(sdl::render_debug_text(&gfx, format!("Audio:    {}", audio_time as f32 / 1000000.0).as_str(), 10.0, 450.0));
        or_die(sdl::render_debug_text(&gfx, format!("Waveform: {}", waveform_time as f32 / 1000000.0).as_str(), 10.0, 470.0));

        or_die(sdl::render_present(&gfx));

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }
}

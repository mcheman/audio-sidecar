extern crate flacenc;
extern crate sdl3_sys;

use crate::sdl::Event;
use config::{Config, FileFormat};
use flacenc::component::BitRepr;
use flacenc::error::Verify;
use sdl3_sys::everything::*;
use std::cmp::{max, min};
use std::env;
use std::path::Path;
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
// todo save performance stats and/or performance stats outside of normal
// todo if audio is for an image, load a thumbnail and display it so it's clearer which file the audio will be associated with. Loading thumbnails rather than the image itself should be both faster and have fewer file formats to deal with. We could even try to load _any_ thumbnail that matches the file in question, say for video files, since we'll only care if there _is_ one. See https://askubuntu.com/questions/1368910/how-to-create-custom-thumbnailers-for-nautilus-nemo-and-caja and https://specifications.freedesktop.org/thumbnail-spec/latest/thumbsave.html
// todo Audio should be saved periodically to some temporary location and always on quit in case the wrong button is pressed. Potentially, the audio could be moved to trash if the X button was clicked, rather than save. Or use hidden files, but what would clean them up?
// todo MVP should ONLY record at end of existing audio.

// todo add error checking, logging, and dad friendly error reporting
// todo add safeguards such as not overwriting existing recordings and/or saving old recordings to a backup directory on overwrite
// todo append new data to file and fixup header when recording to an existing file
// todo show time recorded so far
// todo add big button to stop recording/exit
// todo handle multiple paths sent to this program i.e. drop everything after first
// todo clean up visualization
// todo warn when clipping occurs
// todo figure out how to add to right click menu in nautilus without additional click into scripts submenu
// todo write ffmpeg command such that it will never prompt for user input, such as when attempting to overwrite a file
// todo pin sdl3 version
// todo test on other distros
// todo organize better / refactor / split into separate source files
// todo see if you can get 24bit audio working
// todo add message when quitting if writing out is taking awhile (though probably not needed if writing out as we go)
// todo create a slideshow application that plays the audio with the corresponding picture, advancing to the next once the audio is done. slideshow will play everything in directory
// todo   add optional "music" for background since he wants to put specific music in the background.
// todo   slideshow will also display metadata that was entered such as title and comments etc
// todo add keyboard shortcut to nautilus extension?

// todo periodically check if new audio devices have been added (especially if none of the ideal ones are detected yet), see getaudiorecordingdevices or eventing
// todo assign flac album cover art to image it was created for with extra audio icon????

// todo load values from config file: interface text to search for,

// todo try to select the first audio input, or test both inputs to see which has any audio signal and use that one
// todo display a user facing message about needing to turn the audio interface on/plug in if it isn't detected

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let defaultpath = String::from("/tmp/test.png");
    let filepath = Path::new(args.get(1).unwrap_or(&defaultpath));

    println!("Using input file: {:?}", filepath);

    // window inits as x11 instead of wayland due to lack of fifo-v1 protocol in gnome.
    // fifo-v1 was added here https://gitlab.gnome.org/GNOME/mutter/-/merge_requests/3355 and will be present in gnome 48.
    // The X11 window is responsible for the window flashing on creation. Wayland does not experience this issue.
    // SDL_VIDEO_DRIVER=wayland can force wayland

    let settings = Config::builder()
        .add_source(config::File::new(
            "./audio-sidecar-config",
            FileFormat::Toml,
        ))
        .build()
        .unwrap(); // todo this should have defaults and not panic if config file doesn't exist

    let interface: String = settings
        .get("Interface")
        .expect("Interface key to exist in config file");
    let mut window_width: u32 = settings
        .get("WindowWidth")
        .expect("WindowWidth key to exist in config file");
    let mut window_height: u32 = settings
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

    if let Err(msg) = gfx.set_render_vsync(1) {
        die(format!("SDL vsync failed to enable: {}", msg).as_str());
    }

    let recording_devices = match sdl::get_audio_recording_devices() {
        Ok(a) => a,
        Err(msg) => die(format!("SDL finding audio recording devices failed: {}", msg).as_str()),
    };1

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

    let mut display_waveform: Vec<u32> = Vec::new();
    let mut previous_unchunked_samples: Vec<i32> = Vec::new();

    loop {
        let frame_start = Instant::now();
        // poll until all events are handled and the queue runs dry
        let mut num_events = 0;
        while let Some(event) = sdl::poll_event() {
            num_events += 1;
            match event {
                // todo New events will have to be added both here and in sdl::poll_event()
                // todo check timestamp of event and compare to time to see how much elapsed between when X was clicked and when the event finally got handled to debug the slow close
                Event::Window(event_type, e) => {
                    if event_type == SDL_EventType::WINDOW_RESIZED {
                        window_width = e.data1 as u32;
                        window_height = e.data2 as u32;
                    }
                }
                Event::Quit(_) => {
                    println!("Quitting");

                    if let Err(msg) = sdl::flush_audio_stream(audio_stream) {
                        die(format!("SDL could not flush audio stream: {}", msg).as_str());
                    }
                    sdl::close_audio_device(logical_interface_id);

                    // get last bit of audio
                    let mut samples = match sdl::get_audio_stream_data_i32(audio_stream) {
                        Ok(s) => s,
                        Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
                    };
                    // clip audio to 24 bits by removing quietest 8 bits
                    for s in samples.iter_mut() {
                        *s >>= 8;
                    }

                    recorded_audio.append(&mut samples);

                    //sdl::quit(); // todo hide window while audio exports so it looks immediate? alternately show a progress bar?

                    // save flac audio
                    // todo ensure multithreaded and find out which compression level is used, seems like it defaults to max and it can't be adjusted???
                    let (channels, bits_per_sample, sample_rate) = (1, 24, 44100);
                    let config = flacenc::config::Encoder::default()
                        .into_verified()
                        .expect("Config data error.");
                    let source = flacenc::source::MemSource::from_samples(
                        recorded_audio.as_slice(),
                        channels,
                        bits_per_sample,
                        sample_rate,
                    );
                    let flac_stream =
                        flacenc::encode_with_fixed_block_size(&config, source, config.block_size)
                            .expect("Encode failed.");

                    // `Stream` implements `BitRepr` so you can obtain the encoded stream via
                    // `ByteSink` struct that implements `BitSink`.
                    let mut sink = flacenc::bitsink::ByteSink::new();
                    flac_stream.write(&mut sink).expect("TODO: panic message");

                    // Then, e.g. you can write it to a file.
                    // todo add string at end of filename, before extension, so the audio sidecar sorts after the file
                    let outputfile = filepath.with_extension("flac");
                    std::fs::write(outputfile, sink.as_slice()).expect("Failed to write flac");

                    sdl::quit();
                    exit(0);
                }
                _ => continue,
            }
        }

        let event_time = frame_start.elapsed().as_nanos();

        let begin_audio = Instant::now();

        let mut samples = match sdl::get_audio_stream_data_i32(audio_stream) {
            Ok(s) => s,
            Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
        };

        // clip audio to 24 bits by removing quietest 8 bits
        for s in samples.iter_mut() {
            *s >>= 8;
        }

        recorded_audio.append(&mut samples.clone());

        // combine audio into chunks for display
        const CHUNKSIZE: usize = 44100 / 100; // samples

        previous_unchunked_samples.append(&mut samples);
        let mut max_sample = 0;
        let n = 0;
        for n in 0..previous_unchunked_samples.len() / CHUNKSIZE {
            for i in 0..CHUNKSIZE {
                let v = (previous_unchunked_samples[n * CHUNKSIZE + i] as i64).abs() as u32;
                if v > max_sample {
                    max_sample = v;
                }
            }
            display_waveform.push(max_sample);
            max_sample = 0;
        }
        previous_unchunked_samples = previous_unchunked_samples
            .iter()
            .skip((previous_unchunked_samples.len() / CHUNKSIZE) * CHUNKSIZE)
            .cloned()
            .collect();

        let audio_time = begin_audio.elapsed().as_nanos();

        or_die(gfx.set_render_draw_color(43, 43, 43, 255));
        or_die(gfx.render_clear());
        or_die(gfx.set_render_draw_color(255, 255, 255, 255));

        // todo put this in its own handler widget thing which only renders the new audio to a buffer which can then be scrolled on the screen, rather than line rendering the whole waveform

        let begin_waveform = Instant::now();

        let chunks_to_render = window_width; // one chunk per pixel
        for (x, m) in display_waveform
            .iter()
            .skip(max(0, display_waveform.len() as i64 - chunks_to_render as i64) as usize)
            .enumerate()
        {
            let h = *m as f32 * (400.0 / (i32::MAX >> 8) as f32);
            let y1 = (400.0 / 2.0) - (h / 2.0);
            let y2 = y1 + h;
            if h > 390.0 {
                or_die(gfx.set_render_draw_color(255, 43, 43, 255));
            }
            or_die(gfx.render_line(x as f32, y1, x as f32, y2));
            if h > 390.0 {
                or_die(gfx.set_render_draw_color(255, 255, 255, 255));
            }
        }

        or_die(gfx.render_debug_text(
            format!("{:.3}s", recorded_audio.len() as f64 / 44100.0).as_str(),
            100.0,
            410.0,
        ));

        let waveform_time = begin_waveform.elapsed().as_nanos();

        or_die(gfx.set_render_draw_color(255, 255, 255, 255));
        or_die(gfx.render_debug_text("AudioSidecar", 10.0, 10.0));
        or_die(gfx.render_debug_text(
            format!("Audio:    {:.2}", audio_time as f32 / 1000000.0).as_str(),
            10.0,
            450.0,
        ));
        or_die(gfx.render_debug_text(
            format!("Waveform: {:.2}", waveform_time as f32 / 1000000.0).as_str(),
            10.0,
            470.0,
        ));
        or_die(gfx.render_debug_text(
            format!("Events:   {:.2}", event_time as f32 / 1000000.0).as_str(),
            10.0,
            490.0,
        ));
        or_die(gfx.render_debug_text(
            format!("Num Events: {:.2}", num_events).as_str(),
            10.0,
            510.0,
        ));

        or_die(gfx.render_present());

        // ::std::thread::sleep(
        //     Duration::new(0, 1_000_000_000u32 / 60)
        //         .saturating_sub(Instant::now().duration_since(frame_start)),
        // ); // wait until frame time equals 1/60 sec
    }
}

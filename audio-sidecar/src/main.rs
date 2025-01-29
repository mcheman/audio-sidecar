extern crate flac_sys;
extern crate sdl3_sys;

use self::config::ExistingFileStrategy;
use crate::config::ProgramConfig;
use crate::flac::Encoder;
use crate::gui::{Input, UI};
use crate::sdl::Event;
use crate::utils::die;
use crate::utils::or_die;
use log::{debug, error, info};
use sdl3_sys::everything::*;
use std::any::Any;
use std::backtrace::Backtrace;
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::{env, io, panic};
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt};

mod config;
mod flac;
mod gui;
mod sdl;
mod utils;
// todo copious error checking
// todo save performance stats and/or performance stats outside of normal
// todo if audio is for an image, load a thumbnail and display it so it's clearer which file the audio will be associated with. Loading thumbnails rather than the image itself should be both faster and have fewer file formats to deal with. We could even try to load _any_ thumbnail that matches the file in question, say for video files, since we'll only care if there _is_ one. See https://askubuntu.com/questions/1368910/how-to-create-custom-thumbnailers-for-nautilus-nemo-and-caja and https://specifications.freedesktop.org/thumbnail-spec/latest/thumbsave.html
// todo Audio should be saved periodically to some temporary location (or merely the final desired location!) and always on quit in case the wrong button is pressed. Potentially, the audio could be moved to trash if the X button was clicked, rather than save. Or use hidden files, but what would clean them up?
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
// todo test on other distros
// todo organize better / refactor / split into separate source files
// todo add message when quitting if writing out is taking awhile (though probably not needed if writing out as we go)
// todo create a slideshow application that plays the audio with the corresponding picture, advancing to the next once the audio is done. slideshow will play everything in directory
// todo   add optional "music" for background since he wants to put specific music in the background.
// todo   slideshow will also display metadata that was entered such as title and comments etc
// todo add keyboard shortcut to nautilus extension?

// todo periodically check if new audio devices have been added (especially if none of the ideal ones are detected yet), see getaudiorecordingdevices or eventing
// todo assign flac album cover art to image it was created for with extra audio icon????

// todo try to select the first audio input, or test both inputs to see which has any audio signal and use that one
// todo display a user facing message about needing to turn the audio interface on/plug in if it isn't detected
// todo capture and log panics/backtraces
// todo replace flacenc with reference libFLAC ffi encoder due to quality concerns (author has not maintained library recently, noticed missing metadata when looking at file in nautilus right click menu, one program failed to load these flac files (sound converter), and there was at least one instance where audio was saved distorted)
// todo analyze the audio recorded so far and if its max amplitude (when excluding a few outliers??? like loud pops?) is too low, show a message to raise the gain. Similarly, if clipping regularly, show message asking to lower gain
// todo rearrange code so as much as possible can be tested via test runners
// todo enforce minimum window size to avoid losing the window if it's resized to tiny size
// todo add sdl3_ttf via bindgen ffi like with libFLAC
// todo build action in ci with auto released artifacts?

// redirect panics to log file. woe is me if this panics within the logger itself
fn handle_panic(payload: &(dyn Any + Send), backtrace: Backtrace) {
    error!("Panicked: ");
    if let Some(string) = payload.downcast_ref::<String>() {
        error!("{string}");
    } else if let Some(str) = payload.downcast_ref::<&'static str>() {
        error!("{str}");
    } else {
        error!("{payload:?}");
    }

    error!("Backtrace: {backtrace:#?}");
}

pub fn main() {
    let config = ProgramConfig::from_file().unwrap();

    let log_path = Path::new(&config.log_file);
    let file_appender = tracing_appender::rolling::never(
        log_path.parent().unwrap_or(".".as_ref()),
        log_path.file_name().unwrap_or("audiosidecar.log".as_ref()),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let level = Level::from_str(config.log_level.as_str()).unwrap_or(Level::INFO);

    let subscriber = tracing_subscriber::registry()
        .with(fmt::Layer::new().with_writer(io::stdout.with_max_level(level)))
        .with(
            fmt::Layer::new()
                .with_ansi(false)
                .with_writer(non_blocking.with_max_level(level)),
        );
    subscriber.init();

    panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        handle_panic(info.payload(), backtrace)
    }));

    info!("============= Started =============");

    let args: Vec<String> = env::args().collect();

    let defaultpath = String::from("/tmp/test.png");
    let filepath = Path::new(args.get(1).unwrap_or(&defaultpath));
    info!("Audio associated with file: {:?}", filepath);

    if let Err(msg) = sdl::init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS) {
        die(format!("SDL initialization failed: {}", msg).as_str());
    }

    // window inits as x11 instead of wayland due to lack of fifo-v1 protocol in gnome.
    // fifo-v1 was added here https://gitlab.gnome.org/GNOME/mutter/-/merge_requests/3355 and will be present in gnome 48.
    // The X11 window is responsible for the window flashing on creation. Wayland does not experience this issue.
    // SDL_VIDEO_DRIVER=wayland can force wayland
    let gfx = match sdl::create_window_and_renderer(
        "Record Audio",
        config.window_width,
        config.window_height,
        SDL_WINDOW_RESIZABLE,
    ) {
        Ok(gfx) => gfx,
        Err(msg) => die(format!("SDL window creation failed: {}", msg).as_str()),
    };

    let mut window_width = config.window_width;
    let mut window_height = config.window_height;

    if let Err(msg) = gfx.set_render_vsync(1) {
        die(format!("SDL vsync failed to enable: {}", msg).as_str());
    }

    if let Err(msg) = gfx.set_window_minimum_size(400, 200) {
        die(format!("SDL vsync failed to enable: {}", msg).as_str());
    }

    let recording_devices = match sdl::get_audio_recording_devices() {
        Ok(a) => a,
        Err(msg) => die(format!("SDL finding audio recording devices failed: {}", msg).as_str()),
    };

    let mut desired_interface_id = SDL_AUDIO_DEVICE_DEFAULT_RECORDING;

    info!(
        "Found {} Audio Devices:    (Matching on \"{}\")",
        recording_devices.len(),
        config.interface
    );

    for device in recording_devices {
        let found = if device
            .name
            .to_lowercase()
            .contains(config.interface.as_str())
        {
            desired_interface_id = device.id;
            " <<<< MATCH FOUND <<<<"
        } else {
            ""
        };

        info!("\t{} {}", device.name, found);
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

    let mut display_waveform: Vec<u32> = Vec::new();
    let mut previous_unchunked_samples: Vec<i32> = Vec::new();

    let mut clicking = false;
    let mut mouse_x: f32 = 0.0;
    let mut mouse_y: f32 = 0.0;
    let mut paused = false;

    let mut sample_count = 0u64;

    let filename_base = filepath
        .with_extension("")
        .file_name()
        .expect("filename should be non-empty")
        .to_string_lossy()
        .to_string();
    let filename = filename_base.clone() + "_audio.flac";

    let mut outputfile = filepath.with_file_name(filename);

    if std::fs::exists(&outputfile).unwrap_or(true) {
        // fail safely, assume conflict if can't determine
        // todo handle exists() result io failure and logging explicitly
        info!("File exists at \"{}\"", outputfile.display());

        match config.existing_file_strategy {
            ExistingFileStrategy::RenameToLast => {
                let mut n = 1;
                let mut tries = 100; //give up after 100 tries to avoid infinite loop
                while std::fs::exists(&outputfile).unwrap_or(true) {
                    n += 1;
                    tries -= 1;
                    outputfile = filepath.with_file_name(
                        filename_base.clone() + format!("_audio{}.flac", n).as_str(),
                    );
                    if tries <= 0 {
                        error!("Failed to find next file name in 100 tries. Continuing.");
                        break;
                    }
                }
            }
            ExistingFileStrategy::Replace => {
                // do nothing, the file will be replaced
                // todo, move to trash or something first
            }
            _ => todo!(),
        }
    }

    info!("Saving audio to \"{}\"", outputfile.display());

    let mut encoder_config = flac::EncoderConfig::new();

    encoder_config.set_output_path(&outputfile);

    let encoder = match encoder_config.get_encoder() {
        Ok(e) => e,
        Err(msg) => die(msg.as_str()),
    };

    let mut frame_time = Instant::now();
    let mut max_time = Instant::now();
    let mut max_frame_time = 0.0;

    let mut frames = 0;
    let mut framespersec = 0.0;
    let mut start_sec = Instant::now();

    let mut ui = UI::new(gfx);
    let mut input = Input::default();

    loop {
        // poll until all events are handled and the queue runs dry
        while let Some(event) = sdl::poll_event() {
            match event {
                // todo New events will have to be added both here and in sdl::poll_event()
                Event::Window(event_type, e) => {
                    if event_type == SDL_EventType::WINDOW_RESIZED {
                        window_width = e.data1 as u32;
                        window_height = e.data2 as u32;
                    }
                }
                Event::Button(event_type, _e) => {
                    if event_type == SDL_EventType::MOUSE_BUTTON_DOWN {
                        clicking = true;
                    } else if event_type == SDL_EventType::MOUSE_BUTTON_UP {
                        clicking = false;
                    }
                    input.mouse_button_pressed = clicking;
                }
                Event::Motion(event_type, e) => {
                    if event_type == SDL_EventType::MOUSE_MOTION {
                        mouse_x = e.x;
                        mouse_y = e.y;
                    }
                    input.mouse_x = mouse_x;
                    input.mouse_y = mouse_y;
                }
                Event::Quit(_) => {
                    return save_and_quit(&ui, encoder, logical_interface_id, audio_stream);
                }
                _ => continue,
            }
        }

        ui.apply_input(&input);

        let mut samples = match sdl::get_audio_stream_data_i32(audio_stream) {
            Ok(s) => s,
            Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
        };

        if !paused {
            sample_count += samples.len() as u64;

            or_die(encoder.encode(&samples)); // encode and save to file as we go

            // combine audio into chunks for display
            const CHUNKSIZE: usize = 44100 / 100; // samples

            previous_unchunked_samples.append(&mut samples);
            let mut max_sample = 0;
            for n in 0..previous_unchunked_samples.len() / CHUNKSIZE {
                for i in 0..CHUNKSIZE {
                    let v = previous_unchunked_samples[n * CHUNKSIZE + i].unsigned_abs();
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
        }

        ui.clear();

        const BORDER_SIZE: f32 = 10.0;

        ui.draw_waveform(
            &display_waveform,
            BORDER_SIZE,
            BORDER_SIZE,
            window_width as f32 - BORDER_SIZE * 2.0,
            window_height as f32 - 100.0,
            !paused,
        );

        const CONTROL_HEIGHT: f32 = 100.0 - BORDER_SIZE * 3.0;
        let control_start_y = window_height as f32 - CONTROL_HEIGHT - BORDER_SIZE;

        let p_button_width = 220.0 - BORDER_SIZE * 3.0;
        if ui.button(
            if paused { "Record" } else { "Pause" },
            BORDER_SIZE,
            control_start_y,
            p_button_width,
            CONTROL_HEIGHT,
        ) {
            info!("pressed play/pause audio button");
            paused = !paused;
        }

        ui.draw_text(
            format!(
                "Record Time: {}",
                utils::format_duration(Duration::from_secs_f64(sample_count as f64 / 44100.0,))
            )
            .as_str(),
            BORDER_SIZE + p_button_width + BORDER_SIZE,
            control_start_y + (CONTROL_HEIGHT / 2.0),
            3.0,
            false,
            true,
        );

        let button_width = 200.0;
        let button_height = 100.0 - BORDER_SIZE * 3.0;
        if ui.button(
            "Done",
            window_width as f32 - BORDER_SIZE - button_width,
            control_start_y,
            button_width,
            button_height,
        ) {
            info!("pressed save audio button");

            return save_and_quit(&ui, encoder, logical_interface_id, audio_stream);
        }



        // todo do a quick fade between paused sections of audio

        if max_time.elapsed().as_secs_f64() > 5.0 {
            max_frame_time = 0.0;
            max_time = Instant::now();
        }

        let elapsed = frame_time.elapsed().as_secs_f64();
        if elapsed > max_frame_time {
            max_frame_time = elapsed;
        }

        // // todo toggle debug text on and off through config file
        // let mut debug_text = format!("frametime: {:.2}ms\n", max_frame_time * 1000.0);
        // debug_text += format!("fps: {}\n", framespersec).as_str();
        // debug_text += format!("samples: {}\n", sample_count).as_str();
        // debug_text += format!(
        //     "data size: {:.1}MiB\n",
        //     sample_count as f64 * 4.0 / 1024.0 / 1024.0
        // )
        // .as_str();
        // debug_text += format!(
        //     "waveform size: {:.1}MiB\n",
        //     display_waveform.len() as f64 * 4.0 / 1024.0 / 1024.0
        // )
        // .as_str();
        // ui.debug_view(debug_text.as_str());

        ui.present();
        frames += 1;
        frame_time = Instant::now();

        if start_sec.elapsed().as_secs_f64() > 1.0 {
            start_sec = Instant::now();
            framespersec = frames as f64;
            frames = 0;
        }
    }
}

fn save_and_quit(
    ui: &UI,
    encoder: Encoder,
    logical_interface_id: SDL_AudioDeviceID,
    audio_stream: *mut SDL_AudioStream,
) {
    info!("Shutdown triggered");

    debug!("Capturing final audio samples...");

    if let Err(msg) = sdl::flush_audio_stream(audio_stream) {
        die(format!("SDL could not flush audio stream: {}", msg).as_str());
    }
    sdl::close_audio_device(logical_interface_id);

    // get last bit of audio
    let samples = match sdl::get_audio_stream_data_i32(audio_stream) {
        Ok(s) => s,
        Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
    };

    debug!("Finalizing audio to disk...");

    or_die(encoder.encode(&samples));
    or_die(encoder.finish());

    info!("Audio saved");
    ui.hide();

    let success_sound = match sdl::loadwav("success.wav") {
        Ok(a) => a,
        Err(msg) => die(format!("SDL Failed to load wav: {}", msg).as_str()),
    };

    debug!("Playing success sound...");
    sdl::play_sound(&success_sound); // todo make configurable

    sdl::quit();

    info!("============= Exited =============");

    // exit(0); // todo avoid exiting the program with exit() to allow things to drop, etc.
}

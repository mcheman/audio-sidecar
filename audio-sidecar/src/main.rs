extern crate sdl3_sys;
extern crate flac_sys;

use crate::sdl::{Event, Gfx};
use config::{Config, FileFormat};
use log::{debug, error, info};
use sdl3_sys::everything::*;
use std::cmp::max;
use std::ffi::CString;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;
use std::{env, io, ptr};
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt};

mod sdl;

fn die(s: &str) -> ! {
    error!("{}", s);
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

// todo load values from config file: interface text to search for,

// todo try to select the first audio input, or test both inputs to see which has any audio signal and use that one
// todo display a user facing message about needing to turn the audio interface on/plug in if it isn't detected
// todo capture and log panics/backtraces
// todo replace flacenc with reference libFLAC ffi encoder due to quality concerns (author has not maintained library recently, noticed missing metadata when looking at file in nautilus right click menu, one program failed to load these flac files (sound converter), and there was at least one instance where audio was saved distorted)
// todo analyze the audio recorded so far and if its max amplitude (when excluding a few outliers??? like loud pops?) is too low, show a message to raise the gain. Similarly, if clipping regularly, show message asking to lower gain
// todo rearrange code so as much as possible can be tested via test runners
// todo enforce minimum window size to avoid losing the window if it's resized to tiny size
// todo add sdl3_ttf via bindgen ffi like with libFLAC

#[derive(Debug, PartialEq)]
enum ExistingFileStrategy {
    RenameToLast,
    RenameToFirst,
    Append,
    Replace,
    Ask,
}

impl FromStr for ExistingFileStrategy {
    type Err = ();
    fn from_str(s: &str) -> Result<ExistingFileStrategy, ()> {
        match s {
            "rename-to-last" => Ok(ExistingFileStrategy::RenameToLast),
            "rename-to-first" => Ok(ExistingFileStrategy::RenameToFirst),
            "append" => Ok(ExistingFileStrategy::Append),
            "replace" => Ok(ExistingFileStrategy::Replace),
            "ask" => Ok(ExistingFileStrategy::Ask),
            _ => Err(()),
        }
    }
}

struct ProgramConfig {
    interface: String, // search string for the audio interface to use
    window_width: u32,
    window_height: u32,
    log_file: String,
    log_level: String,
    existing_file_strategy: ExistingFileStrategy,
}

impl ProgramConfig {
    fn from_file() -> Result<ProgramConfig, String> {
        let settings = Config::builder()
            .add_source(config::File::new(
                "./audio-sidecar-config",
                FileFormat::Toml,
            ))
            .build()
            .unwrap(); // todo this should have defaults and not panic if config file doesn't exist

        let interface: String = settings.get("Interface").unwrap_or(String::from(""));
        let window_width: u32 = settings.get("WindowWidth").unwrap_or(1200);
        let window_height: u32 = settings.get("WindowHeight").unwrap_or(600);
        let log_file: String = settings
            .get("LogFile")
            .unwrap_or(String::from("audioSidecar.log"));
        let log_level: String = settings.get("LogLevel").unwrap_or(String::from("debug"));

        let existing_file_strategy = ExistingFileStrategy::from_str(
            settings
                .get("ExistingFileStrategy")
                .unwrap_or(String::from(""))
                .as_str(),
        )
            .unwrap_or(ExistingFileStrategy::RenameToLast);

        Ok(ProgramConfig {
            interface,
            window_width,
            window_height,
            log_file,
            log_level,
            existing_file_strategy,
        })
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();

    let minutes = (seconds / 60.0).floor();
    let seconds = seconds % 60.0;

    let hours = (minutes / 60.0).floor();
    let minutes = minutes % 60.0;

    if hours > 0.0 {
        format!("{}h {}m {:.1}s", hours, minutes, seconds)
    } else if minutes > 0.0 {
        format!("{}m {:.1}s", minutes, seconds)
    } else {
        format!("{:.1}s", seconds)
    }
}

fn draw_waveform(gfx: &Gfx, waveform: &[u32], x: f32, y: f32, width: f32, height: f32) {
    or_die(gfx.set_render_draw_color(43, 43, 43, 255));
    let rect = SDL_FRect {
        x,
        y,
        w: width,
        h: height,
    };
    or_die(gfx.render_fill_rect(&rect));

    or_die(gfx.set_render_draw_color(255, 255, 255, 255));

    // todo put this in its own handler widget thing which only renders the new audio to a buffer which can then be scrolled on the screen, rather than line rendering the whole waveform

    const MAX_AMPLITUDE: u32 = (i32::MAX >> 8) as u32; // 24bits

    let chunks_to_render = width; // one chunk per pixel
    for (col, m) in waveform
        .iter()
        .skip(max(0, waveform.len() as i64 - chunks_to_render as i64) as usize)
        .enumerate()
    {
        let is_clipped = *m >= MAX_AMPLITUDE - 1;

        let h = *m as f32 * (height / MAX_AMPLITUDE as f32);
        let y1 = y + (height / 2.0) - (h / 2.0);
        let y2 = y1 + h;

        if is_clipped {
            or_die(gfx.set_render_draw_color(250, 43, 43, 255));
        }

        or_die(gfx.render_line(x + col as f32, y1, x + col as f32, y2));

        if is_clipped {
            or_die(gfx.set_render_draw_color(255, 255, 255, 255));
        }
    }
}

fn draw_text(gfx: &Gfx, text: &str, x: f32, y: f32, size: f32, centered_x: bool, centered_y: bool) {
    or_die(gfx.set_render_scale(size, size));

    const GLYPH_SIZE: f32 = 8.0;

    let offset_x = if centered_x {
        // note that bitmapped 8x8 pixel font generally has 2 pixels of empty space on the right and 1 pixel of space on the bottom
        //   we subtract 1 pixel here to compensate. subtracting 0.5 pixels from Y results in mushed scaling so we don't do that
        (text.len() as f32 * GLYPH_SIZE) / 2.0 - 1.0
    } else {
        0.0
    };
    let offset_y = if centered_y { GLYPH_SIZE / 2.0 } else { 0.0 };

    or_die(gfx.render_debug_text(text, x / size - offset_x, y / size - offset_y));

    or_die(gfx.set_render_scale(1.0, 1.0));
}

// returns true if button is currently clicked
fn button(
    gfx: &Gfx,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    mouse_x: f32,
    mouse_y: f32,
    clicked: bool,
) -> bool {
    let mouse_colliding = mouse_x > x && mouse_x < x + width && mouse_y > y && mouse_y < y + height;

    if mouse_colliding && clicked {
        or_die(gfx.set_render_draw_color(20, 20, 20, 255));
    } else if mouse_colliding {
        // hover state
        or_die(gfx.set_render_draw_color(80, 90, 90, 255));
    } else {
        or_die(gfx.set_render_draw_color(80, 80, 80, 255));
    }

    let rect = SDL_FRect {
        x,
        y,
        w: width,
        h: height,
    };

    or_die(gfx.render_fill_rect(&rect));

    // button text
    or_die(gfx.set_render_draw_color(255, 255, 255, 255));

    draw_text(
        &gfx,
        text,
        x + width / 2.0,
        y + height / 2.0,
        3.0,
        true,
        true,
    );

    // todo only emit click when mouse button is released

    mouse_colliding && clicked
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

    let mut recorded_audio: Vec<i32> = Vec::new();

    let mut display_waveform: Vec<u32> = Vec::new();
    let mut previous_unchunked_samples: Vec<i32> = Vec::new();

    let mut clicking = false;
    let mut mouse_x: f32 = 0.0;
    let mut mouse_y: f32 = 0.0;
    let mut paused = false;

    let mut is_clicking_save = false;
    let mut is_clicking_pause = false;

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
                }
                Event::Motion(event_type, e) => {
                    if event_type == SDL_EventType::MOUSE_MOTION {
                        mouse_x = e.x;
                        mouse_y = e.y;
                    }
                }
                Event::Quit(_) => {
                    save_and_quit(
                        &gfx,
                        filepath,
                        logical_interface_id,
                        audio_stream,
                        &mut recorded_audio,
                        &config,
                    );
                }
                _ => continue,
            }
        }

        let mut samples = match sdl::get_audio_stream_data_i32(audio_stream) {
            Ok(s) => s,
            Err(msg) => die(format!("SDL GetAudioStreamData failed: {}", msg).as_str()),
        };

        // clip audio to 24 bits by removing quietest 8 bits
        for s in samples.iter_mut() {
            *s >>= 8;
        }

        if !paused {
            recorded_audio.append(&mut samples.clone());

            // combine audio into chunks for display
            const CHUNKSIZE: usize = 44100 / 100; // samples

            previous_unchunked_samples.append(&mut samples);
            let mut max_sample = 0;
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
        }

        or_die(gfx.set_render_draw_color(53, 53, 53, 255));
        or_die(gfx.render_clear());

        const BORDER_SIZE: f32 = 10.0;

        draw_waveform(
            &gfx,
            &display_waveform,
            BORDER_SIZE,
            BORDER_SIZE,
            window_width as f32 - BORDER_SIZE * 2.0,
            window_height as f32 - 100.0,
        );

        draw_text(
            &gfx,
            format!(
                "Record Time: {}",
                format_duration(Duration::from_secs_f64(
                    recorded_audio.len() as f64 / 44100.0,
                ))
            )
                .as_str(),
            BORDER_SIZE,
            window_height as f32 - (100.0 - BORDER_SIZE) / 2.0,
            3.0,
            false,
            true,
        );

        let button_width = 300.0;
        let button_height = 100.0 - BORDER_SIZE * 3.0;
        if button(
            &gfx,
            "Save Audio",
            window_width as f32 - BORDER_SIZE - button_width,
            window_height as f32 - BORDER_SIZE - button_height,
            button_width,
            button_height,
            mouse_x,
            mouse_y,
            clicking,
        ) && !is_clicking_save
        {
            info!("pressed save audio button");

            is_clicking_save = true;

            save_and_quit(
                &gfx,
                filepath,
                logical_interface_id,
                audio_stream,
                &mut recorded_audio,
                &config,
            );
        }

        let p_button_width = 100.0 - BORDER_SIZE * 3.0;
        let p_button_height = 100.0 - BORDER_SIZE * 3.0;
        if button(
            &gfx,
            if paused { "|>" } else { "||" },
            window_width as f32 - BORDER_SIZE - button_width - p_button_width - BORDER_SIZE,
            window_height as f32 - BORDER_SIZE - button_height,
            p_button_width,
            p_button_height,
            mouse_x,
            mouse_y,
            clicking,
        ) && !is_clicking_pause
        {
            info!("pressed play/pause audio button");
            paused = !paused;
            is_clicking_pause = true;
        }

        // todo do a quick fade between paused sections of audio

        if !clicking {
            is_clicking_pause = false;
            is_clicking_save = false;
        }

        or_die(gfx.render_present());
    }
}

fn save_and_quit(
    gfx: &Gfx,
    filepath: &Path,
    logical_interface_id: SDL_AudioDeviceID,
    audio_stream: *mut SDL_AudioStream,
    recorded_audio: &mut Vec<i32>,
    config: &ProgramConfig,
) {
    info!("Shutdown triggered");

    debug!("Capturing final audio samples...");

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

    debug!("Encoding audio...");

    //sdl::quit(); // todo hide window while audio exports so it looks immediate? alternately show a progress bar?


    debug!("Saving audio to disk...");

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
        // todo handle exists() io failure and logging explicitly
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
                // do nothing, the file will be replaced on save
                // todo, move to trash or something first
            }
            _ => todo!(),
        }
    }

    info!("Saving audio to \"{}\"", outputfile.display());

    unsafe {
        let encoder = flac_sys::FLAC__stream_encoder_new();
        if encoder.is_null() {
            die("1");
        }

        // flac__bool is 1 for true, 0 for false
        let mut ok = flac_sys::FLAC__stream_encoder_set_compression_level(encoder, 8);
        ok &= flac_sys::FLAC__stream_encoder_set_channels(encoder, 1);
        ok &= flac_sys::FLAC__stream_encoder_set_bits_per_sample(encoder, 24);
        ok &= flac_sys::FLAC__stream_encoder_set_sample_rate(encoder, 44100);

        if ok == 0 {
            die("2");
        }

        let mut metadata: [*mut flac_sys::FLAC__StreamMetadata; 2] = [ptr::null_mut(); 2];
        metadata[0] = flac_sys::FLAC__metadata_object_new(
            flac_sys::FLAC__MetadataType_FLAC__METADATA_TYPE_VORBIS_COMMENT,
        );
        metadata[1] =
            flac_sys::FLAC__metadata_object_new(flac_sys::FLAC__MetadataType_FLAC__METADATA_TYPE_PADDING);

        // todo check metadatas for NULL

        (*metadata[1]).length = 1234;

        ok = flac_sys::FLAC__stream_encoder_set_metadata(encoder, metadata.as_mut_ptr(), 2);
        if ok == 0 {
            die("3");
        }

        let init_status = flac_sys::FLAC__stream_encoder_init_file(
            encoder,
            CString::new(outputfile.display().to_string())
                .expect("filename to be converted to CString")
                .as_ptr(),
            None,
            ptr::null_mut(),
        );

        if init_status != flac_sys::FLAC__StreamEncoderInitStatus_FLAC__STREAM_ENCODER_INIT_STATUS_OK {
            die("4");
        }


        let ok = flac_sys::FLAC__stream_encoder_process(encoder, &recorded_audio.as_ptr(), recorded_audio.len() as u32);

        let ok = flac_sys::FLAC__stream_encoder_finish(encoder);

        flac_sys::FLAC__metadata_object_delete(metadata[0]);
        flac_sys::FLAC__metadata_object_delete(metadata[1]);
        flac_sys::FLAC__stream_encoder_delete(encoder);
    }


    info!("Audio saved");
    gfx.hide_window();

    let success_sound = match sdl::loadwav("success.wav") {
        Ok(a) => a,
        Err(msg) => die(format!("SDL Failed to load wav: {}", msg).as_str()),
    };

    debug!("Playing success sound...");
    sdl::play_sound(&success_sound);

    sdl::quit();

    info!("============= Exited =============");

    exit(0);
}

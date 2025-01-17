extern crate sdl3_sys;

use std::ffi::{c_float, c_int, CStr};
use std::mem::zeroed;
use std::process::exit;
use std::time::Duration;
use config::{Config, FileFormat};
use sdl3_sys::everything::*;

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
    let settings = Config::builder()
        .add_source(config::File::new("audio-sidecar-config", FileFormat::Toml))
        .build()
        .unwrap();

    let interface: String = settings.get("Interface").unwrap();
    let window_width: u32 = settings.get("WindowWidth").unwrap();
    let window_height: u32 = settings.get("WindowHeight").unwrap();


    if let Err(msg) = sdl::init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS) {
        die(format!("SDL initialization failed: {}", msg).as_str());
    }


    let result = sdl::create_window_and_renderer("Record Audio", window_width, window_height, SDL_WINDOW_RESIZABLE);
    if let Err(msg) = result {
        die(format!("SDL window creation failed: {}", msg).as_str());
    }
    let mut gfx = result.unwrap();



    unsafe {
        let mut num_devices = 0;

        let devices = SDL_GetAudioRecordingDevices(&mut num_devices); // free'd below
        if devices.is_null() || num_devices == 0 {
            SDL_Log(c"No recording devices found!".as_ptr(), SDL_GetError());
            SDL_Quit();
            exit(1);
        }


        let mut desired_interface_id = SDL_AUDIO_DEVICE_DEFAULT_RECORDING;

        println!("Found {} Audio Devices:", num_devices);
        for i in 0..num_devices {
            let deviceid = devices.offset(i as isize);
            let name = CStr::from_ptr(SDL_GetAudioDeviceName(*deviceid)).to_string_lossy().to_string();
            if name.to_lowercase().contains(interface.as_str()) {
                println!("\t{} <<<<<<<<<<<<<<< MATCH FOUND <<<<<", name);
                desired_interface_id = *devices.offset(i as isize);
            } else {
                println!("\t{}", name);
            }
        }

        SDL_free(devices.cast());

        let src_spec = SDL_AudioSpec{
            channels: 1,
            freq: 44100,
            format: SDL_AudioFormat::S32
        };

        let dest_spec = SDL_AudioSpec{
            channels: 1,
            freq: 44100,
            format: SDL_AudioFormat::S32 // todo can I simply truncate 32 bit samples to 24 bit for the flac encoder?
        };

        let logical_interface_id = SDL_OpenAudioDevice(desired_interface_id, &src_spec);

        let audio_stream = SDL_CreateAudioStream(&src_spec, &dest_spec);

        SDL_BindAudioStream(logical_interface_id, audio_stream);

        loop {
            let mut event: SDL_Event = zeroed();
            while SDL_PollEvent(&mut event) {  // poll until all events are handled!
                // decide what to do with this event.
                match SDL_EventType(event.r#type) {
                    SDL_EventType::QUIT => {
                        SDL_FlushAudioStream(audio_stream);
                        SDL_CloseAudioDevice(logical_interface_id);
                        SDL_Quit();
                        exit(0);
                    },
                    _ => continue
                }
            }

            let mut samples = [0i32; 44100/30];

            let bytes_read = SDL_GetAudioStreamData(audio_stream, samples.as_mut_ptr().cast(), (samples.len() * 2) as c_int);

            let mut max = 0;
            for s in samples {
                if s > max {
                    max = s;
                }
            }

            or_die(sdl::set_render_draw_color(&mut gfx, SDL_Color{r: 0, g: 0, b: 0, a: 255}));
            or_die(sdl::render_clear(&mut gfx));
            or_die(sdl::set_render_draw_color(&mut gfx, SDL_Color{r: 255, g: 150, b: 255, a: 255}));


            let rect = SDL_FRect {
                h: (max as f64 * (480.0 / i32::MAX as f64)) as c_float,
                w: 100.0,
                x: 0.0,
                y: 0.0,
            };

            or_die(sdl::render_fill_rect(&mut gfx, &rect));

            or_die(sdl::render_present(&mut gfx));

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
        }
    }


}

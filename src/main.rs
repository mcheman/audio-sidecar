extern crate sdl3_sys;

use std::ffi::{c_float, c_int, CStr, CString};
use std::mem::zeroed;
use std::process::exit;
use std::ptr;
use std::time::Duration;
use sdl3_sys::everything::*;

// todo wrap sdl code in safe crate and hide these variables within, ideally within some created struct
static mut window: *mut SDL_Window = ptr::null_mut();
static mut renderer: *mut SDL_Renderer = ptr::null_mut();


// todo copious error checking

pub fn main() {
    // let a = CString::new(String::from("test")).unwrap();
    // let b = a.as_c_str().to_str().unwrap();
    // let c = a.to_str().unwrap();
    // let d = a.into_string().unwrap();



    unsafe {
        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS);

        SDL_CreateWindowAndRenderer(c"Record Audio".as_ptr(), 640, 480, 0, &raw mut window, &raw mut renderer);

        SDL_RenderClear(renderer);
        SDL_RenderPresent(renderer);
    }



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
            if name.to_lowercase().contains("scarlett") {
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

            SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
            SDL_RenderClear(renderer);
            SDL_SetRenderDrawColor(renderer, 255, 150, 255, 255);


            let rect = SDL_FRect {
                h: (max as f64 * (480.0 / i32::MAX as f64)) as c_float,
                w: 100.0,
                x: 0.0,
                y: 0.0,
            };

            SDL_RenderFillRect(renderer, &rect);

            SDL_RenderPresent(renderer);
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
        }
    }


}

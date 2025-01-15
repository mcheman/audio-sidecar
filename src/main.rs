extern crate sdl3_sys;

use std::ffi::CString;
use std::ptr;
// use sdl3::pixels::Color;
// use sdl3::event::Event;
// use sdl3::keyboard::Keycode;
use std::time::Duration;
use sdl3_sys::everything::*;
// use sdl3::audio::{AudioFormat, AudioSpec};


pub fn main() {
    // let a = CString::new(String::from("test")).unwrap();
    // let b = a.as_c_str().to_str().unwrap();
    // let c = a.to_str().unwrap();
    // let d = a.into_string().unwrap();


    unsafe {
        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS);
        let mut window: *mut SDL_Window = ptr::null_mut();
        let mut renderer: *mut SDL_Renderer = ptr::null_mut();
        SDL_CreateWindowAndRenderer(c"Record Audio".as_ptr(), 640, 480, 0, &mut window, &mut renderer);

        SDL_RenderClear(renderer);
        SDL_RenderPresent(renderer);
    }


    // let audio_subsystem = sdl_context.audio().unwrap();
    //
    // let audio_device_ids = audio_subsystem.audio_recording_device_ids().unwrap();
    // for (index, audio_device_id) in audio_device_ids.iter().enumerate() {
    //
    //     if audio_device_id.name().unwrap().to_lowercase().contains("scarlett") {
    //         println!("Audio device \"{}\" found", audio_device_id.name().unwrap());
    //     }
    // }
    //
    // let audio_settings = AudioSpec::new(Some(44100), Some(1), Some(AudioFormat::F32LE));
    //
    // audio_subsystem.default_recording_device().
    

    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
    //     .position_centered()
    //     .hidden()
    //     .build()
    //     .unwrap();
    //
    // let mut canvas = window.into_canvas();
    // canvas.window_mut().show();
    //
    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.present();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // let mut i = 0;
    // 'running: loop {
    //     i = (i + 1) % 255;
    //     canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
    //     canvas.clear();
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit {..} |
    //             Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
    //                 break 'running
    //             },
    //             _ => {}
    //         }
    //     }
    //     // The rest of the game loop goes here...
    //
    //     canvas.present();
    //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    // }
    ::std::thread::sleep(Duration::new(1, 1_000_000_000u32 / 60));
}
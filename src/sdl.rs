use sdl3_sys::everything::*;
use sdl3_sys::init::SDL_InitFlags;
use std::cmp::min;
use std::ffi::{c_int, CStr, CString};
use std::ptr;
// todo wrap sdl code in safe crate and hide these variables within, ideally within some created struct

pub struct Gfx {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
}

fn get_error() -> String {
    unsafe { CStr::from_ptr(SDL_GetError()).to_string_lossy().to_string() }
}

pub fn init(flags: SDL_InitFlags) -> Result<(), String> {
    unsafe {
        if SDL_Init(flags) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn quit() {
    unsafe {
        SDL_Quit();
    }
}

pub fn create_window_and_renderer(
    title: &str,
    width: u32,
    height: u32,
    window_flags: SDL_WindowFlags,
) -> Result<Gfx, String> {
    let mut gfx = Gfx {
        window: ptr::null_mut(),
        renderer: ptr::null_mut(),
    };

    if unsafe {
        SDL_CreateWindowAndRenderer(
            CString::new(title)
                .expect("window title to be converted to CString")
                .as_ptr(),
            width as i32,
            height as i32,
            window_flags,
            &raw mut gfx.window,
            &raw mut gfx.renderer,
        )
    } {
        Ok(gfx)
    } else {
        Err(get_error())
    }
}

// todo make into trait with gfx as self?
pub fn set_render_draw_color(gfx: &Gfx, color: SDL_Color) -> Result<(), String> {
    unsafe {
        if SDL_SetRenderDrawColor(gfx.renderer, color.r, color.g, color.b, color.a) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn render_clear(gfx: &Gfx) -> Result<(), String> {
    unsafe {
        if SDL_RenderClear(gfx.renderer) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn render_fill_rect(gfx: &Gfx, rect: &SDL_FRect) -> Result<(), String> {
    unsafe {
        if SDL_RenderFillRect(gfx.renderer, rect) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn render_present(gfx: &Gfx) -> Result<(), String> {
    unsafe {
        if SDL_RenderPresent(gfx.renderer) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub struct AudioDevice {
    pub id: SDL_AudioDeviceID,
    pub name: String,
}

pub fn get_audio_recording_devices() -> Result<Vec<AudioDevice>, String> {
    let mut num_devices = 0;

    let devices: *mut SDL_AudioDeviceID = unsafe { SDL_GetAudioRecordingDevices(&mut num_devices) };

    if devices.is_null() || num_devices == 0 {
        Err(format!("No recording devices found: {}", get_error()))
    } else {
        let mut audio_devices = Vec::with_capacity(num_devices as usize);

        for i in 0..num_devices {
            let device_id = unsafe { *(devices.offset(i as isize)) };
            let name =
                unsafe { CStr::from_ptr(SDL_GetAudioDeviceName(device_id)).to_string_lossy() };

            audio_devices.push(AudioDevice {
                id: device_id,
                name: name.to_string(),
            });
        }

        unsafe {
            SDL_free(devices.cast());
        }

        Ok(audio_devices)
    }
}

// get all samples of pending audio
// todo enforce audio is in i32 format when calling this function
pub fn get_audio_stream_data_i32(stream: *mut SDL_AudioStream) -> Result<Vec<i32>, String> {
    let mut samples = Vec::with_capacity(1024);

    let mut sample_buffer = [0i32; 1024];
    let buffer_bytes = (sample_buffer.len() * 4) as c_int;

    loop {
        let bytes_read = unsafe {
            SDL_GetAudioStreamData(stream, sample_buffer.as_mut_ptr().cast(), buffer_bytes)
        };

        if bytes_read == -1 {
            return Err(get_error()); // todo we probably want to handle this better since this could maybe interrupt an in progress recording, say if the audio device got disconnected?
        } else if bytes_read == 0 {
            break;
        }

        let samples_read = (bytes_read / 4) as usize;

        for i in 0..(min(sample_buffer.len(), samples_read)) {
            samples.push(sample_buffer[i]);
        }
    }

    Ok(samples)
}

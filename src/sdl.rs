use sdl3_sys::everything::*;
use sdl3_sys::init::SDL_InitFlags;
use std::cmp::min;
use std::ffi::{c_int, CStr, CString};
use std::ptr;

const AUDIO_SPEC: SDL_AudioSpec = SDL_AudioSpec {
    channels: 1,
    freq: 44100,
    format: SDL_AudioFormat::S32, // todo can I simply truncate 32 bit samples to 24 bit for the flac encoder?
};

pub struct Gfx {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
}

fn get_error() -> String {
    unsafe { CStr::from_ptr(SDL_GetError()).to_string_lossy().to_string() }
}

pub fn init(flags: SDL_InitFlags) -> Result<(), String> {
    if unsafe { SDL_Init(flags) } {
        Ok(())
    } else {
        Err(get_error())
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
pub fn set_render_draw_color(gfx: &Gfx, r: u8, g: u8, b: u8, a: u8) -> Result<(), String> {
    if unsafe { SDL_SetRenderDrawColor(gfx.renderer, r, g, b, a) } {
        Ok(())
    } else {
        Err(get_error())
    }
}

pub fn render_clear(gfx: &Gfx) -> Result<(), String> {
    if unsafe { SDL_RenderClear(gfx.renderer) } {
        Ok(())
    } else {
        Err(get_error())
    }
}

pub fn render_fill_rect(gfx: &Gfx, rect: &SDL_FRect) -> Result<(), String> {
    if unsafe { SDL_RenderFillRect(gfx.renderer, rect) } {
        Ok(())
    } else {
        Err(get_error())
    }
}

pub fn render_present(gfx: &Gfx) -> Result<(), String> {
    if unsafe { SDL_RenderPresent(gfx.renderer) } {
        Ok(())
    } else {
        Err(get_error())
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

pub fn open_audio_device(id: SDL_AudioDeviceID) -> Result<SDL_AudioDeviceID, String> {
    let logical_interface_id = unsafe { SDL_OpenAudioDevice(id, &AUDIO_SPEC) };

    if logical_interface_id != 0 {
        Ok(logical_interface_id)
    } else {
        Err(get_error())
    }
}

// todo, we're really just pretending stuff like this is safe since I could pass garbage as the id. Consider making these actually safe
pub fn close_audio_device(id: SDL_AudioDeviceID) {
    unsafe {
        SDL_CloseAudioDevice(id);
    }
}

pub fn create_audio_stream() -> Result<*mut SDL_AudioStream, String> {
    let audio_steam = unsafe { SDL_CreateAudioStream(&AUDIO_SPEC, &AUDIO_SPEC) };

    if audio_steam.is_null() {
        Err(get_error())
    } else {
        Ok(audio_steam)
    }
}

pub fn bind_audio_stream(
    id: SDL_AudioDeviceID,
    stream: *mut SDL_AudioStream,
) -> Result<(), String> {
    if unsafe { SDL_BindAudioStream(id, stream) } {
        Ok(())
    } else {
        Err(get_error())
    }
}

pub fn flush_audio_stream(stream: *mut SDL_AudioStream) -> Result<(), String> {
    if unsafe { SDL_FlushAudioStream(stream) } {
        Ok(())
    } else {
        Err(get_error())
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

pub fn render_debug_text(gfx: &Gfx, text: &str, x: f32, y: f32) -> Result<(), String> {
    if unsafe {
        SDL_RenderDebugText(gfx.renderer, x, y, CString::new(text)
            .expect("debug text to be converted to CString")
            .as_ptr())
    } {
        Ok(())
    } else {
        Err(get_error())
    }
}

#[allow(dead_code)]
pub enum Event {
    Common(SDL_CommonEvent),
    Display(SDL_DisplayEvent),
    Window(SDL_WindowEvent),
    KDevice(SDL_KeyboardDeviceEvent),
    Key(SDL_KeyboardEvent),
    Edit(SDL_TextEditingEvent),
    EditCandidates(SDL_TextEditingCandidatesEvent),
    Text(SDL_TextInputEvent),
    MDevice(SDL_MouseDeviceEvent),
    Motion(SDL_MouseMotionEvent),
    Button(SDL_MouseButtonEvent),
    Wheel(SDL_MouseWheelEvent),
    JDevice(SDL_JoyDeviceEvent),
    JAxis(SDL_JoyAxisEvent),
    JBall(SDL_JoyBallEvent),
    JHat(SDL_JoyHatEvent),
    JButton(SDL_JoyButtonEvent),
    JBattery(SDL_JoyBatteryEvent),
    GDevice(SDL_GamepadDeviceEvent),
    GAxis(SDL_GamepadAxisEvent),
    GButton(SDL_GamepadButtonEvent),
    GTouchpad(SDL_GamepadTouchpadEvent),
    GSensor(SDL_GamepadSensorEvent),
    ADevice(SDL_AudioDeviceEvent),
    CDevice(SDL_CameraDeviceEvent),
    Sensor(SDL_SensorEvent),
    Quit(SDL_QuitEvent),
    User(SDL_UserEvent),
    TFinger(SDL_TouchFingerEvent),
    PProximity(SDL_PenProximityEvent),
    PTouch(SDL_PenTouchEvent),
    PMotion(SDL_PenMotionEvent),
    PButton(SDL_PenButtonEvent),
    PAxis(SDL_PenAxisEvent),
    Render(SDL_RenderEvent),
    Drop(SDL_DropEvent),
    Clipboard(SDL_ClipboardEvent),
}

pub fn poll_event() -> Option<Event> {
    let mut event = SDL_Event::default();
    if unsafe { SDL_PollEvent(&mut event) } {
        match SDL_EventType(unsafe { event.r#type }) {
            SDL_EventType::QUIT => Some(Event::Quit(unsafe { event.quit })),
            _ => Some(Event::User(unsafe { event.user })), // dummy event so we can decern an unimplemented event (in this function) from NO event
        }
    } else {
        None
    }
}

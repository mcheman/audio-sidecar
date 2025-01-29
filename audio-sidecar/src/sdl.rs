use sdl3_sys::everything::*;
use sdl3_sys::init::SDL_InitFlags;
use std::cmp::min;
use std::ffi::{CStr, CString};
use std::ptr;

const AUDIO_SPEC: SDL_AudioSpec = SDL_AudioSpec {
    channels: 1,
    freq: 44100,
    format: SDL_AudioFormat::S32,
};

pub struct Gfx {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
}

impl Drop for Gfx {
    fn drop(&mut self) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_DestroyRenderer(self.renderer);
            }
        }

        if !self.window.is_null() {
            unsafe {
                SDL_DestroyWindow(self.window);
            }
        }
    }
}

fn ok_or_err(is_ok: bool) -> Result<(), String> {
    if is_ok {
        Ok(())
    } else {
        Err(get_error())
    }
}

fn get_error() -> String {
    unsafe { CStr::from_ptr(SDL_GetError()).to_string_lossy().to_string() }
}

pub fn init(flags: SDL_InitFlags) -> Result<(), String> {
    ok_or_err(unsafe { SDL_Init(flags) })
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

impl Gfx {
    pub fn set_render_draw_color(&self, r: u8, g: u8, b: u8, a: u8) -> Result<(), String> {
        ok_or_err(unsafe { SDL_SetRenderDrawColor(self.renderer, r, g, b, a) })
    }

    pub fn render_clear(&self) -> Result<(), String> {
        ok_or_err(unsafe { SDL_RenderClear(self.renderer) })
    }

    pub fn render_fill_rect(&self, rect: &SDL_FRect) -> Result<(), String> {
        ok_or_err(unsafe { SDL_RenderFillRect(self.renderer, rect) })
    }

    pub fn render_line(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> Result<(), String> {
        ok_or_err(unsafe { SDL_RenderLine(self.renderer, x1, y1, x2, y2) })
    }
    pub fn render_lines(&self, lines: Vec<SDL_FPoint>) -> Result<(), String> {
        ok_or_err(unsafe { SDL_RenderLines(self.renderer, lines.as_ptr(), lines.len() as i32) })
    }

    pub fn render_present(&self) -> Result<(), String> {
        ok_or_err(unsafe { SDL_RenderPresent(self.renderer) })
    }

    pub fn set_render_vsync(&self, vsync: i32) -> Result<(), String> {
        ok_or_err(unsafe { SDL_SetRenderVSync(self.renderer, vsync) })
    }

    pub fn render_debug_text(&self, text: &str, x: f32, y: f32) -> Result<(), String> {
        ok_or_err(unsafe {
            SDL_RenderDebugText(
                self.renderer,
                x,
                y,
                CString::new(text)
                    .expect("debug text to be converted to CString")
                    .as_ptr(),
            )
        })
    }

    pub fn set_render_scale(&self, x_scale: f32, y_scale: f32) -> Result<(), String> {
        ok_or_err(unsafe { SDL_SetRenderScale(self.renderer, x_scale, y_scale) })
    }

    pub fn hide_window(&self) {
        // todo handle return value
        unsafe {
            SDL_HideWindow(self.window);
        }
    }
}

// unsafe {SDL_SetWindowOpacity(gfx.window, 0.5)};

pub struct AudioDevice {
    pub id: SDL_AudioDeviceID,
    pub name: String,
}

pub struct Sound {
    audio_spec: SDL_AudioSpec,
    data: Vec<i32>,
}

pub fn loadwav(path: &str) -> Result<Sound, String> {
    // will be overwritten by SDL_LoadWAV()
    let mut spec = SDL_AudioSpec {
        format: SDL_AUDIO_S32,
        channels: 1,
        freq: 44100,
    };

    let mut audio_buffer: *mut Uint8 = ptr::null_mut();
    let mut audio_len: u32 = 0;

    if unsafe {
        SDL_LoadWAV(
            CString::new(path)
                .expect("path to be converted to CString")
                .as_ptr(),
            &mut spec,
            &mut audio_buffer,
            &mut audio_len,
        )
    } {
        // todo this is hardcore assuming the wav will be encoded as i32 samples and will fail horribly if that's not the case
        let num_samples = audio_len as usize / 4;
        let mut data = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            data.push(unsafe { *(audio_buffer as *mut i32).wrapping_add(i) });
        }
        unsafe { SDL_free(audio_buffer.cast()) };

        Ok(Sound {
            audio_spec: spec,
            data,
        })
    } else {
        Err(get_error())
    }
}

// todo this level of abstraction does not belong here, also fix the safety
pub fn play_sound(sound: &Sound) {
    unsafe {
        let stream = SDL_OpenAudioDeviceStream(
            SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
            &sound.audio_spec,
            SDL_AudioStreamCallback::None,
            ptr::null_mut(),
        );
        SDL_PutAudioStreamData(
            stream,
            sound.data.as_ptr().cast(),
            (sound.data.len() * 4) as i32,
        );
        SDL_SetAudioStreamGain(stream, 0.2);
        SDL_ResumeAudioStreamDevice(stream);

        let seconds = sound.data.len() as f64 / 44100.0;

        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
    }
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
    ok_or_err(unsafe { SDL_BindAudioStream(id, stream) })
}

pub fn flush_audio_stream(stream: *mut SDL_AudioStream) -> Result<(), String> {
    ok_or_err(unsafe { SDL_FlushAudioStream(stream) })
}

// get all samples of pending audio
// todo enforce audio is in i32 format when calling this function
pub fn get_audio_stream_data_i32(stream: *mut SDL_AudioStream) -> Result<Vec<i32>, String> {
    let mut samples = Vec::with_capacity(1024);

    let mut sample_buffer = [0i32; 1024];
    let buffer_bytes = (sample_buffer.len() * 4) as i32;

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
            // clip audio to 24 bits by removing quietest 8 bits
            samples.push(sample_buffer[i] >> 8);
        }
    }

    Ok(samples)
}

#[allow(dead_code)]
pub enum Event {
    Common(SDL_CommonEvent),
    Display(SDL_DisplayEvent),
    Window(SDL_EventType, SDL_WindowEvent),
    KDevice(SDL_KeyboardDeviceEvent),
    Key(SDL_KeyboardEvent),
    Edit(SDL_TextEditingEvent),
    EditCandidates(SDL_TextEditingCandidatesEvent),
    Text(SDL_TextInputEvent),
    MDevice(SDL_MouseDeviceEvent),
    Motion(SDL_EventType, SDL_MouseMotionEvent),
    Button(SDL_EventType, SDL_MouseButtonEvent),
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
            SDL_EventType::WINDOW_RESIZED => {
                Some(Event::Window(SDL_EventType::WINDOW_RESIZED, unsafe {
                    event.window
                }))
            }
            SDL_EventType::MOUSE_MOTION => {
                Some(Event::Motion(SDL_EventType::MOUSE_MOTION, unsafe {
                    event.motion
                }))
            }
            SDL_EventType::MOUSE_BUTTON_DOWN => {
                Some(Event::Button(SDL_EventType::MOUSE_BUTTON_DOWN, unsafe {
                    event.button
                }))
            }
            SDL_EventType::MOUSE_BUTTON_UP => {
                Some(Event::Button(SDL_EventType::MOUSE_BUTTON_UP, unsafe {
                    event.button
                }))
            }
            _ => Some(Event::User(unsafe { event.user })), // dummy event so we can decern an unimplemented event (in this function) from NO event
        }
    } else {
        None
    }
}

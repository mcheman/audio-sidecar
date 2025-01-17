use sdl3_sys::everything::*;
use sdl3_sys::init::SDL_InitFlags;
use std::ffi::{CStr, CString};
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
    unsafe {SDL_Quit();}
}

pub fn create_window_and_renderer(
    title: &str,
    width: u32,
    height: u32,
    window_flags: SDL_WindowFlags,
) -> Result<Gfx, String> {
    let mut window: *mut SDL_Window = ptr::null_mut();
    let mut renderer: *mut SDL_Renderer = ptr::null_mut();

    if unsafe {
        SDL_CreateWindowAndRenderer(
            CString::new(title)
                .expect("window title to be converted to CString")
                .as_ptr(),
            width as i32,
            height as i32,
            window_flags,
            &raw mut window,
            &raw mut renderer,
        )
    } {
        Ok(Gfx { window, renderer })
    } else {
        Err(get_error())
    }
}

pub fn set_render_draw_color(gfx: &mut Gfx, color: SDL_Color) -> Result<(), String> {
    unsafe {
        if SDL_SetRenderDrawColor(gfx.renderer, color.r, color.g, color.b, color.a) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn render_clear(gfx: &mut Gfx) -> Result<(), String> {
    unsafe {
        if SDL_RenderClear(gfx.renderer) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}

pub fn render_fill_rect(gfx: &mut Gfx, rect: &SDL_FRect) -> Result<(), String> {
    unsafe {
        if SDL_RenderFillRect(gfx.renderer, rect) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}


pub fn render_present(gfx: &mut Gfx) -> Result<(), String> {
    unsafe {
        if SDL_RenderPresent(gfx.renderer) {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}


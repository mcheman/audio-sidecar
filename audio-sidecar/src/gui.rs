use sdl3_sys::everything::SDL_FRect;
use std::cmp::max;
use crate::sdl::Gfx;
use crate::utils::or_die;

// returns true if button is currently clicked
pub fn button(
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

pub fn draw_text(gfx: &Gfx, text: &str, x: f32, y: f32, size: f32, centered_x: bool, centered_y: bool) {
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

pub fn draw_waveform(gfx: &Gfx, waveform: &[u32], x: f32, y: f32, width: f32, height: f32) {
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

        // todo consider using render_lines if it's faster to submit one batch
        // todo or use render geometry since it supports separate colors?
        or_die(gfx.render_line(x + col as f32, y1, x + col as f32, y2));

        if is_clipped {
            or_die(gfx.set_render_draw_color(255, 255, 255, 255));
        }
    }
}
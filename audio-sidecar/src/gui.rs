use crate::sdl::Gfx;
use crate::utils::or_die;
use sdl3_sys::everything::SDL_FRect;
use std::cmp::max;

// contains the events that occurred this frame
pub struct UI {
    state: Input,
    prev_state: Input,
    gfx: Gfx,
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_button_pressed: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_button_pressed: false,
        }
    }
}

// todo for panels and nesting, consider an api similar to https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/panels.rs

impl UI {
    pub fn new(gfx: Gfx) -> Self {
        UI {
            state: Input::default(),
            prev_state: Input::default(),
            gfx,
        }
    }

    // take in new input data and change internal state
    pub fn apply_input(&mut self, input: &Input) {
        self.prev_state = self.state;
        self.state = *input;
    }

    pub fn clear(&self) {
        or_die(self.gfx.set_render_draw_color(53, 53, 53, 255));
        or_die(self.gfx.render_clear());
    }

    pub fn present(&self) {
        or_die(self.gfx.render_present());
    }

    pub fn hide(&self) {
        self.gfx.hide_window();
    }

    // return true the first moment the mouse clicks up, otherwise return false
    // this debounces clicking
    fn click_occurred(&self) -> bool {
        !self.state.mouse_button_pressed && self.prev_state.mouse_button_pressed
    }

    // returns true if button is currently clicked
    pub fn button(&self, text: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
        let mouse_colliding = self.state.mouse_x > x
            && self.state.mouse_x < x + width
            && self.state.mouse_y > y
            && self.state.mouse_y < y + height;

        if mouse_colliding && self.click_occurred() {
            or_die(self.gfx.set_render_draw_color(20, 20, 20, 255));
        } else if mouse_colliding {
            // hover state
            or_die(self.gfx.set_render_draw_color(80, 90, 90, 255));
        } else {
            or_die(self.gfx.set_render_draw_color(80, 80, 80, 255));
        }

        let rect = SDL_FRect {
            x,
            y,
            w: width,
            h: height,
        };

        or_die(self.gfx.render_fill_rect(&rect));

        // button text
        or_die(self.gfx.set_render_draw_color(255, 255, 255, 255));

        self.draw_text(text, x + width / 2.0, y + height / 2.0, 3.0, true, true);

        mouse_colliding && self.click_occurred()
    }

    pub fn draw_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        centered_x: bool,
        centered_y: bool,
    ) {
        or_die(self.gfx.set_render_scale(size, size));

        const GLYPH_SIZE: f32 = 8.0;

        let offset_x = if centered_x {
            // note that bitmapped 8x8 pixel font generally has 2 pixels of empty space on the right and 1 pixel of space on the bottom
            //   we subtract 1 pixel here to compensate. subtracting 0.5 pixels from Y results in mushed scaling so we don't do that
            (text.len() as f32 * GLYPH_SIZE) / 2.0 - 1.0
        } else {
            0.0
        };
        let offset_y = if centered_y { GLYPH_SIZE / 2.0 } else { 0.0 };

        or_die(
            self.gfx
                .render_debug_text(text, x / size - offset_x, y / size - offset_y),
        );

        or_die(self.gfx.set_render_scale(1.0, 1.0));
    }

    pub fn draw_waveform(&self, waveform: &[u32], x: f32, y: f32, width: f32, height: f32) {
        or_die(self.gfx.set_render_draw_color(43, 43, 43, 255));
        let rect = SDL_FRect {
            x,
            y,
            w: width,
            h: height,
        };
        or_die(self.gfx.render_fill_rect(&rect));

        or_die(self.gfx.set_render_draw_color(255, 255, 255, 255));

        // todo render the new audio to a buffer which can then be scrolled on the screen, rather than line rendering the whole waveform

        const MAX_AMPLITUDE: u32 = (i32::MAX >> 8) as u32; // 24bits
        let max_conversion_factor: f32 = height / MAX_AMPLITUDE as f32;
        let y_middle = y + height / 2.0;

        let chunks_to_render = width; // one chunk per pixel
        for (col, m) in waveform
            .iter()
            .skip(max(0, waveform.len() as i64 - chunks_to_render as i64) as usize)
            .enumerate()
        {
            // if clipped, draw as red
            if *m >= MAX_AMPLITUDE - 1 {
                or_die(self.gfx.set_render_draw_color(250, 43, 43, 255));
                or_die(
                    self.gfx
                        .render_line(x + col as f32, y, x + col as f32, y + height),
                );
                or_die(self.gfx.set_render_draw_color(255, 255, 255, 255));
            } else {
                let h = *m as f32 * max_conversion_factor;
                let y1 = y_middle - (h / 2.0);
                let y2 = y1 + h;
                or_die(self.gfx.render_line(x + col as f32, y1, x + col as f32, y2));
            }
        }
    }

    pub fn debug_view(&self, text: &str) {
        const GLYPH_SIZE: f32 = 8.0;
        const LINE_SIZE: f32 = GLYPH_SIZE + 4.0;

        let mut y = 0.0;

        for line in text.lines() {
            self.gfx.render_debug_text(line, 0.0, y).unwrap();
            y += LINE_SIZE;
        }
    }
}

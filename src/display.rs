use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const ON: u32 = 0x00FF_FFFF; // white 
const OFF: u32 = 0x0000_0000; // black 

pub struct Display {
    window: Window,
    buffer: [u32; WIDTH * HEIGHT],
    pub screen: [bool; WIDTH * HEIGHT],
}

impl Display {

    pub fn new() -> Self {
        let mut new_window = Window::new("CHIP-8", WIDTH, HEIGHT, WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
            }
        ).expect("failed to create window");

        new_window.set_target_fps(60);
        Self {
            window: new_window,
            buffer: [OFF; WIDTH * HEIGHT],
            screen: [false; WIDTH * HEIGHT],
        }
    }

    // takes a display of bools (or bits ?) and updates the buffer which is drawn to the screen
    pub fn draw(&mut self) {
        for (pixel, &on) in self.buffer.iter_mut().zip(self.screen.iter()) {
            *pixel = if on { ON } else { OFF };
        }

        self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT)
        .expect("failed to update buffer");
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(Key::Escape)
    }

    pub fn clear(&mut self) {
        self.screen = [false; WIDTH * HEIGHT];
    }
}

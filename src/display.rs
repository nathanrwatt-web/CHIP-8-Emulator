use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const ON: u32 = 0x00FF_FFFF; // white 
const OFF: u32 = 0x0000_0000; // black 

pub struct Display {
    window: Option<Window>,
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
            window: Some(new_window),
            buffer: [OFF; WIDTH * HEIGHT],
            screen: [false; WIDTH * HEIGHT],
        }
    }

    #[cfg(test)]
    pub fn headless() -> Self {
        Self {
            window: None,
            buffer: [OFF; WIDTH * HEIGHT],
            screen: [false; WIDTH * HEIGHT],
        }
    }

    pub fn draw(&mut self) {
        for (pixel, &on) in self.buffer.iter_mut().zip(self.screen.iter()) {
            *pixel = if on { ON } else { OFF };
        }

        if let Some(window) = &mut self.window {
            window.update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .expect("failed to update buffer");
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.as_ref().is_none_or(|w| !w.is_open() || w.is_key_down(Key::Escape))
    }

    pub fn clear(&mut self) {
        self.screen = [false; WIDTH * HEIGHT];
    }
    
    // returns true if pixel turned on, false if turned off
    pub fn flip_pixel(&mut self, x_coord: usize, y_coord: usize) -> bool {
        self.screen[ y_coord * WIDTH + x_coord ] =
            match self.screen[ y_coord * WIDTH + x_coord] {
            false => true,
            true => false,
        };

        match self.screen[ y_coord * WIDTH + x_coord] {
            false => false,
            true => true,
        }
    }

    pub fn is_key_down(&self, num: u8) -> bool {
        self.window.as_ref().is_some_and(|w| w.is_key_down(num_to_key(num)))
    }

    pub fn get_pressed_key(&self) -> Option<u8> {
        self.window.as_ref().and_then(key_to_num)
    }
}

// 1 2 3 C      1 2 3 4
// 4 5 6 D      Q W E R
// 7 8 9 E      A S D F
// A 0 B F      Z X C V
fn num_to_key (num: u8) -> minifb::Key {
    match num {
        0x0 => minifb::Key::Key1,
        0x1 => minifb::Key::Key2,
        0x2 => minifb::Key::Key3,
        0x3 => minifb::Key::Key4,
        0x4 => minifb::Key::Q,
        0x5 => minifb::Key::W,
        0x6 => minifb::Key::E, 
        0x7 => minifb::Key::R,
        0x8 => minifb::Key::A,
        0x9 => minifb::Key::S,
        0xA => minifb::Key::D,
        0xB => minifb::Key::F,
        0xC => minifb::Key::Z,
        0xD => minifb::Key::X,
        0xE => minifb::Key::C,
        0xF => minifb::Key::V,
        _ => minifb::Key::V,   // should be unreachable 
    }
}

fn key_to_num(window: &minifb::Window) -> Option<u8> {
    match window.get_keys().first() {
        Some(key) => match key {
            minifb::Key::Key1 => Some(0x0),
            minifb::Key::Key2 => Some(0x1),
            minifb::Key::Key3 => Some(0x2),
            minifb::Key::Key4 => Some(0x3),
            minifb::Key::Q    => Some(0x4),
            minifb::Key::W    => Some(0x5),
            minifb::Key::E    => Some(0x6),
            minifb::Key::R    => Some(0x7),
            minifb::Key::A    => Some(0x8),
            minifb::Key::S    => Some(0x9),
            minifb::Key::D    => Some(0xA),
            minifb::Key::F    => Some(0xB),
            minifb::Key::Z    => Some(0xC),
            minifb::Key::X    => Some(0xD),
            minifb::Key::C    => Some(0xE),
            minifb::Key::V    => Some(0xF),
            _                 => None,
        },
        None => None,
    }
}

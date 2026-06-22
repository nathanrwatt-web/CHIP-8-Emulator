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

    pub fn flip_pixel(&mut self, x_coord: usize, y_coord: usize) {
        // y-coords is read from top left corner down 
        self.screen[ y_coord * WIDTH + x_coord ] = 
            match self.screen[ y_coord * WIDTH + x_coord] {
            false => true,
            true => false,
        };
    }

    pub fn is_key_down(&self, num: u8) -> bool {
        self.window.is_key_down(num_to_key(num))
    }
}

// key helper function
// 0 1 2 3      1 2 3 4
// 4 5 6 7      Q W E R 
// 8 9 A B      A S D F 
// C D E F      Z X C V
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

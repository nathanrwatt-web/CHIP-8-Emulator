mod display;
mod op_code;

use display::Display;
use std::{fs::File, io::{self, Read}};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;


// basic idea // 

/* Inititialize memory 
 * Inititialize stack 
 * Read in the ROM 
 *
 * while display is up
 * execute instruction
 *      parse instruction 
 *      call function
 *      increment PC (or change)
 * update display 
 */ 


fn main() -> io::Result<()> {

    // registers
    let mut v0: u8 = 0;
    let mut v1: u8 = 0;
    let mut v2: u8 = 0;
    let mut v3: u8 = 0;

    let mut pc: u16 = 0;  // program counter 
    let mut i: u16 = 0;   // for memory accessing

    let mut stack: Vec<u16> = Vec::new();

    // ROM processing 
    let mut file_buffer = [0u8; 4096]; // 4kb memory
    let mut file = File::open("files/rom.test")?;
    file.read_exact(&mut file_buffer)?;


    let mut display: Display = Display::new();

    // test 
    for i in 0..HEIGHT.min(WIDTH) {
        display.screen[i * WIDTH + i] = true;
    }

    while !display.should_close() {
        display.draw();
    }

    Ok(())
}


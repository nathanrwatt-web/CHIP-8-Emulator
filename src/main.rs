mod display;

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

    let mut pc: usize = 0;  // program counter, 16 but but usize for rust compliance 
    let mut i: u16 = 0;   // for memory accessing

    let mut stack: Vec<usize> = Vec::new();

    // ROM processing 
    let mut memory = [0u8; 4096]; // 4kb memory
    let mut file = File::open("files/rom.test")?;
    file.read_exact(&mut memory)?;


    let mut display: Display = Display::new();

    // test 
    for i in 0..HEIGHT.min(WIDTH) {
        display.screen[i * WIDTH + i] = true;
    }


    while !&display.should_close() {
        // fetch
        // instructions are two bytes read as one
        let instruction: u16 = (memory[pc] << 8) as u16 + (memory[pc + 1]) as u16;

        // decode + execute 
        pc = execute_instruction(instruction, &mut display, pc, &mut stack);

        display.draw();
    }

    Ok(())
}

// === OP CODES ===
// Reference to the different codes
// registers will be referred to by X,Y ie Register X = VX
//
// there are 17 registers, a couple of which are special
// V0 - VE are 8 bit general use registers
// VF is a flag register 
// I is and index fregister for storing memory addresses 
// DT is a delay timer which automatically decrements at 60Hz when > 0 
// ST is a sound timer which also decrements but plays a sound
//
// PC is the program counter and it is 16 bits instead of 8
// SP is the stack pounter, 8 bit

// === Instructions ===
/* 0NNN: Jumps to subroutine at NNN, deprecated instruction
 *
 * 00E0: Clear screen
 *
 * 1NNN: Set PC to NNN
 * 2NNN: Calls subroutine at NNN (keeps track of previous location to return)
 * 00EE: Pops last address on the stack and jumps there 
 * 3XNN: skips one instruction if VX = NN
 * 4XNN: skips one instruction if VX != NN
 * 5XY0: skips one instruction if VX = VY
 * 9XY0: skips one instruction if VX != VY
 *
 * 6XNN: sets VX to NN
 *
 * 7XNN: adds NN to VX
 * 
 */

// smalled
struct Operation {
    head: u8,
    middle_1: u8,
    middle_2: u8,
    tail: u8,
 }

fn parse_op_code ( instruct: u16) -> Operation {
    Operation {
        head: (instruct >> 12) as u8,
        middle_1: ((instruct >> 8) & 0x000F) as u8,
        middle_2: ((instruct >> 4) & 0x000F) as u8 ,
        tail: (instruct & 0x000F) as u8,
    }
}

pub fn execute_instruction( instruct: u16, display: &mut Display, pc: usize, stack: &mut Vec<usize>) -> usize {

    let op: Operation = parse_op_code(instruct);

    match op.head {
        0x0 => { 
            // clear screen 0x00E0
            if op.tail == 0 && op.middle_2 == 0xE {
               clear_screen(display);
               return pc + 2;
            }
            // pop from the stack 0x00EE
            if op.tail == 0xE && op.middle_2 == 0xE {
                if let Some(value) = stack.pop() {
                    return value;
                }
            }
            pc + 2
             
        },
        0x1 => {
            ((op.middle_1 << 8) + (op.middle_2 << 4) + op.tail) as usize
        },
        0x2 => { 
            stack.push(pc + 2);
            ((op.middle_1 << 8) + (op.middle_2 << 4) + op.tail) as usize
        },
        0x3 => { pc + 2 },
        0x4 => { pc + 2 },
        0x5 => { pc + 2 },
        0x6 => { pc + 2 },
        0x7 => { pc + 2 },
        0x8 => { pc + 2 },
        0x9 => { pc + 2 },
        0xA => { pc + 2 },
        0xB => { pc + 2 },
        0xC => { pc + 2 },
        0xD => { pc + 2 },
        0xE => { pc + 2 },
        0xF => { pc + 2 },
        _ => { pc + 2}

    }
}


fn clear_screen( display: &mut Display) {
    display.clear();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let op: Operation = parse_op_code(0x1234);

        assert_eq!(op.head, 0x1);
        assert_eq!(op.middle_1, 0x2);
        assert_eq!(op.middle_2, 0x3);
        assert_eq!(op.tail, 0x4);
    }


}



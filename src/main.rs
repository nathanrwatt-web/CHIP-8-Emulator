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

    let mut cpu = CPU::new();
    let mut display: Display = Display::new();

    // test 
    for i in 0..HEIGHT.min(WIDTH) {
        display.screen[i * WIDTH + i] = true;
    }


    while !&display.should_close() {
        // fetch
        // instructions are two bytes read as one
        // let instruction: u16 = (memory[pc] << 8) as u16 + (memory[pc + 1]) as u16;

        // decode + execute 
        // pc = execute_instruction(instruction, &mut display, pc, &mut stack);

        // display.draw();
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

struct RegisterBank {
    pub v: [u8; 16],    // General purpose 8-bit 
    pub i: u16,         // index to memory 
    pub pc: u16,        // program counter 
    pub sp: u8,         // stack pointer 
    pub dt: u8,         // delay timer 
    pub st: u8,         // sound timer 
}

impl RegisterBank {
    pub fn new() -> Self {
        Self {
            v: [0u8; 16],
            i: 0,
            pc: 0,
            sp: 0,
            dt: 0,
            st: 0,
        }
    }
 }

struct CPU {
    stack: [u16; 16],   // holds 16 bit addresses for function calls etc.
    memory: [u8; 4096], // 4kb memory 
    register_bank: RegisterBank,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            stack: [0u16; 16],
            memory: [0u8; 4096], 
            /*
             * TODO 
             * Implement initialization of memory 
             * First ~512 bytes need to be sprites for font 
            */
            register_bank: RegisterBank::new(),
        }
    }

    pub fn fetch(&self) -> Operation {
        let position = self.register_bank.pc;
        let instruction = ((self.memory[position as usize] as u16) << 8) + self.memory[(position + 1) as usize] as u16;
        Operation {
            head: (instruction >> 12) as u8,
            middle_1: ((instruction >> 8) & 0x000F) as u8,
            middle_2: ((instruction >> 4) & 0x000F) as u8,
            tail: (instruction & 0x000F) as u8,
        }
    }

    pub fn execute_instruction(&mut self, op: Operation, display: &mut Display) {
        let registers: &mut RegisterBank = &mut self.register_bank;
        let stack = &mut self.stack;
        let memory = &mut self.memory;
        match op.head {
            0x0 => { 
                // clear screen 0x00E0
                if op.tail == 0 && op.middle_2 == 0xE {
                   Self::clear_screen(display);
                   registers.pc += 2;
                   return;
                }
                // pop from the stack 0x00EE
                if op.tail == 0xE && op.middle_2 == 0xE {
                    registers.pc = stack[registers.sp as usize];
                    registers.sp -= 1;
                }
                 
            },
            0x1 => {
                registers.pc = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
            },
            0x2 => { 
                // push 
                stack[(registers.sp + 2) as usize] = registers.pc;
                // update stack pointer 
                registers.sp += 2;
                // update pc 
                registers.pc = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
            },
            0x3 => { 
                // 3XNN if VX = NN skip instruction
                let vx = registers.v[op.middle_1 as usize];
                let nn = (op.middle_2 << 4) + op.tail;
                if vx == nn {
                    registers.pc += 4;
                } else {
                    registers.pc += 2;
                }
            },
            0x4 => {
                // 4XNN if VX != NN skip instruction
                let vx = registers.v[op.middle_1 as usize];
                let nn = (op.middle_2 << 4) + op.tail;
                if vx != nn {
                    registers.pc += 4;
                } else {
                    registers.pc += 2;
                }
            },
            0x5 => {
                // 0x5XY0 if VX == VY skip instruction 
                if op.tail == 0 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];

                    if vx == vy {
                        registers.pc += 4;
                    } else {
                        registers.pc += 2;
                    }
                }
            },
            0x6 => {
                // 0x6XNN let VX = NN
                let nn = (op.middle_2 << 4) + op.tail;
                registers.v[op.middle_1 as usize] = nn;
                registers.pc += 2;
            },
            0x7 => {
                //0x7XNN add NN to VX
                // important note: VF flag is not updated on overflow
                let nn = (op.middle_2 << 4) + op.tail;
                registers.v[op.middle_1 as usize] += nn;
                registers.pc += 2;
            },
            0x8 => {
                if op.tail == 0 {
                    // 0x8XY0 VX = VY
                    registers.v[op.middle_1 as usize] = registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 1 {
                    // 0x8XY1 VX bitwise OR VY
                    registers.v[op.middle_1 as usize] |= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 2 {
                    // 0x8XY2 VX bitwise AND VY
                    registers.v[op.middle_1 as usize] &= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 3 {
                    // 0x8XY3 VX bitwise XOR VY
                    registers.v[op.middle_1 as usize] ^= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 4{
                    // 0x8XY4 VX += VY
                    // if overflow, set VF to flag 
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vx.overflowing_add(vy);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 5 {
                    // 0x8XY5 VX = VX - VY
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vx.overflowing_sub(vy);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 6 {
                    // ambiguous TODO 
                } else if op.tail == 7 {
                    // 0x8XY7 VX = VY - VX
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vy.overflowing_sub(vx);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 0xE {
                    // ambiguous TODO 
                }
            },
            0x9 => {
                // 9XY0 if VX != VY skip instruction
                if op.tail == 0 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];

                    if vx == vy {
                        registers.pc += 4;
                    } else {
                        registers.pc += 2;
                    }
                }
            },
            0xA => {
                // 0xANNN sets I = NNN
                registers.i = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
            },
            0xB => {
                // ambiguous TODO 
            },
            0xC => {
                // 0xCXNN
                // VX = generates random number then ands it with NN
                // debating on using rand crate or not, could implement it using time dt or
                // something
            },
            0xD => {
                // 0xDXYN
                // Draws an N pixel tall sprite from memorh index I 
                // VX is x coord, VY is y coord. 
                // -- quote from Tvil -- 
                // All the pixels that are “on” in the sprite will flip the pixels
                // on the screen that it is drawn to (from left to right,
                // from most to least significant bit). If any pixels on the screen
                // were turned “off” by this, the VF flag register is set to 1.
                // Otherwise, it’s set to 0.
                let x_coord = registers.v[op.middle_1 as usize] % 64;
                let y_coord = registers.v[op.middle_2 as usize] % 32;
                let n = op.tail as usize;

                registers.v[0xF] = 0;

                for row in 0..n {
                    let sprite_row: u8 = memory[registers.i as usize + row];

                    for index in 0..8 {
                        let bit = ((sprite_row << (0x7 - index)) & 0x1) == 1;
                        if bit { display.flip_pixel(row, index) }
                    }
                    // for each bit in sprite_row, need to turn pixel on / off 
                    // stop if botton row is reached 
                    // TODO 
                }
            },
            0xE => {
                if op.middle_2 == 0x9 && op.tail == 0xE {
                    // 0xEX9E skip one instruction if key corresponding to VX value is pressed
                    let key_pressed = registers.v[op.middle_1 as usize];
                    // TODO 
                } else if op.middle_2 == 0xA && op.tail == 0x1 {
                    // 0xEXA1 skips if VX is not pressed
                    let key_not_pressed = registers.v[op.middle_1 as usize];
                    // TODO 
                }
            },
            0xF => {
                if op.middle_2 == 0x0 && op.tail == 0x7 {
                    // 0xFX07 sets VX to DT 
                    registers.v[op.middle_1 as usize] = registers.dt;
                } else if op.middle_2 == 0x0 && op.tail == 0xA {
                    // FX0A 
                    // TODO 
                    // Only increments PC if a certain key is pressed 
                } else if op.middle_2 == 0x1 && op.tail == 0x5 {
                    // 0xFX15 sets DT to VX
                    registers.dt = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0x8 {
                    // 0xFX18 sets ST to VX 
                    registers.st = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0xE {
                    // 0xFX1E register I += VX
                    registers.i += registers.v[op.middle_1 as usize] as u16;
                    // TODO implement overflow 
                } else if op.middle_2 == 0x2 && op.tail == 0x9 {
                    // 0xFX29 I = address of character in VX 
                    // TODO 
                } else if op.middle_2 == 0x3 && op.tail == 0x3 {
                    // 0xFX33 
                    // converts VX to 3 decimal digits, then stores these
                    // contingously beginning at I 
                    // TODO 
                } else if op.middle_2 == 0x5 && op.tail == 0x5 {
                    // TODO mem stuff
                } else if op.middle_2 == 0x6 && op.tail == 0x5 {
                    // TODO more mem stuff 
                }
            },
            _ => { }
        }
    }

    pub fn clear_screen( display: &mut Display) {
        display.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {

        let mut cpu = CPU::new();
        cpu.memory[0] = 0x12;
        cpu.memory[1] = 0x34;
        cpu.register_bank.pc = 0;

        let op = cpu.fetch();

        assert_eq!(op.head, 0x1);
        assert_eq!(op.middle_1, 0x2);
        assert_eq!(op.middle_2, 0x3);
        assert_eq!(op.tail, 0x4);
    }


}



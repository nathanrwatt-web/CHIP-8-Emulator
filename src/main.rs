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
            register_bank: RegisterBank::new(),
        }
    }

    pub fn fetch(&self) -> Operation {
        let position = self.register_bank.pc;
        let instruction = (self.memory[position as usize] << 8) + self.memory[(position + 1) as usize];
        Operation {
            head: (instruction >> 12),
            middle_1: ((instruction >> 8) & 0x000F),
            middle_2: ((instruction >> 4) & 0x000F) ,
            tail: (instruction & 0x000F),
        }
    }

    pub fn execute_instruction(&mut self, op: Operation, display: &mut Display) {
        let registers: &mut RegisterBank = &mut self.register_bank;
        let stack = &mut self.stack;
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

                    if 0xFF - vx < vy {
                        registers.v[0x0F] = 0x1;
                    } else {
                        registers.v[0x0F] = 0x0;
                    }

                    registers.v[op.middle_1 as usize] = vy;
                    registers.pc += 2;
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
            0xA => { },
            0xB => { },
            0xC => { },
            0xD => { },
            0xE => { },
            0xF => { },
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
        let op: Operation = parse_op_code(0x1234);

        assert_eq!(op.head, 0x1);
        assert_eq!(op.middle_1, 0x2);
        assert_eq!(op.middle_2, 0x3);
        assert_eq!(op.tail, 0x4);
    }


}



mod display;
mod font;

use display::Display;
use font::FONTS;
use std::{fs::File, io::{self, Read}};
use rand::Rng;


fn main() -> io::Result<()> {

    let mut cpu = CPU::new();
    let mut display: Display = Display::new();

    cpu.register_bank.pc = 0x200;

    let mut rom = Vec::new();
    File::open("files/test.rom")?.read_to_end(&mut rom)?;
    cpu.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom[..4096 - 0x200]);

    while !&display.should_close() {
        let fetched = cpu.fetch();
        cpu.execute_instruction(fetched, &mut display);
        display.draw();
        cpu.decrement_timers();
    }
    Ok(())
}

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

#[allow(clippy::upper_case_acronyms)]
struct CPU {
    stack: [u16; 16],   // holds 16 bit addresses for function calls etc.
    memory: [u8; 4096], // 4kb memory 
    register_bank: RegisterBank,
}

impl CPU {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        memory[..80].copy_from_slice(&FONTS);
        Self {
            stack: [0u16; 16],
            memory,
            register_bank: RegisterBank::new(),
        }
    }

    pub fn decrement_timers(&mut self) {
        self.register_bank.dt = self.register_bank.dt.saturating_sub(1);
        self.register_bank.st = self.register_bank.st.saturating_sub(1);
        // beep when st > 0: not implemented
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
                if op.tail == 0 && op.middle_2 == 0xE {
                   Self::clear_screen(display);
                   registers.pc += 2;
                   return;
                }
                if op.tail == 0xE && op.middle_2 == 0xE {
                    registers.pc = stack[registers.sp as usize];
                    registers.sp -= 1;
                }
                 
            },
            0x1 => {
                registers.pc = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
            },
            0x2 => {
                stack[(registers.sp + 2) as usize] = registers.pc;
                registers.sp += 2;
                registers.pc = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
            },
            0x3 => {
                let vx = registers.v[op.middle_1 as usize];
                let nn = (op.middle_2 << 4) + op.tail;
                if vx == nn {
                    registers.pc += 4;
                } else {
                    registers.pc += 2;
                }
            },
            0x4 => {
                let vx = registers.v[op.middle_1 as usize];
                let nn = (op.middle_2 << 4) + op.tail;
                if vx != nn {
                    registers.pc += 4;
                } else {
                    registers.pc += 2;
                }
            },
            0x5 => {
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
                let nn = (op.middle_2 << 4) + op.tail;
                registers.v[op.middle_1 as usize] = nn;
                registers.pc += 2;
            },
            0x7 => {
                // VF is not updated on overflow
                let nn = (op.middle_2 << 4) + op.tail;
                registers.v[op.middle_1 as usize] += nn;
                registers.pc += 2;
            },
            0x8 => {
                if op.tail == 0 {
                    registers.v[op.middle_1 as usize] = registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 1 {
                    registers.v[op.middle_1 as usize] |= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 2 {
                    registers.v[op.middle_1 as usize] &= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 3 {
                    registers.v[op.middle_1 as usize] ^= registers.v[op.middle_2 as usize];
                    registers.pc += 2;
                } else if op.tail == 4 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vx.overflowing_add(vy);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 5 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vx.overflowing_sub(vy);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 6 {
                    // ambiguous, choosing 0x8XY6 := 
                    // VX = VY >> 1
                    registers.v[0x0F] = registers.v[op.middle_2 as usize] & 0x1; // VF = shift bit
                    registers.v[op.middle_1 as usize] = registers.v[op.middle_2 as usize] >> 1;
                    registers.pc += 2;
                } else if op.tail == 7 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    let (result, carry) = vy.overflowing_sub(vx);
                    registers.v[op.middle_1 as usize] = result;
                    registers.v[0x0F] = carry as u8;
                    registers.pc += 2;
                } else if op.tail == 0xE {
                    // ambiguous, choosing 0x8XYE := 
                    // VX = VY << 1 
                    registers.v[0x0F] = registers.v[op.middle_2 as usize] >> 7; // VF = shift bit 
                    registers.v[op.middle_1 as usize] = registers.v[op.middle_2 as usize] << 1;
                    registers.pc += 2;
                }
            },
            0x9 => {
                if op.tail == 0 {
                    let vx = registers.v[op.middle_1 as usize];
                    let vy = registers.v[op.middle_2 as usize];
                    if vx != vy {
                        registers.pc += 4;
                    } else {
                        registers.pc += 2;
                    }
                }
            },
            0xA => {
                registers.i = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
                registers.pc += 2;
            },
            0xB => {
                // ambiguous: choosing pc = V0 + NNN
                let nnn: u16 = ((op.middle_1 as u16) << 8) + ((op.middle_2 as u16) << 4) + op.tail as u16;
                registers.pc = registers.v[0x0] as u16 + nnn;
            },
            0xC => {
                let mut rng = rand::rng();
                let rand: u32 = rng.next_u32(); 
                let nn: u32 = ((op.middle_2 as u32) << 4) + op.tail as u32;
                registers.v[op.middle_1 as usize] = (rand & nn) as u8;
            },
            0xD => {
                let x_coord = (registers.v[op.middle_1 as usize] % 64) as usize;
                let y_coord = (registers.v[op.middle_2 as usize] % 32) as usize;
                let n = op.tail as usize;

                registers.v[0xF] = 0;

                for row in 0..n {
                    let sprite_row: u8 = memory[registers.i as usize + row];

                    for col in 0..8 {
                        let bit = ((sprite_row >> (0x7 - col)) & 0x1) == 1;
                        if bit {
                            let on = display.flip_pixel(x_coord + col, y_coord + row);
                            if !on { registers.v[0x0F] = 1; }
                        }
                    }
                }
            },
            0xE => {
                if op.middle_2 == 0x9 && op.tail == 0xE {
                    let key = registers.v[op.middle_1 as usize];
                    if display.is_key_down(key) { registers.pc += 4; }
                    else { registers.pc += 2; }
                } else if op.middle_2 == 0xA && op.tail == 0x1 {
                    let key = registers.v[op.middle_1 as usize];
                    if !display.is_key_down(key) { registers.pc += 4; }
                    else { registers.pc += 2; }
                }
            },
            0xF => {
                if op.middle_2 == 0x0 && op.tail == 0x7 {
                    registers.v[op.middle_1 as usize] = registers.dt;
                } else if op.middle_2 == 0x0 && op.tail == 0xA {
                    if let Some(key_num) = display.get_pressed_key() {
                        registers.v[op.middle_1 as usize] = key_num;
                        registers.pc += 2;
                    }
                } else if op.middle_2 == 0x1 && op.tail == 0x5 {
                    registers.dt = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0x8 {
                    registers.st = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0xE {
                    // TODO: overflow behavior is ambiguous
                    registers.i += registers.v[op.middle_1 as usize] as u16;
                } else if op.middle_2 == 0x2 && op.tail == 0x9 {
                    registers.i = registers.v[op.middle_1 as usize] as u16 * 5;
                } else if op.middle_2 == 0x3 && op.tail == 0x3 {
                    let temp = registers.v[op.middle_1 as usize];
                    let ones = temp % 10;
                    let tens = (temp / 10) % 10;
                    let hundreds = temp / 100;
                    let index = registers.i as usize;
                    memory[index] = hundreds;
                    memory[index + 1] = tens;
                    memory[index + 2] = ones;
                } else if op.middle_2 == 0x5 && op.tail == 0x5 {
                    let x = op.middle_1 as usize;
                    let i = registers.i as usize;
                    memory[i..=i + x].copy_from_slice(&registers.v[..=x]);
                } else if op.middle_2 == 0x6 && op.tail == 0x5 {
                    let x = op.middle_1 as usize;
                    let i = registers.i as usize;
                    registers.v[..=x].copy_from_slice(&memory[i..=i + x]);
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

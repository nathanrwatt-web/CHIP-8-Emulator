mod display;
mod font;

use display::Display;
use font::FONTS;
use clap::Parser;
use std::{fs::File, io::{self, Read}, path::PathBuf, thread};
use std::time::{Duration, Instant};
use rand::Rng;


#[derive(Parser)]
#[command(about = "CHIP-8 emulator")]
struct Cli { rom: PathBuf, } // path to < 4096 - 0x200 byte ROM 

const INTRUCTIONS_PER_FRAME: u32 = 10;
const FRAME_DURATION: Duration = Duration::from_nanos(16666666); // ~60 Hz
                                                                 //
fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let path = &cli.rom;

    let is_rom = path.extension().and_then(|e| e.to_str()) == Some("rom");
    let size_ok = std::fs::metadata(path).is_ok_and(|m| m.len() <= 3584);

    if !is_rom || !size_ok {
        eprintln!("Invalid file type or size: filepath must be to a .rom of size < 3.584 kb");
        std::process::exit(1);
    }

    let mut cpu = CPU::new();
    let mut display: Display = Display::new();

    cpu.register_bank.pc = 0x200;

    let mut rom = Vec::new();
    File::open(path)?.read_to_end(&mut rom)?;
    cpu.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom[..]);

    while !&display.should_close() {
        let frame_start = Instant::now();

        for _ in 0..INTRUCTIONS_PER_FRAME {
            let fetched = cpu.fetch();
            cpu.execute_instruction(fetched, &mut display);
        }

        cpu.decrement_timers();
        display.draw();

        let elapsed_time = frame_start.elapsed();
        if let Some(remaining) = FRAME_DURATION.checked_sub(elapsed_time) {
            thread::sleep(remaining);
        }

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
                   display.clear();
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
                stack[(registers.sp + 1) as usize] = registers.pc + 2;
                registers.sp += 1;
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
                registers.v[op.middle_1 as usize] = registers.v[op.middle_1 as usize].wrapping_add(nn);
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
                    registers.v[0x0F] = !carry as u8;
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
                    registers.v[0x0F] = !carry as u8;
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
                registers.pc += 2;
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
                registers.pc += 2;
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
                    } else { return; }
                } else if op.middle_2 == 0x1 && op.tail == 0x5 {
                    registers.dt = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0x8 {
                    registers.st = registers.v[op.middle_1 as usize];
                } else if op.middle_2 == 0x1 && op.tail == 0xE {
                    registers.i = registers.i.wrapping_add(registers.v[op.middle_1 as usize] as u16);
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
                registers.pc += 2;
            },
            _ => { }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn run(cpu: &mut CPU, high: u8, low: u8) {
        cpu.memory[cpu.register_bank.pc as usize] = high;
        cpu.memory[cpu.register_bank.pc as usize + 1] = low;
        let op = cpu.fetch();
        let mut display = Display::headless();
        cpu.execute_instruction(op, &mut display);
    }

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

    #[test]
    fn test_cls() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        let mut display = Display::headless();
        display.screen[5] = true;
        cpu.memory[0x200] = 0x00;
        cpu.memory[0x201] = 0xE0;
        let op = cpu.fetch();
        cpu.execute_instruction(op, &mut display);

        assert!(!display.screen[5]);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_ret() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.sp = 1;
        cpu.stack[1] = 0x345;
        run(&mut cpu, 0x00, 0xEE);

        assert_eq!(cpu.register_bank.pc, 0x345);
        assert_eq!(cpu.register_bank.sp, 0);
    }

    #[test]
    fn test_jump() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0x12, 0x46);

        assert_eq!(cpu.register_bank.pc, 0x246);
    }

    #[test]
    fn test_call() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0x23, 0x00);

        assert_eq!(cpu.register_bank.sp, 1);
        assert_eq!(cpu.stack[1], 0x202);
        assert_eq!(cpu.register_bank.pc, 0x300);
    }

    #[test]
    fn test_skip_eq_byte() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x42;
        run(&mut cpu, 0x31, 0x42);

        assert_eq!(cpu.register_bank.pc, 0x204);
    }

    #[test]
    fn test_skip_eq_byte_no_skip() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x41;
        run(&mut cpu, 0x31, 0x42);

        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_skip_neq_byte() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x41;
        run(&mut cpu, 0x41, 0x42);

        assert_eq!(cpu.register_bank.pc, 0x204);
    }

    #[test]
    fn test_skip_eq_reg() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x7;
        cpu.register_bank.v[2] = 0x7;
        run(&mut cpu, 0x51, 0x20);

        assert_eq!(cpu.register_bank.pc, 0x204);
    }

    #[test]
    fn test_set_byte() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0x6A, 0x33);

        assert_eq!(cpu.register_bank.v[0xA], 0x33);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_add_byte_wrapping() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0xFF;
        run(&mut cpu, 0x71, 0x02);

        assert_eq!(cpu.register_bank.v[1], 0x01);
        assert_eq!(cpu.register_bank.v[0xF], 0);
    }

    #[test]
    fn test_set_reg() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[2] = 0x55;
        run(&mut cpu, 0x81, 0x20);

        assert_eq!(cpu.register_bank.v[1], 0x55);
    }

    #[test]
    fn test_or() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x0F;
        cpu.register_bank.v[2] = 0xF0;
        run(&mut cpu, 0x81, 0x21);

        assert_eq!(cpu.register_bank.v[1], 0xFF);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x0F;
        cpu.register_bank.v[2] = 0xFC;
        run(&mut cpu, 0x81, 0x22);

        assert_eq!(cpu.register_bank.v[1], 0x0C);
    }

    #[test]
    fn test_xor() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0xFF;
        cpu.register_bank.v[2] = 0x0F;
        run(&mut cpu, 0x81, 0x23);

        assert_eq!(cpu.register_bank.v[1], 0xF0);
    }

    #[test]
    fn test_add_reg_carry() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0xFF;
        cpu.register_bank.v[2] = 0x02;
        run(&mut cpu, 0x81, 0x24);

        assert_eq!(cpu.register_bank.v[1], 0x01);
        assert_eq!(cpu.register_bank.v[0xF], 1);
    }

    #[test]
    fn test_sub_reg_borrow() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x01;
        cpu.register_bank.v[2] = 0x02;
        run(&mut cpu, 0x81, 0x25);

        assert_eq!(cpu.register_bank.v[1], 0xFF);
        assert_eq!(cpu.register_bank.v[0xF], 0);
    }

    #[test]
    fn test_shr() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[2] = 0x05;
        run(&mut cpu, 0x81, 0x26);

        assert_eq!(cpu.register_bank.v[1], 0x02);
        assert_eq!(cpu.register_bank.v[0xF], 1);
    }

    #[test]
    fn test_subn() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x02;
        cpu.register_bank.v[2] = 0x05;
        run(&mut cpu, 0x81, 0x27);

        assert_eq!(cpu.register_bank.v[1], 0x03);
        assert_eq!(cpu.register_bank.v[0xF], 1);
    }

    #[test]
    fn test_shl() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[2] = 0x81;
        run(&mut cpu, 0x81, 0x2E);

        assert_eq!(cpu.register_bank.v[1], 0x02);
        assert_eq!(cpu.register_bank.v[0xF], 1);
    }

    #[test]
    fn test_skip_neq_reg() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x1;
        cpu.register_bank.v[2] = 0x2;
        run(&mut cpu, 0x91, 0x20);

        assert_eq!(cpu.register_bank.pc, 0x204);
    }

    #[test]
    fn test_set_index() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0xA2, 0x46);

        assert_eq!(cpu.register_bank.i, 0x246);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_jump_offset() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[0] = 0x10;
        run(&mut cpu, 0xB2, 0x00);

        assert_eq!(cpu.register_bank.pc, 0x210);
    }

    #[test]
    fn test_random_masked_zero() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0xC1, 0x00);

        assert_eq!(cpu.register_bank.v[1], 0x00);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_draw() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.i = 0x300;
        cpu.memory[0x300] = 0x80;
        cpu.register_bank.v[1] = 0;
        cpu.register_bank.v[2] = 0;
        let mut display = Display::headless();
        cpu.memory[0x200] = 0xD1;
        cpu.memory[0x201] = 0x21;
        let op = cpu.fetch();
        cpu.execute_instruction(op, &mut display);

        assert!(display.screen[0]);
        assert_eq!(cpu.register_bank.v[0xF], 0);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_skip_key_pressed() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x5;
        run(&mut cpu, 0xE1, 0x9E);

        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_skip_key_not_pressed() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x5;
        run(&mut cpu, 0xE1, 0xA1);

        assert_eq!(cpu.register_bank.pc, 0x204);
    }

    #[test]
    fn test_read_delay_timer() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.dt = 0x20;
        run(&mut cpu, 0xF1, 0x07);

        assert_eq!(cpu.register_bank.v[1], 0x20);
        assert_eq!(cpu.register_bank.pc, 0x202);
    }

    #[test]
    fn test_wait_key_no_press() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        run(&mut cpu, 0xF1, 0x0A);

        assert_eq!(cpu.register_bank.pc, 0x200);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x30;
        run(&mut cpu, 0xF1, 0x15);

        assert_eq!(cpu.register_bank.dt, 0x30);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x40;
        run(&mut cpu, 0xF1, 0x18);

        assert_eq!(cpu.register_bank.st, 0x40);
    }

    #[test]
    fn test_add_index() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.i = 0x10;
        cpu.register_bank.v[1] = 0x05;
        run(&mut cpu, 0xF1, 0x1E);

        assert_eq!(cpu.register_bank.i, 0x15);
    }

    #[test]
    fn test_font_address() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.v[1] = 0x3;
        run(&mut cpu, 0xF1, 0x29);

        assert_eq!(cpu.register_bank.i, 0x0F);
    }

    #[test]
    fn test_bcd() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.i = 0x300;
        cpu.register_bank.v[1] = 156;
        run(&mut cpu, 0xF1, 0x33);

        assert_eq!(cpu.memory[0x300], 1);
        assert_eq!(cpu.memory[0x301], 5);
        assert_eq!(cpu.memory[0x302], 6);
    }

    #[test]
    fn test_store_registers() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.i = 0x300;
        cpu.register_bank.v[0] = 0xA;
        cpu.register_bank.v[1] = 0xB;
        cpu.register_bank.v[2] = 0xC;
        run(&mut cpu, 0xF2, 0x55);

        assert_eq!(cpu.memory[0x300], 0xA);
        assert_eq!(cpu.memory[0x301], 0xB);
        assert_eq!(cpu.memory[0x302], 0xC);
    }

    #[test]
    fn test_load_registers() {
        let mut cpu = CPU::new();
        cpu.register_bank.pc = 0x200;
        cpu.register_bank.i = 0x300;
        cpu.memory[0x300] = 0xA;
        cpu.memory[0x301] = 0xB;
        cpu.memory[0x302] = 0xC;
        run(&mut cpu, 0xF2, 0x65);

        assert_eq!(cpu.register_bank.v[0], 0xA);
        assert_eq!(cpu.register_bank.v[1], 0xB);
        assert_eq!(cpu.register_bank.v[2], 0xC);
    }
}

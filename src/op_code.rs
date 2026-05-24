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
struct operation {
    head: u8,
    middle_1: u8,
    middle_2: u8,
    tail: u8,
 }

fn parse_op_code ( instruct: u16) -> operation {
    operation {
        head: (instruct >> 12) as u8,
        middle_1: ((instruct >> 8) & 0x000F) as u8,
        middle_2: ((instruct >> 4) & 0x000F) as u8 ,
        tail: (instruct & 0x000F) as u8,
    }
}

pub fn execute_instruction( instruct: u16 ) {

    let op: operation = parse_op_code(instruct);

    match op.head {
        0x0 => { 
            if (op.tail == 0 && op.middle_2 == 0xE) {
                clear_screen();
            }


        },
        0x1 => {},
        0x2 => {},
        0x3 => {},
        0x4 => {},
        0x5 => {},
        0x6 => {},
        0x7 => {},
        0x8 => {},
        0x9 => {},
        0xA => {},
        0xB => {},
        0xC => {},
        0xD => {},
        0xE => {},
        0xF => {},
        _ => {}

    }
}


fn clear_screen() {

}






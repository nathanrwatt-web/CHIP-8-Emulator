This is (yet another) CHIP-8 emulator written in rust. This is a learning project meant to strengthen my 
knowledge of the fetch-decode-execute loop. This immplentation follows Tobias V. I. Laghoff's blog 
which can be found at https://tobiasvl.github.io/blog/write-a-chip-8-emulator. 


Specs: 
4kb memory
64 x 32 pixel display scaled up by ~16x 
A stack (for now is of variable size)
All original registers (V0 - VR, I, PC, DT, ST)


Implentation: 

=== Hardware ===
CPU is a struct which owns all hardware implementation.
CPU contains three things: 
    1. stack: an array of u16s of length 16. This is strictly for function calls and return values
    2. memory: an array of u8s of length 4096. The CPU::new() intitializes the first 80 bytes of this 
            with font sprites, and in general only bytes past 0x1FF are used for the program. This 
            is to mimick the fact that in the original CHIP-8 the first 512 bytes of memory 
            were reserved for the CHIP-8 interpreter. 
    3. register_bank: This contains 17 registers:
            an array of 16 registers V0-VF which are the general purpose 8 bit
            I: the index register is used to hold addresses in memory and as such is 16bit 
            pc: the program counter 
            sp: The 8-bit stack pointer 
            dt: the delay timer which is decremented every cycle 
            st: the sound timer which is decremented every cycle and beeps when it is zero.
                beeping is not implemented here. 

=== Display === 
The display uses the minifb crate as the frame buffer for rendering pixels (on / off only). 
Display also owns the logic for key press: 
    - get_pressed_key() -> inputs current key being pressed and outputs Option<key_num: u8>
    - is_key_down() -> inputs hex value for key and checks if it is pressed, returning bool 

=== Op-Codes ===
00E0: Clear the screen
00EE: Return from subroutine (pop address from stack and jump there)
1NNN: Jump to NNN
2NNN: Call subroutine at NNN (push current address to stack)
3XNN: Skip next instruction if VX == NN
4XNN: Skip next instruction if VX != NN
5XY0: Skip next instruction if VX == VY
6XNN: Set VX to NN
7XNN: Add NN to VX (no overflow flag)
8XY0: Set VX to VY
8XY1: Set VX to VX | VY
8XY2: Set VX to VX & VY
8XY3: Set VX to VX ^ VY
8XY4: Set VX to VX + VY, VF = carry
8XY5: Set VX to VX - VY, VF = borrow
8XY6: Set VX to VY >> 1, VF = shifted out bit (ambiguous)
8XY7: Set VX to VY - VX, VF = borrow
8XYE: Set VX to VY << 1, VF = shifted out bit (ambiguous)
9XY0: Skip next instruction if VX != VY
ANNN: Set I to NNN
BNNN: Jump to V0 + NNN (ambiguous)
CXNN: Set VX to random byte & NN
DXYN: Draw N-tall sprite from memory[I] at (VX, VY), VF = pixel collision
EX9E: Skip next instruction if key VX is pressed
EXA1: Skip next instruction if key VX is not pressed
FX07: Set VX to DT
FX0A: Block until a key is pressed, store key in VX
FX15: Set DT to VX
FX18: Set ST to VX
FX1E: Add VX to I
FX29: Set I to address of font sprite for digit VX
FX33: Store BCD of VX in memory[I], memory[I+1], memory[I+2]
FX55: Store V0-VX in memory starting at I
FX65: Load V0-VX from memory starting at I

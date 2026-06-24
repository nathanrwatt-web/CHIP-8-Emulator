# CHIP-8 Emulator

A CHIP-8 emulator written in Rust. A learning project meant to strengthen my knowledge of the fetch-decode-execute loop. This implementation follows [Tobias V. I. Langhoff's blog](https://tobiasvl.github.io/blog/write-a-chip-8-emulator).

---

## Specs

| | |
|---|---|
| Memory | 4kb |
| Display | 64 x 32 pixels scaled up by ~16x |
| Stack | Variable size |
| Registers | All original (`V0`-`VF`, `I`, `PC`, `DT`, `ST`) |

---

## Implementation

### Hardware

`CPU` is a struct which owns all hardware implementation. It contains three things:

1. **`stack`** — an array of `u16`s of length 16, strictly for function calls and return values
2. **`memory`** — an array of `u8`s of length 4096. `CPU::new()` initializes the first 80 bytes with font sprites. Only bytes past `0x1FF` are used for the program, mimicking the fact that in the original CHIP-8 the first 512 bytes were reserved for the interpreter.
3. **`register_bank`** — contains 17 registers:

| Register | Description |
|----------|-------------|
| `V0`-`VF` | 16 general purpose 8-bit registers |
| `I` | Index register for holding memory addresses (16-bit) |
| `PC` | Program counter |
| `SP` | 8-bit stack pointer |
| `DT` | Delay timer, decremented every cycle |
| `ST` | Sound timer, decremented every cycle, beeps when non-zero *(not implemented)* |

### Display

The display uses the `minifb` crate as the frame buffer for rendering pixels (on/off only). `Display` also owns the key press logic:

| Method | Description |
|--------|-------------|
| `get_pressed_key()` | Returns `Option<u8>` for the current key being pressed |
| `is_key_down(u8)` | Takes a hex key value, returns `bool` |

### Key Mapping

```
CHIP-8    Keyboard
-------   --------
1 2 3 C   1 2 3 4
4 5 6 D   Q W E R
7 8 9 E   A S D F
A 0 B F   Z X C V
```

---

## Op-Codes

| Opcode | Description |
|--------|-------------|
| `00E0` | Clear the screen |
| `00EE` | Return from subroutine (pop address from stack and jump there) |
| `1NNN` | Jump to `NNN` |
| `2NNN` | Call subroutine at `NNN` (push current address to stack) |
| `3XNN` | Skip next instruction if `VX == NN` |
| `4XNN` | Skip next instruction if `VX != NN` |
| `5XY0` | Skip next instruction if `VX == VY` |
| `6XNN` | Set `VX` to `NN` |
| `7XNN` | Add `NN` to `VX` (no overflow flag) |
| `8XY0` | Set `VX` to `VY` |
| `8XY1` | Set `VX` to `VX \| VY` |
| `8XY2` | Set `VX` to `VX & VY` |
| `8XY3` | Set `VX` to `VX ^ VY` |
| `8XY4` | Set `VX` to `VX + VY`, `VF` = carry |
| `8XY5` | Set `VX` to `VX - VY`, `VF` = borrow |
| `8XY6` | Set `VX` to `VY >> 1`, `VF` = shifted out bit *(ambiguous)* |
| `8XY7` | Set `VX` to `VY - VX`, `VF` = borrow |
| `8XYE` | Set `VX` to `VY << 1`, `VF` = shifted out bit *(ambiguous)* |
| `9XY0` | Skip next instruction if `VX != VY` |
| `ANNN` | Set `I` to `NNN` |
| `BNNN` | Jump to `V0 + NNN` *(ambiguous)* |
| `CXNN` | Set `VX` to random byte `& NN` |
| `DXYN` | Draw `N`-tall sprite from `memory[I]` at `(VX, VY)`, `VF` = pixel collision |
| `EX9E` | Skip next instruction if key `VX` is pressed |
| `EXA1` | Skip next instruction if key `VX` is not pressed |
| `FX07` | Set `VX` to `DT` |
| `FX0A` | Block until a key is pressed, store key in `VX` |
| `FX15` | Set `DT` to `VX` |
| `FX18` | Set `ST` to `VX` |
| `FX1E` | Add `VX` to `I` |
| `FX29` | Set `I` to address of font sprite for digit `VX` |
| `FX33` | Store BCD of `VX` in `memory[I]`, `memory[I+1]`, `memory[I+2]` |
| `FX55` | Store `V0`-`VX` in memory starting at `I` |
| `FX65` | Load `V0`-`VX` from memory starting at `I` |

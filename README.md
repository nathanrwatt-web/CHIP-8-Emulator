This is (yet another) CHIP-8 emulator written in rust. My primary goal is to learn and as such will not use AI to code. 


Specs: 
4kb memory
64 x 32 pixel display scaled up by ~16x 
A stack (for now is of variable size)
All original registers (V0 - VR, I, PC, DT, ST)


Implentation: 

=== Display === 
The display uses the minifb crate as the frame buffer for rendering pixels (on / off only). 

=== Op-Codes ===


=== Memory & Stack ===



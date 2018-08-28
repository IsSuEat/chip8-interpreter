use std::io::{Bytes, Write};


pub struct Cpu {
    opcode : u16,
    mem : [u8; 4096],
    v : [u8; 16],
    i : i16,
    pc : i16,
    gfx : [u8; 64*32],
    stack : [u16; 16],
    stack_pointer : u16,
    key : [u8; 16],
    // timers
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            opcode: 0,
            mem: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [0; 64*32],
            stack: [0; 16],
            stack_pointer: 0,
            key: [0; 16],
        }
    }

    pub fn load_bytes(&mut self, bytes: &Bytes<u8>) {
        //bytes.clone_into(self.mem[512..])
    }
}

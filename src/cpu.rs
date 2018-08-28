use std::io::Bytes;
use std::fs::File;


const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    opcode: u16,
    mem: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    gfx: [u8; 64 * 32],
    stack: [u16; 16],
    stack_pointer: u16,
    key: [u8; 16],
    // timers
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            opcode: 0,
            mem: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            stack: [0; 16],
            stack_pointer: 0,
            key: [0; 16],
        };

        cpu.mem[0..80].clone_from_slice(&FONTSET);
        return cpu;
    }


    pub fn load_bytes(&mut self, bytes: Bytes<File>) {
        // load the file into memory, starting at 0x200 ending at 0xFFF
        for (i, byte) in bytes.enumerate() {
            self.mem[i + 512] = byte.unwrap()
        }
        println!("Loaded into memory");
        self.execute_opcode();
    }

    pub fn execute_opcode(&mut self) {
        // opcode is 2 bytes
        self.opcode = self.mem[self.pc as usize] as u16 | self.mem[self.pc as usize + 1] as u16;
        match self.opcode & 0xF000 {
            0xA000 => self.ANNN(),
            _ => println!("Unknown opcode {}", self.opcode)
        }
    }

    /// Sets I to the address NNN.
    pub fn ANNN(&mut self) {
        self.i = self.opcode & 0x0FF;
        self.pc += 2;
    }
}

use cpu::Cpu;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

pub struct Chip8 {
    cpu : Cpu
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu : Cpu::new()
        }
    }
    pub fn load_rom(&self, path: &Path) {
        let mut file = File::open(path).unwrap();
        for byte in file.bytes() {
            // set memory to read bytes

        }
        print!("Loading rom from path: {:?}", file)
    }
}
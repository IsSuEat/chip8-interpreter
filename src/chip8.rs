use cpu::Cpu;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

pub struct Chip8 {
    cpu: Cpu
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu: Cpu::new()
        }
    }
    pub fn load_rom(&mut self, path: &Path) {
        let file = File::open(path).unwrap();
        println!("Loading rom from path: {:?}", file);

        let bytes = file.bytes();

        self.cpu.load_bytes(bytes);
    }
}
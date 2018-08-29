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
            cpu: Cpu::default().init()
        }

    }
    pub fn load_rom(&mut self, path: &Path) {
        let file = File::open(path).unwrap();
        println!("Loading rom from path: {:?}", file);

        let bytes = file.bytes();

        self.cpu.load_bytes(bytes);
    }
}

#[cfg(test)]
mod tests {
    use chip8::Chip8;
    use std::path::Path;
    #[test]
    fn test_load_rom() {
        let mut c8 = Chip8::new();
        c8.load_rom(Path::new(&String::from("roms/BC_test.ch8")));
        c8.cpu.cycle();
        c8.cpu.cycle();
        
    }
}
use cpu::Cpu;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Chip8 {
    cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu: Cpu::default().init(),
        }
    }
    pub fn load_rom(&mut self, path: &String) {
        let file = File::open(path).unwrap();
        let bytes = file.bytes();

        info!("Read rom from: {}", path);
        self.cpu.load_bytes(bytes);
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.cycle();
        }
    }
}

#[cfg(test)]
mod tests {
    use chip8::Chip8;
    use std::path::Path;
    #[test]
    fn test_load_rom() {
        let mut c8 = Chip8::new();
        c8.load_rom(&String::from("roms/BC_test.ch8"));
        c8.run();
    }
}

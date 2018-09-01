use cpu::Cpu;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use std::time::Duration;

use piston_window::*;

pub struct Chip8 {
    cpu: Cpu,
    window: PistonWindow,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu: Cpu::default().init(),
            window: WindowSettings::new("Hello Piston!", [640, 480])
                .exit_on_esc(true)
                .build()
                .unwrap(),
        }
    }
    pub fn load_rom(&mut self, path: &String) {
        let file = File::open(path).unwrap();
        let bytes = file.bytes();

        info!("Read rom from: {}", path);
        self.cpu.load_bytes(bytes);
    }

    pub fn run(&mut self) {
        while let Some(e) = self.window.next() {
            let foo = self.cpu.redraw;
            let bar = self.cpu.gfx;
            self.window.draw_2d(&e, |c, g| {
                clear(color::BLACK, g);
                //                let square = rectangle::square(0.0, 0.0, 500.0);
                //                let transform = c.transform.trans(-25.0, -25.0);
                let size = 20;
                if foo {
                    let pixel_size = 5;
                    for y in 0..32 {
                        for x in 0..64 {
                            if bar[(x + y * 64) as usize] & 0x01 == 1 {
                                let d = [
                                    (x * size) as f64,
                                    (y * size) as f64,
                                    size as f64,
                                    size as f64,
                                ];
                                Rectangle::new(color::WHITE).draw(d, &c.draw_state, c.transform, g);
                            }
                        }
                    }
                }
            });

            self.cpu.cycle();
        }
    }

    pub fn draw_gfx_memory(&self, buffer: &[u8]) {
        for y in 0u8..32 {
            for x in 0u8..64 {
                if buffer[(x + y * 64) as usize] & 0x01 == 1 {}
            }
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

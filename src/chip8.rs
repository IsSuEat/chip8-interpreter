use cpu::Cpu;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use piston_window::*;
use std::path::Path;

pub struct Chip8 {
    cpu: Cpu,
    window: PistonWindow,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            cpu: Cpu::default().init(),
            window: WindowSettings::new("Chip 8 Interpreter", [640, 320])
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
            if let Some(u) = e.update_args() {
                self.cpu.cycle(u.dt);
            }

            let should_draw = self.cpu.needs_redraw();
            let mem = self.cpu.gfx;
            self.window.draw_2d(&e, |c, g| {
                clear(color::BLACK, g);
                let size = 10;
                if should_draw {
                    for y in 0..32 {
                        for x in 0..64 {
                            if mem[(x + (y * 64)) as usize] & 0x01 == 1 {
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

            if let Some(Button::Keyboard(key)) = e.press_args() {
                if key == Key::Escape {
                    self.dump_memory();
                }
                // A - F
                if key.code() >= 0x61 && key.code() <= 0x66 {
                    let key_in_map = (key.code() - 87) as u8;
                    self.cpu.handle_key_press(key_in_map);
                } else if key.code() >= 0x30 && key.code() <= 0x39 {
                    // 0 - 9
                    let key_in_map = (key.code() - 48) as u8;
                    self.cpu.handle_key_press(key_in_map);
                }
            }
            if let Some(Button::Keyboard(key)) = e.release_args() {
                // A - F
                if key.code() >= 0x61 && key.code() <= 0x66 {
                    let key_in_map = (key.code() - 87) as u8;
                    self.cpu.handle_key_release(key_in_map);
                } else if key.code() >= 0x30 && key.code() <= 0x39 {
                    // 0 - 9
                    let key_in_map = (key.code() - 48) as u8;
                    self.cpu.handle_key_release(key_in_map);
                }
            }
        }
    }

    pub fn dump_memory(&self) {
        let dump_file = Path::new("chip8.memdump");
        if dump_file.exists() {
            fs::rename(dump_file, "chip8.memdump.1").expect("Failed to move memdump");
        }

        let mut file = File::create("chip8.memdump").unwrap();
        file.write_all(self.cpu.dump_memory())
            .expect("Failed to write memdump");
    }
}

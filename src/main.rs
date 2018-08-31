extern crate chip8;
extern crate env_logger;

use chip8::Chip8;

use std::env;
use std::time::Duration;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let mut rom_path = &String::from("roms/sctest.c8");
    if args.len() == 2 {
        rom_path = &args[1];
    }

    let mut c8 = Chip8::new();
    c8.load_rom(rom_path);
    c8.run();
}

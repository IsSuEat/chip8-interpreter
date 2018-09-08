#[macro_use]
extern crate log;
extern crate env_logger;
extern crate piston_window;
extern crate rand;

pub mod chip8;
pub mod cpu;
use chip8::Chip8;

use std::env;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let mut rom_path = &String::from("roms/pong.ch8");
    if args.len() == 2 {
        rom_path = &args[1];
    }

    let mut c8 = Chip8::new();
    c8.load_rom(rom_path);
    c8.run();
}

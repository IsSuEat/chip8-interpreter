extern crate chip8;
extern crate env_logger;
use chip8::Chip8;

use std::env;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    //    if args.len() < 2 {
    //        println!("no path provided");
    //        return;
    //    }
    let rom_path = String::from("roms/BC_test.ch8");

    let mut c8 = Chip8::new();
    c8.load_rom(&rom_path);
    c8.run();
}

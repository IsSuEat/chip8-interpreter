extern crate chip8;
use chip8::Chip8;

use std::path::Path;
use std::env;


fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

//    if args.len() < 2 {
//        println!("no path provided");
//        return;
//    }

    let rom_path = Path::new("/home/armin/projects/rust/chip8/roms/testrom.ch8");

    let mut c8 = Chip8::new();
    c8.load_rom(rom_path);

}

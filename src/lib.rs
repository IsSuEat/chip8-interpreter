#[macro_use]
extern crate log;
extern crate env_logger;

pub mod chip8;
pub mod cpu;
pub use chip8::Chip8;

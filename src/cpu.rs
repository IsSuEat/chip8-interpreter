use std::io::Bytes;
use std::fs::File;


const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    opcode: u16,
    mem: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    gfx: [u8; 64 * 32],
    stack: [u16; 16],
    stack_pointer: u16,
    key: [u8; 16],
    // timers
    delay_timer : u8,
    sound_timer : u8,
}

impl Default for Cpu {
   fn default() -> Cpu {
       Cpu {
        opcode: 0,
        mem: [0; 4096],
        v: [0; 16],
        i: 0,
        pc: 0,
        gfx: [0; 64 * 32],
        stack: [0; 16],
        stack_pointer: 0,
        key: [0; 16],
        delay_timer: 0,
        sound_timer: 0,
       }
   } 
}

impl Cpu {   
    /// Setup fontmap and initialize program counter
    /// Fontmap is loaded into the first 80 bytes 
    /// Programm counter starts at 0x200
    pub fn init(mut self) -> Self {
        self.mem[0..80].clone_from_slice(&FONTSET);
        self.pc = 0x200;
        return self;
    }

    /// Load bytes intor ROM and RAM memory range
    /// Range starts at 0x200 and ends at 0xFFF
    pub fn load_bytes(&mut self, bytes: Bytes<File>) {
        for (i, byte) in bytes.enumerate() {
            self.mem[i + 512] = byte.unwrap()
        }
    }
    
    /// Fetch the opcode from memory
    /// Opcode is 2 bytes
    fn fetch_opcode(&self) -> u16 {
        let a = (self.mem[self.pc as usize] as u16) << 8 ;
        let b = self.mem[self.pc as usize + 1] as u16;
        return a | b;
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);   
        self.handle_timers();
    }

    fn handle_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP");
            }
            self.sound_timer -= 1;
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        self.opcode = opcode;

        match opcode & 0xF000 {
            0xA000 => self.op_annn(),
            0x0000 => match opcode & 0x000F {
                0x0000 => self.op_00e0(),
                _ => println!("Unknown opcode 0x{:X}", opcode)
            }
            _ => println!("Unknown opcode 0x{:X}", opcode)
        }
    }

    /// Sets I to the address NNN.
    fn op_annn(&mut self) {
        self.i = self.opcode & 0x0FFF;
        self.pc += 2;
    }
    
    /// Clears the screen
    fn op_00e0(&mut self) {
        self.gfx = [0; 64*32];
        println!("Clear screen");
    }
}


#[cfg(test)]
mod tests {
    use cpu::*;

    #[test]
    fn test_annn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0xA123);
       
        assert_eq!(cpu.i, 0x123);
    }

    #[test]
    fn test_00e0() {
        let mut cpu = Cpu::default().init();
        cpu.gfx = [1; 64*32];
        assert!(cpu.gfx[1] == 1);

        cpu.execute_opcode(0x00E0);
     
        assert_eq!(cpu.gfx[1], 0);

    }
}
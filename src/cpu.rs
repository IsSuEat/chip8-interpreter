use std::fs::File;
use std::io::Bytes;
use std::num::Wrapping;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
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
    delay_timer: u8,
    sound_timer: u8,
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
        let a = (self.mem[self.pc as usize] as u16) << 8;
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
            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xnn(),
            0x4000 => self.op_4xnn(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xnn(),
            0x7000 => self.op_7xnn(),
            0x0000 => match opcode & 0x000F {
                0x0000 => self.op_00e0(),
                0x000E => self.op_00ee(),
                _ => self.op_unknown(),
            },
            _ => self.op_unknown(),
        }
    }

    fn op_unknown(&mut self) {
        println!(
            "Unknown opcode 0x{:X}, 0x{:X}",
            self.opcode,
            self.opcode & 0xF000
        );
        panic!();
    }

    /// Sets I to the address NNN.
    fn op_annn(&mut self) {
        self.i = self.opcode & 0x0FFF;
        self.pc += 2;
    }

    /// Clears the screen
    fn op_00e0(&mut self) {
        self.gfx = [0; 64 * 32];
        self.pc += 2;
        println!("Clear screen");
    }

    /// Returns from a subroutine
    fn op_00ee(&mut self) {
        println!("Return from sub");
    }

    /// Jumps to address at NNN
    fn op_1nnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        println!("Jumping to {:X}", address);
    }

    /// Calls subroutine at NNN
    fn op_2nnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        println!("Calling {:X}", address);
        //store current program counter
        self.stack[self.stack_pointer as usize] = self.pc;
        self.stack_pointer += 1;
        self.pc = address;
    }

    /// Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx==NN)`
    fn op_3xnn(&mut self) {
        let x = self._x();
        let nn = self._nn();

        if self.v[x as usize] == nn {
            self.pc += 4;
            return;
        }
        self.pc += 2;
    }

    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx!=NN)`
    fn op_4xnn(&mut self) {
        let x = self._x();
        let nn = self._nn();

        if self.v[x as usize] != nn {
            self.pc += 4;
            return;
        }
        self.pc += 2;
    }

    /// Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx==Vy)`
    fn op_5xy0(&mut self) {
        let x = self._x();
        let y = self._y();
        if self.v[x as usize] == self.v[y as usize] {
            self.pc += 4;
            return;
        }
        self.pc += 2;
    }

    /// Sets VX to NN.
    fn op_6xnn(&mut self) {
        let x = self._x();
        let nn = (self.opcode & 0x00FF) as u8;
        println!("Setting V{} to {}", x, nn);
        self.v[x as usize] = nn;
        self.pc += 2;
    }

    /// Adds NN to VX. (Carry flag is not changed)
    /// pseudo c `Vx += NN `
    fn op_7xnn(&mut self) {
        let x = self._x();
        let nn = self._nn();

        self.v[x as usize] = self.v[x as usize].wrapping_add(nn);
    }

    /// Extract X from opcode
    fn _x(&self) -> u8 {
        return ((self.opcode & 0x0F00) >> 8) as u8;
    }

    /// Extract Y from opcode
    fn _y(&self) -> u8 {
        return ((self.opcode & 0x00F0) >> 4) as u8;
    }

    /// Extract NN from opcode
    fn _nn(&self) -> u8 {
        return (self.opcode & 0x00FF) as u8;
    }
}

#[cfg(test)]
mod tests {
    use cpu::*;

    #[test]
    fn test_x_y() {
        let mut cpu = Cpu::default().init();
        cpu.opcode = 0xDEAD;
        assert_eq!(cpu._x(), 0xE);
        assert_eq!(cpu._y(), 0xA);
    }

    #[test]
    fn test_nn() {
        let mut cpu = Cpu::default().init();
        cpu.opcode = 0xDEAD;
        assert_eq!(cpu._nn(), 0xAD);
    }

    #[test]
    fn test_annn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0xA123);

        assert_eq!(cpu.i, 0x123);
    }

    #[test]
    fn test_00e0() {
        let mut cpu = Cpu::default().init();
        cpu.gfx = [1; 64 * 32];
        assert_eq!(cpu.gfx[1], 1);

        cpu.execute_opcode(0x00E0);

        assert_eq!(cpu.gfx[1], 0);
    }

    #[test]
    fn test_00ee() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0x00EE);
    }

    #[test]
    fn test_1nnn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0x1FFF);
    }

    #[test]
    fn test_2nnn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0x2123);
        assert_eq!(cpu.pc, 0x123);
    }

    #[test]
    fn test_3xnn() {
        let mut cpu = Cpu::default().init();
        // set register 2 to 0xFF
        cpu.v[2] = 0xFF;
        // if v[2] == 0xFF
        cpu.execute_opcode(0x32FF);
        // skip 4, starting at 0x200
        assert_eq!(cpu.pc, 0x204);
        // set register 2 to 0xF0
        // now if v[2[ == 0xFF is false, so only increment by 2
        cpu.v[2] = 0xF0;
        // reset pc
        cpu.pc = 0x200;
        cpu.execute_opcode(0x32FF);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_4xnn() {
        let mut cpu = Cpu::default().init();
        // set register 2 to 0xFF
        cpu.v[2] = 0xFF;
        // if v[2] != 0xFF
        cpu.execute_opcode(0x42FD);
        // skip 4, starting at 0x200
        assert_eq!(cpu.pc, 0x204);
        // set register 2 to 0xF0
        // if v[2[ == 0xFF only increment by 2
        cpu.v[2] = 0xF0;
        // reset pc
        cpu.pc = 0x200;
        cpu.execute_opcode(0x42F0);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_5xy0() {
        let mut cpu = Cpu::default().init();
        cpu.v[5] = 0xFF;
        cpu.v[6] = 0xFF;
        cpu.execute_opcode(0x5560);
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_6xnn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0x62FF);
        assert_eq!(cpu.v[2], 0xFF);
    }

    #[test]
    fn test_7xnn() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0x72FF);
        assert_eq!(cpu.v[2], 0xFF);
    }

    #[test]
    fn test_7xnn_overflow() {
        let mut cpu = Cpu::default().init();
        cpu.v[2] = 0xFF;
        cpu.execute_opcode(0x7205);
        assert_eq!(cpu.v[2], 0x04);
    }

}

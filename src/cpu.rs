use std::fs::File;
use std::io::Bytes;

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
                debug!("BEEP");
            }
            self.sound_timer -= 1;
        }
    }
    /// Returns the contents of a register
    fn read_register(&self, register: u8) -> u8 {
        self.v[register as usize]
    }

    /// Write `content` to `register`.
    /// Does not increment program counter
    fn set_register(&mut self, register: u8, content: u8) {
        self.v[register as usize] = content
    }

    /// Increment program counter by two, normal step
    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    fn execute_opcode(&mut self, opcode: u16) {
        self.opcode = opcode;
        debug!("OpCode: 0x{:X}", self.opcode);
        match opcode & 0xF000 {
            0xA000 => {
                let address = self.opcode & 0x0FFF;
                self.set_index_register(address)
            }
            0x1000 => {
                let address = self.opcode & 0x0FFF;
                self.jump_to(address);
            }
            0x2000 => {
                let address = self.opcode & 0x0FFF;
                self.call_sub_at(address);
            }
            0x3000 => {
                let vx = self.read_register(self._x());
                let nn = self._nn();
                self.skip_if_eq(vx, nn)
            }
            0x4000 => {
                let vx = self.read_register(self._x());
                let nn = self._nn();
                self.skip_if_neq(vx, nn);
            }
            0x5000 => {
                let vx = self.read_register(self._x());
                let vy = self.read_register(self._y());
                self.skip_if_eq(vx, vy);
            }
            0x6000 => {
                let vx = self._x();
                let nn = self._nn();
                self.set_register(vx, nn);
                self.inc_pc();
            }
            0x7000 => {
                let vx = self._x();
                let nn = self._nn();
                self.add_to_register(vx, nn);
            }
            0x8000 => match opcode & 0x000F {
                0x0005 => {
                    let vx = self._x();
                    let vY = self._y();
                    let value = self.read_register(vY);
                    self.subtract_from_register(vx, value);
                }
                _ => self.op_unknown(),
            },
            0xD000 => self.draw(),
            0x0000 => match opcode & 0x000F {
                0x0000 => self.clear_screen(),
                0x000E => self.return_from_sub(),
                _ => self.op_unknown(),
            },
            _ => self.op_unknown(),
        }
    }

    fn op_unknown(&mut self) {
        error!(
            "Unknown opcode 0x{:X}, 0x{:X}",
            self.opcode,
            self.opcode & 0xF000
        );
        panic!();
    }

    /// Sets I to the address NNN.
    fn set_index_register(&mut self, address: u16) {
        self.i = address;
        self.inc_pc();
    }

    /// Clears the screen
    fn clear_screen(&mut self) {
        self.gfx = [0; 64 * 32];
        self.inc_pc();
        debug!("Clear screen");
    }

    /// Returns from a subroutine
    fn return_from_sub(&mut self) {
        self.pc = self.stack[self.stack_pointer as usize];
        self.stack_pointer -= 1;
        debug!("Return from sub");
    }

    /// Jumps to address at NNN
    fn jump_to(&mut self, address: u16) {
        self.pc = address;
        debug!("Jumping to {:X}", address);
    }

    /// Calls subroutine at NNN
    fn call_sub_at(&mut self, address: u16) {
        //store current program counter
        self.stack[self.stack_pointer as usize] = self.pc;
        self.stack_pointer += 1;
        self.pc = address;
        debug!("Calling {:X}", address);
    }

    /// Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx==NN)`
    fn skip_if_eq(&mut self, x: u8, y: u8) {
        if x == y {
            self.pc += 4;
            return;
        }
        self.inc_pc();
    }

    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx!=NN)`
    fn skip_if_neq(&mut self, x: u8, y: u8) {
        if x != y {
            self.pc += 4;
            return;
        }
        self.inc_pc();
    }

    /// Add data to the contents of register
    /// Does not change the carry flag
    fn add_to_register(&mut self, register: u8, data: u8) {
        let res = self.read_register(register).wrapping_add(data);
        self.set_register(register, res);
        self.inc_pc();
    }

    /// Subtract `x` from the value stored in `register` and store the result in `register`
    /// VF is set to 0 when there's a borrow, and 1 when there isn't
    fn subtract_from_register(&mut self, register: u8, x: u8) {
        let old_value = self.read_register(register);
        if old_value < x {
            self.set_register(0xF, 1);
        } else {
            self.set_register(0xF, 0);
        }
        let res = self.read_register(register).wrapping_sub(x);
        self.set_register(register, res);
        self.inc_pc();
    }

    /// Fills gfx buffer with sprite data
    ///
    fn draw(&mut self) {
        let x = self._x();
        let y = self._y();
        let n = (self.opcode & 0x000F) as u8;

        debug!("Drawing: x: {} y: {} n: {}", x, y, n);
        // self.inc_pc();
    }

    /// Extract X from opcode
    fn _x(&self) -> u8 {
        ((self.opcode & 0x0F00) >> 8) as u8
    }

    /// Extract Y from opcode
    fn _y(&self) -> u8 {
        ((self.opcode & 0x00F0) >> 4) as u8
    }

    /// Extract NN from opcode
    fn _nn(&self) -> u8 {
        (self.opcode & 0x00FF) as u8
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
    fn test_set_index_register() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0xA123);

        assert_eq!(cpu.i, 0x123);
    }

    #[test]
    fn test_clear_screen() {
        let mut cpu = Cpu::default().init();
        cpu.gfx = [1; 64 * 32];
        assert_eq!(cpu.gfx[1], 1);

        cpu.execute_opcode(0x00E0);

        assert_eq!(cpu.gfx[1], 0);
    }

    //    #[test]
    //    fn test_00ee() {
    //        let mut cpu = Cpu::default().init();
    //        cpu.execute_opcode(0x00EE);
    //    }

    #[test]
    fn test_jump_to() {
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

    #[test]
    fn test_subtract_from_register() {
        let mut cpu = Cpu::default().init();
        cpu.set_register(2, 0xFF);
        cpu.set_register(3, 0x0F);
        cpu.execute_opcode(0x8235);
        assert_eq!(cpu.read_register(2), 0xF0);
        assert_eq!(cpu.pc, 0x202);
    }
}

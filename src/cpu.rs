use rand::prelude::*;
use std::fs::File;
use std::io::Bytes;

const FONTSET_START: usize = 0;
const WIDTH: usize = 64;
const HEIGTH: usize = 32;

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
    pub gfx: [u8; WIDTH * HEIGTH],
    stack: [u16; 16],
    stack_pointer: u16,
    key: [u8; 16],
    // timers
    delay_timer: u8,
    sound_timer: u8,
    pub redraw: bool,
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
            redraw: false,
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

    pub fn cycle(&mut self, dt: f64) {
        let num_instr = (dt * 600.0).round() as u64;
        for _ in 1..num_instr {
            let opcode = self.fetch_opcode();
            self.execute_opcode(opcode);
            self.handle_timers();
        }
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

    pub fn handle_key_press(&mut self, key: u8) {
        debug!("key pressed 0x{:X}", key);
        self.key[key as usize] = 1;
    }

    pub fn handle_key_release(&mut self, key: u8) {
        debug!("key released 0x{:X}", key);
        self.key[key as usize] = 0;
    }

    pub fn dump_memory(&self) -> &[u8; 4096] {
        &self.mem
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

    fn read_mem(&self, offset: u16) -> u8 {
        self.mem[offset as usize]
    }

    fn execute_opcode(&mut self, opcode: u16) {
        self.opcode = opcode;
        debug!("OpCode: 0x{:X}", self.opcode);
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => self.clear_screen(),
                0x000E => self.return_from_sub(),
                _ => self.op_unknown(),
            },
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
                0x0000 => {
                    let x = self._x();
                    let vy = self.read_register(self._y());
                    self.set_register(x, vy);
                    self.inc_pc();
                }
                0x0001 => {
                    let x = self._x();
                    let y = self._y();
                    self.or(x, y);
                }
                0x0002 => {
                    let x = self._x();
                    let y = self._y();
                    self.and(x, y);
                }
                0x0003 => {
                    let x = self._x();
                    let y = self._y();
                    self.xor(x, y);
                }
                0x0004 => {
                    let x = self._x();
                    let vy = self.read_register(self._y());
                    self.add_to_register_with_carry(x, vy);
                }
                0x0005 => {
                    let vx = self._x();
                    let value = self.read_register(self._y());
                    self.subtract_from_register_with_carry(vx, value);
                }
                0x006 => {
                    let vx = self._x();
                    self.right_shift_register(vx);
                }
                0x007 => {
                    let vx = self._x();
                    let vy = self._y();
                    self.subtract_registerx_from_registery_set_registerx(vx, vy);
                }
                0x00E => {
                    let vx = self._x();
                    self.left_shift_register(vx);
                }
                _ => self.op_unknown(),
            },
            0x9000 => {
                let registerx_contents = self.read_register(self._x());
                let registery_contents = self.read_register(self._y());
                self.skip_if_neq(registerx_contents, registery_contents);
            }
            0xA000 => {
                let address = self.opcode & 0x0FFF;
                self.set_index_register(address);
            }
            0xC000 => self.rand(),
            0xD000 => self.draw(),
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    let x = self._x();
                    self.check_key_pressed(x);
                }
                0x00A1 => {
                    let x = self._x();
                    self.check_key_released(x);
                }
                _ => self.op_unknown(),
            },
            0xF000 => match opcode & 0x00FF {
                0x001E => {
                    let vx = self._x();
                    self.add_vx_to_i(vx);
                }
                0x007 => {
                    let vx = self._x();
                    self.get_delay_timer(vx);
                }
                0x0015 => {
                    let vx = self._x();
                    self.set_delay_timer(vx)
                }
                0x0018 => {
                    let x = self._x();
                    self.set_sound_timer(x);
                }
                0x0055 => {
                    let vx = self._x();
                    self.store_registers_up_to(vx);
                }
                0x0029 => {
                    let vx = self._x();
                    self.set_index_register_to_character_sprite(vx);
                }
                0x0033 => {
                    let vx = self.read_register(self._x());
                    self.store_bcd(vx);
                }
                0x0065 => {
                    let vx = self._x();
                    self.fill_registers_up_to(vx);
                }
                _ => self.op_unknown(),
            },
            _ => self.op_unknown(),
        }
    }

    pub fn needs_redraw(&self) -> bool {
        self.redraw
    }

    fn op_unknown(&self) {
        error!(
            "Unknown opcode 0x{:X}, 0x{:X}",
            self.opcode,
            self.opcode & 0xF000
        );
        panic!();
    }

    fn set_delay_timer(&mut self, vx: u8) {
        self.delay_timer = self.read_register(vx);
        self.inc_pc();
    }

    fn set_sound_timer(&mut self, vx: u8) {
        self.sound_timer = self.read_register(vx);
        self.inc_pc();
    }

    fn get_delay_timer(&mut self, vx: u8) {
        let current_delay = self.delay_timer;
        self.set_register(vx, current_delay);
        self.inc_pc();
    }

    /// Sets I to the address NNN.
    fn set_index_register(&mut self, address: u16) {
        self.i = address;
        self.inc_pc();
    }

    /// Clears the screen
    fn clear_screen(&mut self) {
        self.gfx = [0; 64 * 32];
        self.redraw = true;
        self.inc_pc();
        debug!("Clear screen");
    }

    /// Returns from a subroutine
    fn return_from_sub(&mut self) {
        self.stack_pointer -= 1;
        self.pc = self.stack[self.stack_pointer as usize];
        debug!("Return from sub");
        self.inc_pc();
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
            self.inc_pc();
        }
        self.inc_pc();
    }

    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    /// pseudo c `if(Vx!=NN)`
    fn skip_if_neq(&mut self, x: u8, y: u8) {
        if x != y {
            self.inc_pc();
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

    /// Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    /// 8XY4
    fn add_to_register_with_carry(&mut self, register: u8, data: u8) {
        let old_value = self.read_register(register);
        // need to check for carry
        let (result, need_carry) = old_value.overflowing_add(data);
        if need_carry {
            self.set_register(0xF, 1);
        } else {
            self.set_register(0xF, 0);
        }

        self.set_register(register, result);
        self.inc_pc();
    }

    /// Subtract `x` from the value stored in `register` and store the result in `register`
    /// VF is set to 0 when there's a borrow, and 1 when there isn't
    fn subtract_from_register_with_carry(&mut self, register: u8, x: u8) {
        let old_value = self.read_register(register);
        let (result, need_carry) = old_value.overflowing_sub(x);

        if need_carry {
            self.set_register(0xF, 0);
        } else {
            self.set_register(0xF, 1);
        }

        self.set_register(register, result);
        self.inc_pc();
    }

    /// Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    /// 8XY7
    fn subtract_registerx_from_registery_set_registerx(&mut self, x: u8, y: u8) {
        let vx = self.read_register(x);
        let vy = self.read_register(y);

        let (result, need_carry) = vy.overflowing_sub(vx);
        if need_carry {
            self.set_register(0xF, 0);
        } else {
            self.set_register(0xF, 1);
        }
        self.set_register(x, result);
        self.inc_pc();
    }

    /// Stores the least significant bit of VX in VF and then shifts VX to the right by 1.[2]
    /// 8XY6
    fn right_shift_register(&mut self, register: u8) {
        let vx = self.read_register(register);
        let least_significant = vx & 0x1;
        let result = vx >> 1;
        self.set_register(register, result);
        self.set_register(0xF, least_significant);
        self.inc_pc();
    }

    /// Stores the most significant bit of VX in VF and then shifts VX to the left by 1.[3]
    /// 8XYE
    fn left_shift_register(&mut self, register: u8) {
        let vx = self.read_register(register);

        let mut most_significant = 0;
        if vx >= 128 {
            most_significant = 1;
        }

        let result = vx << 1u8;
        self.set_register(register, result);
        self.set_register(0xF, most_significant);
        self.inc_pc();
    }

    /// Sets VX to VX xor VY.
    /// 8XY3
    fn xor(&mut self, registerx: u8, registery: u8) {
        let vx = self.read_register(registerx);
        let vy = self.read_register(registery);

        let result = vx ^ vy;
        self.set_register(registerx, result);
        self.inc_pc();
    }

    /// Sets VX to VX or VY. (Bitwise OR operation)
    ///  8XY1
    fn or(&mut self, registerx: u8, registery: u8) {
        let vx = self.read_register(registerx);
        let vy = self.read_register(registery);

        let result = vx | vy;
        self.set_register(registerx, result);
        self.inc_pc();
    }

    /// Sets VX to VX and VY. (Bitwise AND operation)  
    ///  8XY2
    fn and(&mut self, registerx: u8, registery: u8) {
        let vx = self.read_register(registerx);
        let vy = self.read_register(registery);

        let result = vx & vy;
        self.set_register(registerx, result);
        self.inc_pc();
    }

    /// Fills gfx buffer with sprite data
    ///
    fn draw(&mut self) {
        let x = self._x();
        let start_x = self.read_register(x) as u16;
        let y = self._y();
        let start_y = self.read_register(y) as u16;
        let number_of_lines = self.opcode & 0x000F;

        let mut raster: [[bool; WIDTH]; HEIGTH] = [[false; 64]; 32];
        self.set_register(0xF, 0);
        for line in 0..number_of_lines {
            let pixel = self.mem[(self.i + line) as usize];
            for x_pos in 0..8 {
                if (pixel >> (7 - x_pos)) & 1 == 1 {
                    let x_in_raster = ((start_x + x_pos) % WIDTH as u16) as usize;
                    let y_in_raster = ((line + start_y) % HEIGTH as u16) as usize;

                    raster[y_in_raster][x_in_raster] = true;
                    // need to flip the pixel
                    // check if we need carry
                    let offset = ((start_x as u16 + x_pos as u16) as u16)
                        + (((line + start_y) % HEIGTH as u16) * WIDTH as u16);
                    if offset >= self.gfx.len() as u16 {
                        debug!(
                            "Out of bounds {}, x: {}, y:{}: number of lines: {}",
                            offset, x_in_raster, y_in_raster, number_of_lines
                        );
                        panic!();
                    }
                    if self.gfx[offset as usize] == 1 {
                        self.set_register(0xF, 1);
                    }
                    self.gfx[offset as usize] ^= 1;
                }
            }
        }

        // for (line_nr, line) in raster.iter().enumerate() {
        //     print!("|");
        //     for (col_nr, col) in line.iter().enumerate() {
        //         if *col {
        //             print!("*");
        //         } else {
        //             print!(".");
        //         }
        //     }
        //     println!("|");
        // }

        self.redraw = true;
        self.inc_pc();
    }

    /// Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    /// FX29
    fn set_index_register_to_character_sprite(&mut self, sprite: u8) {
        let address_of_sprite = FONTSET_START + (sprite * 5) as usize;
        self.set_index_register(address_of_sprite as u16);
    }

    /// Fills V0 to VX (including VX) with values from memory starting at address I.
    /// The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    /// FF65
    fn fill_registers_up_to(&mut self, end_register: u8) {
        for idx in 0..=end_register {
            let content = self.read_mem(self.i + idx as u16);
            self.set_register(idx, content);
        }
        let i = self.i;
        self.set_index_register(i + end_register as u16 + 1u16);

        // self.inc_pc();
    }

    /// Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    /// FX55
    fn store_registers_up_to(&mut self, registerx: u8) {
        for idx in 0..=registerx {
            let register_content = self.read_register(idx);
            let store_address = self.i + idx as u16;
            self.mem[store_address as usize] = register_content;
        }
        let i = self.i;
        self.set_index_register(i + registerx as u16 + 1u16);
        // self.inc_pc();
    }

    /// Adds VX to I.[4]
    /// FX1E
    fn add_vx_to_i(&mut self, vx: u8) {
        let register_content = self.read_register(vx);
        let result = self.i + register_content as u16;
        self.set_index_register(result);
        // self.inc_pc();
    }

    /// Stores the binary-coded decimal representation of VX,
    /// with the most significant of three digits at the address in I,
    /// the middle digit at I plus 1, and the least significant digit at I plus 2
    /// FX33
    fn store_bcd(&mut self, x: u8) {
        self.mem[self.i as usize] = x / 100;
        self.mem[(self.i + 1) as usize] = (x / 10) % 10;
        self.mem[(self.i + 2) as usize] = (x % 100) % 10;
        self.inc_pc();
    }

    /// Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    /// CXNN
    fn rand(&mut self) {
        let x = self._x();
        let nn = self._nn();
        let random_nr: u8 = thread_rng().gen_range(0, 255);

        self.set_register(x, random_nr & nn);

        self.inc_pc();
    }

    /// Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    /// EX9E
    fn check_key_pressed(&mut self, register: u8) {
        let vx = self.read_register(register);
        if self.key[vx as usize] != 0 {
            self.inc_pc();
        }
        debug!("Checking if key pressed: 0x{:X}", vx);
        self.inc_pc();
    }

    /// Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
    /// EXA1
    fn check_key_released(&mut self, register: u8) {
        let vx = self.read_register(register);
        if self.key[vx as usize] == 0 {
            self.inc_pc();
        }
        debug!("Checking if key released: 0x{:X}", vx);

        self.inc_pc();
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
        cpu.set_register(2, 0x0F);
        cpu.set_register(3, 0xFF);
        cpu.execute_opcode(0x8235);
        assert_eq!(cpu.read_register(0xF), 0);
    }

    #[test]
    fn test_set_idx_to_sprite() {
        let mut cpu = Cpu::default().init();
        cpu.execute_opcode(0xF029);
        assert_eq!(cpu.i, FONTSET_START as u16);
        cpu.execute_opcode(0xF129);
        assert_eq!(cpu.i, (FONTSET_START + 5) as u16);
    }

    #[test]
    fn test_fill_registers_up_to() {
        let mut cpu = Cpu::default().init();
        cpu.mem[0] = 0xF;
        cpu.mem[1] = 0x1;
        cpu.mem[2] = 0x2;
        cpu.mem[3] = 0x3;
        // fill registers 0 up to (including) 3
        cpu.execute_opcode(0xF365);
        assert_eq!(cpu.read_register(0), 0xF);
        assert_eq!(cpu.read_register(1), 0x1);
        assert_eq!(cpu.read_register(2), 0x2);
        assert_eq!(cpu.read_register(3), 0x3);
    }

    #[test]
    fn test_store_bcd() {
        let mut cpu = Cpu::default().init();
        // 0x94 == 148 dec
        cpu.set_register(0xE, 0x94);
        cpu.execute_opcode(0xFE33);
        assert_eq!(cpu.read_mem(cpu.i), 1);
        assert_eq!(cpu.read_mem(cpu.i + 1), 4);
        assert_eq!(cpu.read_mem(cpu.i + 2), 8);
    }

    #[test]
    fn test_add_to_register_with_carry() {
        let mut cpu = Cpu::default().init();
        cpu.set_register(0x1, 5);
        cpu.set_register(0x2, 5);
        // add 5 + 5 in register 0x1
        cpu.execute_opcode(0x8124);
        assert_eq!(cpu.read_register(0x1), 10);
        assert_eq!(cpu.read_register(0xF), 0);
        cpu.set_register(0x1, 0xFF);
        cpu.set_register(0x2, 0x02);
        cpu.execute_opcode(0x8124);
        assert_eq!(cpu.read_register(0xF), 1);
        assert_eq!(cpu.read_register(0x1), 1);
    }
}

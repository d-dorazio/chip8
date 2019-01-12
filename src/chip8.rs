pub const PIXELS: usize = 32 * 64;
pub const RAM_SIZE: usize = 4096;
pub const PROGRAM_START_PC: usize = 0x200;

// each hex digit has a 4x5 sprite
const FONT_SET: [u8; 80] = [
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

#[derive(Clone)]
pub struct Chip8 {
    registers: [u8; 16],
    i_reg: u16,

    ram: [u8; RAM_SIZE],
    vram: [u8; PIXELS / 8],

    pc: usize,

    stack: [usize; 16],
    sp: usize,

    delay_timer: u8,
    sound_timer: u8,

    waiting_keypress_reg: Option<usize>,
    keyboard: [bool; 16],
}

impl Chip8 {
    pub fn with_program(program: &[u8]) -> Option<Self> {
        if program.len() > RAM_SIZE - PROGRAM_START_PC {
            return None;
        }

        let mut ram = [0; RAM_SIZE];

        (&mut ram[..FONT_SET.len()]).copy_from_slice(&FONT_SET);

        for (dst, src) in (&mut ram[PROGRAM_START_PC..]).iter_mut().zip(program) {
            *dst = *src;
        }

        Some(Chip8 {
            registers: [0; 16],
            i_reg: 0,

            ram,
            vram: [0; PIXELS / 8],

            pc: PROGRAM_START_PC,

            stack: [0; 16],
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,

            waiting_keypress_reg: None,
            keyboard: [false; 16],
        })
    }

    pub fn keypress(&mut self, hex_key: u8) {
        debug_assert!(
            usize::from(hex_key) < self.keyboard.len(),
            "key is not an hex key"
        );

        self.keyboard[usize::from(hex_key)] = true;

        if let Some(x) = self.waiting_keypress_reg {
            self.registers[x] = hex_key;
        }

        self.waiting_keypress_reg = None;
    }

    pub fn emulate_cycle(&mut self) {
        if self.waiting_keypress_reg.is_some() {
            return;
        }

        let instr = (u16::from(self.ram[self.pc]) << 8) | u16::from(self.ram[self.pc + 1]);
        self.pc += 2;

        let opcode = instr >> 12;
        let x = usize::from((instr >> 8) & 0xF);
        let y = usize::from((instr >> 4) & 0xF);
        let nnn = instr & 0xFFF;
        let nn = (instr & 0xFF) as u8;
        let n = (instr & 0xF) as u8;

        match opcode {
            0x0 if nnn == 0xE0 => self.clear_vram(),
            0x0 if nnn == 0xEE => self.ret(),
            0x0 => unimplemented!("call RCA 1802 not implemented"),

            0x1 => self.goto(nnn),
            0x2 => self.call(nnn),
            0x3 => self.skip_if_eq_nn(x, nn),
            0x4 => self.skip_if_ne_nn(x, nn),
            0x5 => self.skip_if_eq_rr(x, y),

            0x6 => self.load(x, nn),
            0x7 => self.add_nn(x, nn),

            0x8 if n == 0x0 => self.assign(x, y),
            0x8 if n == 0x1 => self.or(x, y),
            0x8 if n == 0x2 => self.and(x, y),
            0x8 if n == 0x3 => self.xor(x, y),
            0x8 if n == 0x4 => self.add_rr(x, y),
            0x8 if n == 0x5 => self.sub_rr(x, y),
            0x8 if n == 0x6 => self.shiftr(x),
            0x8 if n == 0x7 => self.sub_rr_inv(x, y),
            0x8 if n == 0xE => self.shiftl(x),

            0x9 => self.skip_if_ne_rr(x, y),

            0xA => self.set_i(nnn),

            0xB => self.goto_off(nnn),

            0xC => unimplemented!("rand"),

            0xD => unimplemented!("draw"),

            0xE if nn == 0x93 => self.skip_if_pressed(x),
            0xE if nn == 0xA1 => self.skip_if_not_pressed(x),

            0xF if nn == 0x07 => self.store_delay(x),
            0xF if nn == 0x0A => self.wait_keypress(x),
            0xF if nn == 0x15 => self.set_delay_timer(x),
            0xF if nn == 0x18 => self.set_sound_timer(x),
            0xF if nn == 0x1E => self.add_i(x),
            0xF if nn == 0x29 => self.font_sprite_addr(x),

            0xF if nn == 0x33 => self.bcd(x),
            0xF if nn == 0x55 => self.dump_regs(x),
            0xF if nn == 0x65 => self.load_regs(x),

            _ => {
                // noop
            }
        };
    }

    // ------------------------------------------------------------------------
    // Flow
    // ------------------------------------------------------------------------
    fn call(&mut self, addr: u16) {
        // note: here self.pc is already after the call op
        self.stack[self.sp] = self.pc;
        self.sp += 1;

        self.pc = usize::from(addr);
    }

    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    fn goto(&mut self, pc: u16) {
        self.pc = usize::from(pc);
    }

    fn goto_off(&mut self, pc: u16) {
        self.pc = usize::from(pc) + usize::from(self.registers[0]);
    }

    // ------------------------------------------------------------------------
    // Cond
    // ------------------------------------------------------------------------
    fn skip_if_eq_nn(&mut self, x: usize, nn: u8) {
        if self.registers[x] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_ne_nn(&mut self, x: usize, nn: u8) {
        if self.registers[x] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_eq_rr(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    fn skip_if_ne_rr(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    // ------------------------------------------------------------------------
    // Const
    // ------------------------------------------------------------------------
    fn load(&mut self, x: usize, nn: u8) {
        self.registers[x] = nn;
    }

    fn assign(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    // ------------------------------------------------------------------------
    // Math
    // ------------------------------------------------------------------------
    fn add_nn(&mut self, x: usize, nn: u8) {
        // self.set_status_reg(u8::max_value() - self.registers[x] < nn);
        self.registers[x] = self.registers[x].wrapping_add(nn);
    }

    fn add_rr(&mut self, x: usize, y: usize) {
        let (rx, c) = self.registers[x].overflowing_add(self.registers[y]);

        self.registers[x] = rx;
        self.registers[0xF] = if c { 1 } else { 0 };
    }

    fn sub_rr(&mut self, x: usize, y: usize) {
        let (rx, c) = self.registers[x].overflowing_sub(self.registers[y]);

        self.registers[x] = rx;
        self.registers[0xF] = if c { 1 } else { 0 };
    }

    fn sub_rr_inv(&mut self, x: usize, y: usize) {
        let (rx, c) = self.registers[y].overflowing_sub(self.registers[x]);

        self.registers[x] = rx;
        self.registers[0xF] = if c { 1 } else { 0 };
    }

    // ------------------------------------------------------------------------
    // Bit ops
    // ------------------------------------------------------------------------
    fn or(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    fn and(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    fn xor(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    fn shiftr(&mut self, x: usize) {
        self.registers[0xF] = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
    }

    fn shiftl(&mut self, x: usize) {
        self.registers[0xF] = self.registers[x] >> 7;
        self.registers[x] <<= 1;
    }

    // ------------------------------------------------------------------------
    // Mem
    // ------------------------------------------------------------------------
    fn set_i(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    fn add_i(&mut self, x: usize) {
        let (i, c) = u16::from(self.registers[x]).overflowing_add(self.i_reg);

        self.i_reg = i;
        self.registers[0xF] = if c { 1 } else { 0 };
    }

    fn font_sprite_addr(&mut self, x: usize) {
        // each font sprite is 4x5
        self.i_reg = u16::from(self.registers[x]) * 5;
    }

    fn dump_regs(&mut self, x: usize) {
        let i = usize::from(self.i_reg);

        self.ram[i..=x + i].copy_from_slice(&self.registers[..=x]);
    }

    fn load_regs(&mut self, x: usize) {
        let i = usize::from(self.i_reg);

        self.registers[..=x].copy_from_slice(&self.ram[i..=i + x]);
    }

    // ------------------------------------------------------------------------
    // Timers
    // ------------------------------------------------------------------------
    fn store_delay(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
    }

    fn set_delay_timer(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
    }

    fn set_sound_timer(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
    }

    // ------------------------------------------------------------------------
    // Keyboard
    // ------------------------------------------------------------------------
    fn skip_if_pressed(&mut self, rk: usize) {
        let k = usize::from(self.registers[rk]);
        if self.keyboard[k] {
            self.pc += 2;
        }
    }

    fn skip_if_not_pressed(&mut self, rk: usize) {
        let k = usize::from(self.registers[rk]);
        if !self.keyboard[k] {
            self.pc += 2;
        }
    }

    fn wait_keypress(&mut self, x: usize) {
        self.waiting_keypress_reg = Some(x);
    }

    // ------------------------------------------------------------------------
    // Misc
    // ------------------------------------------------------------------------
    fn bcd(&mut self, x: usize) {
        let r = self.registers[x];

        let i = usize::from(self.i_reg);
        self.ram[i] = r / 100;
        self.ram[i + 1] = (r / 10) % 10;
        self.ram[i + 2] = r % 10;
    }

    // ------------------------------------------------------------------------
    // Graphics
    // ------------------------------------------------------------------------
    fn clear_vram(&mut self) {
        for b in self.vram.iter_mut() {
            *b = 0;
        }
    }
}

use crate::font::FONT;
use rand::Rng;
use std::cmp::min;

// Constants
const RAMBYTES: usize = 4096;
pub const CHIP_WIDTH: usize = 64;
pub const CHIP_HEIGHT: usize = 32;

pub const NKEYS: usize = 16;
pub const KEYS: &str = "0123456789abcdef";

pub struct Chip8 {
    ram: [u8; RAMBYTES],
    vram: [[bool; CHIP_WIDTH]; CHIP_HEIGHT],
    keyboard: [bool; NKEYS],
    pc: usize,
    i: usize,
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; NKEYS],
}

impl Chip8 {

    pub fn new() -> Self {
	
        // load font
        let mut ram = [0u8; RAMBYTES];
        for (i, &byte) in FONT.iter().enumerate() {
            ram[i] = byte;
        }

        Chip8 {
            ram: ram,
            vram: [[false; CHIP_WIDTH]; CHIP_HEIGHT],
	    keyboard: [false; NKEYS],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0u8,
            sound_timer: 0u8,
            v: [0u8; NKEYS],
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        for (i, &byte) in bytes.iter().enumerate() {
            let addr = 0x200 + i;
            if addr >= RAMBYTES {
                break;
            }   
            self.ram[addr] = byte;
        }
    }

    pub fn update_keyboard(&mut self, keyboard: [bool; NKEYS]) {
	self.keyboard = keyboard;
    }

    pub fn get_vram(&self) -> [[bool; CHIP_WIDTH]; CHIP_HEIGHT] {
	return self.vram;
    }

    pub fn tick(&mut self) {
	let opcode = self.fetch();
	self.process(opcode);

	if self.delay_timer > 0 {
	    self.delay_timer -= 1;
	}
	if self.sound_timer > 0 {
	    self.sound_timer -= 1;
	}
    }

    pub fn should_beep(&self) -> bool {
	return self.sound_timer > 0;
    }
    
    fn fetch(&mut self) -> u16 {
        let byte1 = (self.ram[self.pc] as u16) << 8;
        let byte2 = self.ram[self.pc + 1] as u16;
        self.pc += 2;
        return byte1 | byte2;
    }

    fn process(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        match nibbles {
            (0x00, 0x00, 0x0E, 0x00) => self.op_00e0(),
	    (0x00, 0x00, 0x0E, 0x0E) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
	    (0x02, _, _, _) => self.op_2nnn(nnn),
	    (0x03, _, _, _) => self.op_3xnn(x, nn),
	    (0x04, _, _, _) => self.op_4xnn(x, nn),
	    (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xnn(x, nn),
            (0x07, _, _, _) => self.op_7xnn(x, nn),
	    (0x08, _, _, 0x00) => self.op_8xy0(x, y),
	    (0x08, _, _, 0x01) => self.op_8xy1(x, y),
	    (0x08, _, _, 0x02) => self.op_8xy2(x, y),
	    (0x08, _, _, 0x03) => self.op_8xy3(x, y),
	    (0x08, _, _, 0x04) => self.op_8xy4(x, y),
	    (0x08, _, _, 0x05) => self.op_8xy5(x, y),
	    (0x08, _, _, 0x06) => self.op_8xy6(x, y),
	    (0x08, _, _, 0x07) => self.op_8xy7(x, y),
	    (0x08, _, _, 0x0E) => self.op_8xye(x, y),
	    (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0A, _, _, _) => self.op_annn(nnn),
	    (0x0B, _, _, _) => self.op_bnnn(nnn),
	    (0x0C, _, _, _) => self.op_cxnn(x, nn),
            (0x0D, _, _, _) => self.op_dxyn(x, y, n),
	    (0x0E, _, 0x09, 0x0E) => self.op_ex9e(x),
	    (0x0E, _, 0x0A, 0x01) => self.op_exa1(x),
	    (0x0F, _, 0x00, 0x07) => self.op_fx07(x),
	    (0x0F, _, 0x00, 0x0A) => self.op_fx0a(x),
	    (0x0F, _, 0x01, 0x05) => self.op_fx15(x),
	    (0x0F, _, 0x01, 0x08) => self.op_fx18(x),
	    (0x0F, _, 0x01, 0x0E) => self.op_fx1e(x),
	    (0x0F, _, 0x02, 0x09) => self.op_fx29(x),
	    (0x0F, _, 0x03, 0x03) => self.op_fx33(x),
	    (0x0F, _, 0x05, 0x05) => self.op_fx55(x),
	    (0x0F, _, 0x06, 0x05) => self.op_fx65(x),
            _ => (),
        }
    }

    fn op_00e0(&mut self) {
        for y in 0..CHIP_HEIGHT {
            for x in 0..CHIP_WIDTH {
                self.vram[y][x] = false;
            }
        }
    }

    fn op_00ee(&mut self) {
	if let Some(sp) = self.stack.pop() {
	    self.pc = sp;
	}
    }
    
    fn op_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: usize) {
	self.stack.push(self.pc);
	self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) {
	if self.v[x] == nn {
	    self.pc += 2;
	}
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) {
	if self.v[x] != nn {
	    self.pc += 2;
	}
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
	if self.v[x] == self.v[y] {
	    self.pc += 2;
	}
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
	let result = ((self.v[x] as u16) + (nn as u16)) as u8;
        self.v[x] = result;
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
	self.v[x] = self.v[y];
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
	self.v[x] |= self.v[y];
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
	self.v[x] &= self.v[y];
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
	self.v[x] ^= self.v[y];
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
	let result: u16 = (self.v[x] as u16) + (self.v[y] as u16);
	self.v[x] = result as u8;
	self.v[0xF] = if result > 0xFF {1} else {0};
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
	self.v[0xF] = if self.v[x] > self.v[y] {1} else {0};
	self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    fn op_8xy6(&mut self, x: usize, _y: usize) {
	self.v[0xF] = self.v[x] & 1;
	self.v[x] >>= 1;
    }
    
    fn op_8xy7(&mut self, x: usize, y: usize) {
	self.v[0xF] = if self.v[y] > self.v[x] {1} else {0};
	self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }

    fn op_8xye(&mut self, x: usize, _y: usize) {
	self.v[0xF] = (self.v[x] >> 7) & 1;
	self.v[x] <<= 1;
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
	if self.v[x] != self.v[y] {
	    self.pc += 2;
	}
    }

    fn op_annn(&mut self, nnn: usize) {
        self.i = nnn;
    }

    fn op_bnnn(&mut self, nnn: usize) {
	self.pc = (self.v[0] as usize) + nnn;
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) {
	let mut rng = rand::thread_rng();
	self.v[x] = rng.gen::<u8>() & nn;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        
        let x0 = (self.v[x] as usize) % CHIP_WIDTH;
        let y0 = (self.v[y] as usize) % CHIP_HEIGHT;

        self.v[0xF] = 0;

        for offset in 0..min(CHIP_HEIGHT - y0, n) {
            
            let y = y0 + offset;

            let sprite = self.ram[self.i + offset];

            for bit in 0..min(CHIP_WIDTH - x0, 8) {
                let x = x0 + bit;
                let on: bool = ((sprite >> (7 - bit)) & 1) == 1;
                self.v[0xF] |= (self.vram[y][x] as u8) & (on as u8);
                self.vram[y][x] ^= on;
            }
        }
    }

    fn op_ex9e(&mut self, x: usize) {
	if self.keyboard[self.v[x] as usize] {
	    self.pc += 2;
	}
    }

    fn op_exa1(&mut self, x: usize) {
	if !self.keyboard[self.v[x] as usize] {
	    self.pc += 2;
	}
    }

    fn op_fx07(&mut self, x: usize) {
	self.v[x] = self.delay_timer;
    }

    fn op_fx15(&mut self, x: usize) {
	self.delay_timer = self.v[x];
    }

    fn op_fx18(&mut self, x: usize) {
	self.sound_timer = self.v[x];
    }

    fn op_fx1e(&mut self, x: usize) {
	self.i += self.v[x] as usize;
    }

    fn op_fx0a(&mut self, x: usize) {
	if let Some(key_idx) = self.keyboard.iter().position(|&pressed| pressed) {
	    self.v[x] = key_idx as u8;
	}
	else {
	    self.pc -= 2;
	}
    }

    fn op_fx29(&mut self, x: usize) {
	let font_addr = (5 * self.v[x]) as usize;
	self.i = self.ram[font_addr] as usize;
    }

    fn op_fx33(&mut self, x: usize) {
	let mut num = self.v[x];

	self.ram[self.i] = num / 100;
	num %= 100;
	
	self.ram[self.i + 1] = num / 10;
	num %= 10;
	
	self.ram[self.i + 2] = num;
    }

    
    fn op_fx55(&mut self, x: usize) {
	for reg_idx in 0..=x {
	    self.ram[self.i + reg_idx] = self.v[reg_idx];
	}
    }

    fn op_fx65(&mut self, x: usize) {
	for reg_idx in 0..=x {
	    self.v[reg_idx] = self.ram[self.i + reg_idx];
	}
    }
}

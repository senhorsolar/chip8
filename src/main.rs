extern crate ncurses;

use ncurses::*;

use std::cmp::min;
use std::thread;
use std::time;

use std::fs;
use std::env;


const FONT: [u8; 5*16] = [
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

struct Chip8 {
    ram: [u8; 4096],
    display: [[u8; 64]; 32],
    pc: usize,
    i: usize,
    stack: Vec<u8>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16],
}

impl Chip8 {

    fn new() -> Self {
        // load font
        let mut ram = [0u8; 4096];
        for (i, &byte) in FONT.iter().enumerate() {
            ram[i] = byte;
        }

        Chip8 {
            ram: ram,
            display: [[0u8; 64]; 32],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0u8,
            sound_timer: 0u8,
            v: [0u8; 16],
        }
    }

    fn load_rom(&mut self, bytes: &[u8]) {
        for (i, &byte) in bytes.iter().enumerate() {
            let addr = 0x200 + i;
            if addr >= 4096 {
                break;
            }   
            self.ram[addr] = byte;
        }
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
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x06, _, _, _) => self.op_6xnn(x, nn),
            (0x07, _, _, _) => self.op_7xnn(x, nn),
            (0x0A, _, _, _) => self.op_annn(nnn),
            (0x0D, _, _, _) => self.op_dxyn(x, y, n),
            _ => (),
        }
    }

    fn op_00e0(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.display[y][x] = 0;
            }
        }
    }

    fn op_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
	let result = ((self.v[x] as u16) + (nn as u16)) as u8;
        self.v[x] = result;;
    }

    fn op_annn(&mut self, nnn: usize) {
        self.i = nnn;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        
        let x0 = (self.v[x] as usize) % 64;
        let y0 = (self.v[y] as usize) % 32;

        self.v[0xF] = 0;

        for offset in 0..min(32 - y0, n) {
            
            let y = y0 + offset;

            let sprite = self.ram[self.i + offset];

            for bit in 0..min(64 - x0, 8) {
                let x = x0 + bit;
                let on = (sprite >> (7 - bit)) & 1;
                self.v[0xF] |= self.display[y][x] & on;
                self.display[y][x] ^= on;
            }
        }
    }   
}  

fn main() {

    let sleep_duration = time::Duration::from_millis(5000);

    let args: Vec<String> = env::args().collect();
    let rom_filename = &args[1];
    let rom_bytes = fs::read(rom_filename).expect("Rom file does not exist");
    
    let mut c = Chip8::new();
    c.load_rom(&rom_bytes);

    initscr();
    setlocale(constants::LcCategory::all, "");
    cbreak();
    noecho();

    resize_term(32+4, 64+4);
    refresh();
    
    let window = newwin(32 + 2, 64 + 2, 2, 2);
    

    nodelay(stdscr(), true);
    keypad(stdscr(), true);
    box_(window, 0, 0);
    //waddstr(window, "Hello Rust");
    wrefresh(window);
    
    //println!("Running");
    for i in 0..1000 {
	let opcode = c.fetch();
	c.process(opcode);
	
	for x in 0..64 {
	    for y in 0..32 {
		let ch = match c.display[y][x] {
		    0 => ' ' as u32,
		    _ => ACS_BLOCK()
		};
		mvwaddch(window, (y + 1) as i32, (x + 1) as i32, ch);
	    }
	}
	box_(window, 0, 0);
	wrefresh(window);
    }

    thread::sleep(sleep_duration);
    
    delwin(window);
    endwin();
}

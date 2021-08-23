extern crate ncurses;

mod font;
mod keyboard;
mod screen;
mod chip8;

use keyboard::Keyboard;
use screen::Screen;
use chip8::{Chip8, CHIP_WIDTH, CHIP_HEIGHT};

use ncurses::*;

use std::thread;
use std::time;

use std::fs;
use std::env;

const TWO_MILLIS: time::Duration = time::Duration::from_millis(2);

fn main() {

    // Read ROM
    let args: Vec<String> = env::args().collect();
    let rom_filename = &args[1];
    let rom_bytes = fs::read(rom_filename).expect("Rom file does not exist");
    
    // Setup NCURSES
    let locale_conf = LcCategory::all;
    setlocale(locale_conf, "en_US.UTF-8");
    
    initscr();
    
    cbreak();
    noecho();
    nodelay(stdscr(), true);
    keypad(stdscr(), true);
    
    resize_term(CHIP_HEIGHT as i32 +4, CHIP_WIDTH as i32 +4);
    refresh();
    
    let window = newwin(CHIP_HEIGHT as i32 + 2, CHIP_WIDTH as i32 + 2, 2, 2);
    box_(window, 0, 0);
    wrefresh(window);

    // Components
    let mut chip8 = Chip8::new();
    chip8.load_rom(&rom_bytes);

    let mut keyboard = Keyboard::new();
    let screen = Screen::new(window);
    
    // Start
    keyboard.start_listening();
    
    loop {

	// Exit if 'q' is entered on keyboard
	if keyboard.should_exit() {
	    break;
	}
	
	let start_tick = time::Instant::now();

	// Processor
	chip8.update_keyboard(keyboard.get_pressed());
	chip8.tick();

	// Screen
	screen.update(chip8.get_vram());

	// Sound
	if chip8.should_beep() {
	    beep();
	}
	
	let end_tick = start_tick.elapsed();

	// Try to tick every 2ms
	if end_tick < TWO_MILLIS {
	    let time_delta = TWO_MILLIS - end_tick;
	    thread::sleep(time_delta);
	}
    }

    // Shutdown
    keyboard.stop_listening();
    delwin(window);
    endwin();
}

use ncurses::{ACS_BLOCK, WINDOW, box_, mvwaddch, wrefresh};

use crate::chip8::{CHIP_WIDTH, CHIP_HEIGHT};

pub struct Screen {
    window: WINDOW
}

impl Screen {

    pub fn new(window: WINDOW) -> Self {
	Screen {
	    window: window
	}
    }
    
    pub fn update(&self, vram: [[bool; CHIP_WIDTH]; CHIP_HEIGHT]) {
	for x in 0..CHIP_WIDTH {
	    for y in 0..CHIP_HEIGHT {
		let ch = match vram[y][x] {
		    false => ' ' as u32,
		    true => ACS_BLOCK()
		};
		mvwaddch(self.window, (y + 1) as i32, (x + 1) as i32, ch);
	    }
	}
	box_(self.window, 0, 0);
	wrefresh(self.window);
    }
}

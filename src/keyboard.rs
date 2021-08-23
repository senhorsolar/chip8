extern crate ncurses;

use crate::chip8::{NKEYS, KEYS};

use std::thread;
use std::sync::{Arc, Mutex};
use std::time;
use ncurses::getch;

const KEY_TIME_MS: u128 = 10;

pub struct Keyboard {
    keys: Arc<Mutex<[bool; NKEYS]>>,
    running: Arc<Mutex<bool>>,
    listener: Option<thread::JoinHandle<()>>,
    exit: Arc<Mutex<bool>>
}

impl Keyboard {
    
    pub fn new() -> Self {
	Keyboard {
	    keys: Arc::new(Mutex::new([false; NKEYS])),
	    running: Arc::new(Mutex::new(false)),
	    listener: None,
	    exit: Arc::new(Mutex::new(false))
	}
    }

    pub fn key_to_idx(c: char) -> Option<usize> {
	return KEYS.find(c);
    }

    pub fn is_pressed(&self, idx: usize) -> bool {
	if idx < NKEYS {
	    return self.keys.lock().unwrap()[idx];
	}
	else {
	    return false;
	}
    }

    pub fn get_pressed(&self) -> [bool; NKEYS] {
	let mut keys = [false; NKEYS];
	for idx in 0..NKEYS {
	    keys[idx] = self.is_pressed(idx);
	}
	return keys;
    }
    
    pub fn start_listening(&mut self) {

	let keys = Arc::clone(&self.keys);
	let running = Arc::clone(&self.running);
	let exit = Arc::clone(&self.exit);
	
	*running.lock().unwrap() = true;

	// This thread checks for pressed keys. When it detects a pressed key,
	// it will consider it pressed for at least KEY_TIME_MS. The letter 'q'
	// is for quitting.
	self.listener = Some(thread::spawn(move || {
	    
	    let mut since_key_pressed = [time::Instant::now(); 16];
	    
	    while *running.lock().unwrap() {

		// Reset keys if unpressed for some time
		for i in 0..NKEYS {
		    if keys.lock().unwrap()[i] && since_key_pressed[i].elapsed().as_millis() > KEY_TIME_MS {
			keys.lock().unwrap()[i] = false;
		    }
		}
		
		// Check for pressed keys
		while let (Some(c), true) = (char::from_u32(getch() as u32),
					     *running.lock().unwrap()) {

		    match c {
			'q' => {
			    *exit.lock().unwrap() = true;
			    break;
			}
			_ => {
			    match Keyboard::key_to_idx(c) {
				Some(idx) => {
				    keys.lock().unwrap()[idx] = true;
				    since_key_pressed[idx] = time::Instant::now();
				}
				None => ()
			    };
			}
		    }
		}
	    }
	    
	}));
    }

    pub fn should_exit(&self) -> bool {
	return *self.exit.lock().unwrap();
    }

    pub fn stop_listening(&mut self) {
	*self.running.lock().unwrap() = false;

	if let Some(_) = &self.listener {
	    let _ = self.listener.take().unwrap().join();
	    self.listener = None;
	}
    }
}

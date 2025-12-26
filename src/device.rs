use std::collections::{VecDeque};



pub struct Mouse {
    pub x: u8,
    pub y: u8,
}


impl Mouse {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }
}



/**
 * 
 * Will support alphabet and arrow keys and numbers
 * code:
 * 0b0000_0000: no key
 * 0b0000_0001: a
 * (and so on)
 * 0b0001_1011: UP (27)
 * 0b0001_1100: DOWN (28)
 * 0b0001_1101: LEFT (29)
 * 0b0001_1110: RIGHT (30)
 * 0b0001_1111: ONE (31)
 * 
 * 
 * 
 */
 
pub struct Keyboard {
    queue: VecDeque<u8>, // the u8 is an identifier for one key/character
    // queue acts like a stack; once a key is read it's removed
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn pop_key(&mut self) -> u8 {
        // println!("Popping key...");
        return match self.queue.pop_front() {
            Some(k) => k,
            None => 0b0000_0000,
        }
    }

    pub fn inject_key(&mut self, key: u8) {
        if self.queue.len() == 0 {
            self.queue.push_back(key);
        }
    }

    pub fn status(&mut self) -> u8 {
        let mut status: u8 = 0b0000_0000;
        if self.queue.len() > 0 {
            status = 0b0000_0001;
        }

        // println!("Status: {:08b}", status);

        return status;
    }

    pub fn debug(&mut self) {
        println!("Keys pressed: {:?}", self.queue);
    }
}
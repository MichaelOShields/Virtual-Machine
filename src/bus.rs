
use crate::{Keyboard, Mouse};
use std::ops::Range;

// memory bus
// owns keyboard, mouse, etc
pub struct Bus {
    ram: [u8; 16384],
    mouse: Mouse,
    keyboard: Keyboard,
    mmio_range: Range<u16>,
}



impl Bus {
    pub fn new(mouse: Mouse, keyboard: Keyboard) -> Self {
        Self {
            ram: [0; 16384],
            mouse,
            keyboard,
            mmio_range: 0x1000..0x10FF,
        }
    }

    pub fn get(&mut self, address: u16) -> u8 {

        if self.mmio_range.contains(&address) {
            return self.mmio_get(address);
        }
        return self.ram[address as usize];

    }

    pub fn mmio_get(&mut self, address: u16) -> u8 {
        println!("Getting from MMIO...");
        match address {

            // Keyboard
            0x1000 => {return self.keyboard.status()},
            0x1001 => {println!("Popping key"); return self.keyboard.pop_key();},
            _ => {println!("Unaccounted-for MMIO call"); return 0;},
        };
    }

    pub fn set(&mut self, dest: u16, src: u8) {

        self.ram[dest as usize] = src;

    }

    pub fn get_range(&mut self, a: u16, b: u16) -> &[u8] {
        return &self.ram[a as usize..b as usize];
    }

    pub fn key_inject(&mut self, key: u8) {
        self.keyboard.inject_key(key);
    }

    pub fn status(&mut self) {
        self.keyboard.debug();
    }
}
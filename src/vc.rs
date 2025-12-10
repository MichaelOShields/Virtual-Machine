
/**
 * for 128x128 black/white pixels,
 * 2048 bytes -> 2^11 - 2^12 (2048-4096)
 */

 use crate::memory::Mem;
 use crate::binary::{get_bits_lsb,get_bits_msb};

pub struct VideoController {
    pub width: usize,
    pub height: usize,
    pub framebuffer: Vec<u8>,
    pub vram_base: u16,
}


impl VideoController {
    pub fn new(width: usize, height: usize, vram_base: u16) -> Self {
        Self {
            width,
            height,
            framebuffer: vec![0; (width * height) / 8],
            vram_base,
        }
    }

    pub fn update_framebuffer(&mut self, mem: &[u8]) {
        for i in 0..self.framebuffer.len() {
            self.framebuffer[i] = mem[i];
        }
        // self.print_frame();
    }

    pub fn print_frame(&mut self) {
        println!("{:?}", self.framebuffer);
    }
}
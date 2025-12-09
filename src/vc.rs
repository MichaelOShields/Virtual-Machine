
/**
 * for 128x128 black/white pixels,
 * 2048 bytes -> 2^11 - 2^12 (2048-4096)
 */

pub struct VideoController {
    pub width: usize,
    pub height: usize,
    pub framebuffer: Vec<u8>,
    pub vram_base: u16,
}


impl VideoController {
    pub fn new() -> Self {
        Self {
            width: 1,
            height: 1,
            framebuffer: vec![],
            vram_base: 1,
        }
    }
}
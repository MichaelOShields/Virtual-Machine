


pub struct Mem {
    mem: [u8; 16384],
}



impl Mem {
    pub fn new() -> Self {
        Self {
            mem: [0; 16384],
        }
    }

    pub fn get(&mut self, address: u16) -> u8 {

        return self.mem[address as usize];

    }

    pub fn set(&mut self, dest: u16, src: u8) {

        self.mem[dest as usize] = src;

    }

    pub fn get_range(&mut self, a: u16, b: u16) -> &[u8] {
        return &self.mem[a as usize..b as usize];
    }
}
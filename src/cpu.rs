


pub struct CPU {
    pub regs: [u8; 8],
}

impl CPU {
    pub fn new() -> Self {

        Self {
            regs: [0; 8],
        }

    }


    pub fn status(&self) {
        println!("Registers: {:?}", self.regs)
    }
}
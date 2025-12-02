/*

6 bits opcode
4 bits mode (diff for each instruction)

6 bits register

8 bit operand

*/

#[allow(dead_code)]
struct Flags {
    carry: bool,
    sign: bool,
    zero: bool,
    overflow: bool,
}

#[allow(dead_code)]
pub struct CPU {
    regs: [u8; 8],
    flags: Flags,
}

impl CPU {
    pub fn new() -> Self {
        let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};

        Self {
            regs: [0; 8],
            flags: flags,
        }

    }


    pub fn status(&self) {
        println!("Registers: {:?}", self.regs);
    }
}
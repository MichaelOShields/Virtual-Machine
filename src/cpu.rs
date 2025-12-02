/*

6 bits opcode
4 bits mode (diff for each instruction)

6 bits register

8 bit operand (only for immediate values)


INSTRUCTIONS: dest, src
move (r2 to r1, m2 to r1, r2 to m1)

opcode: 000000

modes:
0000 = r2 to r1
0001 = m2 to r1
0010 = r2 to m1
0011 = i2 to r1
0100 = i2 to m1



INTEGER ARITHMETIC -> dest, src / divisor, dividend

MODES: 
r2 / r1, M2 / r1, r2 / M1, I2 / r1, r2 / I1, M2 / I1

0001 = r2 / r1
0010 = m2 / r1
0011 = r2 / m1
0100 = r2 / i1
0101 = m2 / i1


add 000001
sub 000010
mul 000011
div 000100


example for division: 10 / 2 = 5
storing in r0, taking 10 from R1 and 2 from mem add 0xFFF0-FFF8

DIVISION:

registers:
R0: 0
R1: A

R2: FF
R3: F0



opcode: 000001 
mode (r2 / m1): 0011
register: (mem address) 010 (R1) 001

00000100 11010001

04 D1


MOVING:
opcode: 000000
mode (r2 to r1): 0010

        DEST  SRC
register: 000 001 -> R1 to R0







*/
use crate::memory::mem;



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
    pc: u16,
    sp: u16,
    mem: mem,


}

fn get_bits(number: u16, idx1: u8, idx2: u8) -> u16 {
    let low = idx1.min(idx2);
    let high = idx1.max(idx2);

    let width = high - low + 1;

    (number >> low) & ((1 << width) - 1)
}

impl CPU {
    pub fn new(mem: mem) -> Self {
        let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};

        Self {
            regs: [0; 8],
            flags: flags,
            pc: 0,
            sp: 65535,
            mem: mem,
        }

    }

    fn increment_pc(&mut self, incs: u8) {
        self.pc += incs as u16;
    }

    fn get_operand(&mut self) -> u8 {
        return self.mem.get(self.pc + 2);
    }

    fn op_move(&mut self, mode: u16, reg: u16) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits(reg, 3, 5) as usize];
                let r1 = &mut self.regs[get_bits(reg, 0, 2) as usize];

                *r1 = r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand();
                let r1 = &mut self.regs[get_bits(reg, 0, 2) as usize];


                *r1 = i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            _ => println!("Not accounted for"),
        };
    }   



    pub fn step(&mut self) {
        let instruction: u16 = (((self.mem.get(self.pc) as u16) << 8)) | (self.mem.get(self.pc + 1) as u16);
        let opcode = get_bits(instruction, 0, 5);
        let mode = get_bits(instruction, 6, 9);
        let reg = get_bits(instruction, 10, 15);

        match opcode {
            0_u16 => self.op_move(mode, reg),
            _ => println!("Unaccounted-for operation"),
        }
        
    }


    pub fn status(&self) {
        println!("Registers: {:?}", self.regs);
    }
}
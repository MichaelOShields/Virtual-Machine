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


add 000010
sub 000011
mul 000100
div 000101


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
use crate::bus::Bus;

use crate::binary::{get_bits_lsb, get_bits_msb};



#[allow(dead_code)]
pub struct Flags {
    carry: bool,
    sign: bool,
    zero: bool,
    overflow: bool,
}

#[allow(dead_code)]
pub struct Cpu {
    pub regs: [u8; 8],
    pub flags: Flags,
    pub pc: u16,
    pub sp: u16,
    pub halted: bool,


}

impl Cpu {
    pub fn new() -> Self {
        let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};

        Self {
            regs: [0; 8],
            flags: flags,
            pc: 0,
            sp: 0,
            halted: false,
        }
    }

    fn increment_pc(&mut self, incs: u8) {
        self.pc += incs as u16;
    }

    fn get_operand(&mut self, mem: &mut Bus) -> u8 {
        return mem.get(self.pc + 0x02);
    }

    fn push(&mut self, val: u8, mem: &mut Bus) {
        self.sp -= 1;
        mem.set(self.sp, val);
    }

    fn pop(&mut self, mem: &mut Bus) -> u8 {
        let v = mem.get(self.sp);
        self.sp += 1;
        return v;
    }

    fn op_move(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        // println!("{}", reg);
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize]; // so will add 1 to reg
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address

                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                mem.set(m1, r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        };
    }   


    fn signs_add(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        self.flags.carry = carry;
        self.flags.zero = result == 0;
        self.flags.sign = (result & 0b1000_0000_u8) != 0;

        let a_sign = (a & 0b1000_0000_u8) != 0;
        let b_sign = (b & 0b1000_0000_u8) != 0;
        let result_sign = self.flags.sign;

        self.flags.overflow = (a_sign == b_sign) && (result_sign != a_sign);
    }

    fn signs_sub(&mut self, a: u8, b: u8, result: u8, borrow: bool) {

        self.flags.carry = borrow;
        self.flags.zero = result == 0;
        self.flags.sign = (result & 0b1000_0000_u8) != 0;
        let a_sign = (a & 0b1000_0000_u8) != 0;
        let b_sign = (b & 0b1000_0000_u8) != 0;

        let result_sign = self.flags.sign;
        self.flags.overflow = (a_sign != b_sign) && (result_sign != a_sign);
    }

    fn signs_mul(&mut self, result: u8, overflow: bool) {

        self.flags.carry = overflow;
        self.flags.zero = result == 0;
        self.flags.sign = (result & 0b1000_0000_u8) != 0;
        self.flags.overflow = overflow;
    }


    fn signs_div(&mut self, result: u8) {
        self.flags.carry = false; // no carry
        self.flags.zero = result == 0;
        self.flags.sign = (result & 0b1000_0000_u8) != 0;
        self.flags.overflow = false;
    }

    fn op_add(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = r2;

                let (result, carry) = (*r1).overflowing_add(r2);

                *r1 = result;

                self.signs_add(a, b, result, carry);

                self.increment_pc(2); // increment by # of bytes in instruction

            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize]; // so will add 1 to reg
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = mem.get(m2);
                

                // *r1 = mem.get(m2) + *r1;

                let (result, carry) = (a).overflowing_add(b);


                *r1 = result;


                self.signs_add(a, b, result, carry);



                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                let a = m1val.clone();
                let b = r2;

                let (result, carry) = a.overflowing_add(b);

                mem.set(m1, r2 + m1val);

                self.signs_add(a, b, result, carry);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("{}", i2);
                // println!("reg: {:016b}", reg);
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let a = (*r1).clone();
                let b = i2;

                let (result, carry) = a.overflowing_add(b);


                *r1 = result;

                self.signs_add(a, b, result, carry);


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                let a = m1val.clone();
                let b = i2;

                let (r, c) = a.overflowing_add(b);

                mem.set(m1, r);

                self.signs_add(a, b, r, c);




                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        };

    }


    fn op_sub(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                *r1 = result;

                self.signs_sub(a, b, result, borrow);

                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = mem.get(m2);

                let (result, borrow) = a.overflowing_sub(b);

                *r1 = result;

                self.signs_sub(a, b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let a = mem.get(m1);
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                mem.set(m1, result);

                self.signs_sub(a,b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = *r1;
                let b = i2;
                let (result, borrow) = a.overflowing_sub(b);
                *r1 = result;
                self.signs_sub(a, b, result, borrow);


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                let (result, borrow) = m1val.overflowing_sub(i2);
                mem.set(m1, result);
                self.signs_sub(m1val, i2, result, borrow);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    fn op_div(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let result =  *r1 / r2;
                *r1 = result;
                self.signs_div(result);

                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let result = *r1 / mem.get(m2);
                *r1 = result;
                self.signs_div(result);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                let result = m1val / r2;
                mem.set(m1, result);
                self.signs_div(result);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let result = *r1 / i2;
                *r1 = result;
                self.signs_div(result);


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                let result = m1val / i2;
                mem.set(m1, result);
                self.signs_div(result);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    fn op_mul(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let (result, overflow) = (*r1).overflowing_mul(r2);

                *r1 = result;

                self.signs_mul(result, overflow);
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let (result, overflow) = (*r1).overflowing_mul(mem.get(m2));

                *r1 = result;
                
                self.signs_mul(result, overflow);
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                let (result, overflow) = m1val.overflowing_mul(r2);

                let a = m1val;
                let b = r2;

                self.signs_mul(result, overflow);

                mem.set(m1, result);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let (result, overflow) = (*r1).overflowing_mul(i2);

                *r1 = result;
                    
                self.signs_mul(result, overflow);

                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                let (result, overflow) = m1val.overflowing_mul(i2);
                self.signs_mul(result, overflow);

                mem.set(m1, result);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }



    fn op_mod(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 % r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 % mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                mem.set(m1, m1val % r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 % i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                mem.set(m1, m1val % i2);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }

    fn op_and(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 & r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 & mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                mem.set(m1, m1val & r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 & i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                mem.set(m1, m1val & i2);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    fn op_or(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 | r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 | mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                mem.set(m1, m1val | r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 | i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                mem.set(m1, m1val | i2);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    fn op_xor(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 ^ r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 ^ mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);

                mem.set(m1, m1val ^ r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 ^ i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                mem.set(m1, m1val ^ i2);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }

    fn op_not(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = !r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                // println!("regs: {:016b}", reg);

                // println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                // println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = !mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = mem.get(m1);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = !i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                // let m1val = mem.get(m1);

                mem.set(m1, !i2);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }

    fn single_val(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> u8 {
        match mode {
            0b0000_u16 => {
                // r
                
                let r = self.regs[get_bits_lsb(reg, 3, 5) as usize].clone();

                self.increment_pc(2);
                return r;
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.increment_pc(2);

                return mem.get(m);

            },
            0b0010_u16 => {
                // i
                let i = self.get_operand(mem);

                self.increment_pc(3);

                return i;

            },
            _ => 0b0000_0000_u8,
        }
    }


    fn op_j(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        // println!("Jumping from: {:016b}", self.pc);
        match mode {
            0b0000_u16 => {
                // r
                
                let r11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let r12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (r11 as u16) << 8 | (r12 as u16);

                self.pc = m;
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.pc = (mem.get(m) as u16) << 8 | (mem.get(m + 1) as u16);

            },
            0b0010_u16 => {
                // i
                let i1 = self.get_operand(mem);
                self.increment_pc(1);
                let i2 = self.get_operand(mem);

                let i = (i1 as u16) << 8 | (i2 as u16);

                self.pc = i;
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    fn jump_cond(&mut self, mode: u16, reg: u16, mem: &mut Bus, boolean: bool) {
        if boolean {
            self.op_j(mode, reg, mem);
        }
        else {
            match mode {
                0b0000_u16 => {
                    // r
                    self.increment_pc(2);
                },
                0b0001_u16 => {
                    // m
                    self.increment_pc(2);

                },
                0b0010_u16 => {
                    // i
                    self.increment_pc(4); // jumping code doesn't run so must compensate
                },
                _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
            }
        }
    }


    fn op_jz(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, self.flags.zero);
    }

    fn op_jc(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, self.flags.carry);
    }

    fn op_jo(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, self.flags.overflow);
    }

    fn op_js(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, self.flags.sign);
    }
    fn op_jnz(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, !self.flags.zero);
    }

    fn op_jg(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.jump_cond(mode, reg, mem, !self.flags.zero && !self.flags.sign);
    }

    fn op_jl(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        self.op_js(mode, reg, mem);
    }


    fn op_comp(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a, b, result, borrow);

                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = mem.get(m2);

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a, b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let a = mem.get(m1);
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a,b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = *r1;
                let b = i2;

                let (result, borrow) = (a).overflowing_sub(b);
                
                
                self.signs_sub(a, b, result, borrow);



                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = mem.get(m1);

                let a = m1val;
                let b = i2;

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a, b, result, borrow);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }

    fn op_push(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        let val: u8 = self.single_val(mode, reg, mem);
        self.push(val, mem);
    }

    fn op_call(&mut self, mode: u16, reg: u16, mem: &mut Bus) {

        let pos = self.pc.clone() + match mode {
            0b0000_u16 => 2, // r
            0b0001_u16 => 2, // m
            0b0010_u16 => 4, // i
            _ => 0, // not understood
        };

        self.push((pos >> 8) as u8, mem);
        self.push(pos as u8, mem);

        self.op_j(mode, reg, mem);
    }


    fn op_ret(&mut self, mem: &mut Bus) {
        let m2 = self.pop(mem);
        let m1 = self.pop(mem);

        self.pc = (m1 as u16) << 8 | (m2 as u16);
    }


    fn op_pop(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r1
                let val = self.pop(mem);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = val;

            },
            0b0001_u16 => {
                // m
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m = (m1 as u16) << 8 | (m2 as u16);


                let val = self.pop(mem);
                mem.set(m, val);
            },
            _ => println!("Unaccounted-for mode in pop"),
        }
    }


    fn op_shl(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r
                
                let r = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r <<= 1;

                self.increment_pc(2);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                let unshifted = mem.get(m);

                mem.set(m, unshifted << 1);

                self.increment_pc(2);

            },
            _ => (),
        }
    }


    fn op_shr(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r
                
                let r = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r >>= 1;

                self.increment_pc(2);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                let unshifted = mem.get(m);

                mem.set(m, unshifted >> 1);

                self.increment_pc(2);

            },
            _ => (),
        }
    }

    fn op_sar(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r
                
                let r = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let r_i: i8 = (*r as i8) >> 1;





                *r = r_i as u8;

                self.increment_pc(2);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                let unshifted = mem.get(m);

                let m_i: i8 = (unshifted as i8) >> 1;

                mem.set(m, m_i as u8);

                self.increment_pc(2);

            },
            _ => (),
        }
    }

    fn op_ssp(&mut self, mode: u16, reg: u16, mem: &mut Bus) {
        match mode {
            0b0000_u16 => {
                // r
                
                let r11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let r12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (r11 as u16) << 8 | (r12 as u16);

                self.sp = m;
                self.increment_pc(2);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.sp = (mem.get(m) as u16) << 8 | (mem.get(m + 1) as u16);

                self.increment_pc(2);

            },
            0b0010_u16 => {
                // i
                let i1 = self.get_operand(mem);
                self.increment_pc(1);
                let i2 = self.get_operand(mem);

                let i = (i1 as u16) << 8 | (i2 as u16);

                self.sp = i;
                self.increment_pc(3);
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }

    fn op_skip(&mut self, mode: u16, reg: u16, mem: &mut Bus) { // skip n byte instructions
        // println!("Jumping from: {:016b}", self.pc);
        match mode {
            0b0000_u16 => {
                // r
                
                let r = self.regs[get_bits_lsb(reg, 3, 5) as usize];

                self.increment_pc(2 + r);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.increment_pc(2 + mem.get(m));

            },
            0b0010_u16 => {
                // i
                let i = self.get_operand(mem);

                self.increment_pc(3 + i);
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
    }


    pub fn step(&mut self, mem: &mut Bus) { // 1 for did something, 0 for did nothing
        let instruction: u16 =
        (mem.get(self.pc) as u16) << 8
        | ((mem.get(self.pc + 1) as u16));

        let opcode = get_bits_msb(instruction, 0, 5);
        let mode = get_bits_msb(instruction, 6, 9);
        let reg = get_bits_msb(instruction, 10, 15);


        match opcode {
            0b000000_u16 => {self.increment_pc(1); }, // NO OP
            0b000001_u16 => {self.op_move(mode, reg, mem); }, // MOVE
            0b000010_u16 => {self.op_add(mode, reg, mem); }, // ADD
            0b000011_u16 => {self.op_sub(mode, reg, mem); }, // SUB
            0b000100_u16 => {self.op_mul(mode, reg, mem); }, // MUL
            0b000101_u16 => {self.op_div(mode, reg, mem); }, // DIV
            0b000110_u16 => {self.op_mod(mode, reg, mem); }, // MOD
            0b000111_u16 => {self.op_and(mode, reg, mem); }, // AND
            0b001000_u16 => {self.op_or(mode, reg, mem); }, // OR
            0b001001_u16 => {self.op_xor(mode, reg, mem); }, // XOR
            0b001010_u16 => {self.op_not(mode, reg, mem); }, // NOT
            0b001011_u16 => {self.op_j(mode, reg, mem); }, // JUMP
            0b001100_u16 => {self.op_jz(mode, reg, mem); }, // JUMP Z
            0b001101_u16 => {self.op_jc(mode, reg, mem); }, // JUMP C
            0b001110_u16 => {self.op_jo(mode, reg, mem); }, // JUMP O
            0b001111_u16 => {self.op_js(mode, reg, mem); }, // JUMP S
            0b010000_u16 => {self.op_jnz(mode, reg, mem); }, // JUMP !Z
            0b010001_u16 => {self.op_jg(mode, reg, mem); }, // JUMP >
            0b010010_u16 => {self.op_jl(mode, reg, mem); }, // JUMP <
            0b010011_u16 => {self.op_comp(mode, reg, mem); }, // COMPARE
            0b010100_u16 => {self.op_push(mode, reg, mem); }, // PUSH
            0b010101_u16 => {self.op_pop(mode, reg, mem); }, // POP
            0b010110_u16 => {self.op_call(mode, reg, mem); }, // CALL
            0b010111_u16 => {self.op_ret(mem); }, // RETURN
            0b011000_u16 => {self.op_shl(mode, reg, mem); }, // SHIFT LEFT
            0b011001_u16 => {self.op_shr(mode, reg, mem); }, // LOGICAL SHIFT RIGHT
            0b011010_u16 => {self.op_sar(mode, reg, mem); }, // ARITHMETIC SHIFT RIGHT
            0b011011_u16 => {self.op_ssp(mode, reg, mem); }, // SET STACK POINTER
            0b011100_u16 => {self.op_skip(mode, reg, mem); },
            0b111111_u16 => {self.halted = true; println!("CPU halted")}, // HALT
            _ => {
                println!("Unaccounted-for operation.\nInstruction: {:016b}\nPC: {:x}", instruction, self.pc);
                println!("Halting...");
                for inst in mem.get_range(self.pc - 5, self.pc + 5) {
                    println!("Instruction: {:08b}", inst);

                }
                self.halted = true;
            },
        }

        // self.status();
        
    }


    pub fn status(&self) {
        print!("Registers: [");
        for i in 0..7 {
            print!("{:08b},", self.regs[i]);
        }
        println!("{:08b}]", self.regs[7]);
        if self.halted {
            println!("Halted");
        }
    }
}
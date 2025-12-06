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
use crate::memory::Mem;



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
    mem: Mem,


}

fn get_bits_msb(number: u16, idx1: u8, idx2: u8) -> u16 {
    let msb_low = idx1.min(idx2);
    let msb_high = idx1.max(idx2);

    let lsb_low  = 15 - msb_high;
    let lsb_high = 15 - msb_low;

    let width = lsb_high - lsb_low + 1;

    (number >> lsb_low) & ((1 << width) - 1)
}

fn get_bits_lsb(number: u16, idx1: u8, idx2: u8) -> u16 {
    let lsb_low  = idx1.min(idx2);
    let lsb_high = idx1.max(idx2);

    let width = lsb_high - lsb_low + 1;

    (number >> lsb_low) & ((1 << width) - 1)
}

impl CPU {
    pub fn new(mem: Mem) -> Self {
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
        return self.mem.get(self.pc + 0x02);
    }

    fn op_move(&mut self, mode: u16, reg: u16) {
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

                *r1 = self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address

                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                self.mem.set(m1, r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                self.mem.set(m1, i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        };
    }   


    #[allow(unused_assignments)]
    fn signs_add(&mut self, a: u8, b: u8, result: u8, carry: bool) {
        let mut set_carry = self.flags.carry;
        let mut set_zero = self.flags.zero;
        let mut set_sign = self.flags.sign;
        let mut set_over = self.flags.overflow;

        set_carry = carry;
        set_zero = result == 0;
        set_sign = (result & 0b1000_0000_u8) != 0;

        let a_sign = (a & 0b1000_0000_u8) != 0;
        let b_sign = (b & 0b1000_0000_u8) != 0;

        let result_sign = set_sign;

        set_over = (a_sign == b_sign) && (result_sign != a_sign);


        self.flags.carry = set_carry;
        self.flags.zero = set_zero;
        self.flags.sign = set_sign;
        self.flags.overflow = set_over;

    }

    #[allow(unused_assignments)]
    fn signs_sub(&mut self, a: u8, b: u8, result: u8, borrow: bool) {
        let mut set_carry = self.flags.carry;
        let mut set_zero = self.flags.zero;
        let mut set_sign = self.flags.sign;
        let mut set_over = self.flags.overflow;

        set_carry = borrow;
        set_zero = result == 0;
        set_sign = (result & 0b1000_0000_u8) != 0;

        let a_sign = (a & 0b1000_0000_u8) != 0;
        let b_sign = (b & 0b1000_0000_u8) != 0;

        let result_sign = set_sign;

        set_over = (a_sign != b_sign) && (result_sign != a_sign);


        self.flags.carry = set_carry;
        self.flags.zero = set_zero;
        self.flags.sign = set_sign;
        self.flags.overflow = set_over;

    }


    #[allow(unused_assignments)]
    fn signs_div(&mut self, a: u8, b: u8, result: u8, borrow: bool) {
        let mut set_carry = self.flags.carry;
        let mut set_zero = self.flags.zero;
        let mut set_sign = self.flags.sign;
        let mut set_over = self.flags.overflow;

        set_carry = borrow;
        set_zero = result == 0;
        set_sign = (result & 0b1000_0000_u8) != 0;

        let a_sign = (a & 0b1000_0000_u8) != 0;
        let b_sign = (b & 0b1000_0000_u8) != 0;

        let result_sign = set_sign;

        set_over = (a_sign != b_sign) && (result_sign != a_sign);


        self.flags.carry = set_carry;
        self.flags.zero = set_zero;
        self.flags.sign = set_sign;
        self.flags.overflow = set_over;

    }

    fn op_add(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();
                let b = self.mem.get(m2);
                

                // *r1 = self.mem.get(m2) + *r1;

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

                let m1val = self.mem.get(m1);

                let a = m1val.clone();
                let b = r2;

                let (result, carry) = a.overflowing_add(b);

                self.mem.set(m1, r2 + m1val);

                self.signs_add(a, b, result, carry);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                // println!("{}", i2);
                // println!("reg: {:016b}", reg);
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let a = (*r1).clone();
                let b = i2;

                let (result, carry) = a.overflowing_add(b);


                *r1 = i2 + *r1;

                self.signs_add(a, b, result, carry);


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                let a = m1val.clone();
                let b = i2;

                let (r, c) = a.overflowing_add(b);

                self.mem.set(m1, r);

                self.signs_add(a, b, r, c);




                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        };

    }


    fn op_sub(&mut self, mode: u16, reg: u16) {
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
                let b = self.mem.get(m2);

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

                let a = self.mem.get(m1);
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                self.mem.set(m1, result);

                self.signs_sub(a,b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 - i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val - i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }


    fn op_div(&mut self, mode: u16, reg: u16) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 / r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 / self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val / r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 / i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val / i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }


    fn op_mul(&mut self, mode: u16, reg: u16) {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 * r2;
                self.increment_pc(2); // increment by # of bytes in instruction
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 * self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val * r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 * i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val * i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }



    fn op_mod(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 % self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val % r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 % i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val % i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }

    fn op_and(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 & self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val & r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 & i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val & i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }


    fn op_or(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 | self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val | r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 | i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val | i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }


    fn op_xor(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 ^ self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val ^ r2);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 ^ i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val = self.mem.get(m1);

                self.mem.set(m1, m1val ^ i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }

    fn op_not(&mut self, mode: u16, reg: u16) {
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

                println!("regs: {:016b}", reg);

                println!("r1: {}", self.regs[get_bits_lsb(reg, 0, 2) as usize]);
                println!("m2: {}", m2);

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = !self.mem.get(m2);
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val = self.mem.get(m1);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = !i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(); // 1 byte; got operand, so adds 1 byte to full instruction
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                // let m1val = self.mem.get(m1);

                self.mem.set(m1, !i2);

                self.increment_pc(3);

            },
            _ => println!("Not accounted for"),
        }
    }


    pub fn step(&mut self) -> bool { // 1 for did something, 0 for did nothing
        let instruction: u16 =
        (self.mem.get(self.pc) as u16) << 8
        | ((self.mem.get(self.pc + 1) as u16));

        let opcode = get_bits_msb(instruction, 0, 5);
        let mode = get_bits_msb(instruction, 6, 9);
        let reg = get_bits_msb(instruction, 10, 15);


        return match opcode {
            0_u16 => false, // do nothing
            0b000001_u16 => {self.op_move(mode, reg); return true;}, // MOVE
            0b000010_u16 => {self.op_add(mode, reg); return true;}, // ADD
            0b000011_u16 => {self.op_sub(mode, reg); return true;}, // SUB
            0b000100_u16 => {self.op_mul(mode, reg); return true;}, // MUL
            0b000101_u16 => {self.op_div(mode, reg); return true;}, // DIV
            0b000110_u16 => {self.op_mod(mode, reg); return true;}, // MOD
            0b000111_u16 => {self.op_and(mode, reg); return true;}, // AND
            0b001000_u16 => {self.op_or(mode, reg); return true;}, // OR
            0b001001_u16 => {self.op_xor(mode, reg); return true;}, // XOR
            0b001010_u16 => {self.op_not(mode, reg); return true;}, // NOT
            _ => {println!("Unaccounted-for operation"); return false;},
        }
        
    }


    pub fn status(&self) {
        println!("Registers: {:?}", self.regs);
    }
}
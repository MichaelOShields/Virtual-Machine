
use crate::bus::Bus;

use crate::binary::{get_bits_lsb, get_bits_msb};




struct DoubleVal<'a> {
    a: &'a mut u8,
    b: u8,
}

#[allow(dead_code)]
pub struct Flags {
    carry: bool,
    sign: bool,
    zero: bool,
    overflow: bool,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Access { // reading, writing, executing
    R, // read
    W, // write
    X, // execute
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CPUMode {
    K, // Kernel
    U, // User
}

#[derive(PartialEq, Debug)]
pub enum Fault {
    IllegalInstruction, //
    IllegalMemAccess, // 
    UnknownAction, //
}

#[derive(PartialEq, Debug)]
pub enum CPUExit {
    None,  // idk default error
    Timer, // when we need to return control to kernel
    Halt, // Kill current program; 
    Syscall, // R0-R3 give information re which syscall;
    Fault ( Fault ), // Something went wrong, probably w/ permissions; 
}


#[allow(dead_code)]
pub struct Cpu {
    pub regs: [u8; 8],
    pub flags: Flags,
    pub pc: u16,
    pub sp: u16, // current sp
    pub halted: bool,
    pub mode: CPUMode,
    pub access: Access,
    pub instruction_ctr: u16,
    pub instruction_lim: u16,

    pub kernel_trap_address: u16,




}

impl Cpu {
    pub fn new(trap_addr: u16) -> Self {
        let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};

        Self {
            regs: [0; 8],
            flags: flags,
            pc: 0,
            sp: 0,
            halted: false,
            mode: CPUMode::K,
            access: Access::X,
            instruction_ctr: 0,
            instruction_lim: 50, // allow 50 instructions before returning control
            kernel_trap_address: trap_addr,
        }
    }

    fn increment_pc(&mut self, incs: u8) {
        self.pc += incs as u16;
    }

    fn force_get_operand(&mut self, mem: &mut Bus) -> u8 {
        let result = mem.force_get(self.pc + 0x02);
        // self.access = Access::X;
        result
    }

    fn get_operand(&mut self, mem: &mut Bus) -> Result<u8, CPUExit> {
        let result = self.memget(self.pc + 2, mem)?;
        self.access = Access::X;
        Ok(result)
    }
    // comment so i can commit an idea

    fn push(&mut self, val: u8, mem: &mut Bus) -> Result<(), CPUExit> {
        // self.debug(mem);


        

        self.memset(self.sp, val, mem)?;

        self.sp -= 1;

        // match self.mode {
        //     CPUMode::K => self.ksp = self.sp,
        //     CPUMode::U => self.usp = self.sp,
        // }
        Ok(())
    }

    fn pop(&mut self, mem: &mut Bus) -> Result<u8, CPUExit> {

        self.sp += 1;

        let v = self.memget(self.sp, mem)?;

        // match self.mode {
        //     CPUMode::K => self.ksp = self.sp,
        //     CPUMode::U => self.usp = self.sp,
        // }
        Ok(v)
    }

    fn double_val<'a>(&'a mut self, mode: u16, reg: u16, mem: &'a mut Bus) -> Result<DoubleVal<'a>, CPUExit> {
        return Ok(match mode {
            0_u16 => {
                // r2 to r1
                
                // dest src
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                self.increment_pc(2); // increment by # of bytes in instruction
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

            

                DoubleVal { a: r1, b: r2 }
            },
            0b0001_u16 => {
                // m2 to r1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m21 = self.regs[get_bits_lsb(reg, 0, 2) as usize]; // so will add 1 to reg
                let m22 = self.regs[(get_bits_lsb(reg, 0, 2) + 1) as usize];

                let m2: u16 = (m21 as u16) << 8 | (m22 as u16); // memory address

                let m2val = self.memget(m2, mem)?;

                self.increment_pc(2);


                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

            

                DoubleVal { a: r1, b: m2val }

            },
            0b0010_u16 => {
                // r2 to m1

                // mem loc stored as r0:r1 or r1:r2 etc
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address

                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                self.increment_pc(2);

                
                
                DoubleVal { a: self.memgetmutable(m1, mem)?, b: r2 }

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                self.increment_pc(3); // uses operand -> 3 bytes
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                

                DoubleVal { a: r1, b: i2 }
            },
            0b0100_u16 => {
                // m(i)2 to r1
            
                // dest src

                let i21 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i22 = self.get_operand(mem)?;
                self.increment_pc(1);

                let i = (i21 as u16) << 8 | (i22 as u16);

                let mval = self.memget(i, mem)?;

                self.increment_pc(2); // uses operand -> 3 bytes

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                

                DoubleVal { a: r1, b: mval }
            },
            0b0101_u16 => {
                // r2 to m(i)1
                // dest src

                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];


                let i11 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i12 = self.get_operand(mem)?;
                self.increment_pc(1);

                let i = (i11 as u16) << 8 | (i12 as u16);

                self.increment_pc(2); // uses operand -> 3 bytes

                DoubleVal { a: self.memgetmutable(i, mem)?, b: r2 }
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc); return Err(CPUExit::Fault(Fault::UnknownAction))},
        });
        // Err(CPUExit::Fault(Fault::UnknownAction))
    }



    fn op_mov(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        // println!("{}", reg);
        let DoubleVal { a, b} = self.double_val(mode, reg, mem)?;
        *a = b;
        
        Ok(())
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

    fn op_add(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        let DoubleVal { a, b} = self.double_val(mode, reg, mem)?;
        let (result, carry) = (*a).overflowing_add(b);
        *a = result;
        let aclone = (*a).clone();
        self.signs_add(aclone, b, result, carry);
        Ok(())
    }


    fn op_sub(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        let DoubleVal { a, b} = self.double_val(mode, reg, mem)?;
        let (result, borrow) = (*a).overflowing_sub(b);
        *a = result;
        let aclone = (*a).clone();
        self.signs_sub(aclone, b, result, borrow);
        Ok(())
    }


    fn op_div(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let result = *r1 / m2val;
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

                let m1val: u8 = self.memget(m1, mem)?;

                let result = m1val / r2;
                self.memset(m1, result, mem)?;
                self.signs_div(result);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
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

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                let result = m1val / i2;
                self.memset(m1, result, mem)?;
                self.signs_div(result);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }


    fn op_mul(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let (result, overflow) = (*r1).overflowing_mul(m2val);

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

                let m1val: u8 = self.memget(m1, mem)?;

                let (result, overflow) = m1val.overflowing_mul(r2);

                let a = m1val;
                let b = r2;

                self.signs_mul(result, overflow);

                self.memset(m1, result, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let (result, overflow) = (*r1).overflowing_mul(i2);

                *r1 = result;
                    
                self.signs_mul(result, overflow);

                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                let (result, overflow) = m1val.overflowing_mul(i2);
                self.signs_mul(result, overflow);

                self.memset(m1, result, mem)?;

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }



    fn op_mod(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        match mode {
            0_u16 => {
                // r2 to r1
                // dest src
                
                
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let result = *r1 % r2;

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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let result = *r1 % m2val;

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

                let m1val: u8 = self.memget(m1, mem)?;

                let result = m1val % r2;

                self.signs_div(result);

                self.memset(m1, result, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                let result = *r1 % i2;

                *r1 = result;

                self.signs_div(result);


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn op_and(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 & m2val;
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val & r2, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 & i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val & i2, mem)?;

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }


    fn op_or(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 | m2val;
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val | r2, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 | i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val | i2, mem)?;

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }


    fn op_xor(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = *r1 ^ m2val;
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val ^ r2, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = *r1 ^ i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, m1val ^ i2, mem)?;

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn op_not(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let m2val: u8 = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = !m2val;
                
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // r2 to m1
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1: u16 = (m11 as u16) << 8 | (m12 as u16); // memory address
                let r2 = self.regs[get_bits_lsb(reg, 0, 2) as usize];



                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, !r2, mem)?;
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
                // println!("r1: {:016b}", self.regs[get_bits_lsb(reg, 3, 5) as usize]);
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];


                *r1 = !i2;


                self.increment_pc(3); // uses operand -> 3 bytes
            },
            0b0100_u16 => {
                // i2 to m1
                // dest src

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                self.memset(m1, !i2, mem)?;

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn single_val(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<u8, CPUExit> {
        match mode {
            0b0000_u16 => {
                // r
                
                let r = self.regs[get_bits_lsb(reg, 3, 5) as usize];

                self.increment_pc(2);
                Ok(r)
            },
            0b0001_u16 => {
                // m(r)

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.increment_pc(2);

                Ok(self.memget(m, mem)?)

            },
            0b0010_u16 => {
                // i
                let i = self.get_operand(mem)?;
                self.increment_pc(1);

                self.increment_pc(2);

                Ok(i)

            },
            0b0011_u16 => {
                // m(i)
                let i1 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i2 = self.get_operand(mem)?;
                self.increment_pc(1);

                let m: u16 = (i1 as u16) << 8 | (i2 as u16); // memory address

                let mval = self.memget(m, mem)?;

                self.increment_pc(2);

                Ok(mval)
            },
            _ => Err(CPUExit::Fault(Fault::UnknownAction))
        }
    }


    fn single_val_addr(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<u16, CPUExit> {
        return match mode {
            0b0000_u16 => {
                // r
                
                let r11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let r12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (r11 as u16) << 8 | (r12 as u16);

                self.increment_pc(2);

                Ok(m)
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let r11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let r12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (r11 as u16) << 8 | (r12 as u16);

                let mval = (self.memget(m, mem)? as u16) << 8 | (self.memget(m + 1, mem)? as u16);

                self.increment_pc(2);

                Ok(mval)

            },
            0b0010_u16 => {
                // i
                let i1 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i2 = self.get_operand(mem)?;
                self.increment_pc(1);

                let i = (i1 as u16) << 8 | (i2 as u16);

                // match self.mode {
                //     CPUMode::K => self.ksp = i,
                //     CPUMode::U => self.usp = i,
                // }

                self.increment_pc(2);

                Ok(i)
            },
            0b0011_u16 => {
                // m(i)
                let i1 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i2 = self.get_operand(mem)?;
                self.increment_pc(1);

                let m: u16 = (i1 as u16) << 8 | (i2 as u16); // memory address

                let mval1 = self.memget(m, mem)?;
                let mval2 = self.memget(m + 1, mem)?;

                self.increment_pc(2);

                let final_mem = (mval1 as u16) << 8 | (mval2 as u16);

                Ok(final_mem)
            },
            _ => {println!("Not accounted-for mode: {:04b}", mode); println!("pc: {:0x}", self.pc); Err(CPUExit::Fault(Fault::UnknownAction))},
        }
    }


    fn op_j(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                self.pc = (self.memget(m, mem)? as u16) << 8 | (self.memget(m + 1, mem)?) as u16;

            },
            0b0010_u16 => {
                // i
                let i1 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i2 = self.get_operand(mem)?;

                let i = (i1 as u16) << 8 | (i2 as u16);

                self.pc = i;
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }


    fn jump_cond(&mut self, mode: u16, reg: u16, mem: &mut Bus, boolean: bool) -> Result<(), CPUExit> {
        if boolean {
            self.op_j(mode, reg, mem)?;
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
        Ok(())
    }


    fn op_jz(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, self.flags.zero)?)
    }

    fn op_jc(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, self.flags.carry)?)
    }

    fn op_jo(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, self.flags.overflow)?)
    }

    fn op_js(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, self.flags.sign)?)
    }
    fn op_jnz(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, !self.flags.zero)?)
    }

    fn op_jg(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.jump_cond(mode, reg, mem, !self.flags.zero && !self.flags.sign)?)
    }

    fn op_jl(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        Ok(self.op_js(mode, reg, mem)?)
    }


    fn op_cmp(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let b = self.memget(m2, mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                let a = (*r1).clone();

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

                let a = self.memget(m1, mem)?;
                let b = r2;

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a,b, result, borrow);
                
                self.increment_pc(2);

            },
            0b0011_u16 => {
                // i2 to r1
                // dest src

                let i2 = self.get_operand(mem)?;
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

                let i2 = self.get_operand(mem)?;
                let m11 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m12 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m1 = (m11 as u16) << 8 | (m12 as u16);

                let m1val: u8 = self.memget(m1, mem)?;

                let a = m1val;
                let b = i2;

                let (result, borrow) = a.overflowing_sub(b);

                self.signs_sub(a, b, result, borrow);

                self.increment_pc(3);

            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn op_push(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        let val: u8 = self.single_val(mode, reg, mem)?;    

        self.push(val, mem)?;
        Ok(())
    }

    fn op_call(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {

        let pos = self.pc + match mode {
            0b0000_u16 => 2, // r
            0b0001_u16 => 2, // m
            0b0010_u16 => 4, // i
            _ => return Err(CPUExit::Fault(Fault::UnknownAction)), // not understood
        };


        // println!("SP: {:?}", self.sp);
        let p1 = (pos >> 8) as u8;
        let p2 = pos as u8;

        // panic!("p1: {:08b}\np2: {:08b}\npos:{:016b}", p1, p2, pos);


        self.push(p1, mem)?;
        self.push(p2, mem)?;
        // println!("SP (from call): 0x{:0x}", self.sp);


        // print stack

        Ok(self.op_j(mode, reg, mem)?)
    }


    fn op_ret(&mut self, mem: &mut Bus) -> Result<(), CPUExit> {
        let m2 = self.pop(mem)?;
        let m1 = self.pop(mem)?;

        // println!("m{}", (m1 as u16) << 8 | (m2 as u16));


        self.pc = (m1 as u16) << 8 | (m2 as u16);
        // println!("sp{}", self.pc);
        // panic!();

        Ok(())
    }


    fn op_sys(&mut self, mem: &mut Bus) -> Result<(), CPUExit> {

        // 0b0000_0000: resume quick
        // 0b0000_0001: get key
        self.increment_pc(1);

        // println!("-----------------------------------------------------------------------------------------------------------------------");

        return Err(CPUExit::Syscall);
    }

    fn op_kret(&mut self, mem: &mut Bus) -> Result<(), CPUExit> { // KERNEL RETURN; return after syscall/exit

        // self.access = Access::X;


        let pc2 = self.pop(mem)?;
        let pc1 = self.pop(mem)?;
        // println!("pc2: {:0x}", pc2);
        // println!("pc1: {:0x}", pc1);
        
        // panic!();
        let new_pc = (pc1 as u16) << 8 | (pc2 as u16);
        // println!("new pc: 0x{:x}", new_pc);
        self.pc = new_pc;

        self.mode = CPUMode::U;

        self.access = Access::X;


        // println!("-----------------------------------------------------------------------------------------------------------------------");



        Ok(())

    }


    fn op_pop(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        match mode {
            0b0000_u16 => {
                // r1
                let val = self.pop(mem)?;

                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];

                *r1 = val;
                self.increment_pc(2);

            },
            0b0001_u16 => {
                // m
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize]; // so will add 1 to reg
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m = (m1 as u16) << 8 | (m2 as u16);


                let val = self.pop(mem)?;
                self.memset(m, val, mem)?;
                self.increment_pc(2);
            },
            _ => println!("Unaccounted-for mode in pop"),
        }
        Ok(())
    }


    fn op_shl(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let unshifted = self.memget(m, mem)?;

                self.memset(m, unshifted << 1, mem)?;

                self.increment_pc(2);

            },
            _ => (),
        }
        Ok(())
    }


    fn op_shr(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        let DoubleVal { a, b} = self.double_val(mode, reg, mem)?;
        *a >>= b;

        Ok(())
    }

    fn op_shrw(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        let DoubleVal { a, b} = self.double_val(mode, reg, mem)?;
        *a = (*a).rotate_right(b as u32) as u8;

        Ok(())
    }

    fn op_sar(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
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

                let unshifted = self.memget(m, mem)?;

                let m_i: i8 = (unshifted as i8) >> 1;

                self.memset(m, m_i as u8, mem)?;

                self.increment_pc(2);

            },
            _ => (),
        }
        Ok(())
    }

    fn setsp(&mut self, addr: u16, mem: &mut Bus) {

    }

    fn op_ssp(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        self.sp = self.single_val_addr(mode, reg, mem)?;
        // self.debug(mem);
        // panic!();

        // self.debug(mem);
        Ok(())


    }

    fn op_skip(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> { // skip n byte instructions
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
                let mval = self.memget(m, mem)?;

                self.increment_pc(2 + mval);

            },
            0b0010_u16 => {
                // i
                let i = self.get_operand(mem)?;

                self.increment_pc(3 + i);
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn op_gsp(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        // println!("Jumping from: {:016b}", self.pc);
        let s1 = get_bits_msb(self.sp, 0, 7) as u8;
        let s2 = get_bits_msb(self.sp, 8, 15) as u8;
        match mode {
            0b0000_u16 => {
                // r
                
                let r1 = &mut self.regs[get_bits_lsb(reg, 3, 5) as usize];
                *r1 = s1;
                let r2 = &mut self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];
                *r2 = s2;


                self.increment_pc(2);
            },
            0b0001_u16 => {
                // m

                // mem loc stored as r0:r1 or r1:r2 etc
                let m1 = self.regs[get_bits_lsb(reg, 3, 5) as usize];
                let m2 = self.regs[(get_bits_lsb(reg, 3, 5) + 1) as usize];

                let m: u16 = (m1 as u16) << 8 | (m2 as u16); // memory address

                self.memset(m, s1, mem)?;
                self.memset(m + 1, s2, mem)?;
                self.increment_pc(2);

            },
            0b0010_u16 => {
                // i
                let i1 = self.get_operand(mem)?;
                self.increment_pc(1);
                let i2 = self.get_operand(mem)?;

                let m = (i1 as u16) << 8 | (i2 as u16);

                self.memset(m, s1, mem)?;
                self.memset(m + 1, s2, mem)?;

                self.increment_pc(3);
            },
            _ => {println!("Not accounted-for mode"); println!("pc: {:0x}", self.pc);},
        }
        Ok(())
    }

    fn op_gfls(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {


        // in future, can condense to store all flags in 1 byte; currently too lazy lol

        let addr = self.single_val_addr(mode, reg, mem)?;

        // let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};
        self.memset(addr, self.flags.carry as u8, mem)?;
        self.memset(addr + 1, self.flags.sign as u8, mem)?;
        self.memset(addr + 2, self.flags.zero as u8, mem)?;
        self.memset(addr + 3, self.flags.overflow as u8, mem)?;

        Ok(())
    }

    fn op_sfls(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {


        // in future, can condense to store all flags in 1 byte; currently too lazy lol

        let address = self.single_val_addr(mode, reg, mem)?;

        // let flags = Flags {carry: false, sign: false, zero: false, overflow: false,};
        self.flags.carry = self.memget(address, mem)? != 0;
        self.flags.sign = self.memget(address + 1, mem)? != 0;
        self.flags.zero = self.memget(address + 2, mem)? != 0;
        self.flags.overflow = self.memget(address + 3, mem)? != 0;

        Ok(())
    }

    fn op_pnk(&mut self, mem: &mut Bus) {
        panic!("Panicked @ request.");
    }

    fn op_dbg(&mut self, mode: u16, reg: u16, mem: &mut Bus) -> Result<(), CPUExit> {
        println!("DEBUG:");
        self.status();
        self.debug(mem);
        let val = self.single_val(mode, reg, mem)?;
        println!("Debug num: {}", val);
        Ok(())
    }
    


    pub fn debug(&mut self, mem: &mut Bus) {
        println!("SP: 0x{:0x}", self.sp);
        println!("Next 5 stack items: ");
        for (i,  num) in mem.get_range(self.sp + 1, self.sp + 6).iter().enumerate() {
            println!("{} (0x{:0x}): 0b{:08b}", i + 1, self.sp + i as u16, num);
        }
        println!("PC: 0x{:0x}", self.pc);
        println!("Mode: {:?}", self.mode);
        println!("Access: {:?}", self.access);
        println!("Instruction: {:08b}", mem.force_get(self.pc));
        let size = mem.get_size() as i64;
        // println!("size: {}", size);
        let range = 0;
        let mut start = self.pc as i64 - range;
        if start < 0 {
            start = 0;
        };
        let mut end = self.pc as i64 + range;
        if end > size {
            end = size;
        }
        for inst in mem.get_range(start as u16, end as u16) {
            println!("Instruction: {:08b}", inst);
        }
        self.status();
    }

    fn handle_exit(&mut self, exit: CPUExit, mem: &mut Bus) {

        if !self.halted {

            self.mode = CPUMode::K;

            self.access = Access::X;

            let exit_id: u8 = match exit {
                CPUExit::None => 0b0000,
                CPUExit::Timer => 0b0001,
                CPUExit::Halt => 0b0010,
                CPUExit::Syscall => 0b0011,
                CPUExit::Fault(ref f) => match f {
                    Fault::IllegalInstruction => 0b0100,
                    Fault::IllegalMemAccess => 0b0101,
                    Fault::UnknownAction => 0b0110,
                }
            };

            let pc1: u8 = get_bits_msb(self.pc, 0, 7) as u8;
            let pc2: u8 = get_bits_msb(self.pc, 8, 15) as u8;

            // println!("pc: {:0x}", self.pc);
            // println!("SP: {:0x}", self.sp);


            // save exit pc
            

            match self.push(pc2, mem) {
                Ok(()) => (),
                Err(e) => {
                    println!("push failed w/ exit {:?}", e);
                    // self.debug(mem);
                    panic!();
                }
            };
            match self.push(pc1, mem)  {
                Ok(()) => (),
                Err(_e) => {panic!("push failed");}
            };


            // push exit reason
            match self.memset(0x125A, exit_id, mem) {
                Ok(()) => (),
                Err(_e) => {println!("memset (exit id) failed"); return;}
            };

            // println!("0x125A: {:08b}", mem.force_get(0x125A));
            //println!("Error: {:?}", exit);
            //println!("-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------");

            self.pc = self.kernel_trap_address;
        }
        else {
            panic!("Tried to handle exit but full halted");
        }


    }

    fn memget(&mut self, address: u16, mem: &mut Bus) -> Result<u8, CPUExit> {
        let orig_access = self.access;
        self.access = Access::R;
        let result = self.memgetcore(address, mem)?;
        self.access = orig_access;
        Ok(result)
    }

    fn memgetmutable<'a>(&mut self, address: u16, mem: &'a mut Bus) -> Result<&'a mut u8, CPUExit> {
        // self.access = Access::R;
        Ok(mem.get_mutable_ref(address, self.mode, Access::W)?)
    }   

    fn memgetcore(&mut self, address: u16, mem: &mut Bus) -> Result<u8, CPUExit> {
        Ok(mem.get(address, self.mode, self.access)?)
    }

    fn memset(&mut self, dest: u16, src: u8, mem: &mut Bus) -> Result<(), CPUExit> {
        self.access = Access::W;
        Ok(mem.set(dest, src, self.mode, self.access)?)

    }

    pub fn step(&mut self, mem: &mut Bus) {
        self.access = Access::X;
        // self.debug(mem);
        match self.act(mem) {
            Ok(()) => (),
            Err(e) => self.handle_exit(e, mem),
        }
    }

    fn op_halt(&mut self, mem: &mut Bus) -> Result<(), CPUExit> {
        match self.mode {
            CPUMode::K => {
                self.halted = true;
                println!("Halting...");
                self.debug(mem);
                Ok(())
            },
            CPUMode::U => return Err(CPUExit::Halt),
        }
    }

    fn convert_instruction(&mut self, op: u16) -> String {
        match op {
            0b000_000 => "nop",
            0b000_001 => "mov",
            0b000_010 => "add",
            0b000_011 => "sub",
            0b000_100 => "mul",
            0b000_101 => "div",
            0b000_110 => "mod",
            0b000_111 => "and",
            0b001_000 => "or",
            0b001_001 => "xor",
            0b001_010 => "not",
            0b001_011 => "jmp",
            0b001_100 => "jz",
            0b001_101 => "jc",
            0b001_110 => "jo",
            0b001_111 => "js",
            0b010_000 => "jnz",
            0b010_001 => "jg",
            0b010_010 => "jl",
            0b010_011 => "cmp",
            0b010_100 => "push",
            0b010_101 => "pop",
            0b010_110 => "call",
            0b010_111 => "ret",
            0b011_000 => "shl",
            0b011_001 => "shr",
            0b011_010 => "sar",
            0b011_011 => "ssp",
            0b011_100 => "skip",
            0b011_101 => "sys",
            0b011_110 => "kret",
            0b011_111 => "gsp",
            0b100_000 => "pnk",
            0b100_001 => "dbg",
            0b100_010 => "shrw",
            0b100_011 => "gfls",
            0b100_100 => "sfls",
            0b111_111 => "hlt",
            _ => panic!("Received invalid instruction {:08b}", op),
        }.to_string()

    }


    fn act(&mut self, mem: &mut Bus) -> Result<(), CPUExit> { // 1 for did something, 0 for did nothing


        self.access = Access::X;

        let instruction1 = self.memgetcore(self.pc, mem)?;
        let instruction2 = self.memgetcore(self.pc + 1, mem)?;



        let instruction: u16 =
        (instruction1 as u16) << 8
        | (instruction2 as u16);

        let opcode = get_bits_msb(instruction, 0, 5);
        let mode = get_bits_msb(instruction, 6, 9);
        let reg = get_bits_msb(instruction, 10, 15);


        // println!("\n\n\nInstruction mnemonic: {}", self.convert_instruction(opcode));
        // println!("Instruction: 0b{:08b}_{:08b}\nPC: 0x{:0x}\nSP: 0x{:0x}\nMode: {:?}\nAccess:{:?}", instruction1, instruction2, self.pc, self.sp, self.mode, self.access);
        // mem.status();
        // self.status();


        if self.mode == CPUMode::U {

            match opcode {
                0b011011_u16 => return Err(CPUExit::Fault(Fault::IllegalInstruction)), // ssp
                0b011110_u16 => return Err(CPUExit::Fault(Fault::IllegalInstruction)), // kret
                _ => (),
            }
        }

        


        match opcode {
            0b000000_u16 => {self.increment_pc(1); }, // NO OP
            0b000001_u16 => {self.op_mov(mode, reg, mem)?; }, // MOVE
            0b000010_u16 => {self.op_add(mode, reg, mem)?; }, // ADD
            0b000011_u16 => {self.op_sub(mode, reg, mem)?; }, // SUB
            0b000100_u16 => {self.op_mul(mode, reg, mem)?; }, // MUL
            0b000101_u16 => {self.op_div(mode, reg, mem)?; }, // DIV
            0b000110_u16 => {self.op_mod(mode, reg, mem)?; }, // MOD
            0b000111_u16 => {self.op_and(mode, reg, mem)?; }, // AND
            0b001000_u16 => {self.op_or(mode, reg, mem)?; }, // OR
            0b001001_u16 => {self.op_xor(mode, reg, mem)?; }, // XOR
            0b001010_u16 => {self.op_not(mode, reg, mem)?; }, // NOT
            0b001011_u16 => {self.op_j(mode, reg, mem)?; }, // JUMP
            0b001100_u16 => {self.op_jz(mode, reg, mem)?; }, // JUMP Z
            0b001101_u16 => {self.op_jc(mode, reg, mem)?; }, // JUMP C
            0b001110_u16 => {self.op_jo(mode, reg, mem)?; }, // JUMP O
            0b001111_u16 => {self.op_js(mode, reg, mem)?; }, // JUMP S
            0b010000_u16 => {self.op_jnz(mode, reg, mem)?; }, // JUMP !Z
            0b010001_u16 => {self.op_jg(mode, reg, mem)?; }, // JUMP >
            0b010010_u16 => {self.op_jl(mode, reg, mem)?; }, // JUMP <
            0b010011_u16 => {self.op_cmp(mode, reg, mem)?; }, // COMPARE
            0b010100_u16 => {self.op_push(mode, reg, mem)?; }, // PUSH
            0b010101_u16 => {self.op_pop(mode, reg, mem)?; }, // POP
            0b010110_u16 => {self.op_call(mode, reg, mem)?; }, // CALL
            0b010111_u16 => {self.op_ret(mem)?; }, // RETURN
            0b011000_u16 => {self.op_shl(mode, reg, mem)?; }, // SHIFT LEFT
            0b011001_u16 => {self.op_shr(mode, reg, mem)?; }, // LOGICAL SHIFT RIGHT
            0b011010_u16 => {self.op_sar(mode, reg, mem)?; }, // ARITHMETIC SHIFT RIGHT
            0b011011_u16 => {self.op_ssp(mode, reg, mem)?; }, // SET STACK POINTER
            0b011100_u16 => {self.op_skip(mode, reg, mem)?; },
            0b011101_u16 => {self.op_sys(mem)?; },
            0b011110_u16 => {self.op_kret(mem)?; },
            0b011111_u16 => {self.op_gsp(mode, reg, mem)?; }, // GET STACK PTR
            0b100_000_u16 => {self.op_pnk(mem); }, // PANIC
            0b100_001_u16 => {self.op_dbg(mode, reg, mem)?; }, // debug
            0b100_010_u16 => {self.op_shrw(mode, reg, mem)?; }, // shift right wrap
            0b100_011_u16 => {self.op_gfls(mode, reg, mem)?; }, // get flags
            0b100_100_u16 => {self.op_sfls(mode, reg, mem)?; }, // set flags
            0b111111_u16 => {self.op_halt(mem)?;},
            _ => {
                println!("Unaccounted-for operation.\nInstruction: {:016b}\nPC: {:x}", instruction, self.pc);
                println!("Halting...");
                for inst in mem.get_range(self.pc - 5, self.pc + 5) {
                    println!("Instruction: {:08b}", inst);

                }
                self.halted = true;
                self.debug(mem);
            },
        }

        if self.mode == CPUMode::U {
            self.instruction_ctr += 1;
            if self.instruction_ctr >= self.instruction_lim {
                self.instruction_ctr = 0;
                return Err(CPUExit::Timer);
            }
        }

        // println!("SP: {:0x}", self.sp);

        Ok(())

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
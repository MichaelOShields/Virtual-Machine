mod cpu;
mod memory;

use cpu::CPU;
use memory::mem;



fn main() {
    let mut memory = mem::new();

    memory.set(0x00, 0b0000_0100_u8);
    memory.set(0x01, 0b1100_0000_u8);
    memory.set(0x02, 0b0000_0100_u8);

    // println!("{:08b}", memory.get(0));
    // println!("{:08b}", memory.get(1));
    // println!("{:08b}", memory.get(2));

    let mut cpu = CPU::new(memory);

    cpu.step();

    cpu.status();
}
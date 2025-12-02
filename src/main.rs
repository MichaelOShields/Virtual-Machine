mod cpu;
mod memory;

use cpu::CPU;
use memory::mem;



fn main() {
    let mut memory = mem::new();

    memory.set(0x00, 0b00000000_u8);
    memory.set(0x01, 0b11000000_u8);
    memory.set(0x02, 0b00000100_u8);

    let mut cpu = CPU::new(memory);

    cpu.step();

    cpu.status();
}
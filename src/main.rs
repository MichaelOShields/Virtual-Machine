mod cpu;
mod memory;

use cpu::CPU;
use memory::Mem;



fn main() {
    let mut memory = Mem::new();

    memory.set(0x00, 0b0000_0100_u8); // move command, no modes so far
    memory.set(0x01, 0b1101_0000_u8); // moving i2 -> r1
    memory.set(0x02, 0b0000_0000_u8);
    
    memory.set(0x03, 0b0000_0100_u8); // move command, no modes so far
    memory.set(0x04, 0b1101_1000_u8); // moving i2 -> r1
    memory.set(0x05, 0b10100101_u8);
 
    // setting memory for m2 to r1
    memory.set(0xA5, 0b0000_1100_u8);


    memory.set(0x06, 0b0000_0100_u8); // move, no modes so far
    memory.set(0x07, 0b01_100_010_u8); // move, mode is m2 to r1, so must identify which registers; 
    // r2 and r3 will hold the address and it'll go to r4


    memory.set(0x08, 0b0000_0100_u8);
    memory.set(0x09, 0b1100_0000_u8);
    memory.set(0x0A, 0b0010_0001_u8);


    memory.set(0x0B, 0b0000_1000_u8);
    memory.set(0x0C, 0b0010_0011_u8);

    memory.set(0x0D, 0b0000_1100_u8);
    memory.set(0x0E, 0b0010_0000_u8);


    let mut cpu = CPU::new(memory);

    let mut cpu_running = true;

    while cpu_running {
        cpu_running = cpu.step();
        cpu.status();
    }
    println!("CPU stopped")

}
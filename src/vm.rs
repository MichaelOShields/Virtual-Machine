
use crate::cpu::Cpu;
use crate::memory::Mem;
use crate::vc::VideoController;


pub struct Vm {
    pub mem: Mem,
    pub cpu: Cpu,
    pub video: VideoController,
}

impl Vm {
    pub fn new(mem: Mem, video: VideoController, cpu: Cpu) -> Self {
        Self {
            mem: mem,
            cpu: cpu,
            video: video
        }
    }

    pub fn step(&mut self) {

        if !self.cpu.halted {
            self.cpu.step(&mut self.mem);
            self.cpu.status();
        }
    }

    pub fn run(&mut self) {
        while !self.cpu.halted {
            self.step();
        }
        println!("CPU halted");
    }
}
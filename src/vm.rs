use crate::cpu::Cpu;
use crate::bus::Bus;
use crate::vc::VideoController;




pub struct Vm {
    pub mem: Bus,
    pub cpu: Cpu,
    pub video: VideoController,
}

impl Vm {
    pub fn new(mem: Bus, video: VideoController, cpu: Cpu) -> Self {
        Self {
            mem: mem,
            cpu: cpu,
            video: video,
        }
    }

    pub fn step(&mut self) {

        // if self.cpu.pc >= 0x800 && self.cpu.pc <= 0x0FFF {
        //     self.cpu.debug(&mut self.mem);
        //     panic!("PC ran away: {:04X}", self.cpu.pc);
        // }

        if !self.cpu.halted {
            self.cpu.step(&mut self.mem);
            // self.cpu.status();

            self.video.update_framebuffer(self.mem.get_range(self.video.vram_base, self.video.vram_base + self.video.framebuffer.len() as u16));
        }
        else {
            // println!("CPU halted at {}", self.cpu.pc);
        }
    }

    pub fn step_many(&mut self, n: i32) {
        for _ in 0..n {
            self.step();
            if self.cpu.halted {
                break;
            }
        }
    }
}
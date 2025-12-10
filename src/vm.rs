use crate::cpu::Cpu;
use crate::memory::Mem;
use crate::vc::VideoController;
use crate::pointer::Pointer;




pub struct Vm {
    pub mem: Mem,
    pub cpu: Cpu,
    pub video: VideoController,
    pub ptr: Pointer,
}

impl Vm {
    pub fn new(mem: Mem, video: VideoController, cpu: Cpu, ptr: Pointer) -> Self {
        Self {
            mem: mem,
            cpu: cpu,
            video: video,
            ptr,
        }
    }

    pub fn step(&mut self) {

        if !self.cpu.halted {
            self.cpu.step(&mut self.mem);
            // self.cpu.status();

            self.video.update_framebuffer(self.mem.get_range(self.video.vram_base, self.video.vram_base + self.video.framebuffer.len() as u16));
        }
        else {
            println!("CPU halted at {}", self.cpu.pc);
        }
    }

    pub fn step_many(&mut self, n: i32) {
        for _ in 0..n {
            self.step();
        }
    }

    pub fn print_video(&mut self) {
        self.video.print_frame();
    }
}
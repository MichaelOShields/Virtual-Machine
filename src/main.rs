//! main.rs
mod cpu;
mod memory;
mod vc;
mod vm;
mod binary;
mod pointer;

use cpu::Cpu;
use memory::Mem;
use vc::VideoController;
use vm::Vm;
use pointer::Pointer;

use std::{num::NonZero, rc::Rc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// ── softbuffer replaces pixels ────────────────────────────────────────────────
use softbuffer::{Context, Surface};

// ── constants ─────────────────────────────────────────────────────────────────
const SIZE: u8 = 128;

// ── app wrapper ───────────────────────────────────────────────────────────────
struct App {
    window:   Option<Rc<Window>>,
    context:  Option<Context<Rc<Window>>>,
    surface:  Option<Surface<Rc<Window>, Rc<Window>>>,
    vm:       Vm,
}

impl App {
    fn new(vm: Vm) -> Self {
        Self {
            window:  None,
            context: None,
            surface: None,
            vm,
        }
    }
}

impl ApplicationHandler for App {
    // create window + softbuffer objects once winit says we're ready
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ---- window ---------------------------------------------------------
        let window = Rc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(winit::dpi::PhysicalSize::new(1024, 1024)) // 4x scale
            )
            .unwrap(),
    );
        let size = window.inner_size();

        // ---- softbuffer context + surface -----------------------------------
        // SAFETY: softbuffer requires raw-handle stability; winit guarantees it.
        let context = unsafe { Context::new(window.clone()).unwrap() };
        let mut surface = unsafe { Surface::new(&context, window.clone()).unwrap() };
        surface.resize(Option::expect(NonZero::new(size.width), "hi"), Option::expect(NonZero::new(size.height), "hi")).unwrap();

        // ---- stash ----------------------------------------------------------
        self.window  = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);

        // kick-start first frame
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                // ---- run VM and refresh framebuffer --------------------------
                for _ in 0..30 { // run multiple steps per frame
                    if !self.vm.cpu.halted {
                        self.vm.step();

                        // Debug: print when we hit certain addresses
                        if self.vm.cpu.regs[0] == 0x0C && self.vm.cpu.regs[1] == 0x00 {
                            println!("Halfway! R0={:02X}, R1={:02X}, PC={:04X}", 
                            self.vm.cpu.regs[0], self.vm.cpu.regs[1], self.vm.cpu.pc);
                        }
                    }
                }
                
                self.vm
                    .video
                    .update_framebuffer(self.vm.mem.get_range(0x800, 0xFFF+1));

                // ---- blit VM framebuffer into softbuffer surface -------------
                let surf  = self.surface.as_mut().unwrap();
                surf
                    .resize(Option::expect(NonZero::new(1024), "hi"), Option::expect(NonZero::new(1024), "hi"))
                    .unwrap();

                {
                    // make counter to check how many times inner loop runs
                    let surf = self.surface.as_mut().unwrap();
                    let mut buf = surf.buffer_mut().unwrap();
                    
                    let scale = 8; // 512 / 128
                    for (byte_idx, &byte) in self.vm.video.framebuffer.iter().enumerate() {
                        for bit_idx in 0..8 {
                            let fb_pixel_idx = byte_idx * 8 + bit_idx;
                            let x = fb_pixel_idx % 128;
                            let y = fb_pixel_idx / 128;
                            let color = if (byte >> bit_idx) & 1 == 1 { 0xFFFFFFFF } else { 0xFF000000 };
                            
                            // Draw scale x scale block
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let screen_idx = (y * scale + dy) * 512 + (x * scale + dx);
                                    buf[screen_idx] = color;
                                }
                            }
                        }
                    }
                    
                    buf.present().unwrap();
                } // buffer presented on drop

                // queue next frame
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => {}
        }
    }
}

// ── bootstrap ────────────────────────────────────────────────────────────────
fn main() {
    // ----- sample program in memory -----------------------------------------
    let mut memory = Mem::new();

    // Initialize R0 = 0x08, R1 = 0x00 (address 0x0800)
    memory.set(0x00, 0b00000100);  // MOVE opcode 000001, mode 00
    memory.set(0x01, 0b11000000);  // mode 11 (0011), reg 000000 (R0)
    memory.set(0x02, 0x08);        // value = 0x08

    memory.set(0x03, 0b00000100);  // MOVE opcode 000001, mode 00
    memory.set(0x04, 0b11001000);  // mode 11 (0011), reg 001000 (R1)
    memory.set(0x05, 0x00);        // value = 0x00

    // ===== MAIN LOOP starts at 0x06 =====

    // Compare R0 with 0x10
    memory.set(0x06, 0b01001100);   // COMP opcode 010011, mode 00
    memory.set(0x07, 0b11000000);   // mode 11 (0011), reg 000000 (R0)
    memory.set(0x08, 0x10);         // immediate value = 0x10

    // If R0 == 0x10, jump to HALT at 0x30
    memory.set(0x09, 0b00110000);   // JZ opcode 001100, mode 00
    memory.set(0x0A, 0b10000000);   // mode 10 (0010), reg 000000
    memory.set(0x0B, 0x00);         // high byte of address
    memory.set(0x0C, 0x30);         // low byte = 0x30

    // Write 0xFF to memory at address R0:R1
    memory.set(0x0D, 0b00000101);   // MOVE opcode 000001, mode 01
    memory.set(0x0E, 0b00000000);   // mode 00 (0100), reg 000000 (M at R0:R1)
    memory.set(0x0F, 0xFF);         // value = 0xFF

    // Add 1 to R1
    memory.set(0x10, 0b00001000);   // ADD opcode 000010, mode 00
    memory.set(0x11, 0b11001000);   // mode 11 (0011), reg 001000 (R1)
    memory.set(0x12, 0x01);         // value = 1

    // If carry, jump to increment R0 at 0x20
    memory.set(0x13, 0b00110100);   // JC opcode 001101, mode 00
    memory.set(0x14, 0b10000000);   // mode 10 (0010), reg 000000
    memory.set(0x15, 0x00);         // high byte
    memory.set(0x16, 0x20);         // low byte = 0x20

    // Loop back to 0x06
    memory.set(0x17, 0b00101100);   // JUMP opcode 001011, mode 00
    memory.set(0x18, 0b10000000);   // mode 10 (0010), reg 000000
    memory.set(0x19, 0x00);         // high byte
    memory.set(0x1A, 0x06);         // low byte = 0x06

    // ===== INCREMENT R0 at 0x20 =====
    memory.set(0x20, 0b00001000);   // ADD opcode 000010, mode 00
    memory.set(0x21, 0b11000000);   // mode 11 (0011), reg 000000 (R0)
    memory.set(0x22, 0x01);         // value = 1

    // Jump back to 0x06
    memory.set(0x23, 0b00101100);   // JUMP opcode 001011, mode 00
    memory.set(0x24, 0b10000000);   // mode 10 (0010), reg 000000
    memory.set(0x25, 0x00);         // high byte
    memory.set(0x26, 0x06);         // low byte = 0x06

    // ===== HALT at 0x30 =====
    memory.set(0x30, 0b11111100);   // HALT opcode 111111, mode 00
    memory.set(0x31, 0b00000000);   // (padding)


    

    


    /*
    loop: 
    set vram[vram ptr] = 0
    set vram ptr = vram ptr + 1
    set vram[vram ptr] = 1
    jump to 0xFF
    */



    memory.set(0x2F, 0b1111_1100);

    for n in memory.get_range(0x00, 0x1F) {
        println!("{:08b}", n);
    }



    // ----- build VM ---------------------------------------------------------
    let cpu = Cpu::new();
    let vc  = VideoController::new(128, 128, 0x800);
    let vm  = Vm::new(memory, vc, cpu, Pointer { x: 0, y: 0 });

    // ----- spin up winit -----------------------------------------------------
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(vm);
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("winit error: {e}");
    }
}
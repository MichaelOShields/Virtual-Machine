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
                self.vm.step();
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
    memory.set(0x00, 0b0000_0100);
    memory.set(0x01, 0b1101_0000);
    memory.set(0x02, 0);
    memory.set(0x03, 0b0000_0100);
    memory.set(0x04, 0b1101_1000);
    memory.set(0x05, 0b1010_0101);
    memory.set(0xA5, 0b0000_1100);
    memory.set(0x06, 0b0000_0100);
    memory.set(0x07, 0b0110_0010);
    memory.set(0x08, 0b0000_0100);
    memory.set(0x09, 0b1100_0000);
    memory.set(0x0A, 0b0010_0001);
    memory.set(0x0B, 0b0000_1000);
    memory.set(0x0C, 0b0010_0011);
    memory.set(0x0D, 0b0000_1100);
    memory.set(0x0E, 0b0010_0000);

    memory.set(0x0F, 0b0000_0100);
    memory.set(0x10, 0b1111_1000);
    memory.set(0x11, 0b1111_1111);



    // regs 5 and 6 saved for mem addy
    memory.set(0x012, 0b0000_0100);
    memory.set(0x013, 0b1110_1000);
    memory.set(0x013, 0x00);

    
    memory.set(0x14, 0b0000_0100);
    memory.set(0x15, 0b1111_0000);
    memory.set(0x16, 0xFF);


    // keep track of current position (start at 0x800)
    memory.set(0xFF, 0b0000_0100);
    memory.set(0x100, 0b1100_0000);
    memory.set(0x101, 0b0000_1000);

    memory.set(0x102, 0b0000_0100);
    memory.set(0x103, 0b1100_1000);
    memory.set(0x104, 0b0000_0000);


    // jump
    memory.set(0x17, 0b0010_1100);
    memory.set(0x18, 0b0010_1000);


    // set the program
    // memory.set(0xFF, 0b1111_1100);

    // compare regs 0 and 1 to 4096; if > then halt

    // if reg 0 is @ 16, then halt


    // setting flags
    memory.set(0xFF, 0b0100_1100);
    memory.set(0x100, 0b1100_0000);
    memory.set(0x101, 0b0001_0000);

    // jump zero -> if zero, send to 0x2F (halt)
    memory.set(0x102, 0b0011_0000);
    memory.set(0x103, 0b1000_0000);

    memory.set(0x104, 0b0000_0000);
    memory.set(0x105, 0b0010_1111);

    // otherwise: lock in:

    // loop step 1
    memory.set(0x106, 0b0000_0101);
    memory.set(0x107, 0b0000_0000);
    memory.set(0x108, 0b0000_0000);

    // loop step 2
    
    // Add 1 to lower number (reg 1)
    memory.set(0x109, 0b0000_1000);
    memory.set(0x10A, 0b1100_1000);
    memory.set(0x10B, 0b0000_0001);

    memory.set(0x10C, 0b0011_0100);
    memory.set(0x10D, 0b1000_0000);
    memory.set(0x10E, 0b0000_0000);
    memory.set(0x10F, 0xC8);

    // loop step 3
    memory.set(0x110, 0b0000_0101);
    memory.set(0x111, 0b0000_0000);
    memory.set(0x112, 0b1111_1111);

    memory.set(0x113, 0b0010_1100);
    memory.set(0x114, 0b1000_0000);
    memory.set(0x115, 0b0000_0000);
    memory.set(0x116, 0xFF);
    
    




    memory.set(0xC8, 0b0000_1000);
    memory.set(0xC9, 0b1100_0000);
    memory.set(0xCA, 0b0000_0001);
    memory.set(0xCB, 0b0010_1100);
    memory.set(0xCC, 0b1000_0000);
    memory.set(0xCD, 0b0000_0001);
    memory.set(0xCE, 0b0001_0000);

    

    


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
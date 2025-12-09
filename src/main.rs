mod cpu;
mod memory;
mod vc;
mod vm;
mod binary;

use cpu::Cpu;
use memory::Mem;
use vc::VideoController;
use vm::Vm;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};


#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Render

                self.window.as_ref().unwrap().request_redraw();
            },
            _ => (),
        }
    }
}



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


    memory.set(0x0F, 0b1111_1100_u8);
    let cpu = Cpu::new();

    // 2048 is the mem address for vram
    // 0x800 - 0xFFF

    let vc = VideoController::new(128, 128, 0x800);
    let mut vm: Vm = Vm::new(memory, vc, cpu);

    // vm.run();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::default();
    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(e) => println!("Received event error {}", e),
    };


}
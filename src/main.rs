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

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use pixels::{Pixels, SurfaceTexture};


const SIZE: u8 = 128;


struct App<'a> {
    window: Option<Window>,
    pixels: Option<Pixels<'a>>,
    vm: Vm,
}

impl<'a> App<'a>{
    pub fn new(vm: Vm) -> Self {
        Self {
            window: None,
            pixels: None,
            vm,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        
        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, &window);

        let pixels = Pixels::new(
            self.vm.video.width as u32,
            self.vm.video.height as u32,
            surface,
        ).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
        // self.window = Some(event_loop.create_window(Window::default_attributes()).unwrap());
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
                self.vm.step_many(50_000);
                self.vm.video.update_framebuffer(self.vm.mem.get_range(0x800, 0xFFF));

                let frame = self.pixels.as_mut().unwrap().frame_mut();
                let framebuf = &self.vm.video.framebuffer;

                for (i, pxl) in frame.chunks_exact_mut(4).enumerate() {
                    pxl.copy_from_slice(&framebuf[i].to_le_bytes());
                }

                self.pixels.as_mut().unwrap().render().unwrap();
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
    let vm: Vm = Vm::new(memory, vc, cpu, Pointer{ x: 0, y: 0 });


    // let window = {
    //     let size = LogicalSize::new(SIZE as f64, SIZE as f64);
    //     WindowBuilder::new()
    //         .with_title("Hello Pixels")
    //         .with_inner_size(SIZE)
    //         .with_min_inner_size(SIZE)
    //         .build(&event_loop)
    //         .unwrap()
    // };

    // vm.run();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new(vm);
    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(e) => println!("Received event error {}", e),
    };


}
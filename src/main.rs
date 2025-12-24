//! main.rs
mod cpu;
mod bus;
mod vc;
mod vm;
mod binary;
mod device;
mod assembler;

use cpu::Cpu;
use bus::Bus;
use vc::VideoController;
use vm::Vm;
use device::{Mouse, Keyboard};

use std::{fs, io, num::NonZero, rc::Rc};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    keyboard::{Key, NamedKey}
};

// ── softbuffer replaces pixels ────────────────────────────────────────────────
use softbuffer::{Context, Surface};

use crate::assembler::{Assembler, AssemblerError, Lexer, Parser};

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
                    .with_inner_size(winit::dpi::PhysicalSize::new(512, 512)) // 4x scale
                    .with_resizable(false)
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

            WindowEvent::KeyboardInput {
                event: KeyEvent {logical_key: key, state: ElementState::Pressed, .. },
                ..
            } => {
                let keycode: u8 = match key.as_ref() {
                    Key::Character("a") => 1,
                    Key::Character("b") => 2,
                    Key::Character("c") => 3,
                    Key::Character("d") => 4,
                    Key::Character("e") => 5,
                    Key::Character("f") => 6,
                    Key::Character("g") => 7,
                    Key::Character("h") => 8,
                    Key::Character("i") => 9,
                    Key::Character("j") => 10,
                    Key::Character("k") => 11,
                    Key::Character("l") => 12,
                    Key::Character("m") => 13,
                    Key::Character("n") => 14,
                    Key::Character("o") => 15,
                    Key::Character("p") => 16,
                    Key::Character("q") => 17,
                    Key::Character("r") => 18,
                    Key::Character("s") => 19,
                    Key::Character("t") => 20,
                    Key::Character("u") => 21,
                    Key::Character("v") => 22,
                    Key::Character("w") => 23,
                    Key::Character("x") => 24,
                    Key::Character("y") => 25,
                    Key::Character("z") => 26,
                    Key::Named(NamedKey::ArrowUp) => 27,
                    Key::Named(NamedKey::ArrowDown) => 28,
                    Key::Named(NamedKey::ArrowLeft) => 29,
                    Key::Named(NamedKey::ArrowRight) => 30,
                    Key::Character("1") => 31,
                    Key::Character("2") => 32,
                    Key::Character("3") => 33,
                    Key::Character("4") => 34,
                    Key::Character("5") => 35,
                    Key::Character("6") => 36,
                    Key::Character("7") => 37,
                    Key::Character("8") => 38,
                    Key::Character("9") => 39,
                    Key::Character("0") => 40,
                    Key::Named(NamedKey::Backspace) => 50,
                    Key::Named(NamedKey::Enter) => 51,
                    Key::Named(NamedKey::Escape) => 52,
                    Key::Named(NamedKey::Space) => 53,
                    _ => 0,
                };

                if keycode != 0 {
                    self.vm.mem.key_inject(keycode);
                    println!("Key pressed: {}", keycode);
                }
            },

            WindowEvent::RedrawRequested => {
                self.vm.step_many(100);
                if !self.vm.cpu.halted {
                    // self.vm.cpu.status();
                    // self.vm.mem.status();
                }
                
                self.vm
                    .video
                    .update_framebuffer(self.vm.mem.get_range(0x2400, 0x3200));

                // ---- blit VM framebuffer into softbuffer surface -------------
                let surf  = self.surface.as_mut().unwrap();
                surf
                    .resize(Option::expect(NonZero::new(512), "hi"), Option::expect(NonZero::new(512), "hi"))
                    .unwrap();

                {
                    // make counter to check how many times inner loop runs
                    let surf = self.surface.as_mut().unwrap();
                    let mut buf = surf.buffer_mut().unwrap();
                    
                    let scale = 4; // 512 / 128
                    for (byte_idx, &byte) in self.vm.video.framebuffer.iter().enumerate() {
                        for bit_idx in 0..8 {
                            let fb_pixel_idx = byte_idx * 8 + bit_idx;
                            let x = fb_pixel_idx % 128;
                            let y = fb_pixel_idx / 128;
                            let color = if (byte >> (7 - bit_idx)) & 1 == 1 {
                                0xFFFFFFFF
                            } else {
                                0xFF000000
                            };
                            
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




    


    /*
    core loop:
    if R2 is zero, reset to 1000_0000 and increment vram ptr
    otherwise, logical shr R2
    finally blit
    
    
     */
    // try to have this at 0x5F4 (1524 bytes)
    // step 1: zero carry R2
    // 0b001100
    

    // logical shr

//];

// fn load_program(memory: &mut Bus, bytes: &[u8]) {
//     let base = 0x400;
//     for (i, byte) in bytes.iter().enumerate() {
//         memory.set(base + i as u16, *byte);
//     }
// }

// const LEXING: bool = false; // debugging lexer


fn load_assembly(memory: &mut Bus, file_path: String) {

    let mut output: String = String::new();

    let code = match fs::read_to_string(&(file_path.clone() + ".dnasm")) {
        Ok(s) => s,
        Err(e) => {panic!("Assembler Error: {:?}", e);},
    };
    

    let lex: Lexer = Lexer::new(&code);

    let mut parser: Parser = Parser::new(lex);

    let mut assembler = Assembler::new(match parser.parse() {
        Ok(p) => p,
        Err(e) => {panic!("Assembler Error: {:?}", e);},
    });
    let assembled = assembler.assemble();

    


    if let Ok(map) = assembled {

        let mut bases: Vec<u16> = map.keys().copied().collect();
        bases.sort_unstable();
        for base in bases {
            let bytes = &map[&base];
            for (offset, byte) in bytes.iter().enumerate() {
                let addr = base + offset as u16;
                memory.force_set(addr, *byte); // ideally checked
                output.push_str(format!("Instruction ({:0x}): {:08b}\n", base, *byte).as_str());
            }
            
        }

        match fs::write(file_path + "_out.txt", output) {
            Ok(()) => (),
            Err(e) => {panic!("Unable to write file with error {:?}", e);},
        };
    }
    else if let Err(e) = assembled {
        panic!("Assembler Error: {:?}", e);
    }
    else {
        panic!("Failed");
    }
    
}





fn main() {

    let keyb = Keyboard::new();
    let ms = Mouse::new();
    
    // kernel / system
    let bootloader   = 0x0000..0x0400; // 1 KB
    let kernel_core  = 0x0400..0x1000;
    let kernel_traps = 0x1000..0x1200;
    let kernel_data  = 0x1200..0x1800;
    let kernel_heap  = 0x1800..0x2000;
    let kernel_stack = 0x2000..0x2400;

    // reserved (not passed into Bus::new directly)
    let vram         = 0x2400..0x3400; // 4 KB, currently only using 0x2400-0x2900(?)

    let mmio         = 0x3400..0x3800; // 1 KB

    // user space
    // differentiate tasks for memory safety
    let user_code_0  = 0x3800..0x4000;
    let user_data_0  = 0x4000..0x4200;
    let user_heap_0  = 0x4200..0x4C00;
    let user_stack_0 = 0x4C00..0x5000;

    let user_code_1  = 0x5000..0x5800;
    let user_data_1  = 0x5800..0x5A00;
    let user_heap_1  = 0x5A00..0x6400;
    let user_stack_1 = 0x6400..0x6800;

    let user_code_2  = 0x6800..0x7000;
    let user_data_2  = 0x7000..0x7200;
    let user_heap_2  = 0x7200..0x7C00;
    let user_stack_2 = 0x7C00..0x8000;

    let user_code_3  = 0x8000..0x8800;
    let user_data_3  = 0x8800..0x8A00;
    let user_heap_3  = 0x8A00..0x9400;
    let user_stack_3 = 0x9400..0x9800;

    let user_code_4  = 0x9800..0xA000;
    let user_data_4  = 0xA000..0xA200;
    let user_heap_4  = 0xA200..0xAC00;
    let user_stack_4 = 0xAC00..0xB000;

    let user_code_5  = 0xB000..0xB800;
    let user_data_5  = 0xB800..0xBA00;
    let user_heap_5  = 0xBA00..0xC400;
    let user_stack_5 = 0xC400..0xC800;

    let user_code_6  = 0xC800..0xD000;
    let user_data_6  = 0xD000..0xD200;
    let user_heap_6  = 0xD200..0xDC00;
    let user_stack_6 = 0xDC00..0xE000;

    let user_code_7  = 0xE000..0xE800;
    let user_data_7  = 0xE800..0xEA00;
    let user_heap_7  = 0xEA00..0xF400;
    let user_stack_7 = 0xF400..0xF800;


    let cpu = Cpu::new(kernel_traps.start);
    let vc  = VideoController::new(128, 128, vram.start);

    let mut memory = Bus::new(
        ms,
        keyb,

        bootloader,
        kernel_core,
        kernel_traps,
        kernel_data,
        kernel_heap,
        kernel_stack,
        vram,
        mmio,

        // later: make this way more concise. probably use a vec or array
        user_code_0,
        user_data_0,
        user_heap_0,
        user_stack_0,

        user_code_1,
        user_data_1,
        user_heap_1,
        user_stack_1,

        user_code_2,
        user_data_2,
        user_heap_2,
        user_stack_2,

        user_code_3,
        user_data_3,
        user_heap_3,
        user_stack_3,

        user_code_4,
        user_data_4,
        user_heap_4,
        user_stack_4,

        user_code_5,
        user_data_5,
        user_heap_5,
        user_stack_5,

        user_code_6,
        user_data_6,
        user_heap_6,
        user_stack_6,

        user_code_7,
        user_data_7,
        user_heap_7,
        user_stack_7,

    );


    // load bootloader
    load_assembly(&mut memory, "src\\boot".to_string());

    load_assembly(&mut memory, "src\\kernel".to_string());

    load_assembly(&mut memory, "src\\kernel_data".to_string());

    load_assembly(&mut memory, "src\\kerheap".to_string());

    load_assembly(&mut memory, "src\\program_handling".to_string());

    load_assembly(&mut memory, "src\\mouse".to_string());

    load_assembly(&mut memory, "src\\shell_disp".to_string());
    

    let vm  = Vm::new(memory, vc, cpu);

    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(vm);
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("winit error: {e}");
    }
}
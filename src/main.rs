//! main.rs
mod cpu;
mod bus;
mod vc;
mod vm;
mod binary;
mod device;

use cpu::Cpu;
use bus::Bus;
use vc::VideoController;
use vm::Vm;
use device::{Mouse, Keyboard};

use std::{num::NonZero, rc::Rc};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    keyboard::{Key, NamedKey}
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
                    _ => 0,
                };
                self.vm.mem.key_inject(keycode);
                println!("Key pressed: {}", keycode);
            },

            WindowEvent::RedrawRequested => {
                self.vm.step_many(100);
                if !self.vm.cpu.halted {
                    self.vm.cpu.status();
                    self.vm.mem.status();
                }
                
                self.vm
                    .video
                    .update_framebuffer(self.vm.mem.get_range(0x800, 0xFFF+1));

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



// fn set_mem(memory: &mut Mem, current_pos: u16, src: u8) -> u16 {
//     memory.set(current_pos, src);
//     return current_pos + 1;
// }

// fn init_r0(memory: &mut Mem, mut current_pos: u16) -> u16 {
//     // Initialize R0 = 0x08, R1 = 0x00 (address 0x0800)
//     current_pos = set_mem(memory, current_pos, 0b00000100);  // MOVE opcode 000001, mode 00
//     current_pos = set_mem(memory, current_pos, 0b11000000);  // mode 11 (0011), reg 000000 (R0)
//     current_pos = set_mem(memory, current_pos, 0x08);        // value = 0x08

//     current_pos = set_mem(memory, current_pos, 0b00000100);  // MOVE opcode 000001, mode 00
//     current_pos = set_mem(memory, current_pos, 0b11001000);  // mode 11 (0011), reg 001000 (R1)
//     current_pos = set_mem(memory, current_pos, 0x00);        // value = 0x00

//     current_pos
// }

// fn write_black(memory: &mut Mem, mut current_pos: u16) -> u16 {
//     // Write 0x00 to memory at address R0:R1
//     current_pos = set_mem(memory, current_pos, 0b00000101);   // MOVE opcode 000001, mode 01
//     current_pos = set_mem(memory, current_pos, 0b00000000);   // mode 00 (0100), reg 000000 (M at R0:R1)
//     current_pos = set_mem(memory, current_pos, 0x00);        // value = 0x00
//     current_pos
// }

// fn write_white(memory: &mut Mem, mut current_pos: u16) -> u16 {
//     // Write 0xFF to memory at address R0:R1
//     current_pos = set_mem(memory, current_pos, 0b00000101);   // MOVE opcode 000001, mode 01
//     current_pos = set_mem(memory, current_pos, 0b00000000);   // mode 00 (0100), reg 000000 (M at R0:R1)
//     current_pos = set_mem(memory, current_pos, 0xFF);        // value = 0xFF
//     current_pos
// }

// fn cmp_r0_and_jump(memory: &mut Mem, mut current_pos: u16, imm: u8, target: u16) -> u16 {
//     // Compare R0 with 0x10
//     current_pos = set_mem(memory, current_pos, 0b01001100);   // COMP opcode 010011, mode 00
//     current_pos = set_mem(memory, current_pos, 0b11000000);   // mode 11 (0011), reg 000000 (R0)
//     current_pos = set_mem(memory, current_pos, imm);          // immediate value = 0x10

//     // If R0 == 0x10, jump to HALT at 0x30
//     current_pos = set_mem(memory, current_pos, 0b00110000);   // JZ opcode 001100, mode 00
//     current_pos = set_mem(memory, current_pos, 0b10000000);   // mode 10 (0010), reg 000000
//     current_pos = set_mem(memory, current_pos, (target >> 8) as u8); // high byte of address
//     current_pos = set_mem(memory, current_pos, (target & 0xFF) as u8); // low byte = 0x30

//     current_pos
// }

// fn add_1_r1(memory: &mut Mem, mut current_pos: u16) -> u16 {
//     // Add 1 to R1
//     current_pos = set_mem(memory, current_pos, 0b00001000);   // ADD opcode 000010, mode 00
//     current_pos = set_mem(memory, current_pos, 0b11001000);   // mode 11 (0011), reg 001000 (R1)
//     current_pos = set_mem(memory, current_pos, 1);          // value = 1
//     current_pos
// }

// fn jump_if_carry(memory: &mut Mem, mut current_pos: u16, target: u16) -> u16 {
//     // If carry, jump to increment R0 at 0x20
//     current_pos = set_mem(memory, current_pos, 0b00110100);   // JC opcode 001101, mode 00
//     current_pos = set_mem(memory, current_pos, 0b10000000);   // mode 10 (0010), reg 000000
//     current_pos = set_mem(memory, current_pos, (target >> 8) as u8); // high byte
//     current_pos = set_mem(memory, current_pos, (target & 0xFF) as u8); // low byte = 0x20
//     current_pos
// }

// fn jump(memory: &mut Mem, mut current_pos: u16, target: u16) -> u16 {
//     // If carry, jump to increment R0 at 0x20
//     current_pos = set_mem(memory, current_pos, 0b00101100);   // J opcode 001011, mode 00
//     current_pos = set_mem(memory, current_pos, 0b10000000);   // mode 10 (0010), reg 000000
//     current_pos = set_mem(memory, current_pos, (target >> 8) as u8); // high byte
//     current_pos = set_mem(memory, current_pos, (target & 0xFF) as u8); // low byte = 0x20
//     current_pos
// }


// fn increment_r0_and_jump_back(memory: &mut Mem, mut current_pos: u16, jump_target: u16) -> u16 {


//     // ===== INCREMENT R0 =====
//     current_pos = set_mem(memory, current_pos, 0b00001000);   // ADD opcode 000010, mode 00
//     current_pos = set_mem(memory, current_pos, 0b11000000);   // mode 11 (0011), reg 000000 (R0)
//     current_pos = set_mem(memory, current_pos, 0x01);         // value = 1

//     current_pos = write_white(memory, current_pos);

//     // Jump back to 0x06
//     current_pos = set_mem(memory, current_pos, 0b00101100);   // JUMP opcode 001011, mode 00
//     current_pos = set_mem(memory, current_pos, 0b10000000);   // mode 10 (0010), reg 000000
//     current_pos = set_mem(memory, current_pos, (jump_target >> 8) as u8); // high byte
//     current_pos = set_mem(memory, current_pos, (jump_target & 0xFF) as u8); // low byte = 0x06

//     current_pos
// }


// fn draw_line_anim() -> Mem {
//     let mut memory = Mem::new();
//     let mut current_pos: u16 = 0x00;

//     current_pos = init_r0(&mut memory, current_pos);

//     current_pos = write_white(&mut memory, current_pos);

//     // MAIN LOOP

//     let main_loop_pos = current_pos;

//     current_pos = write_black(&mut memory, current_pos);

//     current_pos = cmp_r0_and_jump(&mut memory, current_pos, 0x10, 0x30);

//     current_pos = add_1_r1(&mut memory, current_pos);

//     current_pos = jump_if_carry(&mut memory, current_pos, 0x100); // 0x100 is where we'll increment

//     current_pos = write_white(&mut memory, current_pos);

//     current_pos = jump(&mut memory, current_pos, main_loop_pos);


//     current_pos = increment_r0_and_jump_back(&mut memory, 0x100, main_loop_pos);


//     // HALT
//     memory.set(current_pos, 0b11111100);   // HALT opcode 111111, mode 00

//     memory
// }



/*
current mem architecture:

0x0000–0x03FF: Bootloader (1 KB)
0x0400–0x07FF: User program + globals (1 KB)
0x0800–0x0FFF: VRAM (2 KB)
0x1000–0x10FF: MMIO registers
0x1100–0x3EFF: Heap / free RAM
0x3F00–0x3FFF: Stack

*/
fn load_bootloader(memory: &mut Bus) {
    // STARTS AT 0x00
    // initialize cpu state (registers, sp, etc)
    // clear ram
    // clear vram
    // jump to program entry point/loop (will determine where, prolly 0x400)

    // init cpu:
    memory.set(0x00, 0b0000_0100); // move, mode 00 (so far)
    memory.set(0x01, 0b1100_0000); // mode 0011 -> i2 to r1, r1 is R0
    memory.set(0x02, 0b0000_0000); // empty reg

    // move r2 (R0) to r1 (R1), copies the zero over
    memory.set(0x03, 0b0000_0100);
    memory.set(0x04, 0b0000_1000);

    // R2 
    memory.set(0x05, 0b0000_0100);
    memory.set(0x06, 0b0001_0000);

    // R3 
    memory.set(0x07, 0b0000_0100);
    memory.set(0x08, 0b0001_1000);

    // R4 
    memory.set(0x09, 0b0000_0100);
    memory.set(0x0A, 0b0010_0000);

    // R5 
    memory.set(0x0B, 0b0000_0100);
    memory.set(0x0C, 0b0010_1000);

    // R6 
    memory.set(0x0D, 0b0000_0100);
    memory.set(0x0E, 0b0011_0000);

    // R7 
    memory.set(0x0F, 0b0000_0100);
    memory.set(0x10, 0b0011_1000);

    // initialize SP (go to 0x3FFF)
    memory.set(0x11, 0b0110_1100);
    memory.set(0x12, 0b1000_0000);
    memory.set(0x13, 0b0011_1111);
    memory.set(0x14, 0b1111_1111);

    // clear VRAM
    // memory.set_range(0x800, 0x0FFF + 1, 0);

    // method: check at 0x15 if R0 >= 16, if so jump to next (later clear r0), otherwise loop

    // set R0 = 8
    memory.set(0x15, 0b0000_0100);
    memory.set(0x16, 0b1100_0000);
    memory.set(0x17, 0b0000_1000);

    // set R1 = 0000
    memory.set(0x18, 0b0000_0100);
    memory.set(0x19, 0b1100_1000);
    memory.set(0x1A, 0b0000_0000);
    

    // so R0:R1 is 2048, or 0x800

    // key loop
    // set R0:R1 to 0

    memory.set(0x1B, 0b0000_0101);
    memory.set(0x1C, 0b0000_0000);
    memory.set(0x1D, 0b0000_0000);

    // compare R0 to 16
    memory.set(0x1E, 0b0100_1100);
    memory.set(0x1F, 0b1100_0000);
    memory.set(0x20, 0b0001_0000);

    // if zero, then r0 = 16
    // jz flag past next few instructions (4 bytes for jz flag) -> jump to 0x36
    memory.set(0x21, 0b0011_0000);
    memory.set(0x22, 0b1000_0000);
    memory.set(0x23, 0b0000_0000);
    memory.set(0x24, 0x37);
    

    // add 1 to r1
    memory.set(0x25, 0b0000_1000);
    memory.set(0x26, 0b1100_1000);
    memory.set(0x27, 0b0000_0001);

    // check carry flag (if so add 1 to r0, otherwise loop)
    // carry jump to add 1 to R0
    memory.set(0x28, 0b0011_0100);
    memory.set(0x29, 0b1000_0000);
    memory.set(0x2A, 0b0000_0000);
    memory.set(0x2B, 0x30);


    // loop back to key loop (no carry)
    memory.set(0x2C, 0b0010_1100);
    memory.set(0x2D, 0b1000_0000);
    memory.set(0x2E, 0b0000_0000);
    memory.set(0x2F, 0x1B);

    // add 1 to R0
    memory.set(0x30, 0b0000_1000);
    memory.set(0x31, 0b1100_0000); // should this be 1100_0000?
    memory.set(0x32, 0b0000_0001);

    // loop back to key loop
    memory.set(0x33, 0b0010_1100);
    memory.set(0x34, 0b1000_0000);
    memory.set(0x35, 0b0000_0000);
    memory.set(0x36, 0x1B);

    // CLEARING VRAM DONE

    // clear heap
    // 0x1100–0x3EFF

    // set R0 = 17
    memory.set(0x37, 0b0000_0100);
    memory.set(0x38, 0b1100_0000);
    memory.set(0x39, 0b0001_0001);

    // set R1 = 0000
    memory.set(0x3A, 0b0000_0100);
    memory.set(0x3B, 0b1100_1000); // should this be 1100_1000?
    memory.set(0x3C, 0b0000_0000);

    // R0:R1 = 4352 (0x1100)

    // key loop
    // set R0:R1 to 0
    memory.set(0x3D, 0b0000_0101);
    memory.set(0x3E, 0b0000_0000);
    memory.set(0x3F, 0b0000_0000);

    // compare R0 to 62
    memory.set(0x40, 0b0100_1100);
    memory.set(0x41, 0b1100_0000);
    memory.set(0x42, 62);

    // JZ to 0x58 (check r1 = 1s)
    memory.set(0x43, 0b0011_0000);
    memory.set(0x44, 0b1000_0000);
    memory.set(0x45, 0b0000_0000);
    memory.set(0x46, 0x59);

    // add 1 to R1
    memory.set(0x47, 0b0000_1000);
    memory.set(0x48, 0b1100_1000);
    memory.set(0x49, 0b0000_0001);

    // JC to add to R0 (0x51)
    memory.set(0x4A, 0b0011_0100);
    memory.set(0x4B, 0b1000_0000);
    memory.set(0x4C, 0b0000_0000);
    memory.set(0x4D, 0x52);

    // JMP back to loop (→ 0x3C)
    memory.set(0x4E, 0b0010_1100);
    memory.set(0x4F, 0b1000_0000);
    memory.set(0x50, 0b0000_0000);
    memory.set(0x51, 0x3D);

    // add 1 to R0
    memory.set(0x52, 0b0000_1000);
    memory.set(0x53, 0b1100_0000);
    memory.set(0x54, 0b0000_0001);

    // JMP back to loop (→ 0x3C)
    memory.set(0x55, 0b0010_1100);
    memory.set(0x56, 0b1000_0000);
    memory.set(0x57, 0b0000_0000);
    memory.set(0x58, 0x3D);


    // check R1 == 11111111 (only happens if R0 == 62)
    memory.set(0x59, 0b0100_1100);
    memory.set(0x5A, 0b1100_1000);
    memory.set(0x5B, 0b1111_1111);

    // JZ past looping back
    memory.set(0x5C, 0b0011_0000);
    memory.set(0x5D, 0b1000_0000);
    memory.set(0x5E, 0b0000_0000); // jump to program
    memory.set(0x5F, 0x67);

    // add 1 to R1
    memory.set(0x60, 0b0000_1000);
    memory.set(0x61, 0b1100_1000);
    memory.set(0x62, 0b0000_0001);

    // JMP back to loop (adding to R1)
    memory.set(0x63, 0b0010_1100);
    memory.set(0x64, 0b1000_0000);
    memory.set(0x65, 0b0000_0000);
    memory.set(0x66, 0x59);

    memory.set(0x67, 0b0000_0100);
    memory.set(0x68, 0b1110_1000);
    memory.set(0x69, 0b0000_1000);

    // memory.set(0x6A, 0b1111_1100);
    memory.set(0x6A, 0b0000_0100); // move, mode 00 (so far)
    memory.set(0x6B, 0b1100_0000); // mode 0011 -> i2 to r1, r1 is R0
    memory.set(0x6C, 0b0000_0000); // empty reg

    memory.set(0x6D, 0b0000_0100);
    memory.set(0x6E, 0b0000_1000);

    memory.set(0x6F, 0b0011_0000);
    memory.set(0x70, 0b1000_0000);
    memory.set(0x71, 0b0000_0100); // jump to program
    memory.set(0x72, 0b0000_0000);

}

static SCREEN_WHITE_PROGRAM: &[u8] = &[
    // set up r0 and r1 for vram
    0b0000_0100,
    0b1100_0000,
    0b0000_1000,

    0b0000_0100,
    0b1100_1000,
    0b0000_0000,

    // set up r2 for bit counter for white thing
    0b0000_0100,
    0b1101_0000,
    // 0b1000_0000,
    0b1111_1111,

    // 0x409, skips ahead to 0x412
    0b0010_1100,
    0b1000_0000,
    0b0000_0100,
    0b0001_0010,


    // 0x40D: update vram ptr to R2, SHR R2
    // set vram ptr to R2, SHR R2
    0b0000_0100,
    0b1000_0010, // R2 -> m1 (R0)

    // SHR
    // 0b0110_0100,
    // 0b0001_0000,
    0b0000_0000,
    0b0000_0000,

    // at end, op_ret
    0b0101_1100,
    // 0b0000_0000,

    // 412:
    // main loop I think
    // call 40D
    0b0101_1000,
    0b1000_0000,
    // address of 40D:
    0b0000_0100,
    0b0000_1101,

    // check if R2 is all zeros -> have iterated 8 times
    // op_comp w/ 0000_0000
    // 0b0100_1100,
    // 0b1101_0000,
    // 0b0000_0000,
    0b0000_0000,
    0b0000_0000,
    0b0000_0000,

    // jz to future where we reset R2 to 1000_0000 & increment R1 and R0
    0b0010_1100, // temp change 2 jump
    0b1000_0000,
    // +2 (41B 41C) to 421:
    0b0000_0100,
    0b0010_0001,

    // go back to main loop (+4)
    // jump to continue
    0b0100_0000, // 41D
    0b1000_0000,
    // 0x412 (or wherever main loop is):
    0b0000_0100,
    0b0001_0010,



    // func to increment R1 and R0:
    0b0000_1000, // 421
    0b1100_1000,
    0b0000_0001,

    // carry jump to reset R1 and increment R0 (later)
    // +4 (424 425 426 427)
    0b0011_0100, // 424 check
    0b1000_0000,
    0b0000_0100, // 42F
    0b0010_1111,
    
    // otherwise jump to continue
    0b0100_0000,
    0b1000_0000,
    // 0x412 (or wherever main loop is):
    0b0000_0100, // 42A
    0b0001_0010,

   

    // reset R2
    0b0000_0100, // 42C
    0b1101_0000,
    0b1111_1111,

    // increment R0:
    0b0000_1000, // 42F
    0b1100_0000,
    0b0000_0001,
    
    // reset R1:
    0b0000_0100, // 432
    0b1100_1000,
    0b0000_0000,

    // check if R0 = 0b0001_0000 => if so done (or reset)
    0b0100_1100, // 435
    0b1100_0000,
    0b0001_0000, // correct I believe

    // zero flag jump to restart
    0b0011_0000,
    0b1000_0000,
    // 0x442 (halts)
    0b0000_0100,
    0x42,

    // return:
    // 0b0101_1100,

    // jump to continue
    0b0101_1000,
    0b1000_0000,
    // 0x412 (or wherever main loop is):
    0b0000_0100,
    0b0001_0010,


    0b1111_1100, // 442
];


static MOUSE_PROGRAM: &[u8] = &[
    // r0 and r1 are m1 (y?), m2 (x?) respectively; initialize @ 0, 0 (0x800, 0x07FF)
    0b0000_0100,
    0b1100_0000,
    0b0000_1000,

    0b0000_0100,
    0b1100_1000,
    0b0000_0000,
    

    // init r2 as all 1s, will move a line before i can figure out how to move 1 pixel
    0b0000_0100,
    0b1101_0000,
    0b1111_1111,

    // CORE LOOP: 0x409:

    // check keyboard status (0x1000?)
    // will do & 0000_0001 to check if a key is available:
    

    // put 0000_0001 into r3
    0b0000_0100,
    0b11_011_000,
    0b0000_0001,
    

    // put address 0x1001 (0b0001_0000__0000_0001) into r4, r5
    0b0000_0100,
    0b1110_0000,
    0b0001_0000,

    0b0000_0100,
    0b1110_1000,
    0b0000_0001,

    0b0001_1100, // AND op: compare m2 (r4, r5) w/ r3
    0b0101_1100, // 011 is R3, 100 is R4

    // COMP op of 0000_0001 w/ R3
    0b0100_1100,
    0b1101_1000,
    0b0000_0001,

    // jump not zero means we have a key to read

    // jump zero means we don't have a key to read -> go back to core loop 0x409/0b100_0000_1001:
    0b0011_0000,
    0b1000_0000,
    0b0000_0100, // means 0x409
    0b0000_1001,

    // otherwise, check key @ 0x1001 and, because we're doing arrow keys, subtract 27 (so an UP key will be 0)
    // then do jump zeros and subtract until we reach the number ONE, then restart cuz its useless
    
    // put address 0x1002 (0b0001_0000__0000_0010) into r4, r5
    0b0000_0100,
    0b11_100_000,
    0b0001_0000,

    0b0000_0100,
    0b11_101_000,
    0b0000_0010,

    // now put 27 into r6
    0b0000_0100,
    0b11_110_000,
    0b0001_1011,

    // now do m2 to r1 (R7)
    0b0000_0100,
    0b01_111_100,

    // now do sub R6 from R7 ; r1 - r2 -> r1 = R7, r2 = R6
    0b0000_1100,
    0b00_111_110,

    // set r0, r1 to black:
    0b000001_01, // i2 to m1
    0b00_000_000,
    255,


    // anyway, now if it's UP it'll be 0. put a jump not zero to dodge the next few instructions which adds 1 to R0
    // instead, added skip; just skip a few bytes?
    0b011100_00,
    0b10_000_000,
    5,

    // sub 1 from R0 (3 bytes); fix later, but currently its addition so it doesnt ruin everything lol
    0b000010_00,
    0b11_000_000,
    0b0000_0001,
    // LATER JUMP TO SCREEN UPDATE FUNCTION (4 bytes)


    // screen update function:
    // set r0, r1 to white
    0b000001_00, // i2 to m1
    0b10_000_010,


    // back to core loop?
    0b001011_00,
    0b10_000_000,
    0b0000_0100, // means 0x409
    0b0000_1001,






    0b111111_00,


    
    
// 0b1000_0000,
// 0b0000_0100,


];
// broken af
// hello world:

static HELLOWORLD: &[u8] = &[
    // draw h:
    
    // init R0 and R1 as 0x800
    0b000001_00,
    0b11_000_000,
    0x09,

    0b000001_00,
    0b11_001_000,
    0x00,

    0b000001_01,
    0b00_000_000,
    0b100_000_00,

    0b000001_00,
    0b11_001_000,
    0x10,

    0b000001_01,
    0b00_000_000,
    0b100_0_111_0,

    0b000001_00,
    0b11_001_000,
    0x20,

    0b000001_01,
    0b00_000_000,
    0b100_0_101_0,

    0b000001_00,
    0b11_001_000,
    0x30,

    0b000001_01,
    0b00_000_000,
    0b111_0_111_0,

    0b000001_00,
    0b11_001_000,
    0x40,

    0b000001_01,
    0b00_000_000,
    0b101_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x50,

    0b000001_01,
    0b00_000_000,
    0b101_0_111_0,

    // NEXT BYTE


    0b000001_00,
    0b11_001_000,
    0x01,

    0b000001_01,
    0b00_000_000,
    0b100_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x11,

    0b000001_01,
    0b00_000_000,
    0b100_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x21,

    0b000001_01,
    0b00_000_000,
    0b100_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x31,

    0b000001_01,
    0b00_000_000,
    0b100_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x41,

    0b000001_01,
    0b00_000_000,
    0b100_0_100_0,

    0b000001_00,
    0b11_001_000,
    0x51,

    0b000001_01,
    0b00_000_000,
    0b110_0_110_0,


    // NEXT BYTE (2)


    0b000001_00,
    0b11_001_000,
    0x02,

    0b000001_01,
    0b00_000_000,
    0b000_0_000_0,

    0b000001_00,
    0b11_001_000,
    0x12,

    0b000001_01,
    0b00_000_000,
    0b000_0_000_0,

    0b000001_00,
    0b11_001_000,
    0x22,

    0b000001_01,
    0b00_000_000,
    0b000_0_000_0,

    0b000001_00,
    0b11_001_000,
    0x32,

    0b000001_01,
    0b00_000_000,
    0b111_0_000_0,

    0b000001_00,
    0b11_001_000,
    0x42,

    0b000001_01,
    0b00_000_000,
    0b101_0_000_0,

    0b000001_00,
    0b11_001_000,
    0x52,

    0b000001_01,
    0b00_000_000,
    0b111_0_000_0,


    0b111111_00,

];

    


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

fn load_program(memory: &mut Bus, bytes: &[u8]) {
    let base = 0x400;
    for (i, byte) in bytes.iter().enumerate() {
        memory.set(base + i as u16, *byte);
    }
}


fn main() {
    
    let keyb = Keyboard::new();
    let ms = Mouse::new();

    let mut memory = Bus::new(ms, keyb);

    load_bootloader(&mut memory);
    load_program(&mut memory, HELLOWORLD);

    


    // ----- build VM ---------------------------------------------------------
    let cpu = Cpu::new();
    let vc  = VideoController::new(128, 128, 0x800);
    let vm  = Vm::new(memory, vc, cpu);

    // ----- spin up winit -----------------------------------------------------
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(vm);
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("winit error: {e}");
    }
}
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

            WindowEvent::RedrawRequested => {
                // ---- run VM and refresh framebuffer --------------------------
                // for _ in 0..3 { // run multiple steps per frame
                //     if !self.vm.cpu.halted {
                //         self.vm.step();

                //         // Debug: print when we hit certain addresses
                //         if self.vm.cpu.regs[0] == 0x0C && self.vm.cpu.regs[1] == 0x00 {
                //             println!("Halfway! R0={:02X}, R1={:02X}, PC={:04X}", 
                //             self.vm.cpu.regs[0], self.vm.cpu.regs[1], self.vm.cpu.pc);
                //         }
                //     }
                // }
                self.vm.step_many(100);
                // self.vm.cpu.status();
                
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
fn load_bootloader(memory: &mut Mem) {
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

static PROGRAM: &[u8] = &[
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

fn load_program(memory: &mut Mem, bytes: &[u8]) {
    let base = 0x400;
    for (i, byte) in bytes.iter().enumerate() {
        memory.set(base + i as u16, *byte);
    }
}


fn main() {
    // ----- sample program in memory -----------------------------------------
    let mut memory = Mem::new();

    load_bootloader(&mut memory);
    load_program(&mut memory, PROGRAM);

    


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
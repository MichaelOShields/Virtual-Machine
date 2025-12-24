
use crate::{Keyboard, Mouse, cpu::{Access, CPUExit, CPUMode, Fault}};
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum MemRange {
    Bootloader ( Range<u16> ), // kernel can read & execute
    KernelCore ( Range<u16> ), // kernel can read & execute
    KernelTraps ( Range<u16> ), // kernel can 
    KernelData ( Range<u16> ),
    KernelHeap ( Range<u16> ),
    KernelStack ( Range<u16> ),
    Vram ( Range<u16> ),
    Mmio ( Range<u16> ),

    UserCode ( Range<u16>, u8 ),
    UserData ( Range<u16>, u8 ),
    UserHeap ( Range<u16>, u8 ),
    UserStack ( Range<u16>, u8 ),
}

impl MemRange {

    pub fn check_access(&self, mode: CPUMode, access: Access, task_index: u8) -> Result<(), CPUExit> {
        match self.allows(mode, access, task_index) {
            true => Ok(()),
            false => Err(CPUExit::Fault(Fault::IllegalMemAccess)),
        }
    }

    pub fn contains(&self, addr: u16) -> bool {
        match self {
            MemRange::Bootloader(r)
            | MemRange::KernelCore(r)
            | MemRange::KernelTraps(r)
            | MemRange::KernelData(r)
            | MemRange::KernelHeap(r)
            | MemRange::KernelStack(r)
            | MemRange::Vram(r)
            | MemRange::Mmio(r)
            | MemRange::UserCode(r, _)
            | MemRange::UserData(r, _)
            | MemRange::UserHeap(r, _)
            | MemRange::UserStack(r, _)
            => r.contains(&addr),
        }
    }

    pub fn allows(&self, mode: CPUMode, access: Access, current_task: u8) -> bool {
        return match self {
            Self::Bootloader(_) => mode == CPUMode::K && matches!(access, Access::X | Access::R),
            Self::KernelCore(_) => mode == CPUMode::K && matches!(access, Access::R | Access::X),
            Self::KernelTraps(_) => mode == CPUMode::K && matches!(access, Access::X | Access::R),
            Self::KernelData(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::KernelHeap(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::KernelStack(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::Vram(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::Mmio(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            
            Self::UserCode(_, t) => {
                // println!("t (code): {}", t);
                (mode == CPUMode::K && matches!(access, Access::X | Access::R)) || 
                (mode == CPUMode::U && matches!(access, Access::X | Access::R) && (*t == current_task))},
            
            Self::UserData(_, t) =>  
                (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                (mode == CPUMode::U && matches!(access, Access::R | Access::W) && (*t == current_task)),
            
            Self::UserHeap(_, t) => 
                (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                (mode == CPUMode::U && matches!(access, Access::R | Access::W) && (*t == current_task)),

            Self::UserStack(_, t) => 
                {
                    // println!("t (stack): {}", t);
                    (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                    (mode == CPUMode::U && matches!(access, Access::R | Access::W) && (*t == current_task))},


            _ => panic!("Couldn't identify memory range."),
            
        }
    }
}

// memory bus
// owns keyboard, mouse, etc
pub struct Bus {
    ram: [u8; 65536],

    mouse: Mouse,
    keyboard: Keyboard,

    // memory ranges
    // bootloader: MemRange,
    // kernel_core: MemRange,
    // kernel_traps: MemRange,
    // kernel_data: MemRange,
    // kernel_heap: MemRange,
    // kernel_stack: MemRange,
    // mmio: MemRange,

    // user_code: MemRange,
    // user_data: MemRange,
    // user_heap: MemRange,
    // user_stack: MemRange,
    ranges: Vec<MemRange>,
    mmio_range: Range<u16>,
}



impl Bus {
    pub fn new(
        mouse: Mouse,
        keyboard: Keyboard,

        bootloader: Range<u16>,
        kernel_core: Range<u16>,
        kernel_traps: Range<u16>,
        kernel_data: Range<u16>,
        kernel_heap: Range<u16>,
        kernel_stack: Range<u16>,
        vram: Range<u16>,
        mmio: Range<u16>,


        // later: make this way more concise. probably use a vec or array
        user_code_0:  Range<u16>,
        user_data_0:  Range<u16>,
        user_heap_0:  Range<u16>,
        user_stack_0: Range<u16>,

        user_code_1:  Range<u16>,
        user_data_1:  Range<u16>,
        user_heap_1:  Range<u16>,
        user_stack_1: Range<u16>,

        user_code_2:  Range<u16>,
        user_data_2:  Range<u16>,
        user_heap_2:  Range<u16>,
        user_stack_2: Range<u16>,

        user_code_3:  Range<u16>,
        user_data_3:  Range<u16>,
        user_heap_3:  Range<u16>,
        user_stack_3: Range<u16>,

        user_code_4:  Range<u16>,
        user_data_4:  Range<u16>,
        user_heap_4:  Range<u16>,
        user_stack_4: Range<u16>,

        user_code_5:  Range<u16>,
        user_data_5:  Range<u16>,
        user_heap_5:  Range<u16>,
        user_stack_5: Range<u16>,

        user_code_6:  Range<u16>,
        user_data_6:  Range<u16>,
        user_heap_6:  Range<u16>,
        user_stack_6: Range<u16>,

        user_code_7:  Range<u16>,
        user_data_7:  Range<u16>,
        user_heap_7:  Range<u16>,
        user_stack_7: Range<u16>,

    ) -> Self {
        let mmio_range = mmio.clone();
        let bootloader= MemRange::Bootloader (bootloader);
        let kernel_core= MemRange::KernelCore (kernel_core);
        let kernel_traps= MemRange::KernelTraps (kernel_traps);
        let kernel_data= MemRange::KernelData (kernel_data);
        let kernel_heap= MemRange::KernelHeap (kernel_heap);
        let kernel_stack= MemRange::KernelStack (kernel_stack);
        let vram = MemRange::Vram (vram);
        let mmio= MemRange::Mmio (mmio);

        let user_code_0  = MemRange::UserCode ( user_code_0, 0);
        let user_data_0  = MemRange::UserData ( user_data_0, 0);
        let user_heap_0  = MemRange::UserHeap ( user_heap_0, 0);
        let user_stack_0 = MemRange::UserStack ( user_stack_0, 0);

        let user_code_1  = MemRange::UserCode ( user_code_1, 1);
        let user_data_1  = MemRange::UserData ( user_data_1, 1);
        let user_heap_1  = MemRange::UserHeap ( user_heap_1, 1);
        let user_stack_1 = MemRange::UserStack ( user_stack_1, 1);

        let user_code_2  = MemRange::UserCode ( user_code_2, 2);
        let user_data_2  = MemRange::UserData ( user_data_2, 2);
        let user_heap_2  = MemRange::UserHeap ( user_heap_2, 2);
        let user_stack_2 = MemRange::UserStack ( user_stack_2, 2);

        let user_code_3  = MemRange::UserCode ( user_code_3, 3);
        let user_data_3  = MemRange::UserData ( user_data_3, 3);
        let user_heap_3  = MemRange::UserHeap ( user_heap_3, 3);
        let user_stack_3 = MemRange::UserStack ( user_stack_3, 3);

        let user_code_4  = MemRange::UserCode ( user_code_4, 4);
        let user_data_4  = MemRange::UserData ( user_data_4, 4);
        let user_heap_4  = MemRange::UserHeap ( user_heap_4, 4);
        let user_stack_4 = MemRange::UserStack ( user_stack_4, 4);

        let user_code_5  = MemRange::UserCode ( user_code_5, 5);
        let user_data_5  = MemRange::UserData ( user_data_5, 5);
        let user_heap_5  = MemRange::UserHeap ( user_heap_5, 5);
        let user_stack_5 = MemRange::UserStack ( user_stack_5, 5);

        let user_code_6  = MemRange::UserCode ( user_code_6, 6);
        let user_data_6  = MemRange::UserData ( user_data_6, 6);
        let user_heap_6  = MemRange::UserHeap ( user_heap_6, 6);
        let user_stack_6 = MemRange::UserStack ( user_stack_6, 6);

        let user_code_7  = MemRange::UserCode ( user_code_7, 7);
        let user_data_7  = MemRange::UserData ( user_data_7, 7);
        let user_heap_7  = MemRange::UserHeap ( user_heap_7, 7);
        let user_stack_7 = MemRange::UserStack ( user_stack_7, 7);


        let ranges = vec![
            bootloader,
            kernel_core,
            kernel_traps,
            kernel_data,
            kernel_heap,
            kernel_stack,
            vram,
            mmio,

            user_code_0,  user_data_0,  user_heap_0,  user_stack_0,
            user_code_1,  user_data_1,  user_heap_1,  user_stack_1,
            user_code_2,  user_data_2,  user_heap_2,  user_stack_2,
            user_code_3,  user_data_3,  user_heap_3,  user_stack_3,
            user_code_4,  user_data_4,  user_heap_4,  user_stack_4,
            user_code_5,  user_data_5,  user_heap_5,  user_stack_5,
            user_code_6,  user_data_6,  user_heap_6,  user_stack_6,
            user_code_7,  user_data_7,  user_heap_7,  user_stack_7,
        ];

        
        Self {
            ram: [0; 65536],

            mouse,
            keyboard,
            ranges,
            mmio_range,
        }
    }

    pub fn get_size(&mut self) -> u16 {
        return (self.ram.len() - 1) as u16;
    }
    pub fn check_access(&mut self, address: u16, mode: CPUMode, access: Access) -> Result<(), CPUExit> {

        for range in &self.ranges {
            let actual_range = match range {
                MemRange::Bootloader(r)
                | MemRange::KernelCore(r)
                | MemRange::KernelTraps(r)
                | MemRange::KernelData(r)
                | MemRange::KernelHeap(r)
                | MemRange::KernelStack(r)
                | MemRange::Vram(r)
                | MemRange::Mmio(r)
                | MemRange::UserCode(r, _)
                | MemRange::UserData(r, _)
                | MemRange::UserHeap(r, _)
                | MemRange::UserStack(r, _) => r,
            };
            if actual_range.contains(&address) {
                // println!("Range: {:?}", range);
                let result = range.check_access(mode, access, self.ram[0x12C8]); // dynamically grab current task
                match &result {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Got CPUExit {:?} at range {:?} (0x{:0x}..0x{:0x})", e, range, actual_range.start, actual_range.end);
                        println!("Precise PC: {:0x}", address);
                        println!("Mode: {:?}\nAccess: {:?}", mode, access);
                        println!("Attempted to access address 0x{:0x}", address);
                    }
                }
                return result;
            }
        }
        
        Err(CPUExit::Fault(Fault::IllegalMemAccess))

    }

    pub fn get(&mut self, address: u16, mode: CPUMode, access: Access) -> Result<u8, CPUExit> {

        self.check_access(address, mode, access)?;

        if self.mmio_range.contains(&address) {
            return Ok(self.mmio_get(address)?);
        }
        return Ok(self.ram[address as usize]);

    }

    pub fn mmio_get(&mut self, address: u16) -> Result<u8, CPUExit> {
        // println!("Getting from MMIO...");
        let keyboard_status: u16 = self.mmio_range.start;
        if address == keyboard_status {
            return Ok(self.keyboard.status());
        }
        else if address == keyboard_status + 1 {
            return Ok(self.keyboard.pop_key());
        }
        else {
            return Err(CPUExit::Fault(Fault::IllegalMemAccess));
        }
    }

    pub fn force_set(&mut self, dest: u16, src: u8) {
        self.ram[dest as usize] = src;
    }

    pub fn force_get(&mut self, dest: u16) -> u8 {
        return self.ram[dest as usize];
    }

    pub fn set(&mut self, dest: u16, src: u8, mode: CPUMode, access: Access) -> Result<(), CPUExit> {
        self.check_access(dest, mode, access)?;
        self.ram[dest as usize] = src;

        Ok(())
    }

    pub fn get_mutable_ref(&mut self, address: u16, mode: CPUMode, access: Access) -> Result<&mut u8, CPUExit> {

        self.check_access(address, mode, access)?;

        if self.mmio_range.contains(&address) {
            return Err(CPUExit::Fault(Fault::IllegalMemAccess))
        }
        return Ok(&mut self.ram[address as usize]);

    }

    pub fn get_range(&mut self, a: u16, b: u16) -> &[u8] { // ONLY EXPOSED TO VM ONLY EXPOSED TO VM ONLY EXPOSED TO VM
        // println!("{:?}", &self.ram[a as usize..b as usize]);
        return &self.ram[a as usize..b as usize];
    }

    pub fn key_inject(&mut self, key: u8) {
        self.keyboard.inject_key(key);
    }

    pub fn status(&mut self) {
        self.keyboard.debug();
    }
}

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
    Mmio ( Range<u16> ),

    UserCode ( Range<u16> ),
    UserData ( Range<u16> ),
    UserHeap ( Range<u16> ),
    UserStack ( Range<u16> ),
}

impl MemRange {

    pub fn check_access(&self, mode: CPUMode, access: Access) -> Result<(), CPUExit> {
        match self.allows(mode, access) {
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
            | MemRange::Mmio(r)
            | MemRange::UserCode(r)
            | MemRange::UserData(r)
            | MemRange::UserHeap(r)
            | MemRange::UserStack(r) => r.contains(&addr),
        }
    }

    pub fn allows(&self, mode: CPUMode, access: Access) -> bool {
        return match self {
            Self::Bootloader(_) => mode == CPUMode::K && matches!(access, Access::X | Access::R),
            Self::KernelCore(_) => mode == CPUMode::K && matches!(access, Access::R | Access::X),
            Self::KernelTraps(_) => mode == CPUMode::K && matches!(access, Access::X),
            Self::KernelData(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::KernelHeap(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::KernelStack(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            Self::Mmio(_) => mode == CPUMode::K && matches!(access, Access::R | Access::W),
            
            Self::UserCode(_) => 
                (mode == CPUMode::K && matches!(access, Access::X)) || 
                (mode == CPUMode::U && matches!(access, Access::X)),
            
            Self::UserData(_) => 
                (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                (mode == CPUMode::U && matches!(access, Access::R | Access::W)),
            
            Self::UserHeap(_) => 
                (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                (mode == CPUMode::U && matches!(access, Access::R | Access::W)),

            Self::UserStack(_) => 
                (mode == CPUMode::K && matches!(access, Access::R | Access::W)) || 
                (mode == CPUMode::U && matches!(access, Access::R | Access::W)),


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
        mmio: Range<u16>,

        user_code: Range<u16>,
        user_data: Range<u16>,
        user_heap: Range<u16>,
        user_stack: Range<u16>,
    ) -> Self {
        let mmio_range = mmio.clone();
        let bootloader= MemRange::Bootloader(bootloader);
        let kernel_core= MemRange::KernelCore(kernel_core);
        let kernel_traps= MemRange::KernelTraps(kernel_traps);
        let kernel_data= MemRange::KernelData(kernel_data);
        let kernel_heap= MemRange::KernelHeap(kernel_heap);
        let kernel_stack= MemRange::KernelStack(kernel_stack);
        let mmio= MemRange::Mmio(mmio);

        let user_code = MemRange::UserCode(user_code);
        let user_data= MemRange::UserData(user_data);
        let user_heap= MemRange::UserHeap(user_heap);
        let user_stack= MemRange::UserStack(user_stack);
        let ranges = vec![bootloader, kernel_core, kernel_traps, kernel_data, kernel_heap, kernel_stack, mmio, user_code, user_data, user_heap, user_stack];
        
        Self {
            ram: [0; 65536],

            mouse,
            keyboard,
            ranges,
            mmio_range,
        }
    }

    fn check_access(&mut self, address: u16, mode: CPUMode, access: Access) -> Result<(), CPUExit> {

        for range in &self.ranges {
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
        println!("Getting from MMIO...");
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

    pub fn set(&mut self, dest: u16, src: u8, mode: CPUMode) -> Result<(), CPUExit> {
        self.check_access(dest, mode, Access::W)?;
        self.ram[dest as usize] = src;

        Ok(())
    }

    pub fn get_range(&mut self, a: u16, b: u16) -> &[u8] { // ONLY EXPOSED TO VM ONLY EXPOSED TO VM ONLY EXPOSED TO VM
        return &self.ram[a as usize..b as usize];
    }

    pub fn key_inject(&mut self, key: u8) {
        self.keyboard.inject_key(key);
    }

    pub fn status(&mut self) {
        self.keyboard.debug();
    }
}
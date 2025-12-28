use super::{MemoryBus, Cartridge};

// Small games of not more than 32 KiB ROM do not require a MBC chip for ROM banking.
pub struct ROM(Vec<u8>);

impl ROM {
    pub fn new(data: Vec<u8>) -> Self {
        ROM(data)
    }
}

impl MemoryBus for ROM {
    fn read_byte(&self, address: u16) -> u8 { self.0[address as usize] }
    // ROM is read-only so no write functionality.
    fn write_byte(&mut self, _: u16, _: u8) {}
}

impl Cartridge for ROM { 

    fn len(&self) -> usize { self.0.len() }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) {}

    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> *const u8 { self.0.as_ptr() }
}
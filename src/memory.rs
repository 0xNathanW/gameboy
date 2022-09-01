use std::path::Path;

use crate::cartridge::{Cartridge, self};


const HRAM_SIZE: usize = 127;        // High RAM.
const WRAM_SIZE:  usize = 32_768;    // 32KB Work RAM.

pub trait MemoryBus {

    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, b: u8);

    fn read_word(&self, address: u16) -> u16 {
        (self.read_byte(address) as u16) | ((self.read_byte(address+1) as u16) << 8)
    }

    fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0xFF) as u8);
        self.write_byte(address+1, (word >> 8) as u8)
    }

}

/*
When the CPU tries to access a given address, it’s the Memory's job to 
determine which piece of underlying hardware that particular address 
belongs to, and to forward the access to that device as appropriate.
*/

pub struct Memory {
    pub cartridge: Box<dyn Cartridge>,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl Memory {
    
    pub fn new(path: &Path) -> Self {
        let mut mem = Self {
            cartridge: cartridge::open_cartridge(path),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
        };
        mem
    }
}

/*
General Memory Map
  0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
  8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
  A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
  C000-CFFF   4KB Work RAM Bank 0 (WRAM)
  D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
  E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
  FE00-FE9F   Sprite Attribute Table (OAM)
  FEA0-FEFF   Not Usable
  FF00-FF7F   I/O Ports
  FF80-FFFE   High RAM (HRAM)
  FFFF        Interrupt Enable Register
*/

impl MemoryBus for Memory {

    fn read_byte(&self, _address: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, _address: u16, _b: u8) {
        todo!()
    }
}
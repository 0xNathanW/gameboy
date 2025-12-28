use std::{path::PathBuf, fs::File, io::Write};

use super::{MemoryBus, Cartridge, Result};
#[cfg(not(target_arch = "wasm32"))]
use super::load_save;
// (max 256 KiB ROM and 512x4 bits RAM)

pub struct MBC2 {
    rom:        Vec<u8>,
    rom_bank:   usize,

    // Enables the reading and writing of external RAM.
    ram_enable: bool,
    ram:        Vec<u8>,

    save_path:  Option<PathBuf>,
}

impl MBC2 {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(rom: Vec<u8>, ram_size: usize, save_path: Option<PathBuf>) -> Self {
        
        let ram = match save_path {
            Some(ref path) => load_save(path, ram_size),
            None => vec![0; ram_size],
        };

        Self { 
            ram,
            ram_enable: false,
            rom,
            rom_bank: 1, 
            save_path, 
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(rom: Vec<u8>, ram_size: usize, save_data: Option<Vec<u8>>) -> Self {
        
        let ram = match save_data {
            Some(data) => data,
            None => vec![0; ram_size],
        };

        Self { 
            ram,
            ram_enable: false,
            rom,
            rom_bank: 1, 
            save_path: None,
        }
    }
}

impl Cartridge for MBC2 {

    fn len(&self) -> usize { self.rom.len() }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) -> Result<()> {
        match &self.save_path {
            Some(path) => {
                let mut file = File::create(path)?;
                file.write_all(&self.ram)?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> Result<*const u8> {
        Ok(self.ram.as_ptr())
    }
}

impl MemoryBus for MBC2 {

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000–3FFF — ROM Bank 0 [read-only]
            0x0000 ..= 0x3FFF => self.rom[address as usize],
            // 4000–7FFF — ROM Bank $01-0F [read-only]
            0x4000 ..= 0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                self.rom[offset + (address as usize - 0x4000)]
            },
            // A000–A1FF — Built-in RAM
            0xA000 ..= 0xA1FF => {
                if self.ram_enable {
                    self.ram[(address - 0xA000) as usize]
                } else {
                    0
                }
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        // All MBC2 writes only use lower 4 bits.
        let b = b & 0x0F;
        match address {
            0x0000 ..= 0x1FFF => {
                if address & 0x0100 == 0 {
                    self.ram_enable = b == 0x0a;
                }
            },
            0x2000 ..= 0x3FFF => {
                if address & 0x0100 != 0 {
                    self.rom_bank = b as usize;
                }
            },
            0xA000 ..= 0xA1FF => {
                if self.ram_enable {
                    self.ram[(address - 0xA000) as usize] = b;
                }
            },
            _ => {},
        }
    }
}
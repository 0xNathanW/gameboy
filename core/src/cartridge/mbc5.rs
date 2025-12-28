use std::{path::PathBuf, fs::File, io::Write};

use crate::{bus::MemoryBus, cartridge::Cartridge};
#[cfg(not(target_arch = "wasm32"))]
use super::load_save;

pub struct MBC5 {
    rom:        Vec<u8>,
    rom_bank:   usize,

    ram:        Vec<u8>,
    ram_bank:   usize,
    ram_enable: bool,

    save_path:  Option<PathBuf>
}

impl MBC5 {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(rom: Vec<u8>, ram_size: usize, save_path: Option<PathBuf>) -> Self {
        
        let ram = match save_path {
            Some(ref path) => load_save(path, ram_size),
            None => vec![0; ram_size],
        };

        Self { 
            ram,
            ram_bank: 0,
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
            ram_bank: 0,
            ram_enable: false,
            rom,
            rom_bank: 1,
            save_path: None, 
        }
    }
}

impl Cartridge for MBC5 {

    fn len(&self) -> usize { self.rom.len() }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) {
        match &self.save_path {
            Some(path) => {
                File::create(path).and_then(
                    |mut f| f.write_all(&self.ram)
                ).unwrap()
            }
            None => {},
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> *const u8 {
        self.ram.as_ptr()
    }
}

impl MemoryBus for MBC5 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000 ..= 0x3FFF => self.rom[address as usize],
            0x4000 ..= 0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                self.rom[offset + (address as usize - 0x4000)]
            },
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    let offset = 0x2000 * self.ram_bank;
                    self.ram[offset + (address as usize - 0xA000)]
                } else {
                    0
                }
            },
            _ => 0,
        }
    }
    
    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x0000 ..= 0x1FFF => self.ram_enable = b & 0x0F == 0xA,
            0x2000 ..= 0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | b as usize,
            0x3000 ..= 0x3FFF => self.rom_bank = (self.rom_bank & 0xFF)  | ((b as usize) << 8), 
            0x4000 ..= 0x5FFF => self.ram_bank = (b & 0xF) as usize,
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    let offset = 0x2000 * self.ram_bank;
                    self.ram[offset + (address as usize - 0xA000)] = b;
                }
            },
            _ => {},
        }
    }
}
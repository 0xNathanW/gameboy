use std::{path::PathBuf, fs::File, io::Read};

use crate::bus::MemoryBus;

pub struct MBC5 {
    rom:        Vec<u8>,
    rom_bank:   usize,

    ram:        Vec<u8>,
    ram_bank:   usize,
    ram_enable: bool,

    save_path:  Option<PathBuf>
}

impl MBC5 {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>, save_path: Option<PathBuf>) -> Self {
        let mut mbc = Self { save_path, ram, rom, ram_enable: false, rom_bank: 1, ram_bank: 0 };
        mbc.load_save();
        mbc
    }

    fn load_save(&mut self) {
        if self.save_path.is_some() {
            match File::open(self.save_path.as_ref().unwrap()) {
                Ok(mut path) => {
                    path.read_to_end(&mut self.ram).unwrap();
                },
                Err(_) => {},
            }
        }
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
            _ => panic!("bad address mbc5 (read): {:#2X}", address),
        }
    }
    
    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x0000 ..= 0x1FFF => self.ram_enable = b & 0x0F == 0xA,
            0x2000 ..= 0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | b as usize,
            0x3000 ..= 0x3FFF => self.rom_bank = (self.rom_bank & 0xFF)  | (((b as u8) as usize) << 8), 
            0x4000 ..= 0x5FFF => self.ram_bank = (b & 0xF) as usize,
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    let offset = 0x2000 * self.ram_bank;
                    self.ram[offset + (address as usize - 0xA000)] = b;
                }
            },
            _ => panic!("bad address mbc5 (write): {:#2X}", address),
        }
    }
}
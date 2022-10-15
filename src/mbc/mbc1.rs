use std::{path::PathBuf, io::Write, fs::File, vec};

use super::super::bus::MemoryBus;
use super::load_save;

/*
In its default configuration, MBC1 supports up to 512 KiB ROM with up to 
32 KiB of banked RAM. Some cartridges wire the MBC differently, where the 
2-bit RAM banking register is wired as an extension of the ROM banking register 
(instead of to RAM) in order to support up to 2 MiB ROM, at the cost of only 
supporting a fixed 8 KiB of cartridge RAM. All MBC1 cartridges with 1 MiB of ROM 
or more use this alternate wiring. Also see the note on MBC1M multi-game 
compilation carts below.
*/
#[derive(Default)]
pub struct MBC1 {
    rom: Vec<u8>, 
    // This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000-7FFF region.
    rom_bank: u8,
    
    // Enables the reading and writing of external RAM.
    ram_enable: bool,
    ram: Vec<u8>,
    // This second 2-bit register can be used to select a RAM Bank in range from $00-$03 
    // (32 KiB ram carts only), or to specify the upper two bits (bits 5-6) of the ROM 
    // Bank number (1 MiB ROM or larger carts only).
    ram_bank: u8,

    // This 1-bit register selects between the two MBC1 banking modes, 
    //controlling the behaviour of the secondary 2-bit banking register (above).
    mode: bool,
    save_path: Option<PathBuf>,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, ram_size: usize, save_path: Option<PathBuf>) -> Self {
        
        let ram = match save_path {
            Some(ref path) => load_save(path, ram_size),
            None => vec![0; ram_size],
        };

        Self { 
            ram, 
            rom, 
            rom_bank: 1, 
            save_path, 
            ..Default::default() 
        }
    }
}

// Saves contents of ram to savefile when dropped.
impl Drop for MBC1 {
    fn drop(&mut self) {
        match &self.save_path {
            Some(path) => {
                File::create(path).and_then(|mut f| f.write_all(&self.ram)).unwrap();
                println!("save write");
            },
            None => {},
        }
    }
}

impl MemoryBus for MBC1 {

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000-3FFF - ROM Bank X0 (Read Only)
            0x0000 ..= 0x3FFF => self.rom[address as usize],
            // 4000-7FFF - ROM Bank 01-7F (Read Only)
            0x4000 ..= 0x7FFF => {
                let offset = 0x4000 * self.rom_bank as usize;
                self.rom[offset + (address as usize - 0x4000)]
            }
            // A000-BFFF - RAM Bank 00-03, if any (Read/Write)
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    let offset = self.ram_bank as usize * 8_192;  
                    self.ram[offset + address as usize - 0xA000] 
                } else { 
                    0 
                }
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            // Registers.
            // Any value with 0xa in the lower 4 bits enables ram.
            0x0000 ..= 0x1FFF => self.ram_enable = b & 0x0F == 0x0A,
            // ROM bank number (write only) - lower 5 bits.
            0x2000 ..= 0x3FFF => {
                let n = if b == 0 { 1 } else { b };
                self.rom_bank = self.rom_bank & 0b0110_0000 | n & 0b0001_1111;
            }
            // RAM Bank Number - or - Upper Bits of ROM Bank Number (Write Only)
            0x4000 ..= 0x5FFF => {
                if self.mode {
                    self.ram_bank = b & 0b11;
                } else {
                    self.rom_bank = self.rom_bank & 0b0001_1111 | (b & 3) << 5;
                }
            },
            // Banking Mode Select (Write Only)
            0x6000 ..= 0x7FFF => self.mode = b == 1,
            _ => {},
        }
    }
}

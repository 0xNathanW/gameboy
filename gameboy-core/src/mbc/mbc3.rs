use std::{
    path::PathBuf, 
    fs::File, 
    io::Write,
    time::SystemTime,
};

use crate::{bus::MemoryBus, cartridge::Cartridge};
#[cfg(not(target_arch = "wasm32"))]
use super::load_save;

/*
(max 2MByte ROM and/or 32KByte RAM and Timer)
Beside for the ability to access up to 2MB ROM (18 banks), and 32KB RAM (4 banks), the MBC3 also includes a built-in Real Time Clock (RTC). 
The RTC requires an external 32.768 kHz Quartz Oscillator, and an external battery (if it should continue to tick when the Game Boy is turned off).
*/

struct RealTimeClock {
    seconds:    u8,
    mintues:    u8,
    hours:      u8,
    dl:         u8,
    dh:         u8,
    pub zero:       u64,
}

impl RealTimeClock {
    fn new(rtc_path: Option<PathBuf>) -> Option<RealTimeClock> {
        match rtc_path {
            Some(path) => {
                let zero = match std::fs::read(path) {
                    Ok(f) => {
                        let mut b = [0_u8; 8];
                        b.copy_from_slice(&f);
                        u64::from_be_bytes(b)
                    }
                    Err(_) => SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                Some(Self {
                    seconds: 0,
                    mintues: 0,
                    hours: 0,
                    dl: 0,
                    dh: 0,
                    zero
                    })
            },
            None => None,
        }
    }

    fn step(&mut self) {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() - self.zero;
        
        self.seconds = (duration % 60) as u8;
        self.mintues = (duration / 60 % 60) as u8;
        self.hours   = (duration / 3600 % 24) as u8;

        let days = (duration / 3600 / 24) as u16;
        self.dl = (days % 256) as u8;
        match days {
            0x0 ..= 0xFF => {},
            0x100 ..= 0x1FF => {
                self.dh |= 1;
            },
            _ => {
                self.dh |= 1;
                self.dh |= 0x80;
            },
        }
    }
}

/*
The Clock Counter Registers
08h  RTC S   Seconds   0-59 (0-3Bh)
09h  RTC M   Minutes   0-59 (0-3Bh)
0Ah  RTC H   Hours     0-23 (0-17h)
0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
      Bit 0  Most significant bit of Day Counter (Bit 8)
      Bit 6  Halt (0=Active, 1=Stop Timer)
      Bit 7  Day Counter Carry Bit (1=Counter Overflow)
*/

impl MemoryBus for RealTimeClock {
    
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x08 => self.seconds,
            0x09 => self.mintues,
            0x0A => self.hours,
            0x0B => self.dl,
            0x0C => self.dh,
            _ => panic!("bad address rtc (read): {:#2X}", address),
        }
    }
    
    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x08 => self.seconds = b,
            0x09 => self.mintues = b,
            0x0A => self.hours = b,
            0x0B => self.dl = b,
            0x0C => self.dh = b,
            _ => panic!("bad address rtc (write): {:#2X}", address),
        }
    }
}

pub struct MBC3 {
    rom:        Vec<u8>,
    rom_bank:   usize,

    ram:        Vec<u8>,
    ram_bank:   usize,
    ram_enable: bool,

    rtc:        Option<RealTimeClock>,
    save_path:  Option<PathBuf>,
}

impl MBC3 {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(rom: Vec<u8>, ram_size: usize, save_path: Option<PathBuf>, rtc_path: Option<PathBuf>) -> Self {
        
        let ram = match save_path {
            Some(ref path) => load_save(path, ram_size),
            None => vec![0; ram_size],
        };

        Self {
            ram,
            ram_bank: 1,
            rom,
            rom_bank: 0,
            ram_enable: false,
            save_path,
            rtc: RealTimeClock::new(rtc_path),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(rom: Vec<u8>, ram_size: usize, save_data: Option<Vec<u8>>, rtc_path: Option<PathBuf>) -> Self {
        
        let ram = match save_data {
            Some(data) => data,
            None => vec![0; ram_size],
        };

        Self {
            ram,
            ram_bank: 1,
            rom,
            rom_bank: 0,
            ram_enable: false,
            save_path: None, 
            rtc: RealTimeClock::new(rtc_path),
        }
    }
}

impl Cartridge for MBC3 {
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) {
        match self.save_path.clone() {
            None => {},
            Some(path) => {
                let mut file = match File::create(path) {
                    Ok(f) => f,
                    Err(_) => return,
                };
                // Write real time clock.
                if self.rtc.is_some() {
                    file.write_all(
                        &self.rtc.as_ref().unwrap().zero.to_be_bytes()
                    ).unwrap();
                }
                // Write ram.
                file.write_all(&self.ram).unwrap();
            },
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> *const u8 {
        self.ram.as_ptr()
    }
}

impl MemoryBus for MBC3 {
    
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000-3FFF - ROM Bank 00 (Read Only)
            0x0000 ..= 0x3FFF => self.rom[address as usize],
            // 4000-7FFF - ROM Bank 01-7F (Read Only)
            0x4000 ..= 0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                self.rom[offset + (address as usize - 0x4000)]
            },
            // A000-BFFF - RAM Bank 00-03, if any (Read/Write)
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 3 {
                        let offset = self.ram_bank * 0x2000;
                        self.ram[offset + (address as usize - 0xA000)]
                    } else {
                        match &self.rtc {
                            Some(rtc) => rtc.read_byte(self.ram_bank as u16),
                            None => 0,
                        }
                    }
                } else {
                    0
                }
            },
            _ => panic!("bad address mbc3 (read): {:#2X}", address),
        }
    }
    
    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            // 0000-1FFF - RAM and Timer Enable (Write Only)
            0x0000 ..= 0x1FFF => self.ram_enable = b & 0xF == 0xA,
            // 2000-3FFF - ROM Bank Number (Write Only)
            // Whole 7 bits of the RAM Bank Number are written directly to this address.
            0x2000 ..= 0x3FFF => {
                let n = b & 0b0111_1111;
                let n = if n == 0 { 1 } else { n };
                self.rom_bank = n as usize;
            },
            // 4000-5FFF - RAM Bank Number - or - RTC Register Select (Write Only)
            0x4000 ..= 0x5FFF => {
                let n = b & 0x0F;
                self.ram_bank = n as usize;
            },
            // 6000-7FFF - Latch Clock Data (Write Only)
            0x6000 ..= 0x7FFF => {
                if b & 1 != 0 && self.rtc.is_some() {
                    self.rtc.as_mut().unwrap().step();
                }
            },
            0xA000 ..= 0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 3 {
                        let offset = 0x2000 * self.ram_bank;
                        self.ram[offset + (address as usize - 0xA000)] = b;
                    } else {
                        match &mut self.rtc {
                            Some(rtc) => rtc.write_byte(address, b),
                            None => {},
                        }
                    }
                }
            },
            _ => panic!("bad address mbc3 (write): {:#2X}", address),
        }
    }
}
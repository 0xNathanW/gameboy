use super::{Cartridge, MemoryBus};
use std::time::SystemTime;

/*
(max 2MByte ROM and/or 32KByte RAM and Timer)
Beside for the ability to access up to 2MB ROM (18 banks), and 32KB RAM (4 banks), the MBC3 also includes a built-in Real Time Clock (RTC).
The RTC requires an external 32.768 kHz Quartz Oscillator, and an external battery (if it should continue to tick when the Game Boy is turned off).
*/

struct RealTimeClock {
    seconds: u8,
    mintues: u8,
    hours: u8,
    dl: u8,
    dh: u8,
    pub zero: u64,
}

impl RealTimeClock {
    fn new(rtc_zero: Option<u64>) -> Option<RealTimeClock> {
        let zero = rtc_zero.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        Some(Self {
            seconds: 0,
            mintues: 0,
            hours: 0,
            dl: 0,
            dh: 0,
            zero,
        })
    }

    fn step(&mut self) {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.zero;

        self.seconds = (duration % 60) as u8;
        self.mintues = (duration / 60 % 60) as u8;
        self.hours = (duration / 3600 % 24) as u8;

        let days = (duration / 3600 / 24) as u16;
        self.dl = (days % 256) as u8;
        match days {
            0x0..=0xFF => {}
            0x100..=0x1FF => {
                self.dh |= 1;
            }
            _ => {
                self.dh |= 1;
                self.dh |= 0x80;
            }
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
            _ => panic!("invalid address rtc (read): {:#2X}", address),
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x08 => self.seconds = b,
            0x09 => self.mintues = b,
            0x0A => self.hours = b,
            0x0B => self.dl = b,
            0x0C => self.dh = b,
            _ => panic!("invalid address rtc (write): {:#2X}", address),
        }
    }
}

pub struct MBC3 {
    rom: Vec<u8>,
    rom_bank: usize,

    ram: Vec<u8>,
    ram_bank: usize,
    ram_enable: bool,

    rtc: Option<RealTimeClock>,
}

impl MBC3 {
    pub fn new(
        rom: Vec<u8>,
        ram_size: usize,
        save_data: Option<Vec<u8>>,
        has_rtc: bool,
        rtc_zero: Option<u64>,
    ) -> Self {
        let ram = save_data.unwrap_or_else(|| vec![0; ram_size]);

        Self {
            ram,
            ram_bank: 1,
            rom,
            rom_bank: 0,
            ram_enable: false,
            rtc: if has_rtc {
                RealTimeClock::new(rtc_zero)
            } else {
                None
            },
        }
    }
}

impl Cartridge for MBC3 {
    fn save_data(&self) -> Option<&[u8]> {
        if self.ram.is_empty() {
            None
        } else {
            Some(&self.ram)
        }
    }

    fn ram_size(&self) -> usize {
        self.ram.len()
    }

    fn rtc_zero(&self) -> Option<u64> {
        self.rtc.as_ref().map(|rtc| rtc.zero)
    }

    fn len(&self) -> usize {
        self.rom.len()
    }
}

impl MemoryBus for MBC3 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000-3FFF - ROM Bank 00 (Read Only)
            0x0000..=0x3FFF => self.rom[address as usize],
            // 4000-7FFF - ROM Bank 01-7F (Read Only)
            0x4000..=0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                self.rom[offset + (address as usize - 0x4000)]
            }
            // A000-BFFF - RAM Bank 00-03, if any (Read/Write)
            0xA000..=0xBFFF => {
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
            }
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            // 0000-1FFF - RAM and Timer Enable (Write Only)
            0x0000..=0x1FFF => self.ram_enable = b & 0xF == 0xA,
            // 2000-3FFF - ROM Bank Number (Write Only)
            // Whole 7 bits of the RAM Bank Number are written directly to this address.
            0x2000..=0x3FFF => {
                let n = b & 0b0111_1111;
                let n = if n == 0 { 1 } else { n };
                self.rom_bank = n as usize;
            }
            // 4000-5FFF - RAM Bank Number - or - RTC Register Select (Write Only)
            0x4000..=0x5FFF => {
                let n = b & 0x0F;
                self.ram_bank = n as usize;
            }
            // 6000-7FFF - Latch Clock Data (Write Only)
            0x6000..=0x7FFF => {
                if b & 1 != 0 && self.rtc.is_some() {
                    self.rtc.as_mut().unwrap().step();
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 3 {
                        let offset = 0x2000 * self.ram_bank;
                        self.ram[offset + (address as usize - 0xA000)] = b;
                    } else {
                        match &mut self.rtc {
                            Some(rtc) => rtc.write_byte(address, b),
                            None => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

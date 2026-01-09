use super::{Cartridge, MemoryBus};

pub struct MBC5 {
    rom: Vec<u8>,
    rom_bank: usize,

    ram: Vec<u8>,
    ram_bank: usize,
    ram_enable: bool,
}

impl MBC5 {
    pub fn new(rom: Vec<u8>, ram_size: usize, save_data: Option<Vec<u8>>) -> Self {
        let ram = save_data.unwrap_or_else(|| vec![0; ram_size]);

        Self {
            ram,
            ram_bank: 0,
            ram_enable: false,
            rom,
            rom_bank: 1,
        }
    }
}

impl Cartridge for MBC5 {
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

    fn len(&self) -> usize {
        self.rom.len()
    }

    fn reset(&mut self) {
        self.rom_bank = 1;
        self.ram_bank = 0;
        self.ram_enable = false;
    }
}

impl MemoryBus for MBC5 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                let idx = offset + (address as usize - 0x4000);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => {
                if self.ram_enable && !self.ram.is_empty() {
                    let offset = 0x2000 * self.ram_bank;
                    let idx = offset + (address as usize - 0xA000);
                    self.ram.get(idx).copied().unwrap_or(0xFF)
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enable = b & 0x0F == 0xA,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | b as usize,
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | ((b as usize) << 8),
            0x4000..=0x5FFF => self.ram_bank = (b & 0xF) as usize,
            0xA000..=0xBFFF => {
                if self.ram_enable && !self.ram.is_empty() {
                    let offset = 0x2000 * self.ram_bank;
                    let idx = offset + (address as usize - 0xA000);
                    if let Some(x) = self.ram.get_mut(idx) {
                        *x = b;
                    }
                }
            }
            _ => {}
        }
    }
}

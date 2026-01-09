use super::{Cartridge, MemoryBus};

// (max 256 KiB ROM and 512x4 bits RAM)
pub struct MBC2 {
    rom: Vec<u8>,
    rom_bank: usize,

    // Enables the reading and writing of external RAM.
    ram_enable: bool,
    ram: Vec<u8>,
}

impl MBC2 {
    pub fn new(rom: Vec<u8>, ram_size: usize, save_data: Option<Vec<u8>>) -> Self {
        let ram = save_data.unwrap_or_else(|| vec![0; ram_size]);

        Self {
            ram,
            ram_enable: false,
            rom,
            rom_bank: 1,
        }
    }
}

impl Cartridge for MBC2 {
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
        self.ram_enable = false;
    }
}

impl MemoryBus for MBC2 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000–3FFF — ROM Bank 0 [read-only]
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            // 4000–7FFF — ROM Bank $01-0F [read-only]
            0x4000..=0x7FFF => {
                let offset = 0x4000 * self.rom_bank;
                let idx = offset + (address as usize - 0x4000);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            // A000–A1FF — Built-in RAM
            0xA000..=0xA1FF => {
                if self.ram_enable && !self.ram.is_empty() {
                    let idx = (address - 0xA000) as usize;
                    self.ram.get(idx).copied().unwrap_or(0xFF)
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        // All MBC2 writes only use lower 4 bits.
        let b = b & 0x0F;
        match address {
            0x0000..=0x1FFF => {
                if address & 0x0100 == 0 {
                    self.ram_enable = b == 0x0a;
                }
            }
            0x2000..=0x3FFF => {
                if address & 0x0100 != 0 {
                    self.rom_bank = b as usize;
                }
            }
            0xA000..=0xA1FF => {
                if self.ram_enable && !self.ram.is_empty() {
                    let idx = (address - 0xA000) as usize;
                    if let Some(x) = self.ram.get_mut(idx) {
                        *x = b;
                    }
                }
            }
            _ => {}
        }
    }
}

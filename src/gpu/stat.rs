use crate::bus::MemoryBus;
use crate::bit::Bit;

// STAT (LCD status) (R/W) - FF41
/*
|Bit| Name                                 | Usage notes          |
| 6 | LYC=LY STAT Interrupt source         | 0=Off, 1=On          |
| 5 | Mode 2 OAM STAT Interrupt source     | 0=Off, 1=On          |
| 4 | Mode 1 VBlank STAT Interrupt source  | 0=Off, 1=On          |
| 3 | Mode 0 HBlank STAT Interrupt source  | 0=Off, 1=On          |
| 2 | LYC=LY Flag                          | 0=Different, 1=Equal |
|1-0| Mode Flag                            | Mode 0-3             |
    > 0 - HBlank, > 1 - VBlank, > 2 - Searching OAM, > 3 Transfer data to LCD
*/

#[derive(PartialEq)]
pub enum Mode {
    HBlank,
    VBlank,
    OAMRead,
    VRAMRead,
}

#[derive(Default)]
pub struct STAT (u8);

impl STAT { 
    pub fn new() -> Self { Default::default() } 

    pub fn lyc_interrupt(&self)     -> bool { self.0.bit(6) }
    pub fn oam_interrupt(&self)     -> bool { self.0.bit(5) }
    pub fn vblank_interrupt(&self)  -> bool { self.0.bit(4) }
    pub fn hblank_interrupt(&self)  -> bool { self.0.bit(3) }

    pub fn mode(&self) -> Mode { 
        match self.0 & 0b11 {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OAMRead,
            3 => Mode::VRAMRead,
            
            _ => Mode::HBlank,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.0 |= match mode {
            Mode::HBlank    => 0,
            Mode::VBlank    => 1,
            Mode::OAMRead   => 2,
            Mode::VRAMRead  => 3, 
        }
    }
}

impl MemoryBus for STAT {
    fn read_byte(&self, _: u16) -> u8 { self.0 }

    // mode flag and ly_compare=ly flag read only
    fn write_byte(&mut self, _: u16, b: u8) { self.0 = b & 0b01111000 }
}
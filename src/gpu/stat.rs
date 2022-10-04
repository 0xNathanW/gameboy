use std::default;

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

#[derive(PartialEq, Debug, Default)]
pub enum Mode {
    #[default]
    HBlank,
    VBlank,
    OAMRead,
    VRAMRead,
}

#[derive(Default)]
pub struct STAT {
    pub lyc_interrupt:      bool,
    pub oam_interrupt:      bool,
    pub vblank_interrupt:   bool,
    pub hblank_interrupt:   bool,
    pub mode:               Mode,
}

impl STAT { pub fn new() -> Self { Default::default() } }

impl MemoryBus for STAT {
    fn read_byte(&self, address: u16) -> u8 {
        assert_eq!(address, 0xFF41);
        let mut b: u8 = 0;
        if self.lyc_interrupt       { b.set(6) }
        if self.oam_interrupt       { b.set(5) }
        if self.vblank_interrupt    { b.set(4) }
        if self.hblank_interrupt    { b.set(3) }
        match self.mode {
            Mode::HBlank    => {},
            Mode::VBlank    => { b.set(0) },
            Mode::OAMRead   => { b.set(1) },
            Mode::VRAMRead  => { b.set(1); b.set(0) },
        }
        b
    }

    // mode flag and ly_compare=ly flag read only
    fn write_byte(&mut self, address: u16, b: u8) { 
        assert_eq!(address, 0xFF41);
        self.lyc_interrupt      = b.bit(6);
        self.oam_interrupt      = b.bit(5);
        self.vblank_interrupt   = b.bit(4);
        self.hblank_interrupt   = b.bit(3);
    }
}

#[cfg(test)]
mod test {
    use crate::bus::MemoryBus;

    use super::STAT;
    use super::Mode;

    #[test]
    fn stat_new() {
        let stat = STAT::new();
        assert!(!stat.lyc_interrupt);
        assert!(!stat.oam_interrupt);
        assert!(!stat.vblank_interrupt);
        assert!(!stat.hblank_interrupt);
        assert_eq!(stat.mode, Mode::HBlank);
    }

    #[test]
    fn stat_mode() {
        let mut stat = STAT::new();
        stat.mode = Mode::VBlank;
        assert_eq!(stat.mode, Mode::VBlank);
        stat.mode = Mode::HBlank;
        assert_eq!(stat.mode, Mode::HBlank);
    }

    #[test]
    fn read_write() {
        let mut stat = STAT::new();
        stat.write_byte(0xFF41, 0b1111_1111);
        assert_eq!(stat.mode, Mode::HBlank);
        assert!(stat.oam_interrupt);
        assert!(stat.vblank_interrupt);
        assert!(stat.lyc_interrupt);
        stat.mode = Mode::OAMRead;
        assert_eq!(stat.read_byte(0xFF41), 0b0111_1010);
    }
}
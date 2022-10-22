use crate::bus::MemoryBus;
use crate::bit::Bit;

// LCDC (LCD control) (R/W) - FF40
    // Main control register, its bits toggle what elements are displayed and how.
/*  
|Bit| Name                          | Usage notes              |
| 7 | LCD and PPU enable            | 0=Off, 1=On              |
| 6 | Window tile map area          | 0=9800-9BFF, 1=9C00-9FFF |
| 5 | Window enable                 | 0=Off, 1=On              |
| 4 | BG and Window tile data area  | 0=8800-97FF, 1=8000-8FFF |
| 3 | BG tile map area              | 0=9800-9BFF, 1=9C00-9FFF |
| 2 | OBJ size                      | 0=8x8, 1=8x16            |
| 1 | OBJ enable                    | 0=Off, 1=On              |
| 0 | BG and Window enable/priority | 0=Off, 1=On              |
*/
#[derive(Default)]
pub struct LCDC {
    pub lcd_enable:         bool,

    pub window_tilemap:     u16,
    pub window_enable:      bool,

    pub bg_window_tilemap:  u16,
    pub bg_window_enable:   bool,

    pub bg_tilemap:         u16,

    pub sprite_size:        u8,
    pub sprite_enable:      bool,
}


impl LCDC { 
    pub fn new() -> Self {
        LCDC {
            lcd_enable:         false,
            window_tilemap:     0x9C00,
            window_enable:      false,
            bg_window_tilemap:  0x8800,
            bg_tilemap:         0x9C00,
            sprite_size:        8,
            sprite_enable:      false,
            bg_window_enable:   false,
        }
    }
}

impl MemoryBus for LCDC {
    fn read_byte(&self, address: u16) -> u8 { 
        assert_eq!(address, 0xFF40);
        let mut lcdc: u8 = 0; 
        if self.lcd_enable                  { lcdc.set(7) }
        if self.window_tilemap == 0x9C00    { lcdc.set(6) }
        if self.window_enable               { lcdc.set(5) }
        if self.bg_window_tilemap == 0x8000 { lcdc.set(4) }
        if self.bg_tilemap == 0x9C00        { lcdc.set(3) }
        if self.sprite_size == 16           { lcdc.set(2) }
        if self.sprite_enable               { lcdc.set(1) }
        if self.bg_window_enable            { lcdc.set(0) }
        lcdc
    }

    fn write_byte(&mut self, address: u16, b: u8) { 
        assert_eq!(address, 0xFF40); 
        self.lcd_enable         = b.bit(7);
        self.window_tilemap     = if b.bit(6) { 0x9C00 } else { 0x9800 };
        self.window_enable      = b.bit(5);
        self.bg_window_tilemap  = if b.bit(4) { 0x8000 } else { 0x8800 };
        self.bg_tilemap         = if b.bit(3) { 0x9C00 } else { 0x9800 };
        self.sprite_size        = if b.bit(2) { 16 } else { 8 };
        self.sprite_enable      = b.bit(1);
        self.bg_window_enable   = b.bit(0);
    }
}

#[cfg(test)]
mod test {
    use crate::bus::MemoryBus;
    use super::LCDC;

    #[test]
    fn lcdc_new() {
        let lcdc = LCDC::new();
        assert!(!lcdc.lcd_enable);
        assert_eq!(lcdc.window_tilemap, 0x9800);
        assert!(!lcdc.window_enable);
        assert_eq!(lcdc.bg_window_tilemap, 0x8800);
        assert_eq!(lcdc.bg_tilemap, 0x9800);
        assert_eq!(lcdc.sprite_size, 8);
        assert!(!lcdc.sprite_enable);
        assert!(!lcdc.bg_window_enable);
    }

    #[test]
    fn read_write() {
        let mut lcdc = LCDC::new();
        assert_eq!(lcdc.read_byte(0xFF40), 0);
        lcdc.write_byte(0xFF40, 0b1111_1111);
        assert_eq!(lcdc.read_byte(0xFF40), 0b1111_1111);
        assert!(lcdc.lcd_enable);
        assert_eq!(lcdc.window_tilemap, 0x9C00);
        assert!(lcdc.window_enable);
        assert_eq!(lcdc.bg_window_tilemap, 0x8000);
        assert_eq!(lcdc.sprite_size, 16);
        assert!(lcdc.sprite_enable);
        assert!(lcdc.bg_window_enable);
    }
}
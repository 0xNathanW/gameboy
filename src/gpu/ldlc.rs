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
pub struct LCDC (u8);

impl LCDC { 
    pub fn new() -> Self { Default::default() }

    pub fn lcd_enable(&self)        -> bool { self.0.bit(7) }

    pub fn window_tilemap(&self)    -> u16  { if self.0.bit(6) { 0x9C00 } else { 0x9800 } }

    pub fn window_enable(&self)     -> bool { self.0.bit(5) }

    pub fn bg_window_tilemap(&self) -> u16  { if self.0.bit(4) { 0x8000 } else { 0x8800 } }

    pub fn bg_tilemap(&self)        -> u16  { if self.0.bit(3) { 0x9C00 } else { 0x9800 } }

    pub fn sprite_size(&self)       -> u8   { if self.0.bit(2) { 16 } else { 8 } }

    pub fn sprite_enable(&self)     -> bool { self.0.bit(1) }

    pub fn bg_window_enable(&self)  -> bool { self.0.bit(0) }
}

impl MemoryBus for LCDC {
    fn read_byte(&self, _: u16) -> u8 { self.0 }

    fn write_byte(&mut self, _: u16, b: u8) { self.0 = b }
}
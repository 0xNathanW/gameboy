use std::cell::RefCell;
use std::rc::Rc;

use super::bit::Bit;
use super::bus::MemoryBus;
use super::intf::Intf;

const VRAM_SIZE: usize = 16_384;
const OAM_SIZE: usize = 160;

pub struct GPU {
    // Tile data is stored in VRAM in the memory area at $8000-$97FF.
    vram: [u8; VRAM_SIZE],
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F.
    oam: [u8; OAM_SIZE],
    
    lcdc: LCDC,
    stat: STAT,

    // 0xFF42 - SCY (scroll Y) | 0xFF43 - SCX (scroll X)
    scroll_y: u8,
    scroll_x: u8,
    // 0xFF44 - LY (LCD Y coord ie. current scanline) | 0xFF45 - LYC (LY compare) 
    ly:  u8,
    ly_compare: u8,
    // 0xFF4A - WY (window y position) | 0xFF4B - WX (window x position + 7)
    window_y: u8,
    window_x: u8,

    // LCD monochrome palettes.
    // If want to implement Colour Gameboy there are extra palettes.
    // 0xFF47 - BGP (BG palette data)
    bgp: u8,
    // 0xFF48 - OBP0 (OBJ palette 0 data)
    obp0: u8,
    // 0xFF49 - OBP1 (OBJ palette 1 data)
    obp1: u8,

    /*
    The LCD controller operates on a 2^22 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines = 
    70224 dots = 16.74 ms. On scanlines 0 through 143, the PPU cycles through modes 2, 3, and 0 once 
    every 456 dots. Scanlines 144 through 153 are mode 1.
    */
    dots: u32,

    // Request for interrupt.
    intf: Rc<RefCell<Intf>>,
}

impl GPU {
    
    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self { 
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcdc: LCDC::new(),
            stat: STAT::new(),
            scroll_y:  0,  scroll_x:  0,
            ly:   0,  ly_compare:  0,
            window_y: 0,  window_x: 0,
            bgp:  0,  obp0: 0, obp1: 0,
            dots: 0,
            intf, 
        }
    }

    fn update(&mut self, cycles: u32) {
        if !self.lcdc.enable || cycles == 0 { return }
        
    }

    fn draw_bg(&mut self) {}

    fn draw_sprites(&mut self) {}
}

impl MemoryBus for GPU {

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000 ..= 0x97FF => self.vram[addr as usize - 0x8000],
            0xFE00 ..= 0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF40 => self.lcdc.read_byte(addr),
            0xFF41 => {
                self.stat.read_byte(addr) | 
                (if self.ly == self.ly_compare { 0b00000100 } else { 0 })
            },
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.ly_compare,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("unsupported gpu address (read): {:?}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, b: u8) {
        match addr {
            0x8000 ..= 0x97FF => self.vram[addr as usize - 0x8000] = b,
            0xFE00 ..= 0xFE9F => self.oam[addr as usize - 0xFE00] = b,
            0xFF40 => self.lcdc.write_byte(addr, b),
            0xFF41 => self.stat.write_byte(addr, b),
            0xFF42 => self.scroll_y  = b,
            0xFF43 => self.scroll_x  = b,
            0xFF44 => self.ly   = b,
            0xFF45 => self.ly_compare  = b,
            0xFF47 => self.bgp  = b,
            0xFF48 => self.obp0 = b,
            0xFF49 => self.obp1 = b,
            0xFF4A => self.window_y   = b,
            0xFF4B => self.window_x   = b,
            _ => panic!("unsupported gpu address (read): {:?}", addr),
        };
    }

}

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
struct LCDC {
    enable: bool,

    bg_tilemap:         bool,

    win_enable:         bool,
    win_tilemap:        bool,
    
    bg_win_enable:      bool, 
    bg_win_tilemap:     bool,
    
    obj_size:           bool,
    obj_enable:         bool, 
}

impl LCDC { fn new() -> Self { Default::default() } }

impl MemoryBus for LCDC {
    fn read_byte(&self, _: u16) -> u8 {
        let mut b = 0;
        if self.enable          { b.set(7) };
        if self.win_tilemap     { b.set(6) };
        if self.win_enable      { b.set(5) };
        if self.bg_win_tilemap  { b.set(4) };
        if self.bg_tilemap      { b.set(3) };
        if self.obj_size        { b.set(2) };
        if self.obj_enable      { b.set(1) };
        if self.bg_win_enable   { b.set(0) };
        b
    }

    fn write_byte(&mut self, _: u16, b: u8) {
        self.enable         = b.bit(7);
        self.win_tilemap    = b.bit(6);
        self.win_enable     = b.bit(5);
        self.bg_win_tilemap = b.bit(4);
        self.bg_tilemap     = b.bit(3);
        self.obj_size       = b.bit(2);
        self.obj_enable     = b.bit(1);
        self.bg_win_enable  = b.bit(0);
    }
}

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

#[derive(Default)]
struct STAT {
    lyc_int:    bool,
    mode2:      bool, 
    mode1:      bool,
    mode0:      bool,
    mode:       u8,
}

impl STAT { fn new() -> Self { Default::default() } }

impl MemoryBus for STAT {
    fn read_byte(&self, _: u16) -> u8 {
        let mut b: u8 = 0;
        if self.lyc_int { b.set(6) };
        if self.mode2   { b.set(5) };
        if self.mode1   { b.set(4) };
        if self.mode0   { b.set(3) };
        b |= self.mode;
        b
    }

    fn write_byte(&mut self, _: u16, b: u8) {
        self.lyc_int = b.bit(6);
        self.mode2   = b.bit(5);
        self.mode1   = b.bit(4);
        self.mode0   = b.bit(3);
        // mode flag and ly_compare=ly flag read only
    }
}

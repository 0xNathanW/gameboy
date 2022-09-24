use std::cell::RefCell;
use std::rc::Rc;

use self::stat::Mode;
use super::bit::Bit;
use super::bus::MemoryBus;
use super::intf::{Intf, InterruptSource};
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

use ldlc::LCDC;
use stat::STAT;

const VRAM_SIZE: usize = 16_384;
const OAM_SIZE: usize = 160;

mod ldlc;
mod stat;

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
    ly:         u8,
    ly_compare: u8,
    // 0xFF4A - WY (window y position) | 0xFF4B - WX (window x position + 7)
    window_y: u8,
    window_x: u8,

    // LCD monochrome palettes.
    // If want to implement Colour Gameboy there are extra palettes.
    // 0xFF47 - BGP (BG palette data)
    bg_palette: u8,
    bg_palette_colours: [u8; 4],
    // 0xFF48 - OBP0 (OBJ palette 0 data)
    palette_0: u8,
    palette_0_colours: [u8; 4],
    // 0xFF49 - OBP1 (OBJ palette 1 data)
    palette_1: u8,
    palette_1_colours: [u8; 4],

    bg_priority: [bool; SCREEN_WIDTH],

    /* The LCD controller operates on a 2^22 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines = 
    70224 dots = 16.74 ms. On scanlines 0 through 143, the PPU cycles through modes 2, 3, and 0 once 
    every 456 dots. Scanlines 144 through 153 are mode 1. */
    dots: u32,

    // Request for interrupt.
    intf: Rc<RefCell<Intf>>,

    // Raw pixel data.
    pub pixels: [[[u8; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT],
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
            bg_palette:  0, bg_palette_colours: [0; 4], 
            palette_0: 0, palette_0_colours: [0; 4],
            palette_1: 0, palette_1_colours: [0; 4],
            bg_priority: [false; SCREEN_WIDTH],
            dots: 0,
            intf,
            pixels: [[[255; 3]; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    // Calculate pixel colours for number of cpu cycles.
    pub fn update(&mut self, mut cycles: u32) {
        if !self.lcdc.lcd_enable() || cycles == 0 { return }

        while cycles > 0 {
            // Limit gpu cycles to 80.
            let gpu_cycles = if cycles >= 80 { 80 } else { cycles };
            self.dots += gpu_cycles;
            cycles -= gpu_cycles;

            // Line takes 456 dots.
            if self.dots >= 456 {
                self.dots -= 456;
                // Frame is 154 scanlines.
                self.ly = (self.ly + 1) % 154;
                
                // Check interrupt on current line.
                if self.stat.lyc_interrupt() && self.ly == self.ly_compare {
                    self.intf.borrow_mut().set_interrupt(InterruptSource::STAT);
                }

                // If last line, switch to vertical blank.
                if self.ly >= SCREEN_HEIGHT as u8 && self.stat.mode() != Mode::VBlank {
                    self.switch_mode(Mode::VBlank);
                }
            }

            if self.ly < SCREEN_HEIGHT as u8 {

                if self.dots <= 80 {
                    if self.stat.mode() != Mode::OAMRead {
                        self.switch_mode(Mode::OAMRead);
                    }
                } else if self.dots <= (80 + 172) {
                    if self.stat.mode() != Mode::VRAMRead {
                        self.switch_mode(Mode::VRAMRead);
                    }
                } else {
                    if self.stat.mode() != Mode::HBlank {
                        self.switch_mode(Mode::HBlank);
                    }
                }
            }
        }
    }

    // Change GPU mode, performing interrupts if neccessary.
    fn switch_mode(&mut self, mode: Mode) {
        self.stat.set_mode(mode);
        let interrupt = match self.stat.mode() {
            // Horizontal blank -> render line.
            Mode::HBlank => {
                self.render_line();
                self.stat.hblank_interrupt()
            },
            // Vertical blank -> interrupt
            Mode::VBlank => {
                self.intf.borrow_mut().set_interrupt(InterruptSource::VBlank);
                self.stat.vblank_interrupt()
            },
            // OAM read -> interrupt
            Mode::OAMRead => { self.stat.oam_interrupt() },
            // VRAM read -> nothing
            Mode::VRAMRead => { false },
        };

        if interrupt { self.intf.borrow_mut().set_interrupt(InterruptSource::STAT) }
    }

    // Draws both background and window.
    fn draw_bg(&mut self) {

        let win_y = 
            if self.lcdc.bg_window_enable() && self.lcdc.window_enable() {
                self.ly as i32 - self.window_y as i32
            } else {
                -1
            };
        if !self.lcdc.bg_window_enable() && win_y < 0 { return }
        
        let win_tile_y = (win_y as u16 >> 3) & 31;
        let bg_y = self.scroll_y.wrapping_add(self.ly);
        let bg_tile_y = (bg_y as u16 >> 3) & 31;
        
        for x in 0..SCREEN_WIDTH {

            let win_x = - (self.window_x as i32 - 7 + x as i32);
            let bg_x = self.scroll_x as u32 + x as u32;

            let (tile_map, tile_y, tile_x, pix_y, pix_x) = 
                if win_y >= 0 && win_x >= 0 {
                    (
                        self.lcdc.window_tilemap(), 
                        win_tile_y,
                        win_x as u16 >> 3,
                        win_x as u16 & 0b111,
                        win_y as u16 & 0b111,
                    )
                } else if self.lcdc.bg_window_enable() {
                    (
                        self.lcdc.bg_tilemap(),
                        bg_tile_y,
                        (bg_x as u16 >> 3) & 31,
                        bg_y as u16 & 0b111,
                        bg_x as u16 & 0b111,
                    )
                } else {
                    continue;
                };
            
            let tile_num: u8 = self.read_byte(tile_map + tile_y * 32 + tile_x);
            let tile_offset = if self.lcdc.bg_window_tilemap() == 0x8000 {
                tile_num as i16
            } else {
                tile_num as i8 as i16 + 128
            } as u16;
            let tile_address = tile_map + tile_offset * 16;

            let data_address = tile_address + (pix_y * 2);
            let data = [self.read_byte(data_address), self.read_byte(data_address + 1)];

            let xbit = 7 - pix_x;
            let colour_num = 
                if data[0] & (1 << xbit) != 0 { 1 } else { 0 } |
                if data[1] & (1 << xbit) != 0 { 2 } else { 0 };
            
            self.bg_priority[x] = if colour_num == 0 { true } else { false };

            self.set_pix(x, self.bg_palette_colours[colour_num]);
        }
    }

    // The Game Boy PPU can display up to 40 sprites either in 8x8 or in 8x16 pixels.
    fn draw_sprites(&mut self) {

        for idx in (0..40).rev() {
            
            // Each sprite consists of 4 bytes.
            let sprite_address = 0xFE00 + (idx as u16) * 4;

            let y = self.ly as i32;
            let size = self.lcdc.sprite_size() as i32;

            // Byte 0 - Y position.
            let sprite_y = self.read_byte(sprite_address) as u16 as i32 - 16;
            if sprite_y > y || y >= sprite_y + size { continue }
            
            // Byte 1 - X position.
            let sprite_x = self.read_byte(sprite_address + 1) as u16 as i32 - 8;
            if sprite_x < -7 || sprite_x >= SCREEN_WIDTH as i32 { continue }

            // Byte 2 - Tile idx.
            let tile_idx = (
                self.read_byte(sprite_address + 2) &
                (if self.lcdc.sprite_size() == 16 { 0xFE } else { 0xFF })
            ) as u16;

            /* Byte 3 - flags
            Bit7   BG and Window over OBJ (0=No, 1=BG and Window colors 1-3 over the OBJ)
            Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
            Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
            Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
            Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
            Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7) */
            let flags = self.read_byte(sprite_address + 3);
            let under_bg    = flags.bit(7);
            let y_flip      = flags.bit(6);
            let x_flip      = flags.bit(5);
            let palette_num = flags.bit(4); // true = 1.

            let tile_y: u16 = 
                if y_flip { (size - 1 - y + sprite_y) as u16 }
                else { (y - sprite_y) as u16 };
            
            let tile_address = 0x8000 + tile_idx * 16 + tile_y * 2;

            let data = [self.read_byte(tile_address), self.read_byte(tile_address + 1)];

            for x in 0..8 {

                if sprite_x + x < 0 || sprite_x + x >= (SCREEN_WIDTH as i32) { continue }

                let xbit = 1 << ( if x_flip { x } else { 7 - x } as u32 );

                let colour_num = 
                    if data[0] & xbit != 0 { 1 } else { 0 } |
                    if data[1] & xbit != 0 { 2 } else { 0 };
                
                if colour_num == 0 { continue }

                if under_bg && !self.bg_priority[(sprite_x + x) as usize] { continue }

                let colour = if palette_num { 
                    self.palette_1_colours[colour_num]
                } else {
                    self.palette_0_colours[colour_num]
                };

                self.set_pix((sprite_x + x) as usize , colour);    
            }
        }
    }

    fn render_line(&mut self) {

        for x in 0..SCREEN_WIDTH {
            self.set_pix(x, 255);
            self.bg_priority[x] = false;
        }

        self.draw_bg();
        if self.lcdc.sprite_enable() { self.draw_sprites() };
    }

    fn set_pix(&mut self, x: usize, colour: u8) {
        self.pixels[self.ly as usize][x] = [colour; 3];
    }

    fn set_colours(&mut self) {
        for idx in 0..4 {
            self.bg_palette_colours[idx]    = Self::get_colour_value(self.bg_palette, idx);
            self.palette_0_colours[idx]     = Self::get_colour_value(self.palette_0, idx);
            self.palette_1_colours[idx]     = Self::get_colour_value(self.palette_1, idx);
        }
    }

    fn get_colour_value(value: u8, index: usize) -> u8 {
        match (value >> 2 * index) & 0x03 {
            0 => 255, 
            1 => 192, 
            2 => 96,
            _ => 0
        }
    }

    fn reset(&mut self) {
        self.dots = 0;
        self.ly = 0;
        self.stat.set_mode(Mode::HBlank);

        for row in self.pixels.iter_mut() {
            for pix in row.iter_mut() {
                *pix = [255, 255, 255];
            }
        }
    }
}

impl MemoryBus for GPU {

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000 ..= 0x9FFF => self.vram[addr as usize - 0x8000],
            0xFE00 ..= 0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF40 => self.lcdc.read_byte(addr),
            0xFF41 => {
                self.stat.read_byte(addr) | 
                (if self.ly == self.ly_compare { 0b100 } else { 0 })
            },
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.ly_compare,
            0xFF47 => self.bg_palette,
            0xFF48 => self.palette_0,
            0xFF49 => self.palette_1,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("unsupported gpu address (read): {:#2X}", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, b: u8) {
        match addr {
            0x8000 ..= 0x9FFF => self.vram[addr as usize - 0x8000] = b,
            0xFE00 ..= 0xFE9F => self.oam[addr as usize - 0xFE00] = b,
            0xFF40 => {
                let prev = self.lcdc.lcd_enable();
                self.lcdc.write_byte(addr, b);
                if prev && !self.lcdc.lcd_enable() { self.reset() }
            },
            0xFF41 => self.stat.write_byte(addr, b),
            0xFF42 => self.scroll_y     = b,
            0xFF43 => self.scroll_x     = b,
            0xFF44 => self.ly           = b,
            0xFF45 => self.ly_compare   = b,
            0xFF47 => { self.bg_palette = b; self.set_colours() },
            0xFF48 => { self.palette_0  = b; self.set_colours() },
            0xFF49 => { self.palette_1  = b; self.set_colours() },
            0xFF4A => self.window_y     = b,
            0xFF4B => self.window_x     = b,
            _ => panic!("unsupported gpu address (write): {:#2X}", addr),
        };
    }
}




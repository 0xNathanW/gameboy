mod ldlc;
mod stat;
mod palette;

use std::cell::RefCell;
use std::rc::Rc;

use self::stat::Mode;
use super::bit::Bit;
use super::bus::MemoryBus;
use super::intf::{Intf, InterruptSource};
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

use ldlc::LCDC;
use stat::STAT;
use palette::Palette;

const VRAM_SIZE: usize = 16_384;
const OAM_SIZE: usize = 160;

#[derive(PartialEq, Copy, Clone)]
enum Priority {
    Colour0,
    None,
}

struct Sprite {
    // Byte 0 - y position.
    y: i16,
    // Byte 1 - x position.
    x: i16,
    // Byte 2 - Specifies the sprites Tile Number (00-FF).
    tile_num: u8,
    // Byte 3 - Attributes.
    // Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    // (Used for both BG and Window. BG color 0 is always behind OBJ)
    below_bg:       bool,
    // Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    y_flip:         bool,   
    // Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    x_flip:         bool,
    // Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    is_palette_1:   bool,
    // Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    // Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)   */
}

pub struct GPU {
    // Tile data is stored in VRAM in the memory area at $8000-$97FF.
    vram: [u8; VRAM_SIZE],
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F.
    oam: [u8; OAM_SIZE],
    
    // Raw pixel data, each pixel one of 3 grey shades.
    #[cfg(not(target_arch = "wasm32"))]
    pub pixels: [u32; SCREEN_HEIGHT * SCREEN_WIDTH],
    // Web requires 4 bytes per pixel in rgba format.
    #[cfg(target_arch = "wasm32")]
    pub pixels: [u8; SCREEN_HEIGHT * SCREEN_WIDTH * 4],

    updated: bool,

    lcdc: LCDC,
    stat: STAT,
    h_blank: bool,

    // 0xFF42 - SCY (scroll Y) | 0xFF43 - SCX (scroll X)
    scroll_y: u8,
    scroll_x: u8,
    // 0xFF44 - LY (LCD Y coord ie. current scanline) | 0xFF45 - LYC (LY compare) 
    ly:         u8,
    ly_compare: u8,
    // 0xFF4A - WY (window y position) | 0xFF4B - WX (window x position + 7)
    window_y: u8,
    window_x: u8,

    // LCD monochrome palettes, CGB has extra palettes.
    // 0xFF47 - BGP (BG palette data)
    // 0xFF48 - OBP0 (OBJ palette 0 data) | 0xFF49 - OBP1 (OBJ palette 1 data)
    bg_palette:         Palette,
    sprite_palette_0:   Palette,
    sprite_palette_1:   Palette,

    bg_priority: [Priority; SCREEN_WIDTH],

    /* The LCD controller operates on a 2^22 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines = 
    70224 dots = 16.74 ms. On scanlines 0 through 143, the PPU cycles through modes 2, 3, and 0 once 
    every 456 dots. Scanlines 144 through 153 are mode 1. */
    dots: u32,

    // Request for interrupt.
    intf: Rc<RefCell<Intf>>,
}

impl GPU {
    
    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self { 
            vram: [0; VRAM_SIZE],
            oam:  [0; OAM_SIZE],

            lcdc: LCDC::new(),
            stat: STAT::new(),
            h_blank: false,

            scroll_y:   0,
            scroll_x:   0,
            ly:         0,
            ly_compare: 0,
            window_y:   0,
            window_x:   0,
            
            bg_palette:         Palette::new(),
            sprite_palette_0:   Palette::new(),
            sprite_palette_1:   Palette::new(),

            bg_priority: [Priority::None; SCREEN_WIDTH],
            dots: 0,
            intf,

            #[cfg(not(target_arch = "wasm32"))]
            pixels: [u32::MAX; SCREEN_WIDTH * SCREEN_HEIGHT],
            #[cfg(target_arch = "wasm32")]
            pixels: [u8::MAX; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            updated: false,
        }
    }

    pub fn update(&mut self, mut cycles: u32) {
        
        if !self.lcdc.lcd_enable { return }
        self.h_blank = false;

        while cycles > 0 {
            
            let current_cycles = if cycles >= 80 { 80 } else { cycles };
            self.dots += current_cycles;
            cycles -= current_cycles;
            
            // Full line.
            if self.dots >= 456 {
                self.dots -= 456;
                self.ly = (self.ly + 1) % 154;

                if self.stat.lyc_interrupt && (self.ly == self.ly_compare) {
                    self.intf.borrow_mut().set_interrupt(InterruptSource::STAT);
                }

                /* Mode 1: This mode is called V-Blank and happens when the last visible row has been processed, 
                which is row 143. There are 10 additional rows, which in total take 4,560 clock cycles to process. 
                After that, we go back to the first row (LY = 0). */
                if self.ly >= SCREEN_HEIGHT as u8 && self.stat.mode != Mode::VBlank {
                    self.switch_mode(Mode::VBlank);
                }
            }

            // Normal line.
            if self.ly < SCREEN_HEIGHT as u8 {
                // Fetch assets from memory.
                if self.dots <= 80 {
                    if self.stat.mode != Mode::OAMRead { self.switch_mode(Mode::OAMRead) }
                
                // Tiles and sprites are rendered.
                } else if self.dots <= (80 + 172) {
                    if self.stat.mode != Mode::VRAMRead { self.switch_mode(Mode::VRAMRead) }
                
                } else if self.stat.mode != Mode::HBlank{
                    self.switch_mode(Mode::HBlank)
                }
            }
        } 
    }

    fn switch_mode(&mut self, mode: Mode) {
        self.stat.mode = mode;

        let interrupt = match self.stat.mode {
            Mode::HBlank => {
                self.render_scanline();
                self.h_blank = true;
                self.stat.hblank_interrupt
            },
            Mode::VBlank => {
                self.intf.borrow_mut().set_interrupt(InterruptSource::VBlank);
                self.updated = true;
                self.stat.vblank_interrupt
            },
            Mode::OAMRead => { self.stat.oam_interrupt },
            Mode::VRAMRead => false,
        };

        if interrupt { self.intf.borrow_mut().set_interrupt(InterruptSource::STAT) }
    }

    fn render_scanline(&mut self) {
        if self.lcdc.bg_window_enable { self.render_bg();      }
        if self.lcdc.sprite_enable    { self.render_sprites(); }
    }

    fn render_bg(&mut self) {
        
        let is_window_y = self.lcdc.window_enable && self.ly >= self.window_y;
        let bg_y = self.ly.wrapping_add(self.scroll_y);

        for x in 0..SCREEN_WIDTH as u8 {

            let is_window_x = self.lcdc.window_enable && x >= self.window_x.wrapping_sub(7);
            let is_window = is_window_x && is_window_y;
            let bg_x = x.wrapping_add(self.scroll_x);

            let tile_address = if is_window {
                let offset_y = self.ly.wrapping_sub(self.window_y);
                let offset_x = x.wrapping_sub(self.window_x.wrapping_sub(7));
                GPU::get_address(self.lcdc.window_tilemap, offset_x, offset_y)
            } else {
                GPU::get_address(self.lcdc.bg_tilemap, bg_x, bg_y)
            };
            let tile = self.read_byte(tile_address);

            let tile_base_address = self.get_tile_address(tile);
            let tile_offset = if is_window {
                (self.ly - self.window_y) % 8 * 2
            } else {
                bg_y % 8 * 2
            } as u16;

            let tile_data_address = tile_base_address + tile_offset;
            let tile_data = [
                self.read_byte(tile_data_address),
                self.read_byte(tile_data_address + 1),
            ];

            let x_bit = if is_window {
                self.window_x.wrapping_sub(x) % 8
            } else {
                7 - (bg_x % 8)
            };

            let colour_num = 
                usize::from(tile_data[0] & (1 << x_bit) > 0) |
                if tile_data[1] & (1 << x_bit) > 0 { 2 } else { 0 };
                
            self.bg_priority[x as usize] = if colour_num == 0 {
                Priority::Colour0
            } else {
                Priority::None
            };

            self.set_pixel(x as usize, self.bg_palette.get_shade(colour_num));
        }   
    }

    fn render_sprites(&mut self) {
        
        let line = self.ly as i16;
        let size = self.lcdc.sprite_size as i16;
        // We reverse as sprites with lower idx have pixel priority.
        for idx in (0..40).rev() {

            let sprite = self.fetch_sprite(idx);
            // Skip if out of bounds.
            if line < sprite.y || line >= sprite.y + size { continue; }

            let tile_base_address = 0x8000 + (sprite.tile_num as u16 * 16);
            let tile_offset = if sprite.y_flip {
                size - 1 - (line - sprite.y)
            } else {
                line - sprite.y
            };
            let tile_data_address = tile_base_address + (tile_offset * 2) as u16;
            
            let tile_data = [
                self.read_byte(tile_data_address),
                self.read_byte(tile_data_address + 1),
            ];

            // Iterate width setting each pixel.
            for x in 0..8 {
                let pix_x = sprite.x + x;
                // Skip out of bounds pixels.
                if !(0..160).contains(&pix_x) { continue; }

                let x_bit = if sprite.x_flip { x } else { 7 - x };
                
                let colour_idx = 
                    usize::from(tile_data[0] & (1 << x_bit) > 0) |
                    if tile_data[1] & (1 << x_bit) > 0 { 2 } else { 0 };
                    
                // Skip transparent pixels.
                if colour_idx == 0 { continue; }

                let colour = if sprite.is_palette_1 {
                    self.sprite_palette_1.get_shade(colour_idx)
                } else {
                    self.sprite_palette_0.get_shade(colour_idx)
                };

                // Skip if background has priority.
                if sprite.below_bg && self.bg_priority[(sprite.x + x) as usize] != Priority::Colour0 {
                    continue;
                }

                self.set_pixel(pix_x as usize, colour);
            }
        }
    }

    // Retrieves sprite from memory, returns None if not rendered on current line.
    fn fetch_sprite(&self, idx: usize) -> Sprite {
        
        let address = 0xFE00 + (idx as u16 * 4);
        let y = self.read_byte(address) as i16 - 16;
        let x = self.read_byte(address + 1) as i16 - 8;
        let attr = self.read_byte(address + 3);
        
        Sprite {
            y,
            x,
            tile_num:       self.read_byte(address + 2) & 
                            if self.lcdc.sprite_size == 16 { 0xFE } else { 0xFF },
            below_bg:       attr.bit(7),
            y_flip:         attr.bit(6),
            x_flip:         attr.bit(5),
            is_palette_1:   attr.bit(4),
        }
    }

    fn get_address(base: u16, x: u8, y: u8) -> u16 {
        base + (y as u16 / 8 * 32) + (x as u16 / 8)
    }

    fn get_tile_address(&self, tile_num: u8) -> u16 {
        match self.lcdc.bg_window_tilemap {
            0x8000 => 0x8000 + tile_num as u16 * 16,
            _      => 0x9000_u16.wrapping_add(((tile_num as i8) as u16).wrapping_mul(16)), 
        }
    }

    fn clear_screen(&mut self) {
        for pix in self.pixels.iter_mut() {
            #[cfg(not(target_arch = "wasm32"))] {
                *pix = u32::MAX;
            }
            #[cfg(target_arch = "wasm32")] {
                *pix = u8::MAX;
            }
        }
        for prio in self.bg_priority.iter_mut() {
            *prio = Priority::None;
        }
        self.updated = true;
    }

    fn set_pixel(&mut self, x: usize, colour: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let p: u32 = 0xFF_00_00_00 | colour;
            self.pixels[(self.ly as usize) * SCREEN_WIDTH + x] = p;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let start = ((self.ly as usize) * (SCREEN_WIDTH) + x) *  4;
            let mut rgba = (colour << 8).to_be_bytes();
            rgba[3] = 0xFF;
            self.pixels[start..start+4].copy_from_slice(&rgba);
        }
    }

    pub fn set_colours(&mut self, colours: [u32; 4]) {
        let old_colours = self.bg_palette.colours();
        let old_ly = self.ly;
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            for y in 0..SCREEN_HEIGHT {
                self.ly = y as u8;
                for x in 0..SCREEN_WIDTH as usize {
                    let idx = (y * 166 + x) as usize;
                    match self.pixels[idx] {
                        c if c == old_colours[0] => self.set_pixel(x, colours[0]),
                        c if c == old_colours[1] => self.set_pixel(x, colours[1]),
                        c if c == old_colours[2] => self.set_pixel(x, colours[2]),
                        c if c == old_colours[3] => self.set_pixel(x, colours[3]),
                        _ => unreachable!(),
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            for y in 0..SCREEN_HEIGHT {
                self.ly = y as u8;
                for x in 0..SCREEN_WIDTH {
                    let rgba = self.get_pixel(x, y);
                    let c = u32::from_be_bytes([0, rgba[0], rgba[1], rgba[2]]);
                    match c {
                        c if c == old_colours[0] => self.set_pixel(x, colours[0]),
                        c if c == old_colours[1] => self.set_pixel(x, colours[1]),
                        c if c == old_colours[2] => self.set_pixel(x, colours[2]),
                        c if c == old_colours[3] => self.set_pixel(x, colours[3]),
                        _ => unreachable!(),
                    }
                }
            }
        }

        self.ly = old_ly;
        self.bg_palette.set_colours(colours);
        self.sprite_palette_0.set_colours(colours);
        self.sprite_palette_1.set_colours(colours);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let start = (y * SCREEN_WIDTH + x) * 4;
        let mut rgba = [0; 4];
        rgba.copy_from_slice(&self.pixels[start..start+4]);
        rgba
    }

    pub fn check_updated(&mut self) -> bool {
        let updated = self.updated;
        self.updated = false;
        updated
    }
}


impl MemoryBus for GPU {

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000 ..= 0x9FFF => self.vram[address as usize - 0x8000],
            0xFE00 ..= 0xFE9F => self.oam[address as usize - 0xFE00],
            0xFF40 => self.lcdc.read_byte(address),
            0xFF41 => {
                let mut stat = self.stat.read_byte(address);
                if self.ly == self.ly_compare { stat.set(2) }
                stat
            },
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.ly_compare,
            0xFF47 => self.bg_palette.read_byte(address),
            0xFF48 => self.sprite_palette_0.read_byte(address),
            0xFF49 => self.sprite_palette_1.read_byte(address),
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("invalid memory read for gpu at {:#2X}", address),
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x8000 ..= 0x9FFF => self.vram[address as usize - 0x8000] = b,
            0xFE00 ..= 0xFE9F => self.oam[address as usize - 0xFE00] = b,
            0xFF40 => {
                let prev = self.lcdc.lcd_enable;
                self.lcdc.write_byte(address, b);
                
                if prev && !self.lcdc.lcd_enable { 
                    self.dots = 0;
                    self.ly   = 0;
                    self.stat.mode = Mode::HBlank;
                    self.clear_screen();
                }

                if !prev && self.lcdc.lcd_enable {
                    self.switch_mode(Mode::OAMRead);
                    self.dots = 4;
                }
            },
            0xFF41 => self.stat.write_byte(address, b),
            0xFF42 => self.scroll_y     = b,
            0xFF43 => self.scroll_x     = b,
            0xFF44 => {},   // Read only.
            0xFF45 => self.ly_compare   = b,
            0xFF47 => self.bg_palette.write_byte(address, b),
            0xFF48 => self.sprite_palette_0.write_byte(address, b),
            0xFF49 => self.sprite_palette_1.write_byte(address, b),
            0xFF4A => self.window_y     = b,
            0xFF4B => self.window_x     = b,
            _ => panic!("invalid memory write for gpu at {:#2X}", address),
        };
    }
}




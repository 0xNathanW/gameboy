use crate::constants::CYCLES_PER_FRAME;
use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    Gameboy, GbKey, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub const DEMO_DATA: &[u8] = include_bytes!("../pocket.gb");

pub struct Emulator {
    gameboy: Gameboy,
    // RGBA buffer for web canvas rendering
    rgba_buffer: Vec<u8>,
}

impl Default for Emulator {
    fn default() -> Self {
        let demo = open_cartridge(DEMO_DATA.to_vec(), None, None).unwrap();
        Self {
            gameboy: Gameboy::new(demo, None),
            rgba_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
        }
    }
}

impl Emulator {
    pub fn new(rom_data: Box<dyn Cartridge>) -> Self {
        Self {
            gameboy: Gameboy::new(rom_data, None),
            rgba_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
        }
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < CYCLES_PER_FRAME {
            let cycles = self.gameboy.tick();
            self.gameboy.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn is_display_updated(&mut self) -> bool {
        self.gameboy.display_updated()
    }

    pub fn key_down(&mut self, key: GbKey) {
        self.gameboy.key_down(key);
    }

    pub fn key_up(&mut self, key: GbKey) {
        self.gameboy.key_up(key);
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.gameboy.set_palette(palette);
    }

    // Returns the display buffer as RGBA bytes for web canvas rendering.
    pub fn display_buffer(&mut self) -> &[u8] {
        let pixels = self.gameboy.display_buffer();
        for (i, &pixel) in pixels.iter().enumerate() {
            let offset = i * 4;
            self.rgba_buffer[offset] = ((pixel >> 16) & 0xFF) as u8; // R
            self.rgba_buffer[offset + 1] = ((pixel >> 8) & 0xFF) as u8; // G
            self.rgba_buffer[offset + 2] = (pixel & 0xFF) as u8; // B
            self.rgba_buffer[offset + 3] = 0xFF; // A
        }
        &self.rgba_buffer
    }

    // TODO: implement saving/loading save data and RTC state
    pub fn _save_data(&self) -> Option<&[u8]> {
        self.gameboy.save_data()
    }

    pub fn _rtc_zero(&self) -> Option<u64> {
        self.gameboy.rtc_zero()
    }
}

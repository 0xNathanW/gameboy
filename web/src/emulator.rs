use crate::constants::CYCLES_PER_FRAME;
use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    Gameboy, GbKey,
};

pub const DEMO_DATA: &[u8] = include_bytes!("../pocket.gb");

pub struct Emulator(Gameboy);

impl Default for Emulator {
    fn default() -> Self {
        let demo = open_cartridge(DEMO_DATA.to_vec(), None).unwrap();
        Self(Gameboy::new(demo, None))
    }
}

impl Emulator {
    pub fn new(rom_data: Box<dyn Cartridge>) -> Self {
        Self(Gameboy::new(rom_data, None))
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < CYCLES_PER_FRAME {
            let cycles = self.0.tick();
            self.0.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn is_display_updated(&mut self) -> bool {
        self.0.display_updated()
    }

    pub fn key_down(&mut self, key: GbKey) {
        self.0.key_down(key);
    }

    pub fn key_up(&mut self, key: GbKey) {
        self.0.key_up(key);
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.0.set_palette(palette);
    }

    pub fn display_buffer(&self) -> &[u8] {
        self.0.display_buffer()
    }
}

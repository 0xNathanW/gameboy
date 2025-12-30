use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    cpu::CPU,
    keypad::GbKey,
};

pub const DEMO_DATA: &'static [u8] = include_bytes!("../pocket.gb");

pub struct Emulator(pub CPU);

impl Default for Emulator {
    fn default() -> Self {
        let demo = open_cartridge(DEMO_DATA.to_vec(), None).unwrap();
        Self(CPU::new(demo, None))
    }
}

impl Emulator {
    pub fn new(rom_data: Box<dyn Cartridge>) -> Self {
        Self(CPU::new(rom_data, None))
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < 69_905 {
            let cycles = self.0.tick();
            self.0.mem.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn is_display_updated(&mut self) -> bool {
        self.0.mem.gpu.check_updated()
    }

    pub fn key_down(&mut self, key: GbKey) {
        self.0.mem.keypad.key_press(key);
    }

    pub fn key_up(&mut self, key: GbKey) {
        self.0.mem.keypad.key_release(key);
    }

    pub fn change_palette(&mut self, palette: [u32; 4]) {
        self.0.mem.gpu.set_colours(palette);
    }
}

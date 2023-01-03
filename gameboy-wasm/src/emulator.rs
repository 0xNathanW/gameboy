use gloo::net::http::Request;
use gameboy_core::cpu::CPU;
use gameboy_core::cartridge::open_cartridge;

pub const DEMO_DATA: &'static [u8] = include_bytes!("../pocket.gb");

pub struct Emulator(pub CPU);

impl Emulator {

    pub fn new() -> Self {
        let demo = open_cartridge(DEMO_DATA.to_vec(), None);
        Self(CPU::new(demo, None))
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

}

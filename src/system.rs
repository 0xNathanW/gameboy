use std::path::Path;
use std::time;
use std::thread;
use::minifb::{Window, Key};

use crate::{SCREEN_WIDTH, SCREEN_HEIGHT};
use super::cartridge;
use super::serial::SerialCallback;
use super::cpu::CPU;
use super::keypad::GbKey;

const FRAME_TIME: time::Duration = time::Duration::from_millis(16);

pub struct Gameboy {
    pub cpu: CPU,
    pub display: Window,
}

impl Gameboy {

    pub fn new(rom_path: &Path, display: Window, callback: SerialCallback) -> Self {
        let cartridge = cartridge::open_cartridge(rom_path);
        Self{
            cpu: CPU::new(cartridge, callback),
            display,
        }
    }

    pub fn run(&mut self) {

        while self.display.is_open() {
            let now = time::Instant::now();

            let mut frame_cycles = 0;
            while frame_cycles <= 69_905 {
                let cycles = self.cpu.step();
                self.cpu.mem.update(cycles);
                frame_cycles += cycles;
            }

            let keys = [
                (Key::Right,  GbKey::Right),
                (Key::Up,     GbKey::Up),
                (Key::Left,   GbKey::Left),
                (Key::Down,   GbKey::Down),
                (Key::J,      GbKey::A),
                (Key::K,      GbKey::B),
                (Key::Space,  GbKey::Select),
                (Key::Enter,  GbKey::Start),
            ];
            
            for (input, key) in keys.iter() {
                if self.display.is_key_down(*input) {
                    self.cpu.mem.keypad.key_press(key.clone());
                } else {
                    self.cpu.mem.keypad.key_release(key.clone());
                }
            }
            
            self.display.update_with_buffer(
                self.cpu.mem.gpu.pixels.as_ref(),
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            ).unwrap();
            
            match FRAME_TIME.checked_sub(now.elapsed()) {
                Some(time) => { thread::sleep(time); },
                None => {},
            }
        }
    }
}

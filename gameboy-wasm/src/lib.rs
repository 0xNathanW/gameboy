mod utils;

use gameboy_core::cartridge;
use wasm_bindgen::prelude::*;
use gameboy_core::cpu::CPU;
use gameboy_core::keypad::GbKey;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Emulator { 
    gb: CPU,
    scale: usize,
    pixels: Vec<u8>,
}

#[wasm_bindgen]
impl Emulator {

    pub fn new(rom_data: Vec<u8>, save_data: Option<Vec<u8>>) -> Self {
        let cartridge = gameboy_core::cartridge::open_cartridge(rom_data, save_data);
        Self { 
            gb: CPU::new(cartridge, None),
            scale: 4,
            pixels: vec![0; 160 * 144 * 8 * 8 * 4],
        }
    }

    pub fn key_press(&mut self, key: u8) {
        match key {
            0 => self.gb.mem.keypad.key_press(GbKey::Down),
            1 => self.gb.mem.keypad.key_press(GbKey::Up),
            2 => self.gb.mem.keypad.key_press(GbKey::Left),
            3 => self.gb.mem.keypad.key_press(GbKey::Right),
            4 => self.gb.mem.keypad.key_press(GbKey::A),
            5 => self.gb.mem.keypad.key_press(GbKey::B),
            6 => self.gb.mem.keypad.key_press(GbKey::Select),
            7 => self.gb.mem.keypad.key_press(GbKey::Start),
            _ => {},
        }
    }

    pub fn key_release(&mut self, key: u8) {
        match key {
            0 => self.gb.mem.keypad.key_release(GbKey::Down),
            1 => self.gb.mem.keypad.key_release(GbKey::Up),
            2 => self.gb.mem.keypad.key_release(GbKey::Left),
            3 => self.gb.mem.keypad.key_release(GbKey::Right),
            4 => self.gb.mem.keypad.key_release(GbKey::A),
            5 => self.gb.mem.keypad.key_release(GbKey::B),
            6 => self.gb.mem.keypad.key_release(GbKey::Select),
            7 => self.gb.mem.keypad.key_release(GbKey::Start),
            _ => {},
        }
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < 69905 {
            let cycles = self.gb.tick();
            self.gb.mem.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn display_updated(&mut self) -> bool {
        self.gb.mem.gpu.check_updated()
    }

    //  Converts numbers from 0RGB -> RGBA in byte form.
    //  Scale by 2.
    //  Rerturns pointer to vec.
    pub fn pixels_ptr(&mut self) -> *const u8 {
        
        let row_pix = 160 * self.scale * 4;
        for (i, raw) in self.gb.mem.gpu.pixels.iter().enumerate() {
            
            let row = i / 160;
            let col = i % 160;
            let mut rgba = (raw << 8).to_be_bytes();
            rgba[3] = 255;  // Opacity.

            for (j, c) in rgba.iter().enumerate() {
                for n in 0..self.scale {
                    for m in 0..self.scale {
                        self.pixels[
                            ((col * self.scale * 4) + (4 * n)) +    // x
                            (((row * 4) + m) * row_pix)             // y
                            + j                                     // offset
                        ] = *c;
                    }
                }
            }
        }
        self.pixels.as_ptr()
    }

    pub fn get_save_data(&self) -> *const u8 {
        self.gb.mem.save()
    }
}

#[wasm_bindgen]
pub fn ram_size(n: usize) -> usize {
    cartridge::ram_size(n as u8)
}


use std::path::Path;
use::minifb::Window;

use crate::{SCREEN_WIDTH as W, SCREEN_HEIGHT as H};
use super::serial::SerialCallback;
use super::cpu::CPU;

pub struct Gameboy {
    cpu: CPU,
    display: Window,
}

impl Gameboy {

    pub fn new(rom_path: &Path, display: Window, callback: SerialCallback) -> Self { 
        Self{
            cpu: CPU::new(rom_path, callback),
            display,
        } 
    }

    pub fn run(&mut self) {

        let mut buf = vec![0; W * H];
        self.display.update_with_buffer(&buf, W, H).unwrap();
 
        loop {

            let cycles = self.cpu.step();
            self.cpu.mem.update(cycles);

            let mut idx = 0;
            for row in self.cpu.mem.gpu.pixels.iter() {
                for pix in row.iter() {
                    
                    let (b, g, r) = (
                        (pix[0] as u32) << 16,
                        (pix[1] as u32) << 8,
                        (pix[2] as u32),
                    );
                    let a = 0xFF00_0000;
                    buf[idx] = a | b | g | r;
                    idx += 1;
                }
            }
            //self.display.update_with_buffer(&buf, W, H).unwrap();
        }
    }
}

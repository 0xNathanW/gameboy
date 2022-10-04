use minifb::{Window, WindowOptions, Scale, Key};
use std::{path::Path, ffi::OsStr, time, thread};

use gameboy::{SCREEN_HEIGHT, SCREEN_WIDTH};
use gameboy::cpu::CPU;
use gameboy::keypad::GbKey;
use gameboy::cartridge;

const FRAME_TIME: time::Duration = time::Duration::from_millis(16);

fn main() {

    let rom_name = std::env::args().nth(1).expect(
        "a path to a rom must be provided as an argument"
    );

    let rom_path = Path::new(&rom_name);
    if !rom_path.exists() { 
        panic!("path does not exist"); 
    }    
    if rom_path.extension() != Some(OsStr::new("gb")) {
        panic!("file provided does not have the extention '.gb'"); 
    }

    let cartridge = cartridge::open_cartridge(rom_path);

    let opts = WindowOptions {
        scale: Scale::X4,
        ..Default::default()
    };

    let mut display = Window::new(
        &cartridge.title(),
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        opts,
    ).unwrap_or_else(|e| { panic!("{}", e) });

    let mut cpu = CPU::new(cartridge, None);

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

    while display.is_open() {
        let now = time::Instant::now();

        let mut frame_cycles = 0;
        while frame_cycles <= 69_905 {
            let cycles = cpu.step();
            cpu.mem.update(cycles);
            frame_cycles += cycles;
        }

        for (input, key) in keys.iter() {
            if display.is_key_down(*input) {
                cpu.mem.keypad.key_press(key.clone());
            } else {
                cpu.mem.keypad.key_release(key.clone());
            }
        }
        
        display.update_with_buffer(
            cpu.mem.gpu.pixels.as_ref(), 
            SCREEN_WIDTH, 
            SCREEN_HEIGHT,
        ).unwrap();

        match FRAME_TIME.checked_sub(now.elapsed()) {
            Some(time) => { thread::sleep(time); },
            None => {},
        }
    }
}

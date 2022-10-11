use cpal::OutputCallbackInfo;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use minifb::{Window, WindowOptions, Scale, Key};
use std::{path::Path, ffi::OsStr};

use gameboy::{SCREEN_HEIGHT, SCREEN_WIDTH};
use gameboy::cpu::CPU;
use gameboy::keypad::GbKey;
use gameboy::cartridge;
use gameboy::apu::APU;

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

    let audio_player = initialise_audio(&mut cpu);

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

        let cycles = cpu.step();
        cpu.mem.update(cycles);

        if cpu.mem.gpu.check_updated() {
            display.update_with_buffer(
                cpu.mem.gpu.pixels.as_ref(), 
                SCREEN_WIDTH, 
                SCREEN_HEIGHT,
            ).unwrap();
        }
        
        for (input, key) in keys.iter() {
            if display.is_key_down(*input) {
                cpu.mem.keypad.key_press(key.clone());
            } else {
                cpu.mem.keypad.key_release(key.clone());
            }
        }

        if !cpu.flip() { continue }
    }
    drop(audio_player);
}

fn initialise_audio(cpu: &mut CPU) -> cpal::Stream {

    let device = cpal::default_host().default_output_device().expect("failed to find output device.");
    let config = device.default_output_config().unwrap();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    let apu = APU::power_up(config.sample_rate().0);
    let stream_buffer = apu.buffer.clone();
    cpu.mem.apu = Some(apu);

    let stream = device.build_output_stream(
        &config.config(), 
        move |out_buf: &mut [f32], _: &OutputCallbackInfo | {
            let mut in_buf = stream_buffer.lock().unwrap();
            let length = std::cmp::min(out_buf.len() / 2, in_buf.len());
            
            for (idx, (data_l, data_r)) in in_buf.drain(..length).enumerate() {
                out_buf[idx * 2] = data_l;
                out_buf[idx * 2 + 1] = data_r;
            }
        },
        err_fn,
    ).unwrap();
    stream.play().unwrap();
    stream
}
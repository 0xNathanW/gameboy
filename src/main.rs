use cpal::OutputCallbackInfo;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use minifb::{Window, WindowOptions, Scale, Key};
use clap::Parser;
use std::{path::Path, ffi::OsStr};

use gameboy::{SCREEN_HEIGHT, SCREEN_WIDTH};
use gameboy::cpu::CPU;
use gameboy::keypad::GbKey;
use gameboy::cartridge;
use gameboy::apu::APU;


#[derive(Parser)]
#[command(author = "Nathanw", about  = "A Rust powered Gameboy emulator.")]
struct Args {
    #[arg(short, long, help = "Path to rom")]
    path:   String,

    #[arg(short = 'x', long, help = "Display scale factor")]
    #[arg(value_enum, default_value_t)]
    scale:  DisplayScale,

    #[arg(short, long, help = "Enable audio")]
    #[arg(default_value = "false")]
    audio:  bool,

    #[arg(short, long, help = "Print serial write to stdout")]
    #[arg(default_value = "false")]
    serial: bool,
}

// Copy of minifb::Scale such that it implements clap::ValueEnum.
#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum DisplayScale {
    X1,
    X2,
    #[default]
    X4,
    X8,
    X16,
    X32,
}

fn main() {

    let args = Args::parse();
    let rom_name = args.path;

    let rom_path = Path::new(&rom_name);
    if !rom_path.exists() { 
        panic!("path provided does not exist"); 
    }
    if rom_path.extension() != Some(OsStr::new("gb")) {
        panic!("file provided does not have the extention '.gb'"); 
    }

    let cartridge = cartridge::open_cartridge(rom_path);

    let opts = WindowOptions {
        scale: match args.scale {
            DisplayScale::X1  => Scale::X1,
            DisplayScale::X2  => Scale::X2,
            DisplayScale::X4  => Scale::X4,
            DisplayScale::X8  => Scale::X8,
            DisplayScale::X16 => Scale::X16,
            DisplayScale::X32 => Scale::X32            
        },
        ..Default::default()
    };

    let mut display = Window::new(
        &cartridge.title().to_lowercase(),
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        opts,
    ).unwrap_or_else(|e| { panic!("error setting up display: {}", e) });
    
    let callback: Option<Box<dyn Fn(u8)>> = if args.serial {
        Some(Box::new(|b: u8| { print!("{}", b as char); }))
    } else {
        None
    };

    let mut cpu = CPU::new(cartridge, callback);

    let audio_stream = if args.audio {
        initialise_audio(&mut cpu)
    } else { 
        None
    };

    let keys = [
        (Key::Right,  GbKey::Right),
        (Key::Up,     GbKey::Up),
        (Key::Left,   GbKey::Left),
        (Key::Down,   GbKey::Down),
        (Key::Z,      GbKey::A),
        (Key::X,      GbKey::B),
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

        if !cpu.flip() { continue; }
    }

    // Drop the audio stream if it exists.
    if audio_stream.is_some() { drop(audio_stream.unwrap()) }
}

fn initialise_audio(cpu: &mut CPU) -> Option<cpal::Stream> {

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
    Some(stream)
}

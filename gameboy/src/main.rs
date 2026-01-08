// A simple frontend for the Gameboy core using minifb for display and cpal for audio.

use anyhow::{ensure, Context, Result};
use clap::Parser;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo,
};
use minifb::{Key, Scale, Window, WindowOptions};
use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use gameboy_core::{cartridge, Gameboy, GbKey, SCREEN_HEIGHT, SCREEN_WIDTH};

const STEP_TIME_MS: u64 = 16;
const STEP_CYCLES: u32 = (STEP_TIME_MS as f64 / (1_000_f64 / 4_194_304_f64)) as u32;

#[derive(Parser)]
#[command(author = "NathanW", about = "A Rust powered Gameboy emulator.")]
struct Args {
    #[arg(help = "Path to rom")]
    path: String,

    #[arg(short = 'x', long, help = "Display scale factor")]
    #[arg(value_enum, default_value_t)]
    scale: DisplayScale,

    #[arg(short, long, help = "Enable audio")]
    #[arg(default_value = "false")]
    audio: bool,

    #[arg(short, long, help = "Print serial write to stdout")]
    #[arg(default_value = "false")]
    serial: bool,

    #[arg(long, help = "Path to save file (default: <rom>.sav)")]
    save: Option<PathBuf>,

    #[arg(long, help = "Path to RTC file (default: <rom>.rtc)")]
    rtc: Option<PathBuf>,
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
}

fn load_file(path: &Path) -> Option<Vec<u8>> {
    File::open(path).ok().and_then(|mut f| {
        let mut data = Vec::new();
        f.read_to_end(&mut data).ok().map(|_| data)
    })
}

fn load_rtc_zero(path: &Path) -> Option<u64> {
    load_file(path).and_then(|data| {
        if data.len() >= 8 {
            Some(u64::from_le_bytes(data[..8].try_into().unwrap()))
        } else {
            None
        }
    })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let rom_name = args.path;

    let rom_path = Path::new(&rom_name);
    ensure!(rom_path.exists(), "file path provided does not exist");
    ensure!(
        rom_path.extension() == Some(OsStr::new("gb")),
        "file provided does not have the extention '.gb'"
    );

    let save_path = args.save.unwrap_or_else(|| rom_path.with_extension("sav"));
    let rtc_path = args.rtc.unwrap_or_else(|| rom_path.with_extension("rtc"));

    let rom_data = std::fs::read(rom_path).context("failed reading ROM file")?;

    let save_data = load_file(&save_path);
    if save_data.is_some() {
        println!("Loaded save file: {}", save_path.display());
    } else {
        println!("No save file found at: {}", save_path.display());
    }

    let rtc_zero = load_rtc_zero(&rtc_path);
    if rtc_zero.is_some() {
        println!("Loaded RTC file: {}", rtc_path.display());
    } else {
        println!("No RTC file found at: {}", rtc_path.display());
    }

    let cartridge = cartridge::open_cartridge(rom_data, save_data, rtc_zero)
        .context("failed loading cartridge")?;

    let opts = WindowOptions {
        scale: match args.scale {
            DisplayScale::X1 => Scale::X1,
            DisplayScale::X2 => Scale::X2,
            DisplayScale::X4 => Scale::X4,
            DisplayScale::X8 => Scale::X8,
            DisplayScale::X16 => Scale::X16,
        },
        ..Default::default()
    };

    let mut display = Window::new(
        &cartridge.title().to_lowercase(),
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        opts,
    )
    .context("failed to create window")?;

    let callback: Option<Box<dyn Fn(u8)>> = if args.serial {
        Some(Box::new(|b: u8| {
            print!("{}", b as char);
        }))
    } else {
        None
    };

    let mut gameboy = Gameboy::new(cartridge, callback);

    let _audio_stream = if args.audio {
        initialise_audio(&mut gameboy).context("failed to initialise audio")?
    } else {
        None
    };

    let keys = [
        (Key::Right, GbKey::Right),
        (Key::Up, GbKey::Up),
        (Key::Left, GbKey::Left),
        (Key::Down, GbKey::Down),
        (Key::Z, GbKey::A),
        (Key::X, GbKey::B),
        (Key::Space, GbKey::Select),
        (Key::Enter, GbKey::Start),
    ];

    let mut step_cycles: u32 = 0;
    let mut step_zero = Instant::now();

    while display.is_open() && !display.is_key_down(Key::Q) {
        if step_cycles > STEP_CYCLES {
            step_cycles -= STEP_CYCLES;
            let now = Instant::now();
            let elapsed = now.duration_since(step_zero);
            let sleep_time = STEP_TIME_MS.saturating_sub(elapsed.as_millis() as u64);
            std::thread::sleep(std::time::Duration::from_millis(sleep_time));
            step_zero = step_zero
                .checked_add(std::time::Duration::from_millis(STEP_TIME_MS))
                .unwrap();
            if now.checked_duration_since(step_zero).is_some() {
                step_zero = now;
            }
        }

        let cycles = gameboy.tick();
        gameboy.update(cycles);
        step_cycles += cycles;

        if gameboy.display_updated() {
            display
                .update_with_buffer(gameboy.display_buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)
                .context("failed to update display")?;
        }

        for (input, key) in keys.iter() {
            if display.is_key_down(*input) {
                gameboy.key_down(*key);
            } else {
                gameboy.key_up(*key);
            }
        }
    }

    // Save RAM data if the cartridge supports it.
    if let Some(data) = gameboy.save_data() {
        File::create(&save_path)
            .and_then(|mut f| f.write_all(data))
            .context("failed to save game")?;
        println!("Saved game to: {}", save_path.display());
    }

    // Save RTC zero if the cartridge has RTC.
    if let Some(rtc) = gameboy.rtc_zero() {
        File::create(&rtc_path)
            .and_then(|mut f| f.write_all(&rtc.to_le_bytes()))
            .context("failed to save RTC")?;
        println!("Saved RTC to: {}", rtc_path.display());
    }

    Ok(())
}

fn initialise_audio(gameboy: &mut Gameboy) -> Result<Option<cpal::Stream>> {
    let device = cpal::default_host()
        .default_output_device()
        .context("failed to find audio output device.")?;
    println!("Using audio output device: {}", device.name()?);
    let config = device.default_output_config()?;
    let err_fn = |err| eprintln!("an error occurred on audio stream: {}", err);

    gameboy.enable_audio(config.sample_rate().0);
    let stream_buffer = gameboy
        .audio_buffer()
        .context("failed to get audio buffer")?;

    let stream = device
        .build_output_stream(
            &config.config(),
            move |out_buf: &mut [f32], _: &OutputCallbackInfo| {
                let mut in_buf = stream_buffer.lock().expect("failed to lock audio buffer");
                let length = std::cmp::min(out_buf.len() / 2, in_buf.len());

                for (idx, (data_l, data_r)) in in_buf.drain(..length).enumerate() {
                    out_buf[idx * 2] = data_l;
                    out_buf[idx * 2 + 1] = data_r;
                }
            },
            err_fn,
        )
        .context("failed to build audio stream")?;

    stream.play().context("failed to play audio stream")?;
    Ok(Some(stream))
}

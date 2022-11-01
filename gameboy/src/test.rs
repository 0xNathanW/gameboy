// Tests all cpu instructions, results printed in std output.
// Thus, make sure to run with "cargo test cpu_instructions -- --nocapture"
#[test]
fn cpu_instructions() {
    use std::path::Path;
    use gameboy_core::cpu::CPU;
    use gameboy_core::cartridge;

    let test_path = Path::new("./test_roms/cpu_instrs/cpu_instrs.gb");
    assert!(test_path.exists());

    let callback = |b: u8| { print!("{}", b as char); };
    let cartridge = cartridge::open_cartridge(test_path);

    let mut cpu = CPU::new(
        cartridge,
        Some(Box::new(callback)),
    );

    let mut total_cycles = 0;
    while total_cycles < 127_605_866 {
        let cycles = cpu.step();
        cpu.mem.update(cycles);
        total_cycles += cycles;
    }

    let mut sum = 0_u32;

    for idx in 0..cpu.mem.gpu.pixels.len() {
                sum = sum.wrapping_add((cpu.mem.gpu.pixels[idx] as u32).wrapping_mul(idx as u32));
    }
    println!("\nchecksum = {}", sum);
}

// Basic b/w display test.
#[test] 
fn minifb_test() {
    use std::thread;
    use minifb::{Window, WindowOptions, Scale};

    let mut window = Window::new(
        "test", 
        gameboy_core::SCREEN_WIDTH, 
        gameboy_core::SCREEN_HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..Default::default()
        }
    ).unwrap();

    let mut buf: Vec<u32> = vec![0; gameboy_core::SCREEN_HEIGHT * gameboy_core::SCREEN_WIDTH];
    
    let mut r: u8 = 0; let mut g: u8 = 0; let mut b: u8 = 0;

    while window.is_open() {
        
        r = r.wrapping_add(10);
        g = g.wrapping_add(10);
        b = b.wrapping_add(10);
        let colour =
            (r as u32) << 16 |
            (g as u32) << 8  |
            (b as u32);

        for pix in buf.iter_mut().step_by(3) {
            *pix = colour;
        }

        thread::sleep(std::time::Duration::from_millis(100));
        window.update_with_buffer(&buf, 160, 144).unwrap();
    }
}

    
// Should play a simmple beep sound.
#[test]
fn cpal_test() {
    use cpal;
    use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

    let host = cpal::default_host();
    let device = host.default_output_device().expect("failed to find output device.");
    println!("Output device: {}", device.name().unwrap());
    
    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);
    println!("{:?}", config.sample_format());

    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    let mut sample_clock = 0f32;
    let mut nxt_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config.config(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                for sample in frame.iter_mut() {
                    *sample = nxt_value();
                }
            }
        },
        err_fn,
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(3000));
}

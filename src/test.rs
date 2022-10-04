const CPU_CHECKSUM: u32 = 3112234583;

// 3112234583
// 3541534464

// Tests all cpu instructions, results printed in std output.
// Thus, make sure to run with "cargo test cpu_instructions -- --nocapture"
#[test]
fn cpu_instructions() {
    use std::path::Path;
    use super::cpu::CPU;
    use super::cartridge;

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
        160, 
        144, 
        WindowOptions {
            scale: Scale::X8,
            ..Default::default()
        }
    ).unwrap();

    let mut buf: Vec<u32> = vec![0; 144 * 160];
    
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

#[test]
fn cpu_instrs_system() {
    use std::path::Path;
    use minifb::{Window, WindowOptions, Scale};
    use crate::system::Gameboy;

    let test_path = Path::new("./test_roms/cpu_instrs/cpu_instrs.gb");
    assert!(test_path.exists());

    let window = Window::new(
        "test", 
        160, 
        144, 
        WindowOptions {
            scale: Scale::X8,
            ..Default::default()
        }
    ).unwrap();

    let mut gb = Gameboy::new(test_path, window, None);
    gb.run();
}
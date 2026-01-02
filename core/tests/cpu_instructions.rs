use gameboy_core::{cartridge, Gameboy};
use std::{cell::RefCell, path::Path, rc::Rc};

/*
Runs the blargg test rom on the CPU to verify cpu instructions work correctly.
Test are validated through the serial port.
If a test fails, it will print an error code.
soure: https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs
*/

const CLOCK_FREQUENCY_HZ: u64 = 4_194_304;
// Will run for amount of cycles emulating 1 minute.
// Won't actually run for this long, more like 10 seconds.
const BLARGG_INSTRUCTION_TEST_CYCLES: u64 = 60 * CLOCK_FREQUENCY_HZ;

#[test]
fn cpu_instructions() {
    let rom_path = Path::new("tests/test_assets/cpu_instrs.gb");
    let rom_data = std::fs::read(rom_path).expect("Failed to read test ROM file");

    let output = Rc::new(RefCell::new(String::new()));
    let output_clone = Rc::clone(&output);
    let callback = move |b: u8| {
        output_clone.borrow_mut().push(b as char);
    };

    let cartridge =
        cartridge::open_cartridge(rom_data, None, None).expect("Failed to load test ROM");
    let mut gameboy = Gameboy::new(cartridge, Some(Box::new(callback)));

    let mut cycles_run = 0;
    while cycles_run < BLARGG_INSTRUCTION_TEST_CYCLES {
        let cycles = gameboy.tick();
        gameboy.update(cycles);
        cycles_run += cycles as u64;
    }

    let serial_output = output.borrow();
    println!("Serial output:\n{}", *serial_output);

    // If all tests pass, will print "Passed all tests"
    assert!(serial_output.contains("Passed all tests"));
}

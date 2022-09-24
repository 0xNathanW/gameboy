use crossterm::event::{Event, read, KeyCode};
use std::{path::Path, sync::Arc};

use crate::cpu::CPU;

// Tests all cpu instructions, results printed in std output.
// Thus, make sure to run with "cargo test cpu_instructions -- --nocapture"
#[test]
fn cpu_instructions() {
    
    let test_path = Path::new("./test_roms/cpu_instrs/individual/01-special.gb");
    assert!(test_path.exists());

    let callback = |b: u8| { print!("{}", b as char); };

    let mut cpu = CPU::new(
        test_path,
        Some(Box::new(callback)),
    );

    let mut total_cycles = 0;
    while total_cycles < 127_605_866 {
        let cycles = cpu.step();
        cpu.mem.update(cycles);
        total_cycles += cycles;
    }
}


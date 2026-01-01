#![allow(clippy::upper_case_acronyms)]

#[cfg(feature = "audio")]
mod apu;
mod bus;
mod cpu;
mod gameboy;
mod keypad;

mod bit;
mod clock;
mod gpu;
mod intf;
mod memory;
mod serial;
mod timer;

pub mod cartridge;
pub use gameboy::Gameboy;
pub use keypad::GbKey;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#![allow(clippy::upper_case_acronyms)]

#[cfg(feature = "audio")]
pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod keypad;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

mod bit;
mod clock;
mod gpu;
mod intf;
mod memory;
mod serial;
mod timer;

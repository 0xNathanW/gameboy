#![allow(clippy::upper_case_acronyms)]

pub mod cpu;
pub mod cartridge;
pub mod keypad;
pub mod bus;
#[cfg(feature = "audio")]
pub mod apu;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

mod memory;
mod gpu;
mod mbc;
mod timer;
mod bit;
mod serial;
mod clock;
mod intf;

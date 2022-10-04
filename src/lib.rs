#![allow(dead_code, unused_imports)]
pub mod cpu;
pub mod cartridge;
pub mod keypad;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

mod memory;
mod gpu;
mod mbc;
mod bus;
mod timer;
mod bit;
mod serial;
mod intf;

#[cfg(test)]
mod test;
#![allow(dead_code, unused_imports)]
pub mod system;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

mod cpu;
mod memory;
mod gpu;
mod cartridge;
mod mbc;
mod bus;
mod timer;
mod keypad;
mod bit;
mod serial;
mod intf;

#[cfg(test)]
mod test;
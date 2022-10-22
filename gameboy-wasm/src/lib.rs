mod utils;

use std::path::Path;

use wasm_bindgen::prelude::*;
use gameboy_core::cartridge::open_cartridge;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn cartridge_title(rom_str: &str) -> String {
    let rom_path = Path::new(rom_str);
    let cartridge = open_cartridge(rom_path);
    cartridge.title()
}
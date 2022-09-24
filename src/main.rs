use minifb::{Window, WindowOptions, Scale};
use std::{path::Path, ffi::OsStr};

use gameboy::{SCREEN_HEIGHT, SCREEN_WIDTH};
use gameboy::system::Gameboy;


fn main() {

    // let rom_name = std::env::args().nth(1).expect(
    //     "a path to a rom must be provided as an argument"
    // );
    ///////////////////////////////////////////
    let p = Path::new("./test_roms/cpu_instrs/cpu_instrs.gb");
    ///////////////////////////////////////////
    // let rom_path = Path::new(&rom_name);
    // if !rom_path.exists() { panic!("path does not exist"); }    
    // if rom_path.extension() != Some(OsStr::new("gb")) {
    //     println!("{}", rom_path.extension().unwrap().to_str().unwrap());
    //     panic!("file provided does not have the extention '.gb'"); 
    // }

    let opts = WindowOptions {
        scale: Scale::X4,
        ..Default::default()
    };

    let display = Window::new(
        // String::as_str(""),
        "",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        opts,
    ).unwrap();

    let callback = |b: u8| { print!("{}", b as char); };

    let mut gb = Gameboy::new(p, display, Some(Box::new(callback)));
    gb.run();
}

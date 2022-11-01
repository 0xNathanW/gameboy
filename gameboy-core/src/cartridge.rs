use std::path::Path;

use super::bus::MemoryBus;
use super::mbc::mbc1::MBC1;
use super::mbc::mbc2::MBC2;
use super::mbc::mbc3::MBC3;
use super::mbc::mbc5::MBC5;

// Nintendo logo bitmap, cartridge address range $0104-$0133 must match.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0104-0133---nintendo-logo
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11,
    0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E,
    0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub trait Cartridge: MemoryBus {
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self);
    
    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> *const u8;

    
    // The Game Boy’s boot procedure first displays the logo and then checks that it matches the dump above. 
    //If it doesn’t, the boot ROM locks itself up.
    fn verify_logo(&self) {
        for i in 0..48 {
            if self.read_byte(0x0104+i) != NINTENDO_LOGO[i as usize] {
                panic!("nintendo logo in cartridge is not correct.");
            }
        }
    }

    // Byte 0x014D contains an 8-bit checksum computed from the cartridge header bytes $0134—$014C.
    fn verify_checksum(&self) {
        let mut checksum: u8 = 0;
        for i in 0x0134..0x014D {
            checksum = checksum.wrapping_sub(self.read_byte(i)).wrapping_sub(1);
        }
        if checksum != self.read_byte(0x014D) {
            panic!("header checksum incorrect");
        }
    }

    // Retrieve title of game in upper-case ASCI.
    fn title(&self) -> String {
        let mut title = String::new();
        for address in 0x134..= 0x143 {
            title.push(self.read_byte(address) as char);
        }
        title
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_cartridge(path: &Path) -> Box<dyn Cartridge>{
    let buf = std::fs::read(path).expect("failed to read file"); 

    let save_path = Some(path.to_path_buf().with_extension("sav"));
    let rtc_path  = Some(path.to_path_buf().with_extension("rtc"));

    // Cartridge has a header addr range $0100—$014F, followed by a JUMP @ $0150
    if buf.len() < 0x0150 {
        panic!("missing info in cartridge header")
    }
    // byte 0x0147 indicates what kind of hardware is present on the cartridge — most notably its mapper.
    let cartridge: Box<dyn Cartridge> = match buf[0x147] {
        // ROM only.
        0x00 => Box::new(ROM::new(buf)),
        // MBC1.
        0x01 => Box::new(MBC1::new(buf, 0, None)),
        // MBC1 + RAM. 
        0x02 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC1::new(buf, ram_size, None))
        },
        // MBC1 + RAM + BATTERY.
        0x03 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC1::new(buf, ram_size, save_path))
        },
        // MBC2.
        0x05 => Box::new(MBC2::new(buf, 512, None)),
        // MBC2 + BATTERY.
        0x06 => Box::new(MBC2::new(buf, 512, save_path)),
        // MBC3 + TIMER + BATTERY.
        0x0F => Box::new(MBC3::new(buf, 0, save_path, rtc_path)),
        // MBC3 + TIMER + RAM + BATTERY. 
        0x10 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, save_path, rtc_path))
        },
        // MBC3.
        0x11 => Box::new(MBC3::new(buf, 0, None, None)),
        // MBC3 + RAM.
        0x12 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, None, None))
        },
        // MBC3 + RAM + BATTERY.
        0x13 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, save_path, None))
        },
        // MBC5.
        0x19 => Box::new(MBC5::new(buf, 0, None)),
        // MBC5 + RAM.
        0x1A => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC5::new(buf, ram_size, None))
        },
        // MBC5 + RAM + BATTERY.
        0x1B => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC5::new(buf, ram_size, save_path))
        },
        unknown => panic!("unsupported cartridge type, {:#X}", unknown),
    };
    
    // If verification of logo or checksum fails, program should panic.
    cartridge.verify_logo();
    cartridge.verify_checksum();
    cartridge
}

#[cfg(target_arch = "wasm32")]
pub fn open_cartridge(buf: Vec<u8>, save_data: Option<Vec<u8>>) -> Box<dyn Cartridge>{

    // Cartridge has a header addr range $0100—$014F, followed by a JUMP @ $0150
    if buf.len() < 0x0150 {
        panic!("missing info in cartridge header")
    }
    // byte 0x0147 indicates what kind of hardware is present on the cartridge — most notably its mapper.
    let cartridge: Box<dyn Cartridge> = match buf[0x147] {
        // ROM only.
        0x00 => Box::new(ROM::new(buf)),
        // MBC1.
        0x01 => Box::new(MBC1::new(buf, 0, None)),
        // MBC1 + RAM. 
        0x02 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC1::new(buf, ram_size, None))
        },
        // MBC1 + RAM + BATTERY.
        0x03 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC1::new(buf, ram_size, save_data))
        },
        // MBC2.
        0x05 => Box::new(MBC2::new(buf, 512, None)),
        // MBC2 + BATTERY.
        0x06 => Box::new(MBC2::new(buf, 512, save_data)),
        // MBC3 + TIMER + BATTERY.
        0x0F => Box::new(MBC3::new(buf, 0, save_data, None)),
        // MBC3 + TIMER + RAM + BATTERY. 
        0x10 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, save_data, None))
        },
        // MBC3.
        0x11 => Box::new(MBC3::new(buf, 0, None, None)),
        // MBC3 + RAM.
        0x12 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, None, None))
        },
        // MBC3 + RAM + BATTERY.
        0x13 => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC3::new(buf, ram_size, save_data, None))
        },
        // MBC5.
        0x19 => Box::new(MBC5::new(buf, 0, None)),
        // MBC5 + RAM.
        0x1A => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC5::new(buf, ram_size, None))
        },
        // MBC5 + RAM + BATTERY.
        0x1B => {
            let ram_size = ram_size(buf[0x149]);
            Box::new(MBC5::new(buf, ram_size, save_data))
        },
        unknown => panic!("unsupported cartridge type, {:#X}", unknown),
    };
    
    // If verification of logo or checksum fails, program should panic.
    cartridge.verify_logo();
    cartridge.verify_checksum();
    cartridge
}

// byte 0x0149 indicates size of RAM, if any.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0149---ram-size
pub fn ram_size(n: u8) -> usize {
    let kb = 1024;
    match n {
        0x00 => 0,
        0x01 => 2   * kb,
        0x02 => 8   * kb,
        0x03 => 32  * kb,
        0x04 => 128 * kb,
        0x05 => 64  * kb,
        _ => 0,
    }
}

// Small games of not more than 32 KiB ROM do not require a MBC chip for ROM banking.
pub struct ROM(Vec<u8>);

impl ROM {
    pub fn new(data: Vec<u8>) -> Self {
        ROM(data)
    }
}

impl MemoryBus for ROM {
    fn read_byte(&self, address: u16) -> u8 { self.0[address as usize] }
    // ROM is read-only so no write functionality.
    fn write_byte(&mut self, _: u16, _: u8) {}
}

impl Cartridge for ROM { 
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) {}

    #[cfg(target_arch = "wasm32")]
    fn save(&self) -> *const u8 { self.0.as_ptr() }
}

#[cfg(test)]
mod test {

    use std::path::Path;
    use crate::cartridge::open_cartridge;

    // ROMs with different cartridge architecture.
    // https://b13rg.github.io/Gameboy-MBC-Analysis/#no-mbc
    // Checks logo + checksum verification.
    #[test]
    fn rom_only() {
        let test_path = Path::new("./test_roms/ThisIsATest.gb");
        assert!(test_path.exists());
        open_cartridge(test_path);

        let dr_mario = Path::new("./test_roms/drMario.gb");
        assert!(dr_mario.exists());
        open_cartridge(dr_mario);
    }

    #[test]
    fn mbc1() {
        let test_path = Path::new("./test_roms/cpu_instrs/individual/01-special.gb");
        assert!(test_path.exists());
        let cart = open_cartridge(test_path);

        assert_eq!(cart.read_byte(0x4000), 0xC3);
    }
}
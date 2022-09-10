use std::io::Read;
use std::path::Path;
use std::fs::File;

use super::bus::MemoryBus;

// Nintendo logo bitmap, cartridge address range $0104-$0133 must match.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0104-0133---nintendo-logo
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11,
    0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E,
    0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];


pub trait Cartridge: MemoryBus {
    
    // The Game Boy’s boot procedure first displays the logo and then checks that it matches the dump above. 
    //If it doesn’t, the boot ROM locks itself up.
    fn verify_logo(&self) {
        for i in 0..48 {
            if self.read_byte(0x0104+i) != NINTENDO_LOGO[i as usize] {
                panic!("logo bytes do not match")
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
            panic!("header checksum incorrect")
        }
    }

    fn title(&self) {} //TODO
 
}


pub fn open_cartridge(p: &Path) -> Box<dyn Cartridge>{

    let mut buf = Vec::new();
    File::open(p).and_then(|mut f| f.read_to_end(&mut buf)).unwrap();  
    // Cartridge has a header addr range $0100—$014F, followed by a JUMP @ $0150
    if buf.len() < 0x0150 {
        panic!("missing info in cartridge header")
    }

    // byte 0x0147 indicates what kind of hardware is present on the cartridge — most notably its mapper.
    let cartridge: Box<dyn Cartridge> = match buf[0x0147] {
        // ROM only
        0x00 => Box::new(ROM::new(buf)),
        // TODO: others

        e => panic!("unsupported cartridge type, {}", e)
    };
    
    // If verification of logo or checksum fails, program should panic.
    cartridge.verify_logo();
    cartridge.verify_checksum();
    cartridge
}

// byte 0x0149 indicates size of RAM, if any.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0149---ram-size
fn _ram_size(n: u8) -> usize {
    let kb = 1024;
    match n {
        0x0..=0x01 => 0,
        0x02 => 8 * kb,
        0x03 => 32 * kb,
        0x04 => 128 * kb,
        0x05 => 64 * kb,
        e => panic!("unsupported RAM size, {:#02x}", e) 
    }
}

// Small games of not more than 32 KiB ROM do not require a MBC chip for ROM banking.
pub struct ROM(Vec<u8>);

impl ROM {
    fn new(data: Vec<u8>) -> Self {
        ROM(data)
    }
}

impl MemoryBus for ROM {
    fn read_byte(&self, address: u16) -> u8 {
        self.0[address as usize]
    }
    // ROM is read-only so no write functionality.
    fn write_byte(&mut self, _: u16, _: u8) {}
}

impl Cartridge for ROM {}


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

}
use std::default;
use std::io::Read;
use std::path::Path;
use std::fs::File;

// Nintendo logo bitmap, cartridge address range $0104-$0133 must match.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0104-0133---nintendo-logo
const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11,
    0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E,
    0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub fn open_cartridge(p: &Path) {
    let buf = Vec::new();
    let mut f = File::open(p).unwrap();
    f.read_to_end(&mut buf);
    
    // Cartridge has a header addr range $0100—$014F, followed by a JUMP @ $0150
    if buf.len() < 0x0150 {
        panic!("missing info in cartridge header")
    }
    let rom_size = rom_size(buf[0x0148]);
    let ram_size = ram_size(buf[0x0]);



    


}

// byte 0x0148 indicates size of ROM.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0148---rom-size
fn rom_size(n: u8) -> usize {
    let bank = 16384;
    match n {
        0x00..=0x08 => 32_768 * (1 << n),
        0x52 => 72 * bank,
        0x53 => 80 * bank,
        0x54 => 96 * bank,
        e => panic!("unsupported ROM size: {}", e)
    }
}

// byte 0x0149 indicates size of RAM, if any.
// https://gbdev.io/pandocs/The_Cartridge_Header.html#0149---ram-size
fn ram_size(n: u8) -> usize {
    match n {
        
    }
}

// byte 0x0147
fn cartridge_type(n: u8) -> usize {

}

#[cfg(test)]
mod test {

    #[test]
    fn rom_size() {
    }
}
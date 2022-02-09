use std::path::Path;
use std::io::Read;

/*
General Memory Map
  0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
  4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
  8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
  A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
  C000-CFFF   4KB Work RAM Bank 0 (WRAM)
  D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
  E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
  FE00-FE9F   Sprite Attribute Table (OAM)
  FEA0-FEFF   Not Usable
  FF00-FF7F   I/O Ports
  FF80-FFFE   High RAM (HRAM)
  FFFF        Interrupt Enable Register
*/

const WRAM_SIZE: usize = 8_192;     // 8KB Work RAM.
const VRAM_SIZE: usize = 8_192;     // 8KB Video RAM.
const HRAM_SIZE: usize = 127;       // High RAM.
const ROM_SIZE:  usize = 32_768;    // 32KB ROM.

// Memory management unit.
struct MMU {

    // Cartridge ROM - read only, 2x16KB banks = 32KB.
    rom: [u8; ROM_SIZE],

    // Work RAM - 2x8KB banks = 16KB.
    wram: [u8; WRAM_SIZE],

    // High RAM - 127 bytes.
    hram: [u8; HRAM_SIZE],

}

impl Memory {

    fn new(cartridge: &Path) -> Memory {
        
        Read::read_to_end(&mut self, buf)


    }
}
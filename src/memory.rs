use std::path::Path;
use super::cartridge;
use super::bus::MemoryBus;
use super::timer::Timer;
use super::gpu::GPU;

/*
When the CPU tries to access a given address, it’s the Memory's job to 
determine which piece of underlying hardware that particular address 
belongs to, and to forward the access to that device as appropriate.
*/

/*
General Memory Map
    cartridge.
    0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
    4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
    gpu. 
    8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
    cartridge.
    A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
    wram.
    C000-CFFF   4KB Work RAM Bank 0 (WRAM)
    D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
    also wram.
    E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
    gpu.
    FE00-FE9F   Sprite Attribute Table (OAM)
    not used.
    FEA0-FEFF   Not Usable
    io.
    FF00-FF7F   I/O Ports
        > FF00          Joypad input
        > FF01-FF02     Serial transfer
        > FF04-FF07     Timer and divider
        > FF10-FF26     Sound
        > FF30-FF3F     Wave pattern
        > FF40-FF4B     LCD control
        > FF4F          VRAM bank select
        > FF50          Set to non-zero to disable baot rom
        > FF51-FF55     VRAM DNA
        > FF68-FF69     BG / OBJ palettes
        > FF70          WRAM bank select 
    FF80-FFFE   High RAM (HRAM)
    FFFF        Interrupt Enable Register
*/

const HRAM_SIZE: usize = 127;        // High RAM.
const WRAM_SIZE:  usize = 32_768;    // 32KB Work RAM.

pub struct Memory {
    cartridge: Box<dyn cartridge::Cartridge>,
    gpu: GPU,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    timer: Timer,
}

impl Memory {
    
    pub fn new(path: &Path) -> Self {
        let mut memory = Self {
            cartridge: cartridge::open_cartridge(path),
            gpu: GPU::new(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            timer: Timer::new(),
        };
        memory.initialise();
        memory
    }
}

impl MemoryBus for Memory {

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000 ..= 0x7FFF => self.cartridge.read_byte(address),
            
            0xA000 ..= 0xBFFF => self.cartridge.read_byte(address),

            0xC000 ..= 0xDFFF => self.wram[address as usize - 0xC000],
            0xE000 ..= 0xEFFF => self.wram[address as usize - 0xE000],

            0xFF04 ..= 0xFF07 => self.timer.read_byte(address),
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x0000 ..= 0x7FFF => self.cartridge.write_byte(address, b),
            
            0xA000 ..= 0xBFFF => self.cartridge.write_byte(address, b),

            0xC000 ..= 0xDFFF => { self.wram[address as usize - 0xC000] = b },
            0xE000 ..= 0xEFFF => { self.wram[address as usize - 0xE000] = b },

            0xFF04 ..= 0xFF07 => { self.timer.write_byte(address, b) },
            _ => {},
        }
    }
}

impl Memory {
    
    // Set inital values, rest should be randomised but we can also set to 0.
    fn initialise(&mut self) {
        // http://www.codeslinger.co.uk/pages/projects/gameboy/hardware.html
        self.write_byte(0xFF05, 0x00);
        self.write_byte(0xFF06, 0x00);
        self.write_byte(0xFF07, 0x00);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0x00);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xBF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0x00);
        self.write_byte(0xFF22, 0x00);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0x00);
        self.write_byte(0xFF43, 0x00);
        self.write_byte(0xFF45, 0x00);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0x00);
        self.write_byte(0xFF4B, 0x00);
        self.write_byte(0xFFFF, 0x00);
    }

}
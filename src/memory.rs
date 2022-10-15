use std::cell::RefCell;
use std::rc::Rc;

use crate::SCREEN_WIDTH;
use super::cartridge::Cartridge;
use super::serial::SerialCallback;
use super::cartridge;
use super::bus::MemoryBus;
use super::timer::Timer;
use super::gpu::GPU;
use super::keypad::KeyPad;
use super::intf::Intf;
use super::serial::Serial;
use super::apu::APU;

const HRAM_SIZE: usize = 127;        // High RAM.
const WRAM_SIZE:  usize = 8_192;    // 8KB Work RAM.

pub struct Memory {
    
    cartridge:      Box<dyn cartridge::Cartridge>,    
    wram:           [u8; WRAM_SIZE],
    hram:           [u8; HRAM_SIZE],
    timer:          Timer,
    
    // IO
    pub gpu:        GPU,
    pub keypad:     KeyPad,
    pub apu:        Option<APU>,
    serial:         Serial,
    
    // inte is written to buy game.
    inte:           u8,
    // intf can be written to by components to request interrupts.
    // needs to be shared and have interior mutability.
    intf:           Rc<RefCell<Intf>>,
}

impl Memory {
    pub fn new(cartridge: Box<dyn Cartridge>, callback: SerialCallback) -> Self {
        let intf = Rc::new(RefCell::new(Intf::new()));
        let mut memory = Self {
            cartridge,
            apu:        None,
            gpu:        GPU::new(intf.clone()),
            wram:       [0; WRAM_SIZE],
            hram:       [0; HRAM_SIZE],
            timer:      Timer::new(intf.clone()),
            keypad:     KeyPad::new(intf.clone()),
            serial:     Serial::new(intf.clone(), callback),
            inte:       0,
            intf:       intf.clone(),
        };
        memory.initialise();
        memory
    }
}

impl MemoryBus for Memory {

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
            // 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
            0x0000 ..= 0x7FFF => self.cartridge.read_byte(address),
            
            // 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
            0x8000 ..= 0x9FFF => self.gpu.read_byte(address),

            //A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
            0xA000 ..= 0xBFFF => self.cartridge.read_byte(address),

            // C000-CFFF   4KB Work RAM Bank 0 (WRAM)
            // D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
            0xC000 ..= 0xDFFF => self.wram[address as usize - 0xC000],
            // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
            0xE000 ..= 0xEFFF => self.wram[address as usize - 0xE000],

            // FE00-FE9F   Sprite Attribute Table (OAM)
            0xFE00 ..= 0xFE9F => self.gpu.read_byte(address),

            // I/O Ports 
            0xFF00 => self.keypad.read_byte(address),                     // Joypad input
            0xFF01 ..= 0xFF02 => self.serial.read_byte(address),
            0xFF04 ..= 0xFF07 => self.timer.read_byte(address),           // Timer/Divider
            0xFF0F => self.intf.borrow().read_byte(address),
            0xFF10 ..= 0xFF3F => match &self.apu {
                Some(apu) => apu.read_byte(address),
                None => 0, 
            },
            0xFF40 ..= 0xFF4B => self.gpu.read_byte(address),

            // FF80-FFFE   High RAM (HRAM)
            0xFF80 ..= 0xFFFE => self.hram[address as usize - 0xFF80],

            // 0xFFFF   Interrupt Enable (R/W)
            0xFFFF => self.inte,
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0x0000 ..= 0x7FFF => self.cartridge.write_byte(address, b),
            0x8000 ..= 0x9FFF => self.gpu.write_byte(address, b),
            0xA000 ..= 0xBFFF => self.cartridge.write_byte(address, b),
            0xC000 ..= 0xDFFF => self.wram[address as usize - 0xC000] = b,
            0xE000 ..= 0xEFFF => self.wram[address as usize - 0xE000] = b,
            0xFE00 ..= 0xFE9F => self.gpu.write_byte(address, b),
            0xFF00 => self.keypad.write_byte(address, b),
            0xFF01 ..= 0xFF02 => self.serial.write_byte(address, b),
            0xFF04 ..= 0xFF07 => self.timer.write_byte(address, b),
            0xFF0F => self.intf.borrow_mut().write_byte(address, b),
            0xFF10 ..= 0xFF3F => match &mut self.apu {
                Some(apu) => apu.write_byte(address, b),
                None => {},
            },
            0xFF40 ..= 0xFF45 => self.gpu.write_byte(address, b),
            0xFF46 => self.dma_transfer(b),
            0xFF47 ..= 0xFF4B => self.gpu.write_byte(address, b),
            0xFF80 ..= 0xFFFE => self.hram[address as usize - 0xFF80] = b,
            0xFFFF => self.inte = b,
            _ => {},
        }
    }
}

impl Memory {

    pub fn update(&mut self, cycles: u32) {
        self.timer.update(cycles);
        self.gpu.update(cycles);
        self.apu.as_mut().map_or((), |apu| apu.next(cycles));
    } 

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

    // Direct memory transfer (DMA) from ROM/RAm to OAM.
    fn dma_transfer(&mut self, src_address: u8) {
        let base_address: u16 = (src_address as u16) << 8;
        for x in 0..SCREEN_WIDTH as u16 {
            self.gpu.write_byte(0xFE00 + x, self.read_byte(base_address + x));
        }
    }
}

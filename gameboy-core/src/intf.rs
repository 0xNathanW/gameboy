use crate::bus::MemoryBus;

pub enum InterruptSource {
    VBlank  = 0b00000001,
    STAT    = 0b00000010,    
    Timer   = 0b00000100,
    Serial  = 0b00001000,
    Keypad  = 0b00010000,
}
// Info on interrupts - http://www.codeslinger.co.uk/pages/projects/gameboy/interupts.html
#[derive(Clone, Default)]
pub struct Intf (u8);

impl Intf {
    pub fn new() -> Self { Intf(0) }

    pub fn set_interrupt(&mut self, src: InterruptSource) {
        self.0 |= src as u8;
    }
}

impl MemoryBus for Intf {

    fn read_byte(&self, address: u16) -> u8 {
        assert_eq!(address, 0xFF0F);
        self.0
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        assert_eq!(address, 0xFF0F);
        self.0 = b;
    }
}
pub trait MemoryBus {

    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, b: u8);

    fn read_word(&self, address: u16) -> u16 {
        (self.read_byte(address) as u16) | ((self.read_byte(address+1) as u16) << 8)
    }

    fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0xFF) as u8);
        self.write_byte(address+1, (word >> 8) as u8)
    }
}
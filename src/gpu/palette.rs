use crate::bus::MemoryBus;

/*  Shades
    White   = 255,
    Light   = 192,
    Dark    = 96,
    Black   = 0,
*/

#[derive(Default)]
pub struct Palette {
    data: u8,
    colours: [u8; 4],
}

impl Palette {
    
    pub fn new() -> Self { Self::default() }

    pub fn get_shade(&self, idx: usize) -> u8 {
        match (self.data >> 2 * idx) & 0b11 {
            0b00 => 255,
            0b01 => 192,
            0b10 => 96, 
            0b11 => 0,
            _    => panic!("invalid colour"),
         }
    }

    pub fn update(&mut self) {
        for idx in 0..4 { self.colours[idx] = self.get_shade(idx) }
    }
}

impl MemoryBus for Palette {
    fn read_byte(&self, _: u16) -> u8 { self.data }

    fn write_byte(&mut self, _: u16, b: u8) { self.data = b; self.update() }
}

#[cfg(test)]
mod test {
    use crate::bus::MemoryBus;

    use super::Palette;

    #[test] 
    fn palette() {
        let mut pal = Palette::new();
        pal.write_byte(0x0000, 0b1011_0010);
        pal.update();
        assert_eq!(pal.colours[0], 96);
        assert_eq!(pal.colours[1], 255);
        assert_eq!(pal.colours[2], 0);
        assert_eq!(pal.colours[3], 96);
    }
}

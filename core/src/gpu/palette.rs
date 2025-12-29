use crate::bus::MemoryBus;

/*  Shades - B/W (u8)
    White   = 255,
    Light   = 192,
    Dark    = 96,
    Black   = 0
*/

#[derive(Default)]
pub struct Palette {
    data: u8,
    shades: [u32; 4],
    colours: [u32; 4],
}

impl Palette {
    pub fn new() -> Self {
        Self {
            colours: [0xe0f8d0, 0x88c070, 0x346856, 0x081820], // Classic Gameboy colours.
            ..Default::default()
        }
    }

    pub fn get_shade(&self, idx: usize) -> u32 {
        match self.data >> (2 * idx) & 3 {
            0b00 => self.colours[0], // Lightest
            0b01 => self.colours[1],
            0b10 => self.colours[2],
            0b11 => self.colours[3], // Darkest
            _ => unreachable!(),
        }
    }

    pub fn set_colours(&mut self, colours: [u32; 4]) {
        self.colours = colours;
    }

    pub fn colours(&self) -> [u32; 4] {
        self.colours
    }

    pub fn update(&mut self) {
        for idx in 0..4 {
            self.shades[idx] = self.get_shade(idx)
        }
    }
}

impl MemoryBus for Palette {
    fn read_byte(&self, _: u16) -> u8 {
        self.data
    }

    fn write_byte(&mut self, _: u16, b: u8) {
        self.data = b;
        self.update()
    }
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
        assert_eq!(pal.shades[0], 0x346856);
        assert_eq!(pal.shades[1], 0xe0f8d0);
        assert_eq!(pal.shades[2], 0x081820);
        assert_eq!(pal.shades[3], 0x346856);
    }
}

use crate::{
    bit::Bit,
    bus::MemoryBus,
    intf::{InterruptSource, Intf},
};
use std::{cell::RefCell, rc::Rc};

// FF00 - P1/JOYP - Joypad (R/W)
//
// Bit 7 - Not used
// Bit 6 - Not used
// Bit 5 - P15 Select Button Keys      (0=Select)
// Bit 4 - P14 Select Direction Keys   (0=Select)
// Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
// Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
// Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
// Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)

#[derive(Clone)]
pub enum GbKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

/*
The eight Game Boy action/direction buttons are arranged as a 2x4 matrix.
Select either action or direction buttons by writing to this register, then read out the bits 0-3.
*/
pub struct KeyPad {
    reg: [u8; 2],
    select: u8,
    intf: Rc<RefCell<Intf>>,
}

impl KeyPad {
    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self {
            reg: [0xF, 0xF],
            select: 0,
            intf,
        }
    }

    pub fn key_press(&mut self, key: GbKey) {
        match key {
            GbKey::Right => self.reg[1] &= 0b1110,
            GbKey::Left => self.reg[1] &= 0b1101,
            GbKey::Up => self.reg[1] &= 0b1011,
            GbKey::Down => self.reg[1] &= 0b0111,

            GbKey::A => self.reg[0] &= 0b1110,
            GbKey::B => self.reg[0] &= 0b1101,
            GbKey::Select => self.reg[0] &= 0b1011,
            GbKey::Start => self.reg[0] &= 0b0111,
        }
        self.intf
            .borrow_mut()
            .set_interrupt(InterruptSource::Keypad);
    }

    pub fn key_release(&mut self, key: GbKey) {
        match key {
            GbKey::Right => self.reg[1] |= !(0b1110),
            GbKey::Left => self.reg[1] |= !(0b1101),
            GbKey::Up => self.reg[1] |= !(0b1011),
            GbKey::Down => self.reg[1] |= !(0b0111),

            GbKey::A => self.reg[0] |= !(0b1110),
            GbKey::B => self.reg[0] |= !(0b1101),
            GbKey::Select => self.reg[0] |= !(0b1011),
            GbKey::Start => self.reg[0] |= !(0b0111),
        };
    }
}

impl MemoryBus for KeyPad {
    fn read_byte(&self, address: u16) -> u8 {
        assert_eq!(address, 0xFF00);
        if self.select.bit(4) {
            self.reg[0]
        } else if self.select.bit(5) {
            self.reg[1]
        } else {
            assert_eq!(self.select, 0);
            0
        }
    }

    // The only keypad write is to switch which keys are read (directions or actions).
    fn write_byte(&mut self, address: u16, b: u8) {
        assert_eq!(address, 0xFF00);
        self.select = b & 0b0011_0000;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_keypad() -> KeyPad {
        KeyPad::new(Rc::new(RefCell::new(Intf::new())))
    }

    #[test]
    fn direction_action_independence() {
        let mut keypad = new_keypad();

        // Action button registers
        keypad.key_press(GbKey::A);
        keypad.write_byte(0xFF00, 0x10);
        assert_eq!(keypad.read_byte(0xFF00), 0b1110);

        // Direction register unaffected
        keypad.write_byte(0xFF00, 0x20);
        assert_eq!(keypad.read_byte(0xFF00), 0b1111);
    }

    #[test]
    fn interrupt_triggered_on_key_press() {
        let intf = Rc::new(RefCell::new(Intf::new()));
        let mut keypad = KeyPad::new(intf.clone());
        assert_eq!(intf.borrow().read_byte(0xFF0F), 0);
        keypad.key_press(GbKey::Start);

        // Keypad interrupt flag (bit 4) should be set
        assert_eq!(intf.borrow().read_byte(0xFF0F) & 0b10000, 0b10000);
    }
}

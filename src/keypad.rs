use std::{rc::Rc, cell::RefCell};
use super::bit::Bit;
use super::intf::Intf;
use super::bus::MemoryBus;

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

pub enum Key {
    Right  = 0b0000_0001,
    Left   = 0b0000_0010,
    Up     = 0b0000_0100,
    Down   = 0b0000_1000,
    A      = 0b0001_0000,
    B      = 0b0010_0000,
    Select = 0b0100_0000,
    Start  = 0b1000_0000,
}

/*
The eight Game Boy action/direction buttons are arranged as a 2x4 matrix. 
Select either action or direction buttons by writing to this register, then read out the bits 0-3.
*/
pub struct KeyPad {
    reg:        u8,
    select:     u8,
    intf:       Rc<RefCell<Intf>>
}

impl KeyPad {

    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self {
            reg:    0xFF,
            select: 0,
            intf,
        }
    }

    pub fn key_press(&mut self, key: Key) {
        self.reg &= !(key as u8);
        // TODO interrupt.
    }

    pub fn key_release(&mut self, key: Key) {
        self.reg.set(key as usize);
    }
}

impl MemoryBus for KeyPad {

    fn read_byte(&self, _: u16) -> u8 {
        if self.select.bit(4) { return self.select | 0b1111 }         
        if self.select.bit(5) { return self.select | 0b1111 }
        self.select
    }

    fn write_byte(&mut self, _: u16, b: u8) {
        self.select = b;        
    }
}
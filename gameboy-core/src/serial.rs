use std::rc::Rc;
use std::cell::RefCell;

use super::bus::MemoryBus;
use super::intf::{Intf, InterruptSource};

// Serial is for gameboy multiplayer functionality.
// Since the emulator has no multiplayer it is used for testing puposes instead.
// This is because test roms often send results to the serial memory address.
pub type SerialCallback = Option<Box<dyn Fn(u8)>>;

pub struct Serial {
    // Before a transfer, it holds the next byte that will go out.
    data: u8,
    /*
    Bit 7 - Transfer Start Flag (0=No transfer is in progress or requested, 1=Transfer in progress, or requested)
    Bit 1 - Clock Speed (0=Normal, 1=Fast) ** CGB Mode Only **
    Bit 0 - Shift Clock (0=External Clock, 1=Internal Clock)
    */
    control: u8,

    callback: SerialCallback, 
    
    intf: Rc<RefCell<Intf>>
}

impl Serial {
    pub fn new(intf: Rc<RefCell<Intf>>, callback: SerialCallback) -> Self { 
        Self { intf, data: 0, control: 0, callback } 
    }
}

impl MemoryBus for Serial {
    
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0xFF01 => self.data = b,
            0xFF02 => {
                self.control = b;
                if b == 0x81 {
                    match &self.callback {
                        Some(callback) => {
                            (callback)(self.data);
                            self.data = b;
                            self.intf.borrow_mut().set_interrupt(InterruptSource::Serial);
                        },
                        None => {},
                    }
                }
            },
            _ => unreachable!(),
        }
    }
}


use std::cell::RefCell;
use std::rc::Rc;

use super::bit::Bit;
use super::bus::MemoryBus;
use super::intf::{Intf, InterruptSource};

#[derive(Default)]
pub struct Timer {
    // FF04 - Divider register (R/W).
    // Incremented at rate of 16_384 Hz.
    // Reset when write or stop instruction.
    divider: u8,
    
    // FF05 - Timer counter (R/W).
    // Incremented at clock freq specified by TAC.
    // When overflow, reset to value in TMA.
    counter: u8,
    
    // FF06 - Timer modulo (R/W).
    // Holds value to set TIMA when reset.
    modulo: u8,

    // FF07 - Timer control (R/W).
    /*  Bit  2   - Timer Enable
    Bits 1-0 - Input Clock Select
           00: CPU Clock / 1024 (DMG, SGB2, CGB Single Speed Mode:   4096 Hz, SGB1:   ~4194 Hz, CGB Double Speed Mode:   8192 Hz)
           01: CPU Clock / 16   (DMG, SGB2, CGB Single Speed Mode: 262144 Hz, SGB1: ~268400 Hz, CGB Double Speed Mode: 524288 Hz)
           10: CPU Clock / 64   (DMG, SGB2, CGB Single Speed Mode:  65536 Hz, SGB1:  ~67110 Hz, CGB Double Speed Mode: 131072 Hz)
           11: CPU Clock / 256  (DMG, SGB2, CGB Single Speed Mode:  16384 Hz, SGB1:  ~16780 Hz, CGB Double Speed Mode:  32768 Hz)
    */
    enable: bool,

    period:             u32,
    internal_divider:   u32,
    internal_counter:   u32,

    intf:   Rc<RefCell<Intf>>
}

impl MemoryBus for Timer {

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => {
                let mut b: u8 = 0;
                if self.enable { b.set(2) };
                match self.period {
                    1024 => b.set(0),
                    16   => b.set(1),
                    64   => b.set(2),
                    256  => b.set(3),
                    _ => panic!("timer period not supported (read) {:4X}", self.period)
                }
                b
            },
            _ => panic!("address for timer not supported: {:?}", addr)
        }
    }

    fn write_byte(&mut self, addr: u16, b: u8) {
        match addr {
            0xFF04 => { self.divider = b },
            0xFF05 => { self.counter = b },
            0xFF06 => { self.modulo = b },
            0xFF07 => {
                self.enable = b.bit(2);
                self.period = match b & 0b11 {
                    0 => 1024,
                    1 => 16, 
                    2 => 64,
                    3 => 256,
                    _ => panic!("timer period not supported (write): {:4X}", self.period) 
                }
            }
            _ => panic!("address for timer not supported: {:?}", addr)
        };
    }
}

impl Timer {
    
    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self {
            period: 256,
            intf, 
            ..Timer::default()
        }
    }

    pub fn update(&mut self, cycles: u32) {
        self.increment_div(cycles);
        
        if self.enable {
            self.internal_counter += cycles ;

            // TIMA updates at freq set by self.period.
            while self.internal_counter > self.period {
                self.counter = self.counter.wrapping_add(1);
            
                // Timer about to overflow, TIMA set to TMA + interrupt.
                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.intf.borrow_mut().set_interrupt(InterruptSource::Timer);
                }                

                self.internal_counter  -= self.period;
            }
        }
    }

    // The divider register increments at a fixed rate of 16,384 Hz. 
    // This means that we have to increment DIV’s value every 256 clock cycles (4,194,304 / 16,384 = 256).
    fn increment_div(&mut self, cycles: u32) {
        self.internal_divider += cycles;
        while self.internal_divider >= 256 {
            self.divider = self.divider.wrapping_add(1);
            self.internal_divider -= 256;
        }
    }
}

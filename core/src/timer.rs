use crate::{
    bit::Bit,
    bus::MemoryBus,
    clock::Clock,
    intf::{InterruptSource, Intf},
};
use std::cell::RefCell;
use std::rc::Rc;

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

    div_clock: Clock,
    mod_clock: Clock,

    intf: Rc<RefCell<Intf>>,
}

impl MemoryBus for Timer {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => {
                let mut b: u8 = 0;
                if self.enable {
                    b.set(2)
                };
                match self.mod_clock.period {
                    1024 => b.set(0),
                    16 => b.set(1),
                    64 => b.set(2),
                    256 => b.set(3),
                    _ => unreachable!(),
                }
                b
            }
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, b: u8) {
        match address {
            0xFF04 => self.divider = b,
            0xFF05 => self.counter = b,
            0xFF06 => self.modulo = b,
            0xFF07 => {
                self.enable = b.bit(2);
                self.mod_clock.period = match b & 0b11 {
                    0 => 1024,
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
    }
}

impl Timer {
    pub fn new(intf: Rc<RefCell<Intf>>) -> Self {
        Self {
            div_clock: Clock::new(256),
            mod_clock: Clock::new(1024),
            intf,
            ..Timer::default()
        }
    }

    pub fn update(&mut self, cycles: u32) {
        self.divider = self.divider.wrapping_add(self.div_clock.tick(cycles) as u8);

        if self.enable {
            for _ in 0..self.mod_clock.tick(cycles) {
                self.counter = self.counter.wrapping_add(1);

                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.intf.borrow_mut().set_interrupt(InterruptSource::Timer);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_timer() -> Timer {
        Timer::new(Rc::new(RefCell::new(Intf::new())))
    }

    #[test]
    fn divider_increments() {
        let mut timer = new_timer();
        // Divider clock has period 256, so after 256 cycles divider should increment
        timer.update(256);
        assert_eq!(timer.read_byte(0xFF04), 1);

        timer.update(512); // 2 periods
        assert_eq!(timer.read_byte(0xFF04), 3);
    }

    #[test]
    fn counter_disabled() {
        let mut timer = new_timer();
        timer.write_byte(0xFF05, 0); // Set counter to 0

        // Counter should not change
        timer.update(10000);
        assert_eq!(timer.read_byte(0xFF05), 0);
    }

    #[test]
    fn counter_enabled() {
        let mut timer = new_timer();
        // Enable timer with period 16
        timer.write_byte(0xFF07, 0b101);

        timer.update(16);
        assert_eq!(timer.read_byte(0xFF05), 1);

        timer.update(32);
        assert_eq!(timer.read_byte(0xFF05), 3);
    }

    #[test]
    fn counter_overflow_reset() {
        let mut timer = new_timer();
        timer.write_byte(0xFF06, 0x50); // Set modulo to 0x50
        timer.write_byte(0xFF05, 0xFF); // Set counter near overflow
        timer.write_byte(0xFF07, 0b101); // Enable with fast clock

        // After 16 cycles, counter overflows: 0xFF + 1 = 0x00, then reset back to 0x50
        timer.update(16);
        assert_eq!(timer.read_byte(0xFF05), 0x50);
    }

    #[test]
    fn overflow_interrupt() {
        let intf = Rc::new(RefCell::new(Intf::new()));
        let mut timer = Timer::new(intf.clone());

        // Trigger overflow
        timer.write_byte(0xFF05, 0xFF);
        timer.write_byte(0xFF07, 0b101);
        assert_eq!(intf.borrow().read_byte(0xFF0F), 0);
        timer.update(16);

        // Timer interrupt flag set
        assert_eq!(intf.borrow().read_byte(0xFF0F) & 0b100, 0b100);
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use super::registers::Registers;
use super::memory::MemoryBus;

pub struct CPU {
    pub regs: Registers,
    pub mem: Rc<RefCell<dyn MemoryBus>>, // Provides access to MemUnit.
    pub halted: bool,
}
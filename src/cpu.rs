use std::path::{self, Path};

use crate::memory::Memory;
use super::registers::Registers;

pub struct CPU {
    pub regs: Registers,
    pub memory: Memory, 
    pub halted: bool,
}

impl CPU {
    
    pub fn new(path: &Path) -> CPU {
        CPU {
            regs: Registers::new(),
            memory: Memory::new(path),
            halted: false,
        }
    }

}

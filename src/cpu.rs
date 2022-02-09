use super::registers::Registers;
use super::memory::Memory;

// CPU freq = 4.194304 MHz
const CPU_CLOCK: int = 4_194_304;

pub struct CPU {
    registers:  Registers,
    pub memory: Memory,
}

impl CPU {

    // Executes a single instruction, returning the number of cycles.
    fn execute(&mut self, opcode: u8) -> u8{

        match opcode {
            0x00 => self.registers.pc += 1,
            
            // 8-bit loads.

        }
    }
    
}

use std::path::Path;
use super::bus::MemoryBus;
use super::memory::Memory;

mod registers;
use registers::Registers;

mod opcodes;

pub struct CPU {
    regs:       Registers,
    pub mem:    Memory,
    /*
    halt is an instruction that pauses the CPU (during which less power is consumed) when executed. 
    The CPU wakes up as soon as an interrupt is pending, that is, when the bitwise AND of IE and IF 
    is non-zero.
    */
    halted:     bool,
    // Flag for enabling interrupts in the IE register.
    // Not accessble via i/o address, only through instructions.
    ei:         bool,
}

impl CPU {
    
    pub fn new(path: &Path, cpu_test: bool) -> Self {
        Self {
            regs: Registers::new(),
            mem: Memory::new(path, cpu_test),
            halted: false,
            ei: false,
        }
    }

    // Reads next byte at stack pointer, increments pointer.
    fn nxt_byte(&mut self) -> u8 {
        let b = self.mem.read_byte(self.regs.pc);
        self.regs.pc += 1;
        b
    }

    // Reads next 2 bytes (word) at stack pointer, incrementing twice.
    fn nxt_word(&mut self) -> u16 {
        let w = self.mem.read_word(self.regs.pc);
        self.regs.pc += 2;
        w
    }

    fn step(&mut self) -> u32 {
        let interrupt_cycles = self.check_interrupts();
        if interrupt_cycles != 0 { interrupt_cycles }
        else if self.halted { 4 }
        else {
            let opcode = self.nxt_byte(); 
            self.execute(opcode) 
        }
    }

    /*
        Any set bits in the IF register are only requesting an interrupt. 
        The actual execution of the interrupt handler happens only if both the 
        IME flag and the corresponding bit in the IE register are set; otherwise 
        the interrupt “waits” until both IME and IE allow it to be serviced.
    */
    fn check_interrupts(&mut self) -> u32 {
        // Neither halted not master interrupt flag set.
        if !self.halted && !self.ei { return 0 }
        // Check for requests from interrupt registers.
        let intf = self.mem.read_byte(0xFFFF);
        let inte = self.mem.read_byte(0xFF0F);
        
        if inte & intf == 0 { return 0 }
        // Halt is reset in case of interrupt.
        self.halted = false;

        if !self.ei { return 0 }
        // Prevent further interrupts until program re-enables them.
        self.ei = false;
        // Handle the interrupt.
        self.handle_interrupt(intf);
        16
    }

    fn handle_interrupt(&mut self, mut int: u8) {
        // The priorities follow the order of the bits in the IE and IF registers.
        let n = int.trailing_zeros();
        int &= !(1 << n);
        // Write back to register.
        self.mem.write_byte(0xFF0F, int);
        // Push pc on stack and jump to address of interrupt handler.
        self.stack_push(self.regs.pc);
        self.regs.pc = 0x0040 | ((n as u16) << 3);
    }

}

#[cfg(test)]
mod test {
    // Load and execute a cpu instruction test cartridge.
    
    use std::path::Path;
    use super::CPU;
    

    #[test]
    fn cpu_instructions() {
        let rom_path = Path::new("./test_roms/drMario.gb");
        assert!(rom_path.exists());
        let mut cpu = CPU::new(rom_path, true);
        



    }
}
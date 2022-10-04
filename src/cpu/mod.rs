use std::path::Path;

use super::cartridge::Cartridge;
use super::bus::MemoryBus;
use super::memory::Memory;
use super::serial::SerialCallback;

mod registers;
use registers::Registers;

mod opcodes;

pub struct CPU {
    regs:       Registers,
    pub mem:    Memory,
    
    /* Halt is an instruction that pauses the CPU (during which less power is consumed) when executed. 
    The CPU wakes up as soon as an interrupt is pending, that is, when the bitwise AND of IE and IF 
    is non-zero. */
    halted:     bool,
    
    // Flag for enabling interrupts in the IE register.
    // Not accessble via i/o address, only through instructions.
    ime:         bool,

    disable_interrupt: u8,
    enable_interrupt:  u8,    
}

impl CPU {
    
    pub fn new(cartridge: Box<dyn Cartridge>, callback: SerialCallback) -> Self {
        Self {
            regs:                 Registers::new(),
            mem:                  Memory::new(cartridge, callback),
            halted:               false,
            ime:                  true,
            disable_interrupt:    0,
            enable_interrupt:     0,
        }
    }

    // Reads next byte at stack pointer, increments pointer.
    fn next_byte(&mut self) -> u8 {
        let byte = self.mem.read_byte(self.regs.pc);
        self.regs.pc += 1;
        byte
    }

    // Reads next 2 bytes (word) at stack pointer, incrementing twice.
    fn next_word(&mut self) -> u16 {
        let word = self.mem.read_word(self.regs.pc);
        self.regs.pc += 2;
        word
    }

    fn stack_push(&mut self, val: u16) {
        self.regs.sp -= 2;
        self.mem.write_word(self.regs.sp, val);
    }

    fn stack_pop(&mut self) -> u16 {
        let val = self.mem.read_word(self.regs.sp);
        self.regs.sp += 2;
        val
    }
}

impl CPU {

    pub fn step(&mut self) -> u32 {
        self.update_ime();

        let interrupt_cycles = self.check_interrupts();
        if interrupt_cycles != 0 { 
            return interrupt_cycles 
        }
        // If halted simulate nop instruction.
        if self.halted { 
            4 
        } else {
            let opcode = self.next_byte();
            self.execute(opcode) 
        }
    }

    /* Any set bits in the IF register are only requesting an interrupt. 
    The actual execution of the interrupt handler happens only if both the 
    IME flag and the corresponding bit in the IE register are set; otherwise 
    the interrupt “waits” until both IME and IE allow it to be serviced. */
    fn check_interrupts(&mut self) -> u32 {
        
        // Neither halted not master interrupt flag set.
        if !self.halted && !self.ime { return 0 }

        // Check for requests from interrupt registers.
        let intf = self.mem.read_byte(0xFFFF);
        let inte = self.mem.read_byte(0xFF0F);
        
        let pending_interrupts = intf & inte;
        if pending_interrupts == 0 { return 0 }

        // Halt is reset in case of interrupt.
        self.halted = false;
        // Prevent further interrupts until program re-enables them.
        if !self.ime { return 0 }
        self.ime = false;

        // Handle the interrupt.
        self.handle_interrupt(pending_interrupts);
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

    // Enabling and disabling of interrupts is delayed by one instruction.
    fn update_ime(&mut self) {
        self.disable_interrupt = match self.disable_interrupt {
            2 => 1,
            1 => { self.ime = false; 0 },
            _ => 0,
        };
        self.enable_interrupt = match self.enable_interrupt {
            2 => 1,
            1 => { self.ime = true; 0 },
            _ => 0, 
        };
    }
}

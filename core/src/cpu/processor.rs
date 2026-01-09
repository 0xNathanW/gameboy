use super::registers::Registers;
use crate::{bus::MemoryBus, memory::Memory};

pub struct Cpu {
    pub regs: Registers,
    pub halted: bool,
    pub ime: bool,
    pub disable_interrupt: u8,
    pub enable_interrupt: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            halted: false,
            ime: true,
            disable_interrupt: 0,
            enable_interrupt: 0,
        }
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.halted = false;
        self.ime = true;
        self.disable_interrupt = 0;
        self.enable_interrupt = 0;
    }

    pub fn next_byte(&mut self, mem: &Memory) -> u8 {
        let byte = mem.read_byte(self.regs.pc);
        self.regs.pc += 1;
        byte
    }

    pub fn next_word(&mut self, mem: &Memory) -> u16 {
        let word = mem.read_word(self.regs.pc);
        self.regs.pc += 2;
        word
    }

    pub fn stack_push(&mut self, mem: &mut Memory, val: u16) {
        self.regs.sp -= 2;
        mem.write_word(self.regs.sp, val);
    }

    pub fn stack_pop(&mut self, mem: &mut Memory) -> u16 {
        let val = mem.read_word(self.regs.sp);
        self.regs.sp += 2;
        val
    }

    // Performs a singular instruction or interrupt event.
    pub fn tick(&mut self, mem: &mut Memory) -> u32 {
        self.update_ime();

        let interrupt_cycles = self.check_interrupts(mem);
        if interrupt_cycles != 0 {
            return interrupt_cycles;
        }
        // If halted simulate nop instruction.
        if self.halted {
            4
        } else {
            let opcode = self.next_byte(mem);
            self.execute(opcode, mem)
        }
    }

    fn check_interrupts(&mut self, mem: &mut Memory) -> u32 {
        if !self.halted && !self.ime {
            return 0;
        }

        let intf = mem.read_byte(0xFFFF);
        let inte = mem.read_byte(0xFF0F);

        let pending_interrupts = intf & inte;
        if pending_interrupts == 0 {
            return 0;
        }

        self.halted = false;
        if !self.ime {
            return 0;
        }
        self.ime = false;

        self.handle_interrupt(mem, pending_interrupts);
        16
    }

    fn handle_interrupt(&mut self, mem: &mut Memory, mut int: u8) {
        let n = int.trailing_zeros();
        int &= !(1 << n);

        mem.write_byte(0xFF0F, int);

        self.stack_push(mem, self.regs.pc);
        self.regs.pc = 0x0040 | ((n as u16) << 3);
    }

    fn update_ime(&mut self) {
        self.disable_interrupt = match self.disable_interrupt {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };
        self.enable_interrupt = match self.enable_interrupt {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    #[cfg(feature = "inspect")]
    pub fn state(&self) -> crate::inspect::CpuState {
        crate::inspect::CpuState {
            a: self.regs.a,
            b: self.regs.b,
            c: self.regs.c,
            d: self.regs.d,
            e: self.regs.e,
            h: self.regs.h,
            l: self.regs.l,
            f: self.regs.flags(),
            sp: self.regs.sp,
            pc: self.regs.pc,
            ime: self.ime,
            halted: self.halted,
        }
    }
}

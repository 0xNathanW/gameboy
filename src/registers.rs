use std::convert::From;

// CPU registers.
// Registers af, bc, de and hl can be combined 
// to form a 16-bit register pair.
pub struct Registers {
    pub a:  u8,      // Accumulator.
    f:      u8,      // Flags.
    pub b:  u8,
    pub c:  u8,
    pub d:  u8,
    pub e:  u8,
    pub h:  u8,
    pub l:  u8,
    pub sp: u16,    // Stack pointer.
    pub pc: u16,    // Program counter.
}

enum Flag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
}

// Flags implemented in bitmask.
const ZERO_FLAG: u8         = 0b10000000;  
const SUBTRACT_FLAG: u8     = 0b01000000;
const HALF_CARRY_FLAG: u8   = 0b00100000;
const CARRY_FLAG: u8        = 0b00010000;

impl Registers {

    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0x100,
        }
    }

    // Getters.
    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }
    
    fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero =>       self.f & ZERO_FLAG != 0,
            Flag::Subtract =>   self.f & SUBTRACT_FLAG != 0,
            Flag::HalfCarry =>  self.f & HALF_CARRY_FLAG != 0,
            Flag::Carry =>      self.f & CARRY_FLAG != 0,
        }
    }

    // Setters.
    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Zero =>       self.f = if value { self.f | ZERO_FLAG } else { self.f & !ZERO_FLAG },
            Flag::Subtract =>   self.f = if value { self.f | SUBTRACT_FLAG } else { self.f & !SUBTRACT_FLAG },
            Flag::HalfCarry =>  self.f = if value { self.f | HALF_CARRY_FLAG } else { self.f & !HALF_CARRY_FLAG },
            Flag::Carry =>      self.f = if value { self.f | CARRY_FLAG } else { self.f & !CARRY_FLAG },
        }
    }
}

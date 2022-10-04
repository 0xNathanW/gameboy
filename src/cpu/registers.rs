use std::fmt::Debug;

// CPU registers.
// Registers af, bc, de and hl can be combined 
// to form a 16-bit register pair.
#[derive(Default)]
pub struct Registers {
    pub a:  u8,      
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

pub enum Flag { Z, N, H, C }

// Flags implemented in bitmask.
const ZERO_FLAG: u8         = 0b1000_0000;  
const SUBTRACT_FLAG: u8     = 0b0100_0000;
const HALF_CARRY_FLAG: u8   = 0b0010_0000;
const CARRY_FLAG: u8        = 0b0001_0000;

impl Registers {

    pub fn new() -> Self {
        let mut reg = Self {
            sp: 0xFFFE,
            pc: 0x100,
            ..Default::default()
        };
        // Initial register values.
        reg.set_af(0x01B0);
        reg.set_bc(0x0013);
        reg.set_de(0x00D8);
        reg.set_hl(0x014D);
        reg
    }

    // Getters for 16-bit registers.
    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }
    
    // Setters for 16-bit registers.
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xF0) as u8;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z =>  self.f & ZERO_FLAG != 0,
            Flag::N =>  self.f & SUBTRACT_FLAG != 0,
            Flag::H =>  self.f & HALF_CARRY_FLAG != 0,
            Flag::C =>  self.f & CARRY_FLAG != 0,
        }
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Z =>  self.f = if value { self.f | ZERO_FLAG } else { self.f & !ZERO_FLAG },
            Flag::N =>  self.f = if value { self.f | SUBTRACT_FLAG } else { self.f & !SUBTRACT_FLAG },
            Flag::H =>  self.f = if value { self.f | HALF_CARRY_FLAG } else { self.f & !HALF_CARRY_FLAG },
            Flag::C =>  self.f = if value { self.f | CARRY_FLAG } else { self.f & !CARRY_FLAG },
        }
    }
}

impl Debug for Registers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Registers {{
                af: {:#06X},
                bc: {:#06X},
                de: {:#06X},
                hl: {:#06X},
                flags - Z: {}, N: {}, H: {}, C: {},
                sp: {:#06X},
                pc: {:#06X},
            }}", 
            self.get_af(), self.get_bc(), self.get_de(), self.get_hl(),
            self.get_flag(Flag::Z), self.get_flag(Flag::N), 
            self.get_flag(Flag::H), self.get_flag(Flag::C),
            self.sp, self.pc
        )}
}

#[cfg(test)]
mod test {

    use super::Registers;
    use super::Flag::{Z, H, N};

    #[test]
    fn new() {
        let reg = Registers::new();
        assert_eq!(reg.get_af(), 0x01B0);
        assert_eq!(reg.get_bc(), 0x0013);
        assert_eq!(reg.get_de(), 0x00D8);
        assert_eq!(reg.get_hl(), 0x014D);
    }

    #[test]
    fn combined_registers() {
        let mut reg = Registers::new();
        
        reg.a = 0b00000001;
        reg.f = 0b00000010;
        assert_eq!(reg.get_af(), 0b00000001_00000010);

        reg.b = 0b00110001;
        reg.c = 0b11000111;
        assert_eq!(reg.get_bc(), 0b00110001_11000111);

        reg.set_af(15786);
        assert_eq!(reg.a, 0b00111101);
        assert_eq!(reg.f, 0b10100000);
    }

    #[test]
    fn flags() {
        let mut reg = Registers::new();

        reg.set_flag(Z, true);
        reg.set_flag(H, true);
        reg.set_flag(N, false);

        assert!(reg.get_flag(Z));
        assert!(reg.get_flag(H));
        assert!(!reg.get_flag(N));
    }
}


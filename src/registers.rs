
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

pub enum Flag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
}

// Flags implemented in bitmask.
const ZERO_FLAG: u8         = 0b1000_0000;  
const SUBTRACT_FLAG: u8     = 0b0100_0000;
const HALF_CARRY_FLAG: u8   = 0b0010_0000;
const CARRY_FLAG: u8        = 0b0001_0000;

impl Registers {

    pub fn new() -> Self {
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
    
    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero =>       self.f & ZERO_FLAG != 0,
            Flag::Subtract =>   self.f & SUBTRACT_FLAG != 0,
            Flag::HalfCarry =>  self.f & HALF_CARRY_FLAG != 0,
            Flag::Carry =>      self.f & CARRY_FLAG != 0,
        }
    }

    // Setters.
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Zero =>       self.f = if value { self.f | ZERO_FLAG } else { self.f & !ZERO_FLAG },
            Flag::Subtract =>   self.f = if value { self.f | SUBTRACT_FLAG } else { self.f & !SUBTRACT_FLAG },
            Flag::HalfCarry =>  self.f = if value { self.f | HALF_CARRY_FLAG } else { self.f & !HALF_CARRY_FLAG },
            Flag::Carry =>      self.f = if value { self.f | CARRY_FLAG } else { self.f & !CARRY_FLAG },
        }
    }
}

#[cfg(test)]
mod test {

    use super::Registers;

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
        assert_eq!(reg.get_af(), 0b0011110110101010)
    }

    fn flags() {
        let mut reg = Registers::new();
    }

}
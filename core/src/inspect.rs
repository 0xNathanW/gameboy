// Read-only inspection interface for debugging Game Boy internals.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuState {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub halted: bool,
}

impl CpuState {
    pub fn flag_z(&self) -> bool {
        self.f & 0b1000_0000 != 0
    }

    pub fn flag_n(&self) -> bool {
        self.f & 0b0100_0000 != 0
    }

    pub fn flag_h(&self) -> bool {
        self.f & 0b0010_0000 != 0
    }

    pub fn flag_c(&self) -> bool {
        self.f & 0b0001_0000 != 0
    }

    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuMode {
    HBlank,
    VBlank,
    OamScan,
    Drawing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuState {
    pub ly: u8,
    pub lyc: u8,
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub window_x: u8,
    pub window_y: u8,
    pub mode: GpuMode,
    pub lcd_enable: bool,
    pub window_enable: bool,
    pub sprite_enable: bool,
    pub bg_enable: bool,
    pub sprite_size: u8,
}

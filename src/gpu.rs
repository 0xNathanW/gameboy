use super::bus::MemoryBus;

const VRAM_SIZE: usize = 16_384;
const OAM_SIZE: usize = 160;

pub struct GPU {
    // Tile data is stored in VRAM in the memory area at $8000-$97FF.
    vram: [u8; VRAM_SIZE],
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F.
    oam: [u8; OAM_SIZE],
    
    // LCDC (LCD control) (R/W) - FF40
    // Main control register, its bits toggle what elements are displayed and how.
    /*  
    |Bit| Name                          | Usage notes              |
    | 7 | LCD and PPU enable            | 0=Off, 1=On              |
    | 6 | Window tile map area          | 0=9800-9BFF, 1=9C00-9FFF |
    | 5 | Window enable                 | 0=Off, 1=On              |
    | 4 | BG and Window tile data area  | 0=8800-97FF, 1=8000-8FFF |
    | 3 | BG tile map area              | 0=9800-9BFF, 1=9C00-9FFF |
    | 2 | OBJ size                      | 0=8x8, 1=8x16            |
    | 1 | OBJ enable                    | 0=Off, 1=On              |
    | 0 | BG and Window enable/priority | 0=Off, 1=On              |
    */
    lcd_enable:         bool,
    bg_tilemap:         bool,

    win_enable:         bool,
    win_tilemap:        bool,
    
    bg_win_enable:      bool, 
    bg_win_tilemap:     bool,
    
    obj_size:           bool,
    obj_enable:         bool, 
    
    // STAT (LCD status) (R/W) - FF41
    /*
    |Bit| Name                                 | Usage notes          |
    | 6 | LYC=LY STAT Interrupt source         | 0=Off, 1=On          |
    | 5 | Mode 2 OAM STAT Interrupt source     | 0=Off, 1=On          |
    | 4 | Mode 1 VBlank STAT Interrupt source  | 0=Off, 1=On          |
    | 3 | Mode 0 HBlank STAT Interrupt source  | 0=Off, 1=On          |
    | 2 | LYC=LY Flag                          | 0=Different, 1=Equal |
    |1-0| Mode Flag                            | Mode 0-3             |
        > 0 - HBlank, > 1 - VBlank, > 2 - Searching OAM, > 3 Transfer data to LCD
    */
    lyc_int_src:    bool,
    m2_int_src:     bool, 
    m1_int_src:     bool,
    m0_int_src:     bool,
    lyc_eq_ly:      bool,
    mode:           u8,

    // 0xFF42 - SCY (scroll Y) | 0xFF43 - SCX (scroll X)
    scy: u8,
    scx: u8,
    // 0xFF44 - LY (LCD Y coord) | 0xFF45 - LYC (LY compare) 
    ly: u8,
    lyc: u8,
    // 0xFF4A - WY (window y position) | 0xFF4B - WX (window x position + 7)
    wy: u8,
    wx: u8,

    // LCD monochrome palettes.
    // If want to implement Colour Gameboy there are extra palettes.
    // 0xFF47 - BGP (BG palette data)
    bgp: u8,
    // 0xFF48 - OBP0 (OBJ palette 0 data)
    obp0: u8,
    // 0xFF49 - OBP1 (OBJ palette 1 data)
    obp1: u8,
}

impl GPU {
    
    pub fn new() -> Self {
        Self { 
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcd_enable: false, bg_tilemap: false, 
            win_enable: false, win_tilemap: false,
            bg_win_enable: false, bg_win_tilemap: false,
            obj_size: false, obj_enable: false,
            lyc_int_src: false,
            m2_int_src: false, m1_int_src: false, m0_int_src: false,
            lyc_eq_ly: false,
            mode: 0,
            scy: 0, scx: 0,
            ly: 0, lyc: 0,
            wy: 0, wx: 0,
            bgp: 0, obp0: 0, obp1: 0,
        }
    }
}

impl MemoryBus for GPU {

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000 ..= 0x97FF => self.vram[addr as usize - 0x8000],

        }
    }

    fn write_byte(&mut self, addr: u16, b: u8) {}

}
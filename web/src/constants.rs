// Timing constants
// Approximate frame time in milliseconds for ~60 FPS (1000ms / 60 = 16.67ms)
pub const FRAME_TIME_MS: u32 = 16;

// CPU cycles per frame: 4,194,304 Hz / ~60 FPS = 69,905 cycles
pub const CYCLES_PER_FRAME: u32 = 69_905;

// Clock speed settings (Game Boy runs at 4.194304 MHz)
pub const GB_CLOCK_FREQ: f32 = 4.194304;

// Display constants
pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
pub const MIN_SCALE: u32 = 1;
pub const MAX_SCALE: u32 = 8;

pub struct Palette {
    pub name: &'static str,
    pub colours: [u32; 4],
}

pub const PALETTES: &[Palette] = &[
    Palette {
        name: "Classic",
        colours: [0xe0f8d0, 0x88c070, 0x346856, 0x081820],
    },
    Palette {
        name: "2Bit Demichrome",
        colours: [0xe9efec, 0xa0a08b, 0x555568, 0x211e20],
    },
    Palette {
        name: "Ice Cream",
        colours: [0xfff6d3, 0xf9a875, 0xeb6b6f, 0x7c3f58],
    },
    Palette {
        name: "Bicycle",
        colours: [0xf0f0f0, 0x8f9bf6, 0xab4646, 0x161616],
    },
    Palette {
        name: "Lopsec",
        colours: [0xc7c6c6, 0x7c6d80, 0x382843, 0x000000],
    },
    Palette {
        name: "Autumn Chill",
        colours: [0xdad3af, 0xd58863, 0xc23a73, 0x2c1e74],
    },
    Palette {
        name: "Red Dead",
        colours: [0xfffcfe, 0xff0015, 0x860020, 0x11070a],
    },
    Palette {
        name: "Blue Dream",
        colours: [0xecf2cb, 0x98d8b1, 0x4b849a, 0x1f285d],
    },
    Palette {
        name: "Lollipop",
        colours: [0xe6f2ef, 0xf783b0, 0x3f6d9e, 0x151640],
    },
    Palette {
        name: "Soviet",
        colours: [0xe8d6c0, 0x92938d, 0xa1281c, 0x000000],
    },
];

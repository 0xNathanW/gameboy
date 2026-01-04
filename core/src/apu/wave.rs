use super::components::{wave_period, FrequencyTimer, LengthCounter};

// Volume shift amounts for wave channel
const VOLUME_SHIFT: [u8; 4] = [4, 0, 1, 2]; // 0%, 100%, 50%, 25%

pub struct WaveChannel {
    // Registers
    nrx0: u8, // DAC power
    nrx1: u8, // Length
    nrx2: u8, // Volume
    nrx3: u8, // Frequency low
    nrx4: u8, // Trigger + length enable + frequency high

    // Wave RAM (32 4-bit samples in 16 bytes)
    pub wave_ram: [u8; 16],

    // Components
    pub length: LengthCounter,

    // State
    pub enabled: bool,
    timer: FrequencyTimer,
    position: usize, // 0-31 sample position
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            nrx0: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
            wave_ram: [0; 16],
            length: LengthCounter::new(256),
            enabled: false,
            timer: FrequencyTimer::new(8192),
            position: 0,
        }
    }

    fn frequency(&self) -> u16 {
        u16::from(self.nrx4 & 0x07) << 8 | u16::from(self.nrx3)
    }

    fn dac_enabled(&self) -> bool {
        self.nrx0 & 0x80 != 0
    }

    fn volume_shift(&self) -> u8 {
        VOLUME_SHIFT[((self.nrx2 >> 5) & 0x03) as usize]
    }

    // Tick the channel, returns number of samples advanced
    pub fn tick(&mut self, cycles: u32) -> u32 {
        let clocks = self.timer.tick(cycles);
        for _ in 0..clocks {
            self.position = (self.position + 1) % 32;
        }
        clocks
    }

    // Get current output amplitude
    pub fn output(&self) -> i32 {
        if !self.enabled || !self.dac_enabled() {
            return 0;
        }

        // Get 4-bit sample from wave RAM
        let byte = self.wave_ram[self.position / 2];
        let sample = if self.position % 2 == 0 {
            byte >> 4
        } else {
            byte & 0x0F
        };

        let shifted = sample >> self.volume_shift();
        // Center around 0 (wave samples are 0-15, center at 7.5)
        i32::from(shifted) - i32::from(8 >> self.volume_shift())
    }

    pub fn clock_length(&mut self) {
        if self.length.clock() {
            self.enabled = false;
        }
    }

    pub fn read_register(&self, reg: u8) -> u8 {
        match reg {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.nrx2,
            3 => self.nrx3,
            4 => self.nrx4,
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => {
                self.nrx0 = value;
                if !self.dac_enabled() {
                    self.enabled = false;
                }
            }
            1 => {
                self.nrx1 = value;
                self.length.load_full(u16::from(value));
            }
            2 => {
                self.nrx2 = value;
            }
            3 => {
                self.nrx3 = value;
                self.timer.period = wave_period(self.frequency());
            }
            4 => {
                self.nrx4 = value;
                self.length.enabled = value & 0x40 != 0;
                self.timer.period = wave_period(self.frequency());

                // Trigger
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    pub fn read_wave_ram(&self, index: usize) -> u8 {
        self.wave_ram[index & 0x0F]
    }

    pub fn write_wave_ram(&mut self, index: usize, value: u8) {
        self.wave_ram[index & 0x0F] = value;
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled();
        self.length.trigger();
        self.position = 0;
        self.timer.period = wave_period(self.frequency());
        self.timer.reload();
    }

    pub fn power_off(&mut self) {
        self.nrx0 = 0;
        self.nrx1 = 0;
        self.nrx2 = 0;
        self.nrx3 = 0;
        self.nrx4 = 0;
        self.enabled = false;
        self.length = LengthCounter::new(256);
        // Wave RAM is NOT cleared on power off
    }
}

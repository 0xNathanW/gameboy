use super::components::{noise_period, Envelope, FrequencyTimer, LengthCounter, Lfsr};

pub struct NoiseChannel {
    // Registers
    nrx0: u8, // Unused
    nrx1: u8, // Length
    nrx2: u8, // Envelope
    nrx3: u8, // Clock shift + width mode + divider
    nrx4: u8, // Trigger + length enable

    // Components
    pub length: LengthCounter,
    pub envelope: Envelope,
    lfsr: Lfsr,

    // State
    pub enabled: bool,
    timer: FrequencyTimer,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            nrx0: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
            length: LengthCounter::new(64),
            envelope: Envelope::new(),
            lfsr: Lfsr::new(),
            enabled: false,
            timer: FrequencyTimer::new(8),
        }
    }

    fn clock_shift(&self) -> u8 {
        self.nrx3 >> 4
    }

    fn width_mode(&self) -> bool {
        self.nrx3 & 0x08 != 0
    }

    fn divider_code(&self) -> u8 {
        self.nrx3 & 0x07
    }

    fn dac_enabled(&self) -> bool {
        self.envelope.dac_enabled()
    }

    // Tick the channel, returns number of LFSR clocks
    pub fn tick(&mut self, cycles: u32) -> u32 {
        self.timer.tick(cycles)
    }

    // Clock the LFSR once
    pub fn clock_lfsr(&mut self) {
        self.lfsr.clock();
    }

    // Get current output amplitude
    pub fn output(&self) -> i32 {
        if !self.enabled || !self.dac_enabled() || self.envelope.volume == 0 {
            return 0;
        }

        // LFSR output determines sign
        if self.lfsr.output {
            i32::from(self.envelope.volume)
        } else {
            i32::from(self.envelope.volume) * -1
        }
    }

    pub fn clock_length(&mut self) {
        if self.length.clock() {
            self.enabled = false;
        }
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub fn read_register(&self, reg: u8) -> u8 {
        match reg {
            0 => self.nrx0,
            1 => self.nrx1,
            2 => self.envelope.read_register(),
            3 => self.nrx3,
            4 => self.nrx4,
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => {
                self.nrx0 = value;
            }
            1 => {
                self.nrx1 = value;
                self.length.load(value & 0x3F);
            }
            2 => {
                self.nrx2 = value;
                self.envelope.write_register(value);
                if !self.dac_enabled() {
                    self.enabled = false;
                }
            }
            3 => {
                self.nrx3 = value;
                self.lfsr.set_width_mode(self.width_mode());
                self.timer.period = noise_period(self.divider_code(), self.clock_shift());
            }
            4 => {
                self.nrx4 = value;
                self.length.enabled = value & 0x40 != 0;

                // Trigger
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled();
        self.length.trigger();
        self.envelope.trigger();
        self.lfsr.trigger();
        self.timer.period = noise_period(self.divider_code(), self.clock_shift());
        self.timer.reload();
    }

    pub fn power_off(&mut self) {
        self.nrx0 = 0;
        self.nrx1 = 0;
        self.nrx2 = 0;
        self.nrx3 = 0;
        self.nrx4 = 0;
        self.enabled = false;
        self.envelope = Envelope::new();
        self.length = LengthCounter::new(64);
        self.lfsr = Lfsr::new();
    }
}

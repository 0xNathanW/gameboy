use super::components::{square_period, Envelope, FrequencyTimer, LengthCounter, Sweep};

// Duty cycle waveforms (8 steps each)
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

pub struct SquareChannel {
    // Registers
    pub nrx0: u8, // NR10 for ch1, unused for ch2
    nrx1: u8,     // Duty + length
    nrx2: u8,     // Envelope
    nrx3: u8,     // Frequency low
    nrx4: u8,     // Trigger + length enable + frequency high

    // Components
    pub length: LengthCounter,
    pub envelope: Envelope,
    pub sweep: Option<Sweep>,

    // State
    pub enabled: bool,
    timer: FrequencyTimer,
    duty_position: u8,
}

impl SquareChannel {
    pub fn new(has_sweep: bool) -> Self {
        Self {
            nrx0: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
            length: LengthCounter::new(64),
            envelope: Envelope::new(),
            sweep: if has_sweep { Some(Sweep::new()) } else { None },
            enabled: false,
            timer: FrequencyTimer::new(8192),
            duty_position: 0,
        }
    }

    fn frequency(&self) -> u16 {
        u16::from(self.nrx4 & 0x07) << 8 | u16::from(self.nrx3)
    }

    fn set_frequency(&mut self, freq: u16) {
        self.nrx3 = freq as u8;
        self.nrx4 = (self.nrx4 & 0xF8) | ((freq >> 8) as u8 & 0x07);
    }

    fn duty(&self) -> usize {
        (self.nrx1 >> 6) as usize
    }

    fn dac_enabled(&self) -> bool {
        self.envelope.dac_enabled()
    }

    // Tick the channel, returns number of waveform steps
    pub fn tick(&mut self, cycles: u32) -> u32 {
        let clocks = self.timer.tick(cycles);
        for _ in 0..clocks {
            self.duty_position = (self.duty_position + 1) % 8;
        }
        clocks
    }

    // Get current output amplitude
    pub fn output(&self) -> i32 {
        if !self.enabled || !self.dac_enabled() {
            return 0;
        }

        let duty_output = DUTY_TABLE[self.duty()][self.duty_position as usize];
        if duty_output != 0 {
            i32::from(self.envelope.volume)
        } else {
            -i32::from(self.envelope.volume)
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

    pub fn clock_sweep(&mut self) {
        let (new_freq, disabled) = if let Some(ref mut sweep) = self.sweep {
            (sweep.clock(), sweep.channel_disabled)
        } else {
            (None, false)
        };

        if let Some(freq) = new_freq {
            self.set_frequency(freq);
            self.timer.period = square_period(freq);
        }
        if disabled {
            self.enabled = false;
        }
    }

    pub fn read_register(&self, reg: u8) -> u8 {
        match reg {
            0 => self.sweep.as_ref().map_or(0xFF, |s| s.read_register()),
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
                if let Some(ref mut sweep) = self.sweep {
                    sweep.write_register(value);
                }
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
                self.timer.period = square_period(self.frequency());
            }
            4 => {
                self.nrx4 = value;
                self.length.enabled = value & 0x40 != 0;
                self.timer.period = square_period(self.frequency());

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
        let freq = self.frequency();
        self.timer.period = square_period(freq);
        self.timer.reload();

        let disabled = if let Some(ref mut sweep) = self.sweep {
            sweep.trigger(freq);
            sweep.channel_disabled
        } else {
            false
        };

        if disabled {
            self.enabled = false;
        }
    }

    pub fn power_off(&mut self) {
        self.nrx0 = 0;
        self.nrx1 = 0;
        self.nrx2 = 0;
        self.nrx3 = 0;
        self.nrx4 = 0;
        self.enabled = false;
        self.envelope = Envelope::new();
        if let Some(ref mut sweep) = self.sweep {
            *sweep = Sweep::new();
        }
        self.length = LengthCounter::new(64);
    }
}

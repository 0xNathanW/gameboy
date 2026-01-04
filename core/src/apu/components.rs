// Length counter - disables channel when it counts down to zero
// Clocked at 256 Hz by the frame sequencer
pub struct LengthCounter {
    counter: u16,
    max_length: u16,
    pub enabled: bool,
}

impl LengthCounter {
    pub fn new(max_length: u16) -> Self {
        Self {
            counter: 0,
            max_length,
            enabled: false,
        }
    }

    pub fn load(&mut self, value: u8) {
        self.counter = self.max_length - u16::from(value);
    }

    pub fn load_full(&mut self, value: u16) {
        self.counter = self.max_length - value;
    }

    // Returns true if channel should be disabled
    pub fn clock(&mut self) -> bool {
        if self.enabled && self.counter > 0 {
            self.counter -= 1;
            return self.counter == 0;
        }
        false
    }

    pub fn trigger(&mut self) {
        if self.counter == 0 {
            self.counter = self.max_length;
        }
    }
}

// Volume envelope - modulates volume over time
// Clocked at 64 Hz by the frame sequencer
pub struct Envelope {
    pub volume: u8,
    initial_volume: u8,
    add_mode: bool,
    period: u8,
    timer: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            volume: 0,
            initial_volume: 0,
            add_mode: false,
            period: 0,
            timer: 0,
        }
    }

    // Called when NRx2 is written
    pub fn write_register(&mut self, value: u8) {
        self.initial_volume = value >> 4;
        self.add_mode = value & 0x08 != 0;
        self.period = value & 0x07;
    }

    // Returns register value for NRx2 reads
    pub fn read_register(&self) -> u8 {
        (self.initial_volume << 4) | (if self.add_mode { 0x08 } else { 0 }) | self.period
    }

    // Returns true if DAC is enabled (volume or add_mode set)
    pub fn dac_enabled(&self) -> bool {
        self.initial_volume != 0 || self.add_mode
    }

    pub fn trigger(&mut self) {
        self.volume = self.initial_volume;
        // Period of 0 is treated as 8
        self.timer = if self.period == 0 { 8 } else { self.period };
    }

    pub fn clock(&mut self) {
        if self.period == 0 {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = if self.period == 0 { 8 } else { self.period };

            let new_volume = if self.add_mode {
                self.volume.wrapping_add(1)
            } else {
                self.volume.wrapping_sub(1)
            };

            if new_volume <= 15 {
                self.volume = new_volume;
            }
        }
    }
}

// Frequency sweep - periodically adjusts frequency (Square1 only)
// Clocked at 128 Hz by the frame sequencer
pub struct Sweep {
    enabled: bool,
    shadow_freq: u16,
    timer: u8,
    period: u8,
    negate: bool,
    shift: u8,
    // Track if sweep has disabled the channel
    pub channel_disabled: bool,
}

impl Sweep {
    pub fn new() -> Self {
        Self {
            enabled: false,
            shadow_freq: 0,
            timer: 0,
            period: 0,
            negate: false,
            shift: 0,
            channel_disabled: false,
        }
    }

    // Called when NR10 is written
    pub fn write_register(&mut self, value: u8) {
        self.period = (value >> 4) & 0x07;
        self.negate = value & 0x08 != 0;
        self.shift = value & 0x07;
    }

    // Returns register value for NR10 reads
    pub fn read_register(&self) -> u8 {
        (self.period << 4) | (if self.negate { 0x08 } else { 0 }) | self.shift
    }

    // Called on channel trigger, returns new frequency if changed
    pub fn trigger(&mut self, frequency: u16) -> Option<u16> {
        self.shadow_freq = frequency;
        self.timer = if self.period == 0 { 8 } else { self.period };
        self.enabled = self.period != 0 || self.shift != 0;
        self.channel_disabled = false;

        // If shift is non-zero, calculate and check overflow immediately
        if self.shift != 0 {
            let new_freq = self.calculate();
            if new_freq > 2047 {
                self.channel_disabled = true;
            }
        }

        None
    }

    fn calculate(&self) -> u16 {
        let offset = self.shadow_freq >> self.shift;
        if self.negate {
            self.shadow_freq.wrapping_sub(offset)
        } else {
            self.shadow_freq.wrapping_add(offset)
        }
    }

    // Returns Some(new_frequency) if frequency should be updated
    pub fn clock(&mut self) -> Option<u16> {
        if !self.enabled {
            return None;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = if self.period == 0 { 8 } else { self.period };

            if self.period != 0 {
                let new_freq = self.calculate();

                if new_freq > 2047 {
                    self.channel_disabled = true;
                    return None;
                }

                if self.shift != 0 {
                    self.shadow_freq = new_freq;

                    // Calculate again and check for overflow
                    let check_freq = self.calculate();
                    if check_freq > 2047 {
                        self.channel_disabled = true;
                        return None;
                    }

                    return Some(new_freq);
                }
            }
        }

        None
    }
}

// Linear Feedback Shift Register - generates pseudo-random noise
pub struct Lfsr {
    register: u16,
    width_mode: bool, // true = 7-bit, false = 15-bit
    pub output: bool, // Current output state
}

impl Lfsr {
    pub fn new() -> Self {
        Self {
            register: 0x7FFF, // All bits set
            width_mode: false,
            output: false,
        }
    }

    pub fn set_width_mode(&mut self, narrow: bool) {
        self.width_mode = narrow;
    }

    pub fn trigger(&mut self) {
        self.register = 0x7FFF;
        self.output = false;
    }

    // Clock the LFSR, updates output state
    pub fn clock(&mut self) {
        let xor_result = (self.register & 0x01) ^ ((self.register >> 1) & 0x01);
        self.register >>= 1;
        self.register |= xor_result << 14;

        if self.width_mode {
            self.register = (self.register & !0x40) | (xor_result << 6);
        }

        // Output is inverted bit 0
        self.output = self.register & 0x01 == 0;
    }
}

// Frequency timer helper
pub struct FrequencyTimer {
    pub period: u32,
    counter: u32,
}

impl FrequencyTimer {
    pub fn new(period: u32) -> Self {
        Self { period, counter: 0 }
    }

    // Returns number of times the timer clocked
    pub fn tick(&mut self, cycles: u32) -> u32 {
        self.counter += cycles;
        let clocks = self.counter / self.period;
        self.counter %= self.period;
        clocks
    }

    pub fn reload(&mut self) {
        self.counter = 0;
    }
}

// Calculate square/wave channel period from frequency
pub fn square_period(frequency: u16) -> u32 {
    4 * (2048 - u32::from(frequency))
}

pub fn wave_period(frequency: u16) -> u32 {
    2 * (2048 - u32::from(frequency))
}

// Calculate noise channel period from divider and shift
pub fn noise_period(divider_code: u8, clock_shift: u8) -> u32 {
    let base = match divider_code {
        0 => 8,
        n => u32::from(n) * 16,
    };
    base << clock_shift
}

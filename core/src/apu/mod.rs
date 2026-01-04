mod components;
mod noise;
mod square;
mod wave;

use crate::bus::MemoryBus;
use blip_buf::BlipBuf;
use noise::NoiseChannel;
use square::SquareChannel;
use std::sync::{Arc, Mutex};
use wave::WaveChannel;

const CLOCK_FREQUENCY: u32 = 4_194_304;
const FRAME_SEQUENCER_RATE: u32 = 512;
const CYCLES_PER_FRAME_STEP: u32 = CLOCK_FREQUENCY / FRAME_SEQUENCER_RATE;

// Masks for reading registers (OR'd with value)
const READ_MASKS: [u8; 48] = [
    0x80, 0x3F, 0x00, 0xFF, 0xBF, // NR10-NR14
    0xFF, 0x3F, 0x00, 0xFF, 0xBF, // NR20-NR24
    0x7F, 0xFF, 0x9F, 0xFF, 0xBF, // NR30-NR34
    0xFF, 0xFF, 0x00, 0x00, 0xBF, // NR40-NR44
    0x00, 0x00, 0x70, // NR50-NR52
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // FF27-FF2F
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Wave RAM
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// Frame sequencer step determines which components are clocked
#[derive(Clone, Copy)]
enum FrameStep {
    Length,
    LengthAndSweep,
    Envelope,
    Nothing,
}

impl FrameStep {
    fn from_step(step: u8) -> Self {
        match step {
            0 | 4 => FrameStep::Length,
            2 | 6 => FrameStep::LengthAndSweep,
            7 => FrameStep::Envelope,
            _ => FrameStep::Nothing,
        }
    }
}

struct FrameSequencer {
    step: u8,
    timer: u32,
}

impl FrameSequencer {
    fn new() -> Self {
        Self { step: 0, timer: 0 }
    }

    fn tick(&mut self, cycles: u32) -> Option<FrameStep> {
        self.timer += cycles;
        if self.timer >= CYCLES_PER_FRAME_STEP {
            self.timer -= CYCLES_PER_FRAME_STEP;
            let event = FrameStep::from_step(self.step);
            self.step = (self.step + 1) % 8;
            Some(event)
        } else {
            None
        }
    }
}

struct ChannelBlip {
    blip: BlipBuf,
    last_amp: i32,
    time: u32,
}

impl ChannelBlip {
    fn new(sample_rate: u32) -> Self {
        let mut blip = BlipBuf::new(sample_rate);
        blip.set_rates(f64::from(CLOCK_FREQUENCY), f64::from(sample_rate));
        Self {
            blip,
            last_amp: 0,
            time: 0,
        }
    }

    fn add_delta(&mut self, amplitude: i32) {
        let delta = amplitude - self.last_amp;
        if delta != 0 {
            self.blip.add_delta(self.time, delta);
            self.last_amp = amplitude;
        }
    }

    fn advance(&mut self, cycles: u32) {
        self.time += cycles;
    }

    fn end_frame(&mut self) {
        self.blip.end_frame(self.time);
        self.time = 0;
    }

    fn samples_avail(&self) -> u32 {
        self.blip.samples_avail() as u32
    }

    fn read_samples(&mut self, buf: &mut [i16]) -> usize {
        self.blip.read_samples(buf, false)
    }
}

struct Mixer {
    ch1: ChannelBlip,
    ch2: ChannelBlip,
    ch3: ChannelBlip,
    ch4: ChannelBlip,
    sample_rate: u32,
}

impl Mixer {
    fn new(sample_rate: u32) -> Self {
        Self {
            ch1: ChannelBlip::new(sample_rate),
            ch2: ChannelBlip::new(sample_rate),
            ch3: ChannelBlip::new(sample_rate),
            ch4: ChannelBlip::new(sample_rate),
            sample_rate,
        }
    }

    fn update(&mut self, ch1: i32, ch2: i32, ch3: i32, ch4: i32) {
        self.ch1.add_delta(ch1);
        self.ch2.add_delta(ch2);
        self.ch3.add_delta(ch3);
        self.ch4.add_delta(ch4);
    }

    fn advance(&mut self, cycles: u32) {
        self.ch1.advance(cycles);
        self.ch2.advance(cycles);
        self.ch3.advance(cycles);
        self.ch4.advance(cycles);
    }

    fn end_frame(&mut self) {
        self.ch1.end_frame();
        self.ch2.end_frame();
        self.ch3.end_frame();
        self.ch4.end_frame();
    }

    fn mix(&mut self, nr50: u8, nr51: u8, buffer: &mut Vec<(f32, f32)>) {
        let count = self.ch1.samples_avail() as usize;
        if count == 0 {
            return;
        }

        let l_vol = f32::from((nr50 >> 4) & 0x07) / 7.0 * (1.0 / 15.0) * 0.5;
        let r_vol = f32::from(nr50 & 0x07) / 7.0 * (1.0 / 15.0) * 0.5;

        let mut buf1 = [0i16; 1024];
        let mut buf2 = [0i16; 1024];
        let mut buf3 = [0i16; 1024];
        let mut buf4 = [0i16; 1024];

        let mut offset = 0;
        while offset < count {
            let chunk = (count - offset).min(1024);

            let n1 = self.ch1.read_samples(&mut buf1[..chunk]);
            let n2 = self.ch2.read_samples(&mut buf2[..chunk]);
            let n3 = self.ch3.read_samples(&mut buf3[..chunk]);
            let n4 = self.ch4.read_samples(&mut buf4[..chunk]);

            let n = n1.min(n2).min(n3).min(n4);

            for i in 0..n {
                // Don't buffer more than 1 second
                if buffer.len() > self.sample_rate as usize {
                    return;
                }

                let mut left = 0.0f32;
                let mut right = 0.0f32;

                // Channel 1
                let s1 = f32::from(buf1[i]);
                if nr51 & 0x10 != 0 {
                    left += s1 * l_vol;
                }
                if nr51 & 0x01 != 0 {
                    right += s1 * r_vol;
                }

                // Channel 2
                let s2 = f32::from(buf2[i]);
                if nr51 & 0x20 != 0 {
                    left += s2 * l_vol;
                }
                if nr51 & 0x02 != 0 {
                    right += s2 * r_vol;
                }

                // Channel 3
                let s3 = f32::from(buf3[i]);
                if nr51 & 0x40 != 0 {
                    left += s3 * l_vol;
                }
                if nr51 & 0x04 != 0 {
                    right += s3 * r_vol;
                }

                // Channel 4
                let s4 = f32::from(buf4[i]);
                if nr51 & 0x80 != 0 {
                    left += s4 * l_vol;
                }
                if nr51 & 0x08 != 0 {
                    right += s4 * r_vol;
                }

                buffer.push((left, right));
            }

            offset += n;
        }
    }
}

pub struct APU {
    pub buffer: Arc<Mutex<Vec<(f32, f32)>>>,

    // Channels
    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,

    // Control registers
    nr50: u8, // Master volume
    nr51: u8, // Channel panning
    nr52: u8, // Power control

    // Timing
    frame_sequencer: FrameSequencer,
    mixer: Mixer,
}

impl APU {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            ch1: SquareChannel::new(true),  // Has sweep
            ch2: SquareChannel::new(false), // No sweep
            ch3: WaveChannel::new(),
            ch4: NoiseChannel::new(),
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            frame_sequencer: FrameSequencer::new(),
            mixer: Mixer::new(sample_rate),
        }
    }

    fn power_on(&self) -> bool {
        self.nr52 & 0x80 != 0
    }

    pub fn next(&mut self, cycles: u32) {
        if !self.power_on() {
            return;
        }

        // Update channels
        self.ch1.tick(cycles);
        self.ch2.tick(cycles);
        self.ch3.tick(cycles);
        let noise_clocks = self.ch4.tick(cycles);
        for _ in 0..noise_clocks {
            self.ch4.clock_lfsr();
        }

        // Update mixer with current outputs
        self.mixer.update(
            self.ch1.output(),
            self.ch2.output(),
            self.ch3.output(),
            self.ch4.output(),
        );
        self.mixer.advance(cycles);

        // Frame sequencer
        if let Some(step) = self.frame_sequencer.tick(cycles) {
            match step {
                FrameStep::Length => {
                    self.ch1.clock_length();
                    self.ch2.clock_length();
                    self.ch3.clock_length();
                    self.ch4.clock_length();
                }
                FrameStep::LengthAndSweep => {
                    self.ch1.clock_length();
                    self.ch2.clock_length();
                    self.ch3.clock_length();
                    self.ch4.clock_length();
                    self.ch1.clock_sweep();
                }
                FrameStep::Envelope => {
                    self.ch1.clock_envelope();
                    self.ch2.clock_envelope();
                    self.ch4.clock_envelope();
                }
                FrameStep::Nothing => {}
            }

            // End frame and mix
            self.mixer.end_frame();
            if let Ok(mut buffer) = self.buffer.lock() {
                self.mixer.mix(self.nr50, self.nr51, &mut buffer);
            }
        }
    }

    fn channel_status(&self) -> u8 {
        let ch1 = if self.ch1.enabled { 0x01 } else { 0 };
        let ch2 = if self.ch2.enabled { 0x02 } else { 0 };
        let ch3 = if self.ch3.enabled { 0x04 } else { 0 };
        let ch4 = if self.ch4.enabled { 0x08 } else { 0 };
        ch1 | ch2 | ch3 | ch4
    }

    fn power_off_channels(&mut self) {
        self.ch1.power_off();
        self.ch2.power_off();
        self.ch3.power_off();
        self.ch4.power_off();
        self.nr50 = 0;
        self.nr51 = 0;
    }
}

impl MemoryBus for APU {
    fn read_byte(&self, addr: u16) -> u8 {
        let value = match addr {
            // Square 1: NR10-NR14
            0xFF10..=0xFF14 => self.ch1.read_register((addr - 0xFF10) as u8),
            // Square 2: NR20-NR24 (0xFF15 unused)
            0xFF15..=0xFF19 => self.ch2.read_register((addr - 0xFF15) as u8),
            // Wave: NR30-NR34
            0xFF1A..=0xFF1E => self.ch3.read_register((addr - 0xFF1A) as u8),
            // Noise: NR40-NR44 (0xFF1F unused)
            0xFF1F..=0xFF23 => self.ch4.read_register((addr - 0xFF1F) as u8),
            // Control
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => (self.nr52 & 0x80) | self.channel_status(),
            // Unused
            0xFF27..=0xFF2F => 0x00,
            // Wave RAM
            0xFF30..=0xFF3F => self.ch3.read_wave_ram((addr - 0xFF30) as usize),
            _ => 0xFF,
        };
        value | READ_MASKS[(addr - 0xFF10) as usize]
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        // NR52 can always be written
        if addr == 0xFF26 {
            let was_on = self.power_on();
            self.nr52 = value & 0x80;
            if was_on && !self.power_on() {
                self.power_off_channels();
            }
            return;
        }

        // Other registers only writable when powered on
        if !self.power_on() {
            return;
        }

        match addr {
            // Square 1: NR10-NR14
            0xFF10..=0xFF14 => self.ch1.write_register((addr - 0xFF10) as u8, value),
            // Square 2: NR20-NR24 (0xFF15 unused)
            0xFF15..=0xFF19 => self.ch2.write_register((addr - 0xFF15) as u8, value),
            // Wave: NR30-NR34
            0xFF1A..=0xFF1E => self.ch3.write_register((addr - 0xFF1A) as u8, value),
            // Noise: NR40-NR44 (0xFF1F unused)
            0xFF1F..=0xFF23 => self.ch4.write_register((addr - 0xFF1F) as u8, value),
            // Control
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            // Unused
            0xFF27..=0xFF2F => {}
            // Wave RAM
            0xFF30..=0xFF3F => self.ch3.write_wave_ram((addr - 0xFF30) as usize, value),
            _ => {}
        }
    }
}

use crate::{audio::AudioOutput, constants::CYCLES_PER_FRAME};
use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    Gameboy, GbKey, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub const DEMO_DATA: &[u8] = include_bytes!("../pocket.gb");

pub struct Emulator {
    gameboy: Gameboy,
    // RGBA buffer for web canvas rendering
    rgba_buffer: Vec<u8>,
    audio_output: Option<AudioOutput>,
}

impl Default for Emulator {
    fn default() -> Self {
        let demo = open_cartridge(DEMO_DATA.to_vec(), None, None).unwrap();
        let gameboy = Gameboy::new(demo, None);
        // Note: APU is enabled lazily when user enables audio, not during init
        // This avoids potential issues with WebAudio context during startup
        Self {
            gameboy,
            rgba_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            audio_output: None,
        }
    }
}

impl Emulator {
    pub fn new(rom_data: Box<dyn Cartridge>) -> Self {
        let gameboy = Gameboy::new(rom_data, None);
        // Note: APU is enabled lazily when user enables audio
        Self {
            gameboy,
            rgba_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            audio_output: None,
        }
    }

    pub fn disable_audio(&mut self) {
        self.audio_output = None;
    }

    pub fn enable_audio(&mut self) -> bool {
        if self.audio_output.is_some() {
            return true;
        }

        let sample_rate = AudioOutput::default_sample_rate();
        self.gameboy.enable_audio(sample_rate);

        let Some(buffer) = self.gameboy.audio_buffer() else {
            web_sys::console::error_1(&"failed to get audio buffer".into());
            return false;
        };

        match AudioOutput::new(buffer, sample_rate) {
            Ok((output, actual_rate)) => {
                if actual_rate != sample_rate {
                    self.gameboy.enable_audio(actual_rate);
                }
                self.audio_output = Some(output);
                true
            }
            Err(e) => {
                web_sys::console::error_1(&format!("failed to create audio: {}", e).into());
                false
            }
        }
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < CYCLES_PER_FRAME {
            let cycles = self.gameboy.tick();
            self.gameboy.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn is_display_updated(&mut self) -> bool {
        self.gameboy.display_updated()
    }

    pub fn key_down(&mut self, key: GbKey) {
        self.gameboy.key_down(key);
    }

    pub fn key_up(&mut self, key: GbKey) {
        self.gameboy.key_up(key);
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.gameboy.set_palette(palette);
    }

    // Returns the display buffer as RGBA bytes for web canvas rendering.
    pub fn display_buffer(&mut self) -> &[u8] {
        let pixels = self.gameboy.display_buffer();
        for (i, &pixel) in pixels.iter().enumerate() {
            let offset = i * 4;
            self.rgba_buffer[offset] = ((pixel >> 16) & 0xFF) as u8; // R
            self.rgba_buffer[offset + 1] = ((pixel >> 8) & 0xFF) as u8; // G
            self.rgba_buffer[offset + 2] = (pixel & 0xFF) as u8; // B
            self.rgba_buffer[offset + 3] = 0xFF; // A
        }
        &self.rgba_buffer
    }

    // TODO: implement saving/loading save data and RTC state
    pub fn _save_data(&self) -> Option<&[u8]> {
        self.gameboy.save_data()
    }

    pub fn _rtc_zero(&self) -> Option<u64> {
        self.gameboy.rtc_zero()
    }
}

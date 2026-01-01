#[cfg(feature = "audio")]
use crate::apu::APU;
use crate::{
    cartridge::{CartError, Cartridge},
    cpu::Cpu,
    keypad::GbKey,
    memory::Memory,
    serial::SerialCallback,
};
#[cfg(feature = "audio")]
use std::sync::{Arc, Mutex};

const STEP_TIME: u32 = 16;
const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1_000_f64 / 4_194_304_f64)) as u32;

pub struct Gameboy {
    cpu: Cpu,
    mem: Memory,
    // Provide control over speed of cpu clock.
    step_cycles: u32,
    #[cfg(not(target_arch = "wasm32"))]
    step_zero: std::time::Instant,
}

impl Gameboy {
    pub fn new(cartridge: Box<dyn Cartridge>, callback: SerialCallback) -> Self {
        Self {
            cpu: Cpu::new(),
            mem: Memory::new(cartridge, callback),
            step_cycles: 0,
            #[cfg(not(target_arch = "wasm32"))]
            step_zero: std::time::Instant::now(),
        }
    }

    pub fn tick(&mut self) -> u32 {
        self.cpu.tick(&mut self.mem)
    }

    pub fn update(&mut self, cycles: u32) {
        self.mem.update(cycles);
    }

    // Run at documented 4.19 MHz with timing control (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn step(&mut self) -> u32 {
        if self.step_cycles > STEP_CYCLES {
            self.step_cycles -= STEP_CYCLES;
            let now = std::time::Instant::now();

            let d = now.duration_since(self.step_zero);
            let sleep_time = (STEP_TIME.saturating_sub(d.as_millis() as u32)) as u64;
            std::thread::sleep(std::time::Duration::from_millis(sleep_time));
            self.step_zero = self
                .step_zero
                .checked_add(std::time::Duration::from_millis(STEP_TIME as u64))
                .unwrap();

            if now.checked_duration_since(self.step_zero).is_some() {
                self.step_zero = now;
            }
        }

        let cycles = self.tick();
        self.step_cycles += cycles;
        cycles
    }

    // Check if the display buffer has been updated since last call.
    pub fn display_updated(&mut self) -> bool {
        self.mem.gpu.check_updated()
    }

    // Get a reference to the pixel buffer for rendering.
    // Returns &[u32] on native (160*144 pixels).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn display_buffer(&self) -> &[u32] {
        &self.mem.gpu.pixels
    }

    // Get a reference to the pixel buffer for rendering.
    // Returns &[u8] on WASM (160*144*4 bytes in RGBA format).
    #[cfg(target_arch = "wasm32")]
    pub fn display_buffer(&self) -> &[u8] {
        &self.mem.gpu.pixels
    }

    pub fn set_palette(&mut self, palette: [u32; 4]) {
        self.mem.gpu.set_colours(palette);
    }

    pub fn key_down(&mut self, key: GbKey) {
        self.mem.keypad.key_press(key);
    }

    pub fn key_up(&mut self, key: GbKey) {
        self.mem.keypad.key_release(key);
    }

    // Initialize the APU with the given sample rate.
    #[cfg(feature = "audio")]
    pub fn enable_audio(&mut self, sample_rate: u32) {
        self.mem.apu = Some(APU::power_up(sample_rate));
    }

    // Get a shared reference to the audio buffer for streaming.
    #[cfg(feature = "audio")]
    pub fn audio_buffer(&self) -> Option<Arc<Mutex<Vec<(f32, f32)>>>> {
        self.mem.apu.as_ref().map(|apu| apu.buffer.clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> Result<(), CartError> {
        self.mem.save()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) -> Result<*const u8, CartError> {
        self.mem.save()
    }
}

#[cfg(feature = "audio")]
use crate::apu::APU;
use crate::{
    cartridge::Cartridge, cpu::Cpu, keypad::GbKey, memory::Memory, serial::SerialCallback,
};
#[cfg(feature = "audio")]
use std::sync::{Arc, Mutex};

pub struct Gameboy {
    cpu: Cpu,
    mem: Memory,
}

impl Gameboy {
    pub fn new(cartridge: Box<dyn Cartridge>, callback: SerialCallback) -> Self {
        Self {
            cpu: Cpu::new(),
            mem: Memory::new(cartridge, callback),
        }
    }

    pub fn tick(&mut self) -> u32 {
        self.cpu.tick(&mut self.mem)
    }

    pub fn update(&mut self, cycles: u32) {
        self.mem.update(cycles);
    }

    // Check if the display buffer has been updated since last call.
    pub fn display_updated(&mut self) -> bool {
        self.mem.gpu.check_updated()
    }

    // Get a reference to the pixel buffer for rendering (160*144 pixels)
    pub fn display_buffer(&self) -> &[u32] {
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
        self.mem.apu = Some(APU::new(sample_rate));
    }

    // Get a shared reference to the audio buffer for streaming.
    #[cfg(feature = "audio")]
    pub fn audio_buffer(&self) -> Option<Arc<Mutex<Vec<(f32, f32)>>>> {
        self.mem.apu.as_ref().map(|apu| apu.buffer.clone())
    }

    // Set the audio volume (0.0 to 1.0).
    #[cfg(feature = "audio")]
    pub fn set_volume(&mut self, volume: f32) {
        if let Some(apu) = &mut self.mem.apu {
            apu.set_volume(volume);
        }
    }

    #[cfg(feature = "audio")]
    pub fn volume(&self) -> f32 {
        self.mem.apu.as_ref().map_or(1.0, |apu| apu.volume())
    }

    // Returns the save RAM data if the cartridge supports saves.
    pub fn save_data(&self) -> Option<&[u8]> {
        self.mem.save_data()
    }

    // Returns the RTC zero timestamp if the cartridge has RTC support.
    pub fn rtc_zero(&self) -> Option<u64> {
        self.mem.rtc_zero()
    }

    #[cfg(feature = "inspect")]
    pub fn cpu_state(&self) -> crate::inspect::CpuState {
        self.cpu.state()
    }

    #[cfg(feature = "inspect")]
    pub fn gpu_state(&self) -> crate::inspect::GpuState {
        self.mem.gpu.state()
    }

    #[cfg(feature = "inspect")]
    pub fn vram(&self) -> &[u8] {
        self.mem.gpu.vram()
    }

    #[cfg(feature = "inspect")]
    pub fn oam(&self) -> &[u8] {
        self.mem.gpu.oam()
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.mem.reset();
    }
}

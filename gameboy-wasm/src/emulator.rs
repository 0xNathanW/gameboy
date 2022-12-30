use gameboy_core::cpu::CPU;
pub struct Emulator(pub CPU);

impl Emulator {

    pub fn new() -> Self {
        let demo = std::fs::read("./pocket.gb").unwrap();
        let cart = open_cartridge(demo, None);
        Self(CPU::new(cart, None))
    }

    pub fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < 69_905 {
            let cycles = self.0.tick();
            self.0.mem.update(cycles);
            frame_cycles += cycles;
        }
    }

    pub fn is_display_updated(&mut self) -> bool {
        self.0.mem.gpu.check_updated()
    }

    pub fn pixel_buffer(&self) -> Vec<u8> {
        let row_pix = 160 * 4 * 4;
        let mut buf = vec![0; row_pix * 144 * 4 * 4];
        for (i, raw) in self.0.mem.gpu.pixels.iter().enumerate() {

            let col = i % 160;
            let row = i / 160;
            let mut rgba = (raw << 8).to_be_bytes();
            rgba[3] = 0xFF; // Opacity.

            for (j, c) in rgba.iter().enumerate() {
                for n in 0..4 {
                    for m in 0..4 {
                        buf[
                            ((col * 4 * 4) + (4 * n)) +     // x
                            (((row * 4) + m ) * row_pix)    // y
                            + j                             // offset
                        ] = *c;
                    }}}}   
        buf
    }
}

use std::sync::Arc;

use yew::prelude::*;
use gloo::timers::callback::Interval;
use web_sys::{
    HtmlCanvasElement,
    ImageData,
};
use wasm_bindgen::JsCast;
use gameboy_core::{
    cpu::CPU,
    cartridge::open_cartridge,
};

pub struct Emulator {
    cpu: CPU,
    rom_name: String,
    canvas: NodeRef,
}

impl Emulator {

    fn tick(&mut self) {
        let mut frame_cycles = 0;
        while frame_cycles < 69_905 {
            let cycles = self.cpu.tick();
            self.cpu.mem.update(cycles);
            frame_cycles += cycles;
        }
    }

    fn is_display_updated(&mut self) -> bool {
        self.cpu.mem.gpu.check_updated()
    }

    fn pixel_buffer(&self) -> Vec<u8> {
        let row_pix = 160 * 4 * 4;
        let mut buf = vec![0; row_pix * 144 * 4 * 4];
        for (i, raw) in self.cpu.mem.gpu.pixels.iter().enumerate() {

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

enum Msg {
    Start,
}

impl Component for Emulator {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        let demo = std::fs::read("./pocket.gb").unwrap();
        let cart = open_cartridge(demo, None);
        Self {
            cpu: CPU::new(cart, None),
            rom_name: "demo".to_string(),
            canvas: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                let node_ref = self.canvas.clone();
                let canvas = node_ref.cast::<HtmlCanvasElement>().unwrap();
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();
                let interval = Interval::new(16, move || {

                    // Tick cpu.
                    self.tick();
                    // Update display if neccessary.
                    if self.is_display_updated() {
                        let buf = self.pixel_buffer();
                        let img_data = ImageData::new_with_u8_clamped_array(
                            wasm_bindgen::Clamped(&buf),
                            160 * 4,    // Wdith.
                        ).unwrap();
                        ctx.put_image_data(&img_data, 0.0, 0.0).unwrap();
                    }
                });
                true
            }
        }
    
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <div>
            <canvas 
                id="emulator_canvas"
                width={(160 * 4).to_string()}
                height={(144 * 4).to_string()}
                ref={self.canvas.clone()}>
            </canvas>
            </div>
        }
    }
}



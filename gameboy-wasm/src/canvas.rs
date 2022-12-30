use std::{rc::Rc, cell::RefCell};
use yew::prelude::*;
use gloo::timers::callback::Interval;
use web_sys::{
    HtmlCanvasElement,
    ImageData,
};
use wasm_bindgen::JsCast;
use gameboy_core::cartridge::open_cartridge;
use super::emulator::Emulator;

pub struct Canvas {
    emulator: Rc<RefCell<Emulator>>,
    rom_name: String,
    canvas: NodeRef,
}

enum Msg {
    Start,
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            emulator: Rc::new(RefCell::new(Emulator::new())),
            rom_name: "demo".to_string(),
            canvas: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                
                let node_ref = self.canvas.clone();
                let mut emu = self.emulator.borrow_mut();
                let canvas = node_ref.cast::<HtmlCanvasElement>().unwrap();
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                let interval = Interval::new(16, move || {
                    // Tick cpu.
                    emu.tick();
                    // Update display if neccessary.
                    if emu.is_display_updated() {
                        let buf = emu.pixel_buffer();
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



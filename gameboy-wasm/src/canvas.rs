use std::{rc::Rc, cell::RefCell};
use yew::prelude::*;
use gloo::timers::callback::Interval;
use gloo::console::log;
use web_sys::{
    HtmlCanvasElement,
    ImageData, 
    Node, 
    CanvasRenderingContext2d,
    window,
    console,
};
use wasm_bindgen::JsCast;
use gameboy_core::cartridge::open_cartridge;
use super::emulator::Emulator;

const FRAME_TIME: u32 = 16; // Approx 60 FPS.
const SCALE: f64 = 4.0;

pub struct Canvas {
    emulator:   Emulator,
    canvas:     NodeRef,
    ctx:        Option<CanvasRenderingContext2d>,
    interval:   Interval,
}

pub enum Msg {
    Tick, 
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let interval = {
            let link = ctx.link().clone();
            Interval::new(FRAME_TIME, move || {
                link.send_message(Msg::Tick);
            })
        };

        Self {
            emulator: Emulator::new(),
            canvas: NodeRef::default(),
            ctx: None,
            interval,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.emulator.tick();
                if self.emulator.is_display_updated() {
                    self.render_frame();
                }
                true
            },
        }
    
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <div>
            <canvas 
                id="emulator_canvas"
                width={(160 * SCALE as usize).to_string()}
                height={(144 * SCALE as usize).to_string()}
                ref={self.canvas.clone()}>
            </canvas>
            </div>
        }
    }
}

impl Canvas {
    
    fn render_frame(&mut self) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        let ctx = match &self.ctx {
            Some(ctx) => ctx,
            None => {
                let ctx = canvas.get_context("2d")
                    .unwrap()
                    .unwrap();
                let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                ctx.scale(SCALE, SCALE).unwrap();
                self.ctx = Some(ctx);
                self.ctx.as_ref().unwrap()
            }
        };
        
        let clamped_arr = wasm_bindgen::Clamped(self.emulator.0.mem.gpu.pixels.as_slice());
        let img_data = ImageData::new_with_u8_clamped_array(
            clamped_arr,
            160,
        ).unwrap();

        ctx.put_image_data(&img_data, 0.0, 0.0).unwrap();
        ctx.draw_image_with_html_canvas_element(&ctx.canvas().unwrap(), 0_f64, 0_f64).unwrap();
    }

}


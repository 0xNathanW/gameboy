use std::{rc::Rc, cell::RefCell};
use yew::prelude::*;
use gloo::{
    timers::callback::Interval, 
    utils::document, 
    events::EventListener,
    console::log,
};
use web_sys::{
    HtmlCanvasElement,
    ImageData, 
    Node, 
    CanvasRenderingContext2d,
};
use futures::channel::mpsc;
use wasm_bindgen::JsCast;
use gameboy_core::keypad::GbKey;
use super::emulator::Emulator;

const FRAME_TIME: u32 = 16; // Approx 60 FPS.
const SCALE: f64 = 4.0;

pub struct Canvas {
    emulator:           Emulator,
    canvas:             NodeRef,
    ctx:                Option<CanvasRenderingContext2d>,
    interval:           Interval,
    key_up_listen:      EventListener,
    key_down_listen:    EventListener,
}


pub enum Msg {
    Tick,
    KeyDown(GbKey),
    KeyUp(GbKey),
}

#[derive(Clone, PartialEq, Properties)]
pub struct CanvasProps {
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        
        // Update frame every 16ms.
        let interval = {
            let link = ctx.link().clone();
            Interval::new(FRAME_TIME, move || {
                link.send_message(Msg::Tick);
            })
        };

        // Callbacks for key events.
        let on_key_down = {
            let link = ctx.link().clone();
            Callback::from(move |e: KeyboardEvent| {
                match e.key().as_str() {
                    "ArrowUp"       => link.send_message(Msg::KeyDown(GbKey::Up)),
                    "ArrowDown"     => link.send_message(Msg::KeyDown(GbKey::Down)),
                    "ArrowLeft"     => link.send_message(Msg::KeyDown(GbKey::Left)),
                    "ArrowRight"    => link.send_message(Msg::KeyDown(GbKey::Right)),
                    "z"             => link.send_message(Msg::KeyDown(GbKey::A)),
                    "x"             => link.send_message(Msg::KeyDown(GbKey::B)),
                    "Enter"         => link.send_message(Msg::KeyDown(GbKey::Start)),
                    "Shift"         => link.send_message(Msg::KeyDown(GbKey::Select)),
                    _ => return,
                };
            })
        };

        let on_key_up = {
            let link = ctx.link().clone();
            Callback::from(move |e: KeyboardEvent| {
                match e.key().as_str() {
                    "ArrowUp"       => link.send_message(Msg::KeyUp(GbKey::Up)),
                    "ArrowDown"     => link.send_message(Msg::KeyUp(GbKey::Down)),
                    "ArrowLeft"     => link.send_message(Msg::KeyUp(GbKey::Left)),
                    "ArrowRight"    => link.send_message(Msg::KeyUp(GbKey::Right)),
                    "z"             => link.send_message(Msg::KeyUp(GbKey::A)),
                    "x"             => link.send_message(Msg::KeyUp(GbKey::B)),
                    "Enter"         => link.send_message(Msg::KeyUp(GbKey::Start)),
                    "Shift"         => link.send_message(Msg::KeyUp(GbKey::Select)),
                    _ => return,
                };
            })
        };

        // Attach key listeners to document.
        let doc = document();
        let key_down = EventListener::new(&doc, "keydown", move |event| {
            let key_event = event.clone().dyn_into::<KeyboardEvent>().unwrap();
            if !key_event.repeat() {
                on_key_down.emit(key_event);
            }    
        });
        let key_up = EventListener::new(&doc, "keyup", move |event| {
            let key_event = event.clone().dyn_into::<KeyboardEvent>().unwrap();
            if !key_event.repeat() {
                on_key_up.emit(key_event);
            }    
        });

        Self {
            emulator: Emulator::new(),
            canvas: NodeRef::default(),
            ctx: None,
            interval,
            key_up_listen: key_up,
            key_down_listen: key_down,
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
            Msg::KeyDown(key) => {
                self.emulator.key_down(key);
                false
            },
            Msg::KeyUp(key) => {
                self.emulator.key_up(key);
                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        html! {
            <div>
            <canvas 
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


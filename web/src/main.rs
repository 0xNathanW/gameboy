use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    keypad::GbKey,
};
use emulator::Emulator;
use gloo::{
    dialogs::alert, events::EventListener, file::File, timers::callback::Interval, utils::document,
};
use panel::{InfoProps, Panel};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::prelude::*;
use yew::props;

const FRAME_TIME: u32 = 16; // Approx 60 FPS.
const SCALE: f64 = 4.0;
const PALETTES: [(&str, [u32; 4]); 10] = [
    ("Classic", [0xe0f8d0, 0x88c070, 0x346856, 0x081820]),
    ("2Bit Demichrome", [0xe9efec, 0xa0a08b, 0x555568, 0x211e20]),
    ("Ice Cream", [0xfff6d3, 0xf9a875, 0xeb6b6f, 0x7c3f58]),
    ("Bicycle", [0xf0f0f0, 0x8f9bf6, 0xab4646, 0x161616]),
    ("Lopsec", [0xc7c6c6, 0x7c6d80, 0x382843, 0x000000]),
    ("Autumn Chill", [0xdad3af, 0xd58863, 0xc23a73, 0x2c1e74]),
    ("Red Dead", [0xfffcfe, 0xff0015, 0x860020, 0x11070a]),
    ("Blue Dream", [0xecf2cb, 0x98d8b1, 0x4b849a, 0x1f285d]),
    ("Lollipop", [0xe6f2ef, 0xf783b0, 0x3f6d9e, 0x151640]),
    ("Soviet", [0xe8d6c0, 0x92938d, 0xa1281c, 0x000000]),
];

mod emulator;
mod panel;

fn main() {
    yew::Renderer::<App>::new().render();
}

pub struct App {
    emulator: Emulator,
    is_cgb: bool,

    rom_name: AttrValue,
    rom_size: usize,
    cart_type: AttrValue,
    saveable: bool,

    pallette_idx: usize,

    canvas: NodeRef,
    ctx: Option<CanvasRenderingContext2d>,
    // Dropping interval will stop it from ticking.
    interval: Interval,
    paused: bool,
    // Dropping these listeners will remove them from the document.
    _key_up_listen: EventListener,
    _key_down_listen: EventListener,

    file_reader: Option<gloo::file::callbacks::FileReader>,
}

pub enum Msg {
    Tick,
    Pause,
    KeyDown(GbKey),
    KeyUp(GbKey),
    FileUpload(File),
    NewROM(Box<dyn Cartridge>),
    CyclePalette,
}

impl Component for App {
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
                    "ArrowUp" => link.send_message(Msg::KeyDown(GbKey::Up)),
                    "ArrowDown" => link.send_message(Msg::KeyDown(GbKey::Down)),
                    "ArrowLeft" => link.send_message(Msg::KeyDown(GbKey::Left)),
                    "ArrowRight" => link.send_message(Msg::KeyDown(GbKey::Right)),
                    "z" => link.send_message(Msg::KeyDown(GbKey::A)),
                    "x" => link.send_message(Msg::KeyDown(GbKey::B)),
                    "Enter" => link.send_message(Msg::KeyDown(GbKey::Start)),
                    "Shift" => link.send_message(Msg::KeyDown(GbKey::Select)),
                    _ => return,
                };
            })
        };

        let on_key_up = {
            let link = ctx.link().clone();
            Callback::from(move |e: KeyboardEvent| {
                match e.key().as_str() {
                    "ArrowUp" => link.send_message(Msg::KeyUp(GbKey::Up)),
                    "ArrowDown" => link.send_message(Msg::KeyUp(GbKey::Down)),
                    "ArrowLeft" => link.send_message(Msg::KeyUp(GbKey::Left)),
                    "ArrowRight" => link.send_message(Msg::KeyUp(GbKey::Right)),
                    "z" => link.send_message(Msg::KeyUp(GbKey::A)),
                    "x" => link.send_message(Msg::KeyUp(GbKey::B)),
                    "Enter" => link.send_message(Msg::KeyUp(GbKey::Start)),
                    "Shift" => link.send_message(Msg::KeyUp(GbKey::Select)),
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
            emulator: Emulator::default(),
            is_cgb: false,
            rom_name: "Demo".into(),
            rom_size: 0,
            saveable: false,
            cart_type: "ROM only".into(),
            canvas: NodeRef::default(),
            pallette_idx: 1,
            ctx: None,
            interval,
            paused: false,
            _key_up_listen: key_up,
            _key_down_listen: key_down,
            file_reader: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                if self.paused {
                    return false;
                }
                self.emulator.tick();
                if self.emulator.is_display_updated() {
                    self.render_frame();
                }
                true
            }

            Msg::Pause => {
                self.paused = !self.paused;
                true
            }

            Msg::KeyDown(key) => {
                self.emulator.key_down(key);
                false
            }

            Msg::KeyUp(key) => {
                self.emulator.key_up(key);
                false
            }

            Msg::FileUpload(file) => {
                let link = ctx.link().clone();
                self.file_reader = Some(gloo::file::callbacks::read_as_bytes(
                    &file,
                    move |bytes| match bytes {
                        Ok(bytes) => match open_cartridge(bytes, None) {
                            Ok(cartridge) => {
                                link.send_message(Msg::NewROM(cartridge));
                            }

                            Err(e) => alert(&format!("Error loading ROM: {}", e)),
                        },

                        Err(e) => alert(&format!("Failed to read bytes: {}", e)),
                    },
                ));
                self.paused = false;
                true
            }

            Msg::NewROM(cartridge) => {
                self.rom_name = cartridge.title().into();
                self.rom_size = cartridge.len();
                self.is_cgb = cartridge.is_cgb();
                self.cart_type = cartridge.cartridge_type().into();
                self.saveable = cartridge.is_saveable();
                self.emulator = Emulator::new(cartridge);
                true
            }

            Msg::CyclePalette => {
                self.pallette_idx = {
                    let idx = self.pallette_idx + 1;
                    if idx >= 10 {
                        0
                    } else {
                        idx
                    }
                };
                self.emulator.change_palette(PALETTES[self.pallette_idx].1);
                self.render_frame();

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let info_props = props!(InfoProps {
            is_cgb: self.is_cgb,
            rom_name: self.rom_name.clone(),
            rom_size: self.rom_size,
            cart_type: self.cart_type.clone(),
            saveable: self.saveable,
            pallette: AttrValue::from(PALETTES[self.pallette_idx].0),
        });

        html! {
            <>
            <div class="upper">

                <h1>{"GameBoy.WASM"}</h1>
                <div class="canvas">

                    <canvas
                        width={(160 * SCALE as usize).to_string()}
                        height={(144 * SCALE as usize).to_string()}
                        ref={self.canvas.clone()}>
                    </canvas>

                    <div class="button-row">

                        <input
                            id="file-input"
                            type="file"
                            multiple=false
                            accept=".gb"
                            onchange={
                                ctx.link().batch_callback(move |event: Event| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    if let Some(file) = input.files().map(|list| list.get(0)).flatten() {
                                        Some(Msg::FileUpload(file.into()))
                                    } else {
                                        None
                                    }
                                })
                            }
                        />
                        <label for="file-input" class="file-input-label">
                            <i class="gg-software-upload"></i>
                            {"\u{00a0}Upload ROM"}
                        </label>

                        <input
                            id="play-pause"
                            type="checkbox"
                            class="control-button"
                            checked={self.paused}
                            onclick={ctx.link().callback(|_| Msg::Pause)}
                        />
                        <label for="play-pause" class="control-button" id="play-pause-label">
                        {""}
                        </label>

                        <button onclick={ctx.link().callback(|_| Msg::CyclePalette)} class="control-button">
                            <i class="gg-color-picker"></i>
                            {"\u{00a0}Change Palette"}
                        </button>

                    </div>
                </div>
            </div>
            <br/>
            <br/>
            <Panel ..info_props/>
            </>
        }
    }
}

impl App {
    fn render_frame(&mut self) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        let ctx = match &self.ctx {
            Some(ctx) => ctx,
            None => {
                let ctx = canvas.get_context("2d").unwrap().unwrap();
                let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                ctx.scale(SCALE, SCALE).unwrap();
                self.ctx = Some(ctx);
                self.ctx.as_ref().unwrap()
            }
        };

        let clamped_arr = wasm_bindgen::Clamped(self.emulator.0.mem.gpu.pixels.as_slice());
        let img_data = ImageData::new_with_u8_clamped_array(clamped_arr, 160).unwrap();

        ctx.put_image_data(&img_data, 0.0, 0.0).unwrap();
        ctx.draw_image_with_html_canvas_element(&ctx.canvas().unwrap(), 0_f64, 0_f64)
            .unwrap();
    }
}

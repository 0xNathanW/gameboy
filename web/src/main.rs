use emulator::Emulator;
use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    GbKey,
};
use gloo::{
    dialogs::alert, events::EventListener, file::File, timers::callback::Interval, utils::document,
};
use panel::{InfoProps, Panel};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::prelude::*;
use yew::props;

mod constants;
mod emulator;
mod panel;

use constants::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

pub struct App {
    emulator: Emulator,

    // ROM info
    is_cgb: bool,
    rom_name: AttrValue,
    rom_size: usize,
    cart_type: AttrValue,
    saveable: bool,

    // UI state
    palette_idx: usize,
    paused: bool,

    canvas: NodeRef,
    ctx: Option<CanvasRenderingContext2d>,

    // Events
    _interval: Interval,
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
        let _interval = {
            let link = ctx.link().clone();
            Interval::new(FRAME_TIME_MS, move || {
                link.send_message(Msg::Tick);
            })
        };

        // Callbacks for key events.
        let on_key_down = create_key_callback(ctx, true);
        let on_key_up = create_key_callback(ctx, false);
        let doc = document();
        let key_down = EventListener::new(&doc, "keydown", move |event| {
            if let Some(key_event) = event.clone().dyn_into::<KeyboardEvent>().ok() {
                if !key_event.repeat() {
                    on_key_down.emit(key_event);
                }
            }
        });
        let key_up = EventListener::new(&doc, "keyup", move |event| {
            if let Some(key_event) = event.clone().dyn_into::<KeyboardEvent>().ok() {
                if !key_event.repeat() {
                    on_key_up.emit(key_event);
                }
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
            palette_idx: 1,
            ctx: None,
            _interval,
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
                self.palette_idx = (self.palette_idx + 1) % 10;
                self.emulator
                    .set_palette(PALETTES[self.palette_idx].colours);
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
            pallette: AttrValue::from(PALETTES[self.palette_idx].name),
        });

        html! {
            <>
            <div class="upper">

                <h1>{"GameBoy.WASM"}</h1>
                <div class="canvas">

                    <canvas
                        width={(SCREEN_WIDTH * DISPLAY_SCALE as u32).to_string()}
                        height={(SCREEN_HEIGHT * DISPLAY_SCALE as u32).to_string()}
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
        let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() else {
            web_sys::console::error_1(&"render_frame: canvas NodeRef not bound".into());
            return;
        };

        let ctx = match &self.ctx {
            Some(ctx) => ctx,
            None => {
                let ctx = match canvas.get_context("2d") {
                    Ok(Some(ctx)) => ctx,
                    Ok(None) => {
                        web_sys::console::error_1(&"render_frame: canvas context unavailable".into());
                        return;
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("render_frame: get_context failed: {:?}", e).into());
                        return;
                    }
                };

                let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() else {
                    web_sys::console::error_1(&"render_frame: context is not CanvasRenderingContext2d".into());
                    return;
                };

                if let Err(e) = ctx.scale(DISPLAY_SCALE, DISPLAY_SCALE) {
                    web_sys::console::error_1(&format!("render_frame: scale failed: {:?}", e).into());
                    return;
                }

                self.ctx = Some(ctx);
                self.ctx.as_ref().expect("just assigned")
            }
        };

        let clamped_arr = wasm_bindgen::Clamped(self.emulator.display_buffer());
        let img_data = match ImageData::new_with_u8_clamped_array(clamped_arr, 160) {
            Ok(data) => data,
            Err(e) => {
                web_sys::console::error_1(&format!("render_frame: ImageData creation failed: {:?}", e).into());
                return;
            }
        };

        if let Err(e) = ctx.put_image_data(&img_data, 0.0, 0.0) {
            web_sys::console::error_1(&format!("render_frame: put_image_data failed: {:?}", e).into());
            return;
        }

        let Some(canvas_element) = ctx.canvas() else {
            web_sys::console::error_1(&"render_frame: ctx.canvas() returned None".into());
            return;
        };

        if let Err(e) = ctx.draw_image_with_html_canvas_element(&canvas_element, 0.0, 0.0) {
            web_sys::console::error_1(&format!("render_frame: draw_image failed: {:?}", e).into());
        }
    }
}

fn create_key_callback(ctx: &Context<App>, is_down: bool) -> Callback<KeyboardEvent> {
    let link = ctx.link().clone();
    Callback::from(move |e: KeyboardEvent| {
        let key = match e.key().as_str() {
            "ArrowUp" => GbKey::Up,
            "ArrowDown" => GbKey::Down,
            "ArrowLeft" => GbKey::Left,
            "ArrowRight" => GbKey::Right,
            "z" => GbKey::A,
            "x" => GbKey::B,
            "Enter" => GbKey::Start,
            "Shift" => GbKey::Select,
            _ => return,
        };
        let msg = if is_down {
            Msg::KeyDown(key)
        } else {
            Msg::KeyUp(key)
        };
        link.send_message(msg);
    })
}

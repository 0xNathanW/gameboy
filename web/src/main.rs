use emulator::{decode_tiles, Emulator, DEMO_DATA};
use gameboy_core::{
    cartridge::{open_cartridge, Cartridge},
    GbKey,
};
use gloo::{
    dialogs::alert, events::EventListener, file::File, timers::callback::Interval, utils::document,
};
use panel::{DebugState, Panel, PanelProps};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::prelude::*;
use yew::props;

mod audio;
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
    scale: u32,
    speed: f32,
    canvas: NodeRef,
    audio_enabled: bool,
    volume: u8,

    // Events
    _interval: Interval,
    _key_up_listen: EventListener,
    _key_down_listen: EventListener,
    file_reader: Option<gloo::file::callbacks::FileReader>,
}

pub enum Msg {
    Tick,
    Pause,
    Reset,
    KeyDown(GbKey),
    KeyUp(GbKey),
    FileUpload(File),
    NewROM(Box<dyn Cartridge>),
    CyclePalette(i32),
    SetScale(u32),
    SetSpeed(f32),
    ResetSpeed,
    ToggleAudio,
    SetVolume(u8),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let emulator = Emulator::default();

        let on_key_down = create_key_callback(ctx, true);
        let on_key_up = create_key_callback(ctx, false);
        let doc = document();
        let key_down = EventListener::new(&doc, "keydown", move |event| {
            if let Ok(key_event) = event.clone().dyn_into::<KeyboardEvent>() {
                if !key_event.repeat() {
                    on_key_down.emit(key_event);
                }
            }
        });
        let key_up = EventListener::new(&doc, "keyup", move |event| {
            if let Ok(key_event) = event.clone().dyn_into::<KeyboardEvent>() {
                if !key_event.repeat() {
                    on_key_up.emit(key_event);
                }
            }
        });
        let _interval = {
            let link = ctx.link().clone();
            Interval::new(FRAME_TIME_MS, move || {
                link.send_message(Msg::Tick);
            })
        };

        Self {
            emulator,
            is_cgb: false,
            rom_name: "Demo".into(),
            rom_size: DEMO_DATA.len(),
            saveable: false,
            cart_type: "ROM only".into(),
            canvas: NodeRef::default(),
            palette_idx: 1,
            _interval,
            paused: false,
            scale: 3,
            speed: 1.0,
            audio_enabled: false,
            volume: 50,
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
                let cycles = (CYCLES_PER_FRAME as f32 * self.speed) as u32;
                self.emulator.tick(cycles);
                if self.emulator.is_display_updated() {
                    self.render_frame();
                    true
                } else {
                    false
                }
            }

            Msg::Pause => {
                self.paused = !self.paused;
                true
            }

            Msg::Reset => {
                self.emulator.reset();
                let cycles = (CYCLES_PER_FRAME as f32 * self.speed) as u32;
                self.emulator.tick(cycles);
                self.render_frame();
                true
            }

            Msg::SetSpeed(delta) => {
                self.speed = (self.speed + delta).clamp(0.1, 1.0);
                true
            }

            Msg::ResetSpeed => {
                self.speed = 1.0;
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
                        Ok(bytes) => match open_cartridge(bytes, None, None) {
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
                if self.audio_enabled {
                    self.emulator.enable_audio();
                }
                true
            }

            Msg::CyclePalette(dir) => {
                let len = PALETTES.len() as i32;
                self.palette_idx = ((self.palette_idx as i32 + dir).rem_euclid(len)) as usize;
                self.emulator
                    .set_palette(PALETTES[self.palette_idx].colours);
                self.render_frame();
                true
            }

            Msg::SetScale(scale) => {
                if scale != self.scale && (MIN_SCALE..=MAX_SCALE).contains(&scale) {
                    self.scale = scale;
                    true
                } else {
                    false
                }
            }

            Msg::ToggleAudio => {
                if !self.audio_enabled {
                    if self.emulator.enable_audio() {
                        self.audio_enabled = true;
                    }
                } else {
                    self.emulator.disable_audio();
                    self.audio_enabled = false;
                }
                true
            }

            Msg::SetVolume(volume) => {
                self.volume = volume.min(100);
                self.emulator.set_volume(self.volume);
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        // Canvas is cleared when resized, so redraw after any view update.
        self.render_frame();
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let debug_state = DebugState {
            cpu: self.emulator.cpu_state(),
            gpu: self.emulator.gpu_state(),
            tiles: decode_tiles(self.emulator.vram(), PALETTES[self.palette_idx].colours),
        };

        let panel_props = props!(PanelProps {
            is_cgb: self.is_cgb,
            rom_name: self.rom_name.clone(),
            rom_size: self.rom_size,
            cart_type: self.cart_type.clone(),
            saveable: self.saveable,
            palette: AttrValue::from(PALETTES[self.palette_idx].name),
            paused: self.paused,
            scale: self.scale,
            speed: AttrValue::from(format!("{:.2} MHz", self.speed * GB_CLOCK_FREQ)),
            audio_enabled: self.audio_enabled,
            volume: self.volume,
            debug_state: Some(debug_state),
            on_file_upload: ctx
                .link()
                .callback(|file: web_sys::File| Msg::FileUpload(file.into())),
            on_pause: ctx.link().callback(|_| Msg::Pause),
            on_reset: ctx.link().callback(|_| Msg::Reset),
            on_cycle_palette: ctx.link().callback(Msg::CyclePalette),
            on_set_scale: ctx.link().callback(Msg::SetScale),
            on_set_speed: ctx.link().callback(Msg::SetSpeed),
            on_reset_speed: ctx.link().callback(|_| Msg::ResetSpeed),
            on_toggle_audio: ctx.link().callback(|_| Msg::ToggleAudio),
            on_set_volume: ctx.link().callback(Msg::SetVolume),
        });

        html! {
            <div class="app-container">
                <header>
                    <h1>{"GameBoy.WASM"}</h1>
                    <a href="https://github.com/0xNathanW/gameboy" target="_blank" rel="noopener noreferrer">
                        {"GitHub"}
                    </a>
                </header>

                <div class="main-content">
                    <Panel ..panel_props />

                    <div class="canvas-wrapper">
                        <canvas
                            width={SCREEN_WIDTH.to_string()}
                            height={SCREEN_HEIGHT.to_string()}
                            style={format!("width: {}px; height: {}px;",
                                SCREEN_WIDTH * self.scale,
                                SCREEN_HEIGHT * self.scale)}
                            ref={self.canvas.clone()}>
                        </canvas>
                    </div>
                </div>
            </div>
        }
    }
}

impl App {
    // Applies the current display buffer to the canvas.
    fn render_frame(&mut self) {
        let Some(ctx) = self
            .canvas
            .cast::<HtmlCanvasElement>()
            .and_then(|canvas| canvas.get_context("2d").ok())
            .and_then(|ctx| ctx.and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok()))
        else {
            return;
        };

        let clamped_arr = wasm_bindgen::Clamped(self.emulator.display_buffer());
        let Ok(img_data) = ImageData::new_with_u8_clamped_array(clamped_arr, SCREEN_WIDTH) else {
            return;
        };

        ctx.put_image_data(&img_data, 0.0, 0.0).ok();
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

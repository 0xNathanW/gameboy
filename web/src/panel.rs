use crate::constants::{MAX_SCALE, MIN_SCALE};
use gameboy_core::{CpuState, GpuMode, GpuState};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DebugState {
    pub cpu: CpuState,
    pub gpu: GpuState,
    pub tiles: Vec<u8>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PanelProps {
    #[prop_or_default]
    pub is_cgb: bool,

    #[prop_or(AttrValue::from("Unknown"))]
    pub rom_name: AttrValue,

    #[prop_or_default]
    pub rom_size: usize,

    #[prop_or(AttrValue::from("Unknown"))]
    pub cart_type: AttrValue,

    #[prop_or_default]
    pub saveable: bool,

    #[prop_or(AttrValue::from("Unknown"))]
    pub palette: AttrValue,

    #[prop_or_default]
    pub paused: bool,

    #[prop_or(3)]
    pub scale: u32,

    #[prop_or_default]
    pub audio_enabled: bool,

    #[prop_or(50)]
    pub volume: u8,

    pub on_file_upload: Callback<web_sys::File>,
    pub on_pause: Callback<()>,
    pub on_step: Callback<()>,
    pub on_reset: Callback<()>,
    pub on_cycle_palette: Callback<i32>,
    pub on_set_scale: Callback<u32>,
    pub on_toggle_audio: Callback<()>,
    pub on_set_volume: Callback<u8>,

    #[prop_or_default]
    pub debug_state: Option<DebugState>,
}

#[function_component]
pub fn Panel(props: &PanelProps) -> Html {
    let cart_collapsed = use_state(|| false);
    let controls_collapsed = use_state(|| true);
    let settings_collapsed = use_state(|| true);
    let debug_collapsed = use_state(|| false);

    // Clones are cheap here, callback is wrapped Rc.
    let on_file_change = {
        let callback = props.on_file_upload.clone();
        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target_unchecked_into();
            if let Some(file) = input.files().and_then(|list| list.get(0)) {
                callback.emit(file);
            }
        })
    };

    let on_pause_click = {
        let callback = props.on_pause.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(());
        })
    };

    let on_step_click = {
        let callback = props.on_step.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(());
        })
    };

    let on_reset_click = {
        let callback = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(());
        })
    };

    let on_palette_prev = {
        let callback = props.on_cycle_palette.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(-1);
        })
    };

    let on_palette_next = {
        let callback = props.on_cycle_palette.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(1);
        })
    };

    let on_scale_down = {
        let callback = props.on_set_scale.clone();
        let scale = props.scale;
        Callback::from(move |_: MouseEvent| {
            if scale > MIN_SCALE {
                callback.emit(scale - 1);
            }
        })
    };

    let on_scale_up = {
        let callback = props.on_set_scale.clone();
        let scale = props.scale;
        Callback::from(move |_: MouseEvent| {
            if scale < MAX_SCALE {
                callback.emit(scale + 1);
            }
        })
    };

    let on_audio_click = {
        let callback = props.on_toggle_audio.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(());
        })
    };

    let on_volume_down = {
        let callback = props.on_set_volume.clone();
        let volume = props.volume;
        Callback::from(move |_: MouseEvent| {
            callback.emit(volume.saturating_sub(10));
        })
    };

    let on_volume_up = {
        let callback = props.on_set_volume.clone();
        let volume = props.volume;
        Callback::from(move |_: MouseEvent| {
            callback.emit(volume.saturating_add(10).min(100));
        })
    };

    let toggle_cart = {
        let collapsed = cart_collapsed.clone();
        Callback::from(move |_: MouseEvent| collapsed.set(!*collapsed))
    };

    let toggle_controls = {
        let collapsed = controls_collapsed.clone();
        Callback::from(move |_: MouseEvent| collapsed.set(!*collapsed))
    };

    let toggle_settings = {
        let collapsed = settings_collapsed.clone();
        Callback::from(move |_: MouseEvent| collapsed.set(!*collapsed))
    };

    let toggle_debug = {
        let collapsed = debug_collapsed.clone();
        Callback::from(move |_: MouseEvent| collapsed.set(!*collapsed))
    };

    let toggle_class = |collapsed: bool| {
        if collapsed {
            "section-toggle collapsed"
        } else {
            "section-toggle"
        }
    };

    let content_class = |collapsed: bool| {
        if collapsed {
            "section-content collapsed"
        } else {
            "section-content"
        }
    };

    html! {
        <div class="sidebar">
            // Control buttons at top
            <div class="panel-top">
                <div class="button-row">
                    <button onclick={on_pause_click} class="btn">
                        {if props.paused { "Resume" } else { "Pause" }}
                    </button>
                    <button onclick={on_step_click} class="btn">
                        {"Step"}
                    </button>
                    <button onclick={on_reset_click} class="btn">
                        {"Reset"}
                    </button>
                </div>
            </div>

            // Cartridge Section
            <div class="section">
                <div class="section-header" onclick={toggle_cart}>
                    <span class="section-title">{"Cartridge"}</span>
                    <span class={toggle_class(*cart_collapsed)}>{"▼"}</span>
                </div>
                <div class={content_class(*cart_collapsed)}>
                    <div class="panel-section">
                        <div class="info-row">
                            <span class="info-label">{"Name"}</span>
                            <span class="info-value">{&props.rom_name}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">{"Console"}</span>
                            <span class="info-value">
                                {if props.is_cgb { "GBC" } else { "DMG" }}
                            </span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">{"Size"}</span>
                            <span class="info-value">{format!("{} KB", props.rom_size / 1024)}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">{"Type"}</span>
                            <span class="info-value">{&props.cart_type}</span>
                        </div>
                        <div class="info-row">
                            <span class="info-label">{"Battery"}</span>
                            <span class="info-value">{if props.saveable { "Yes" } else { "No" }}</span>
                        </div>
                    </div>

                    <div class="panel-section">
                        <input
                            id="file-input"
                            type="file"
                            multiple=false
                            accept=".gb"
                            onchange={on_file_change}
                        />
                        <label for="file-input" class="btn btn-full">
                            {"Load ROM"}
                        </label>
                    </div>
                </div>
            </div>

            // Controls Section
            <div class="section">
                <div class="section-header" onclick={toggle_controls}>
                    <span class="section-title">{"Controls"}</span>
                    <span class={toggle_class(*controls_collapsed)}>{"▼"}</span>
                </div>
                <div class={content_class(*controls_collapsed)}>
                    <div class="panel-section">
                        <div class="control-item">
                            <span class="control-action">{"D-Pad"}</span>
                            <span class="control-key">{"Arrow Keys"}</span>
                        </div>
                        <div class="control-item">
                            <span class="control-action">{"A Button"}</span>
                            <span class="control-key">{"Z"}</span>
                        </div>
                        <div class="control-item">
                            <span class="control-action">{"B Button"}</span>
                            <span class="control-key">{"X"}</span>
                        </div>
                        <div class="control-item">
                            <span class="control-action">{"Start"}</span>
                            <span class="control-key">{"Enter"}</span>
                        </div>
                        <div class="control-item">
                            <span class="control-action">{"Select"}</span>
                            <span class="control-key">{"Shift"}</span>
                        </div>
                    </div>
                </div>
            </div>

            // Settings Section
            <div class="section">
                <div class="section-header" onclick={toggle_settings}>
                    <span class="section-title">{"Settings"}</span>
                    <span class={toggle_class(*settings_collapsed)}>{"▼"}</span>
                </div>
                <div class={content_class(*settings_collapsed)}>
                    <div class="panel-section">
                        <div class="stepper-row">
                            <span class="stepper-label">{"Scale"}</span>
                            <div class="stepper">
                                <button class="stepper-btn" onclick={on_scale_down}>{"◀"}</button>
                                <span class="stepper-value">{format!("{}x", props.scale)}</span>
                                <button class="stepper-btn" onclick={on_scale_up}>{"▶"}</button>
                            </div>
                        </div>
                    </div>

                    <div class="panel-section">
                        <div class="stepper-row">
                            <span class="stepper-label">{"Palette"}</span>
                            <div class="stepper">
                                <button class="stepper-btn" onclick={on_palette_prev}>{"◀"}</button>
                                <span class="stepper-value">{&props.palette}</span>
                                <button class="stepper-btn" onclick={on_palette_next}>{"▶"}</button>
                            </div>
                        </div>
                    </div>

                    <div class="panel-section">
                        <div class="stepper-row">
                            <span class="stepper-label">{"Audio"}</span>
                            <div class="stepper">
                                <button class="stepper-btn" onclick={on_audio_click}>
                                    {if props.audio_enabled { "On" } else { "Off" }}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="panel-section">
                        <div class="stepper-row">
                            <span class="stepper-label">{"Volume"}</span>
                            <div class="stepper">
                                <button class="stepper-btn" onclick={on_volume_down}>{"◀"}</button>
                                <span class="stepper-value">{format!("{}%", props.volume)}</span>
                                <button class="stepper-btn" onclick={on_volume_up}>{"▶"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Debug Section
            <div class="section">
                <div class="section-header" onclick={toggle_debug}>
                    <span class="section-title">{"Debug"}</span>
                    <span class={toggle_class(*debug_collapsed)}>{"▼"}</span>
                </div>
                <div class={content_class(*debug_collapsed)}>
                    {if let Some(ref debug) = props.debug_state {
                        html! {
                            <>
                                // CPU Registers
                                <div class="panel-section">
                                    <div class="subsection-header">{"CPU Registers"}</div>
                                    <div class="debug-registers">
                                        <div class="register-row">
                                            <span class="register-name">{"PC"}</span>
                                            <span class="register-value wide">{format!("{:04X}", debug.cpu.pc)}</span>
                                            <span class="register-name">{"SP"}</span>
                                            <span class="register-value wide">{format!("{:04X}", debug.cpu.sp)}</span>
                                        </div>
                                        <div class="register-row">
                                            <span class="register-name">{"A"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.a)}</span>
                                            <span class="register-name">{"F"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.f)}</span>
                                        </div>
                                        <div class="register-row">
                                            <span class="register-name">{"B"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.b)}</span>
                                            <span class="register-name">{"C"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.c)}</span>
                                        </div>
                                        <div class="register-row">
                                            <span class="register-name">{"D"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.d)}</span>
                                            <span class="register-name">{"E"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.e)}</span>
                                        </div>
                                        <div class="register-row">
                                            <span class="register-name">{"H"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.h)}</span>
                                            <span class="register-name">{"L"}</span>
                                            <span class="register-value">{format!("{:02X}", debug.cpu.l)}</span>
                                        </div>
                                        <div class="flags-row">
                                            <span class="flags-label">{"Flags"}</span>
                                            <span class={if debug.cpu.flag_z() { "flag set" } else { "flag" }}>{"Z"}</span>
                                            <span class={if debug.cpu.flag_n() { "flag set" } else { "flag" }}>{"N"}</span>
                                            <span class={if debug.cpu.flag_h() { "flag set" } else { "flag" }}>{"H"}</span>
                                            <span class={if debug.cpu.flag_c() { "flag set" } else { "flag" }}>{"C"}</span>
                                        </div>
                                        <div class="status-row">
                                            <span class="status-label">{"IME"}</span>
                                            <span class="status-value">{if debug.cpu.ime { "1" } else { "0" }}</span>
                                            <span class="status-label">{"HALT"}</span>
                                            <span class="status-value">{if debug.cpu.halted { "1" } else { "0" }}</span>
                                        </div>
                                    </div>
                                </div>

                                // GPU State
                                <div class="panel-section">
                                    <div class="subsection-header">{"GPU State"}</div>
                                    <div class="debug-gpu">
                                        <div class="info-row">
                                            <span class="info-label">{"LY"}</span>
                                            <span class="info-value">{debug.gpu.ly}</span>
                                        </div>
                                        <div class="info-row">
                                            <span class="info-label">{"Mode"}</span>
                                            <span class="info-value">{
                                                match debug.gpu.mode {
                                                    GpuMode::HBlank => "HBlank",
                                                    GpuMode::VBlank => "VBlank",
                                                    GpuMode::OamScan => "OAM Scan",
                                                    GpuMode::Drawing => "Drawing",
                                                }
                                            }</span>
                                        </div>
                                        <div class="info-row">
                                            <span class="info-label">{"Scroll"}</span>
                                            <span class="info-value">{format!("{},{}", debug.gpu.scroll_x, debug.gpu.scroll_y)}</span>
                                        </div>
                                        <div class="info-row">
                                            <span class="info-label">{"Window"}</span>
                                            <span class="info-value">{format!("{},{}", debug.gpu.window_x, debug.gpu.window_y)}</span>
                                        </div>
                                        <div class="gpu-flags">
                                            <span class="flags-label">{"Enable"}</span>
                                            <span class={if debug.gpu.lcd_enable { "flag set" } else { "flag" }}>{"LCD"}</span>
                                            <span class={if debug.gpu.bg_enable { "flag set" } else { "flag" }}>{"BG"}</span>
                                            <span class={if debug.gpu.window_enable { "flag set" } else { "flag" }}>{"WIN"}</span>
                                            <span class={if debug.gpu.sprite_enable { "flag set" } else { "flag" }}>{"SPR"}</span>
                                        </div>
                                    </div>
                                </div>

                                // VRAM Tile Viewer
                                <div class="panel-section">
                                    <div class="subsection-header">{"VRAM Tiles"}</div>
                                    <TileViewer tiles={debug.tiles.clone()} />
                                </div>
                            </>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct TileViewerProps {
    pub tiles: Vec<u8>,
}

#[function_component]
pub fn TileViewer(props: &TileViewerProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let tiles = props.tiles.clone();
        use_effect_with(tiles, move |tiles| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Ok(Some(ctx)) = canvas.get_context("2d") {
                    if let Ok(ctx) = ctx.dyn_into::<CanvasRenderingContext2d>() {
                        let clamped = wasm_bindgen::Clamped(tiles.as_slice());
                        if let Ok(img_data) = ImageData::new_with_u8_clamped_array(clamped, 128) {
                            ctx.put_image_data(&img_data, 0.0, 0.0).ok();
                        }
                    }
                }
            }
            || ()
        });
    }

    html! {
        <div class="tile-viewer">
            <canvas
                ref={canvas_ref}
                width="128"
                height="192"
                class="tile-canvas"
            />
        </div>
    }
}

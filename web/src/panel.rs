use crate::constants::{MAX_SCALE, MIN_SCALE};
use web_sys::HtmlInputElement;
use yew::prelude::*;

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

    pub on_file_upload: Callback<web_sys::File>,
    pub on_pause: Callback<()>,
    pub on_cycle_palette: Callback<i32>,
    pub on_set_scale: Callback<u32>,
}

#[function_component]
pub fn Panel(props: &PanelProps) -> Html {
    let cart_collapsed = use_state(|| false);
    let controls_collapsed = use_state(|| true);
    let settings_collapsed = use_state(|| true);

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
            // Pause button at top
            <div class="panel-top">
                <button onclick={on_pause_click} class="btn btn-full">
                    {if props.paused { "Resume" } else { "Pause" }}
                </button>
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
                </div>
            </div>
        </div>
    }
}

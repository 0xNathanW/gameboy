use gloo::{console::log, utils::document};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct InfoProps {
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
    pub pallette: AttrValue,
}

#[function_component]
pub fn Panel(props: &InfoProps) -> Html {
    // A callback to show content of clicked tab, and hide the rest
    let cb = Callback::from(move |tab: String| {
        let tabs = document().get_elements_by_class_name("panel-content");

        for i in 0..tabs.length() {
            if let Some(tab) = tabs.item(i) {
                log!(tab.id());
                tab.set_attribute("style", "display:none").unwrap();
            }
        }

        let active = document()
            .get_element_by_id(&tab)
            .unwrap()
            .set_attribute("style", "display:block")
            .unwrap();
    });

    let info_cb = cb.clone();
    let about_cb = cb.clone();

    html! {
        <div class="panel">
            <div class="panel-inner">
                <div class="panel-button-row">

                    <input
                        class="panel-input"
                        onclick={move |_| { info_cb.emit("info".to_string()) }}
                        type="radio"
                        name="tabs"
                        value="info"
                        id="info-tab"
                        checked=true
                    />
                    <label for="info-tab" class="panel-buttons">{"Info"}</label>

                    <input
                        class="panel-input"
                        onclick={move |_| { about_cb.emit("about".to_string()) }}
                        type="radio"
                        name="tabs"
                        value="about"
                        id="about-tab"
                    />
                    <label for="about-tab" class="panel-buttons">{"About"}</label>

                    <input
                        class="panel-input"
                        onclick={move |_| { cb.emit("controls".to_string()) }}
                        type="radio"
                        name="tabs"
                        value="controls"
                        id="controls-tab"
                    />
                    <label for="controls-tab" class="panel-buttons">{"Controls"}</label>
                </div>

                <div class="panel-content" id="info">
                    <p>
                        {"Console: "}
                        <span style="float:right;">
                            {if props.is_cgb { "Gameboy Colour" } else { "Gameboy Classic" }}
                        </span>
                    </p>
                    <p>
                        {"ROM Name: "}
                        <span style="float:right;">
                            {props.rom_name.clone()}
                        </span>
                    </p>
                    <p>
                        {"ROM Size: "}
                        <span style="float:right;">
                            {format!("{} KB", props.rom_size / 1024)}
                        </span>
                    </p>
                    <p>
                        {"Cart Type: "}
                        <span style="float:right;">
                            {props.cart_type.clone()}
                        </span>
                    </p>
                    <p>
                        {"Saveable: "}
                        <span style="float:right;">
                            {if props.saveable { "Yes" } else { "No" }}
                        </span>
                    </p>
                    <p>
                        {"Pallette: "}
                        <span style="float:right;">
                            {props.pallette.clone()}
                        </span>
                    </p>
                </div>

                <div class="panel-content" id="about" style="display:none">
                    <About />
                </div>

                <div class="panel-content" id="controls" style="display:none">
                    <Controls />
                </div>
            </div>
        </div>
    }
}

#[function_component]
fn About() -> Html {
    html! {
        <>
            <p>{"A Gameboy emulator built in Rust and delivered to the web using WebAssembly and Yew."}</p>
            <p>{"Link to repository: "}
                <a
                    href="https://github.com/0xNathanW/gameboy"
                    target="_blank"
                    rel="noopener noreferrer"
                >{"github.com/0xNathanW/gameboy"}</a>
            </p>
            <p>{"TODO (hopefully):"}</p>
            <ul>
                <li>{"Web audio support."}</li>
                <li>{"Save states."}</li>
                <li>{"Gameboy Color support."}</li>
                <li>{"Fix scaling/quality tradeoff."}</li>
                <li>{"Debugging tools."}</li>
            </ul>
            <p>{"Made by: Nathan W."}</p>
        </>
    }
}

#[function_component]
fn Controls() -> Html {
    html! {
        <div class="controls">
            <div class="item">
                <svg xmlns="http://www.w3.org/2000/svg" width="70" height="70" fill="currentColor" class="bi bi-dpad" viewBox="0 0 16 16">
                    <path d="m7.788 2.34-.799 1.278A.25.25 0 0 0 7.201 4h1.598a.25.25 0 0 0 .212-.382l-.799-1.279a.25.25 0 0 0-.424 0Zm0 11.32-.799-1.277A.25.25 0 0 1 7.201 12h1.598a.25.25 0 0 1 .212.383l-.799 1.278a.25.25 0 0 1-.424 0ZM3.617 9.01 2.34 8.213a.25.25 0 0 1 0-.424l1.278-.799A.25.25 0 0 1 4 7.201V8.8a.25.25 0 0 1-.383.212Zm10.043-.798-1.277.799A.25.25 0 0 1 12 8.799V7.2a.25.25 0 0 1 .383-.212l1.278.799a.25.25 0 0 1 0 .424Z"/>
                    <path d="M6.5 0A1.5 1.5 0 0 0 5 1.5v3a.5.5 0 0 1-.5.5h-3A1.5 1.5 0 0 0 0 6.5v3A1.5 1.5 0 0 0 1.5 11h3a.5.5 0 0 1 .5.5v3A1.5 1.5 0 0 0 6.5 16h3a1.5 1.5 0 0 0 1.5-1.5v-3a.5.5 0 0 1 .5-.5h3A1.5 1.5 0 0 0 16 9.5v-3A1.5 1.5 0 0 0 14.5 5h-3a.5.5 0 0 1-.5-.5v-3A1.5 1.5 0 0 0 9.5 0h-3ZM6 1.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 .5.5v3A1.5 1.5 0 0 0 11.5 6h3a.5.5 0 0 1 .5.5v3a.5.5 0 0 1-.5.5h-3a1.5 1.5 0 0 0-1.5 1.5v3a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-3A1.5 1.5 0 0 0 4.5 10h-3a.5.5 0 0 1-.5-.5v-3a.5.5 0 0 1 .5-.5h3A1.5 1.5 0 0 0 6 4.5v-3Z"/>
                </svg>
            </div>

            <div class="item" id="arrows">
                <button class="kbc-button kbc-button-sm">{"🡡"}</button>
                <div id="arrow-bottom">
                    <button class="kbc-button kbc-button-sm">{"🡠"}</button>
                    <button class="kbc-button kbc-button-sm">{"🡣"}</button>
                    <button class="kbc-button kbc-button-sm">{"🡢"}</button>
                </div>
            </div>

            <div class="item">
                <svg width="50" height="50">
                    <circle cx="25" cy="25" r="15" fill="#B53737" stroke="black" stroke-width="3"/>
                    <text x="50%" y="50%" text-anchor="middle" fill="white" font-size="14px" font-family="Arial" font-weight="bolder" dy=".3em">{"A"}</text>
                </svg>

                <svg width="50" height="50">
                    <circle cx="25" cy="25" r="15" fill="#B53737" stroke="black" stroke-width="3"/>
                    <text x="50%" y="50%" text-anchor="middle" fill="white" font-size="14px" font-family="Arial" font-weight="bolder" dy=".3em">{"B"}</text>
                </svg>

            </div>

            <div class="item">
                <button class="kbc-button kbc-button-sm">{"Z"}</button>
                <button class="kbc-button kbc-button-sm">{"X"}</button>
            </div>

            <div class="item">
                <button class="start-select">{"Start"}</button>
                <button class="start-select">{"Select"}</button>
            </div>

            <div class="item">
                <button class="kbc-button kbc-button-sm">{"Enter"}</button>
                <button class="kbc-button kbc-button-sm">{"Shift"}</button>
            </div>

        </div>
    }
}

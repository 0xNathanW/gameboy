use yew::prelude::*;
use gloo::utils::document;
use gloo::console::log;
use web_sys::{
    Document,
    HtmlCollection,
};

#[function_component]
pub fn Panel() -> Html {
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
                    <h3>{"Info"}</h3>
                </div>

                <div class="panel-content" id="about" style="display:none">
                    <h3>{"About"}</h3>
                </div>

                <div class="panel-content" id="controls" style="display:none">
                    <h3>{"Controls"}</h3>
                </div>
            </div>
        </div>
    }
}



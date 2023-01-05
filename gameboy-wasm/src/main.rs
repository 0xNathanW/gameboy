#![allow(unused)]
use yew::prelude::*;
use canvas::Canvas;

mod canvas;
mod emulator;

fn main() {
    yew::Renderer::<App>::new().render();
}

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        App
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        html! {
            <>
                <div class="upper">
                    <h1>{ "GameBoy.WASM" }</h1>
                    <div class="canvas">
                        <Canvas />
                        <div class="button-row">
                            <FileInput />
                            <button>{"Pallette"}</button>
                            <button>{"Scale"}</button>
                        </div>
                    </div>        
                </div>
            </>
        }
    }
}

#[function_component]
fn FileInput() -> Html {
    html! {
        <>
            <label for="file-input" class="file-input-label">
                <i class="fas fa-cloud"></i>
                <span>{"Upload ROM"}</span>
            </label>
            <input
                id="file-input"
                type="file"
                multiple=false
                accept=".gb"
                onchange={ctx.link().to_parent()}
            />
        </>
    }
}
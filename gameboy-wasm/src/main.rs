#[allow(unused)]

use yew::prelude::*;

mod gameboy;
use gameboy::Emulator;

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
            <h1>{"GameBoy.WASM"}</h1>
            <h3>{"A GameBoy emulator written in Rust and WASM"}</h3>
            <Emulator />
            </>
        }
    }
}

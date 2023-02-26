use cfg_if::cfg_if;

pub mod app;
pub mod pokemon;
pub mod poll;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::{wasm_bindgen, Closure};
        use crate::app::*;
        use leptos::*;

        #[wasm_bindgen(raw_module = "../truffle-wrapper.js")]
        extern "C" {
            pub fn set_callback(c: &Closure<dyn Fn(String)>);
            pub fn minimize();
            pub fn maximize();
        }

        #[wasm_bindgen]
        pub fn hydrate() {
            console_error_panic_hook::set_once();
            _ = console_log::init_with_level(log::Level::Debug);

            leptos::mount_to_body(|cx| {
                view! { cx,  <App/> }
            });
        }

    } else if #[cfg(feature = "ssr")] {
        use leptos::ServerFn;
        pub fn register_server_functions() {
            _ = app::GetPokemon::register();
            _ = poll::GetIsStreamer::register();
            _ = poll::GetCurrentPoll::register();
            _ = poll::SetCurrentVote::register();
            _ = poll::GetCurrentVote::register();
            _ = poll::GeneratePoll::register();
            _ = poll::GetPollResult::register();
            _ = poll::EndPoll::register();
        }

        pub fn minimize() {}

        pub fn maximize() {}
    }
}

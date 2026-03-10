use leptos::prelude::*;

mod models;
mod disasm;
mod database;
mod app;

use crate::app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::logging::log!("Starting main");
    mount_to_body(|| view! { <App /> });
}

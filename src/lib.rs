#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::eval_order_dependence)]

use wasm_bindgen::prelude::*;
use yew::utils::document;

use app::App;

pub mod app;
pub mod components;
pub mod routes;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("Launch App!");
    let element = document().get_element_by_id("main").expect("failed get_element_by_id");
    yew::App::<App>::new().mount(element);
}

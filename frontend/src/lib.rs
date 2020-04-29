mod utils;
extern crate console_log;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn app_main() {
    utils::set_panic_hook();
    console_log::init_with_level(log::Level::Debug).expect("problems initing logger");
    log::debug!("console log initialised");

    greet("alert-ee");
}

#[wasm_bindgen]
pub fn greet(subj: &str) {
    alert(&format!("Hello, {}!", subj));
}


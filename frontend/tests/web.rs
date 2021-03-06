//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
extern crate wasm_bindgen_futures;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
fn fail() {
    assert_eq!(1, 2);
}

#[wasm_bindgen_test(async)]
async fn my_async_test() {
    // Create a promise that is ready on the next tick of the micro task queue.
    let promise = js_sys::Promise::resolve(&JsValue::from(42));

    // Convert that promise into a future and make the test wait on it.
    let x = JsFuture::from(promise).await.unwrap();
    assert_eq!(x, 42);
}

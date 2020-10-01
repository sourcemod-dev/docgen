extern crate cfg_if;
extern crate wasm_bindgen;
extern crate wee_alloc;

mod utils;

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, wasm-worker!".to_string()
}

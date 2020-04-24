mod utils;

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
pub fn greet() {
    alert("Hello, wasm-ppm!");
}

#[wasm_bindgen]
pub fn image_passthrough(data: &[u8]) -> Vec<u8> {

    alert(&format!("image data: {:?}", data));
    let mut ret = Vec::new();
    ret.extend_from_slice(data);

    return ret;
}

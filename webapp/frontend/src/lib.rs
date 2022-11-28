use wasm_bindgen::{prelude::*};

mod utils;
use utils::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn hash(value: String) -> String {
    let data = value.as_bytes().to_vec();

    let digest = sha256(&data);

    to_base64(digest)
}
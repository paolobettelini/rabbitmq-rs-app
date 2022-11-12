#![macro_use] // export macros

use wasm_bindgen::prelude::*;
use sha2::Digest;
pub use base64::encode as to_base64;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub fn sha256(data: &Vec<u8>) -> Vec<u8> {
    let res = sha2::Sha256::digest(data);

    // let x: [u8; 32] = res.as_slice().try_into().unwrap();

    res.as_slice().to_vec()
}
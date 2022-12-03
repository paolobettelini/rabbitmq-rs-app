use wasm_bindgen::{prelude::*};

mod utils;
use utils::*;
use protocol::validation;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn hash(value: String) -> String {
    let data = value.as_bytes().to_vec();

    let digest = sha256(&data);

    to_base64(digest)
}

#[wasm_bindgen]
pub fn validate_email(value: String) -> bool {
    validation::validate_email(&value)
}

#[wasm_bindgen]
pub fn validate_password(value: String) -> bool {
    validation::validate_password(&value)
}

#[wasm_bindgen]
pub fn validate_username(value: String) -> bool {
    validation::validate_username(&value)
}
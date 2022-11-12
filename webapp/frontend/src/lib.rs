use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use wasm_bindgen::{JsCast, prelude::*};

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

#[wasm_bindgen]
pub fn translate(data: Vec<u8>) -> String {
    // Translate protocol packet and return .to_string
    // (implement Display trait)
    
    String::from("")
}

#[wasm_bindgen]
pub fn start_websocket() -> Result<(), JsValue> {
    let ws = WebSocket::new("wss://echo.websocket.events")?;
    
    // Messages -> ArrayBuffer
    // Images -> Blob
    
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    
    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            on_arraybuf_message(buf);
        } else {
            console_log!("Unknown type of message receivd");
        }
    });
    
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();
    
    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();
    
    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        console_log!("socket opened");
        
        /*
        match cloned_ws.send_with_u8_array(&vec![0, 1, 2, 3]) {
            Ok(_) => console_log!("binary message successfully sent"),
            Err(err) => console_log!("error sending message: {:?}", err),
        }*/
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    
    Ok(())
}

fn on_arraybuf_message(data: js_sys::ArrayBuffer) {
    console_log!("Received ArrayBuffer: {:?}", data);
    
    let raw = js_sys::Uint8Array::new(&data).to_vec();
}
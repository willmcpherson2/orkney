use bevy::prelude::*;
use shared::ServerMessage;
use std::{collections::VecDeque, sync::Arc, sync::Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

#[derive(Resource)]
pub struct Inbox {
    pub queue: Arc<Mutex<VecDeque<ServerMessage>>>,
}

impl Inbox {
    pub fn new(ws: &WebSocket) -> Inbox {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let queue_clone = queue.clone();
        let on_message = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(array) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let bytes = js_sys::Uint8Array::new(&array).to_vec();
                match bincode::deserialize::<ServerMessage>(&bytes) {
                    Ok(msg) => {
                        info!("received message: {:?}", msg);
                        queue_clone.lock().unwrap().push_back(msg);
                    }
                    Err(err) => {
                        info!("message error: {:?}", err);
                    }
                }
            } else {
                info!("received unknown: {:?}", e.data());
            }
        });
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        Inbox { queue }
    }
}

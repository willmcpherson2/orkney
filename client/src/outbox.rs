use bevy::prelude::*;
use shared::ClientMessage;
use std::{collections::VecDeque, sync::Arc, sync::Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

pub struct Outbox {
    pub ws: WebSocket,
    pub queue: Arc<Mutex<VecDeque<ClientMessage>>>,
}

impl Outbox {
    pub fn new(ws: &WebSocket) -> Outbox {
        let ws_clone = ws.clone();
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let queue_clone = queue.clone();
        let on_open = Closure::<dyn FnMut(_)>::new(move |_: MessageEvent| {
            info!("websocket opened");
            let mut queue = queue_clone.lock().unwrap();
            while let Some(msg) = queue.pop_front() {
                info!("sending message: {:?}", msg);
                let bytes = bincode::serialize(&msg).unwrap();
                ws_clone.send_with_u8_array(&bytes).unwrap();
            }
        });
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();
        Outbox {
            ws: ws.clone(),
            queue,
        }
    }

    pub fn send(&mut self, msg: ClientMessage) {
        if self.is_open() {
            info!("sending message: {:?}", msg);
            let bytes = bincode::serialize(&msg).unwrap();
            self.ws.send_with_u8_array(&bytes).unwrap();
        } else {
            info!("queueing message: {:?}", msg);
            self.queue.lock().unwrap().push_back(msg);
        }
    }

    pub fn is_open(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}

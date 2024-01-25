use bevy::prelude::*;
use shared::{ClientMessage, ClientId, ServerMessage};
use std::{collections::VecDeque, sync::Arc, sync::Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

fn main() {
    let ws = WebSocket::new("ws://localhost:3000/ws").unwrap();
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let inbox = Inbox::new(&ws);
    let outbox = Outbox::new(&ws);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(inbox)
        .insert_non_send_resource(outbox)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Resource)]
struct Inbox {
    queue: Arc<Mutex<VecDeque<ServerMessage>>>,
}

impl Inbox {
    fn new(ws: &WebSocket) -> Inbox {
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

struct Outbox {
    ws: WebSocket,
    queue: Arc<Mutex<VecDeque<ClientMessage>>>,
}

impl Outbox {
    fn new(ws: &WebSocket) -> Outbox {
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

    fn send(&mut self, msg: ClientMessage) {
        if self.is_open() {
            info!("sending message: {:?}", msg);
            let bytes = bincode::serialize(&msg).unwrap();
            self.ws.send_with_u8_array(&bytes).unwrap();
        } else {
            info!("queueing message: {:?}", msg);
            self.queue.lock().unwrap().push_back(msg);
        }
    }

    fn is_open(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outbox: NonSendMut<Outbox>,
) {
    outbox.send(ClientMessage::RequestClientId);

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn update(mut commands: Commands, inbox: ResMut<Inbox>) {
    for msg in inbox.queue.lock().unwrap().drain(..) {
        match msg {
            ServerMessage::NewClientId(id) => {
                info!("received client id: {:?}", id);
                commands.insert_resource(id);
            }
        }
    }
}

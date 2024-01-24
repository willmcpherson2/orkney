use bevy::prelude::*;
use shared::{ClientMessage, ServerMessage};
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, start_websocket)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

fn start_websocket() {
    let ws = WebSocket::new("ws://localhost:3000/ws").unwrap();

    let on_message = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            match serde_json::from_str(&text.as_string().unwrap()) {
                Ok(msg) => {
                    info!("received message: {:?}", msg);
                    receive(msg);
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

    let msg = ClientMessage::RequestId;
    let json = serde_json::to_string(&msg).unwrap();

    let ws_sender = ws.clone();
    let on_open = Closure::<dyn FnMut()>::new(move || match ws_sender.send_with_str(&json) {
        Ok(_) => info!("sent message: {:?}", msg),
        Err(err) => info!("error sending message: {:?}", err),
    });
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
}

fn receive(msg: ServerMessage) {
    match msg {
        ServerMessage::NewId(id) => {
            info!("new ID: {:?}", id);
        }
    }
}

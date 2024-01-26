mod inbox;
mod outbox;

use bevy::prelude::*;
use inbox::Inbox;
use outbox::Outbox;
use shared::{ClientMessage, ServerMessage};
use web_sys::WebSocket;

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

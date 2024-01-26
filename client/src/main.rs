mod inbox;
mod outbox;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use inbox::Inbox;
use outbox::Outbox;
use shared::{ClientMessage, ServerMessage};
use web_sys::WebSocket;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(Resource)]
struct Username(String);

#[derive(Resource)]
struct Lobby(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_state::<AppState>()
        .add_systems(OnEnter(AppState::Menu), enter_menu)
        .add_systems(Update, update_menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnEnter(AppState::Game), enter_game)
        .add_systems(Update, update_game.run_if(in_state(AppState::Game)))
        .run();
}

fn enter_menu(mut commands: Commands) {
    commands.insert_resource(Username("Anonymous".to_string()));
    commands.insert_resource(Lobby("Public".to_string()));
}

fn update_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut username: ResMut<Username>,
    mut lobby: ResMut<Lobby>,
) {
    let ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Menu");
        ui.horizontal(|ui| {
            ui.label("Theme:");
            egui::global_dark_light_mode_buttons(ui);
        });
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut username.0);
        });
        ui.horizontal(|ui| {
            ui.label("Lobby:");
            ui.text_edit_singleline(&mut lobby.0);
        });
        if ui.add(egui::Button::new("Join")).clicked() {
            next_state.set(AppState::Game);
        }
    });
}

fn connect(world: &mut World) {
    let ws = WebSocket::new("ws://localhost:3000/ws").unwrap();
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let inbox = Inbox::new(&ws);
    let mut outbox = Outbox::new(&ws);
    outbox.send(ClientMessage::RequestClientId);

    world.insert_resource(inbox);
    world.insert_non_send_resource(outbox);
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

fn enter_game(
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

fn update_game() {}

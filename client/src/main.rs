use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bincode::{deserialize, serialize};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use shared::{ClientMessage, Lobby, ServerMessage, Username};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Game,
}

struct Sender(WsSender);

impl Sender {
    fn send(&mut self, msg: ClientMessage) {
        let bytes = serialize(&msg).unwrap();
        self.0.send(WsMessage::Binary(bytes));
    }
}

#[derive(Resource)]
struct Receiver(Mutex<WsReceiver>);

impl Receiver {
    fn try_recv(&self) -> Option<WsEvent> {
        self.0.lock().unwrap().try_recv()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_state::<AppState>()
        .add_systems(OnEnter(AppState::Menu), enter_menu)
        .add_systems(Update, update_menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnEnter(AppState::Game), join_game)
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
    mut exit: EventWriter<AppExit>,
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
        if ui.add(egui::Button::new("Exit")).clicked() {
            exit.send(AppExit);
        }
    });
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

fn join_game(world: &mut World) {
    let lobby = world.get_resource::<Lobby>().unwrap();
    let username = world.get_resource::<Username>().unwrap();

    let url = format!("ws://localhost:3000/join/{}/{}", &lobby.0, &username.0);
    info!("connecting to {}", url);
    let (sender, receiver) = ewebsock::connect(url).unwrap();

    world.insert_non_send_resource(Sender(sender));
    world.insert_resource(Receiver(Mutex::new(receiver)));
}

fn update_game(mut sender: NonSendMut<Sender>, receiver: ResMut<Receiver>) {
    while let Some(event) = receiver.try_recv() {
        match event {
            WsEvent::Message(msg) => match msg {
                WsMessage::Binary(bytes) => match deserialize::<ServerMessage>(&bytes) {
                    Ok(msg) => {
                        info!("received message: {:?}", msg);
                    }
                    Err(err) => {
                        info!("message error: {:?}", err);
                    }
                },
                msg => info!("non-binary message: {:?}", msg),
            },
            WsEvent::Opened => {
                let msg = ClientMessage::HelloFromClient;
                info!("sending message: {:?}", msg);
                sender.send(msg);
            }
            WsEvent::Closed => {
                info!("websocket closed");
            }
            WsEvent::Error(err) => {
                info!("websocket error: {:?}", err);
            }
        }
    }
}

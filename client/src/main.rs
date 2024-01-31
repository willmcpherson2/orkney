use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_matchbox::prelude::*;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Game,
}

type Socket = MatchboxSocket<SingleChannel>;

#[derive(Serialize, Deserialize, Debug, Resource)]
struct Lobby(String);

#[derive(Serialize, Deserialize, Debug, Resource)]
struct Username(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Message {
    Hello,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Orkney".to_string(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_state::<AppState>()
        .add_systems(OnEnter(AppState::Menu), enter_menu)
        .add_systems(Update, update_menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnEnter(AppState::Game), (join_game, enter_game))
        .add_systems(
            Update,
            (handle_socket, handle_keys).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), leave_game)
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

fn join_game(mut commands: Commands, _username: Res<Username>, _lobby: Res<Lobby>) {
    let url = "ws://0.0.0.0:3536/orkney";
    let socket = MatchboxSocket::new_reliable(url);
    commands.insert_resource(socket);
    info!("connected to {url}");
}

fn enter_game() {}

fn handle_socket(mut socket: ResMut<Socket>) {
    socket.update_peers();
    for (peer, bytes) in socket.receive() {
        let msg = deserialize::<Message>(bytes.as_ref()).unwrap();
        let id = socket.id().unwrap();
        info!("local {:?} <- peer {:?}: {:?}", id, peer, msg);
    }
}

fn handle_keys(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut socket: ResMut<Socket>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
    if keyboard_input.just_pressed(KeyCode::M) {
        info!("sending messages...");
        let peers = socket.connected_peers().collect::<Vec<PeerId>>();
        for peer in peers {
            let msg = Message::Hello;
            let bytes = serialize(&msg).unwrap();
            socket.send(bytes.into_boxed_slice(), peer);
            let id = socket.id().unwrap();
            info!("local {:?} -> peer {:?}: {:?}", id, peer, msg);
        }
    }
}

fn leave_game(mut commands: Commands) {
    commands.remove_resource::<Socket>();
}

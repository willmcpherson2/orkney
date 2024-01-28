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

fn join_game(world: &mut World) {
    let lobby = world.get_resource::<Lobby>().unwrap();
    let username = world.get_resource::<Username>().unwrap();

    let url = format!("ws://localhost:3000/join/{}/{}", &lobby.0, &username.0);
    info!("connecting to {}", url);
    let (sender, receiver) = ewebsock::connect(url).unwrap();

    world.insert_non_send_resource(Sender(sender));
    world.insert_resource(Receiver(Mutex::new(receiver)));
}

fn enter_game() {}

fn handle_socket(mut sender: NonSendMut<Sender>, receiver: ResMut<Receiver>) {
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

fn handle_keys(mut next_state: ResMut<NextState<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
}

fn leave_game(world: &mut World) {
    world.remove_non_send_resource::<Sender>();
    world.remove_resource::<Receiver>();
}

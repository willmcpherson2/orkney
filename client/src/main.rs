use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_matchbox::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Message {
    Hello,
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Waypoint(Option<Vec3>);

#[derive(SystemParam)]
struct ScreenToGround<'w, 's> {
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    ground_transform: Query<'w, 's, &'static GlobalTransform, With<Ground>>,
    window: Query<'w, 's, &'static Window>,
}

impl<'w, 's> ScreenToGround<'w, 's> {
    fn ground_transform(&self) -> GlobalTransform {
        *self.ground_transform.single()
    }

    fn position(&self) -> Option<Vec3> {
        let (camera, camera_transform) = self.camera.single();
        let ground_transform = self.ground_transform();
        let cursor_position = self.window.single().cursor_position()?;

        let ray = camera.viewport_to_world(camera_transform, cursor_position)?;
        let distance = ray.intersect_plane(
            ground_transform.translation(),
            InfinitePlane3d::new(ground_transform.up()),
        )?;
        Some(ray.get_point(distance))
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
        .add_plugins(PanOrbitCameraPlugin)
        .init_state::<AppState>()
        .add_systems(Startup, startup)
        .add_systems(Update, update_menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnEnter(AppState::Game), join_game)
        .add_systems(
            Update,
            (
                handle_socket,
                handle_keys,
                handle_mouse,
                update_player,
                update_camera,
                draw_cursor,
            )
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), leave_game)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Lobby("Public".to_string()));

    commands.spawn((
        Ground,
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(20., 20.)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
    ));

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::default()),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(1.0, 0.5, 1.0),
        ..default()
    });

    commands.spawn((
        Player,
        Waypoint(None),
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::srgb(0.6, 0.7, 0.9)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
    ));

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(1.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(5.0, 5.0, 5.0),
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            ..default()
        },
    ));
}

fn update_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
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
            ui.label("Lobby:");
            ui.text_edit_singleline(&mut lobby.0);
        });
        if ui.add(egui::Button::new("Join")).clicked() {
            next_state.set(AppState::Game);
        }
        if ui.add(egui::Button::new("Exit")).clicked() {
            exit.send(AppExit::Success);
        }
    });
}

fn join_game(mut commands: Commands, lobby: Res<Lobby>) {
    let url = format!("ws://0.0.0.0:3001/{}", lobby.0);
    let socket = MatchboxSocket::new_reliable(&url);
    commands.insert_resource(socket);
    info!("connected to {url}");
}

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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut socket: ResMut<Socket>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
    if keyboard_input.just_pressed(KeyCode::KeyM) {
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

fn handle_mouse(
    mut events: EventReader<MouseButtonInput>,
    screen_to_ground: ScreenToGround,
    mut waypoint: Query<&mut Waypoint, With<Player>>,
) {
    let ground_transform = screen_to_ground.ground_transform();

    for event in events.read() {
        match (event.button, event.state) {
            (MouseButton::Left, ButtonState::Released) => {
                if let Some(ground_position) = screen_to_ground.position() {
                    let ground_position = ground_position + ground_transform.up() * 0.5;
                    for mut waypoint in waypoint.iter_mut() {
                        waypoint.0 = Some(ground_position);
                    }
                }
            }
            (MouseButton::Right, ButtonState::Released) => {
                for mut waypoint in waypoint.iter_mut() {
                    waypoint.0 = None;
                }
            }
            _ => {}
        }
    }
}

fn update_player(
    waypoint: Query<&Waypoint, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Some(waypoint) = waypoint.single().0 else {
        return;
    };
    let mut transform = player_transform.single_mut();

    let direction = waypoint - transform.translation;
    let distance = direction.length();
    let speed = 10.0;
    let velocity = direction.normalize() * speed;
    let delta = velocity * time.delta_seconds();
    if delta.length() < distance {
        transform.translation += delta;
    } else {
        transform.translation = waypoint;
    }
}

fn update_camera(
    player_transform: Query<&Transform, With<Player>>,
    mut camera: Query<&mut PanOrbitCamera>,
) {
    let player = player_transform.single().translation;
    let mut camera = camera.single_mut();
    camera.target_focus = player;
}

fn draw_cursor(screen_to_ground: ScreenToGround, mut gizmos: Gizmos) {
    let ground_transform = screen_to_ground.ground_transform();
    let Some(ground_position) = screen_to_ground.position() else {
        return;
    };

    gizmos.circle(
        ground_position + ground_transform.up() * 0.01,
        ground_transform.up(),
        0.2,
        Color::WHITE,
    );
}

fn leave_game(mut commands: Commands) {
    commands.remove_resource::<Socket>();
}

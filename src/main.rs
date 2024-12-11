use bevy::prelude::*;

#[derive(Resource)]
pub struct Health {
    pub level: f32,
}

#[derive(Component)]
pub struct PlayerHealth {
    pub level: f32,
}

#[derive(Component)]
pub struct BadGuyMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fox GT".to_string(),
                resolution: (854., 480.).into(),
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Health { level: 100. })
        .add_systems(Startup, setup)
        .add_systems(Update, keyboad_input_change_system)
        .add_systems(Update, keyboard_input_system)
        .run();
}

fn setup(mut commands: Commands, assest_server: Res<AssetServer>) {
    commands.spawn((Camera2d { ..default() },));

    commands
        .spawn((
            PlayerHealth { level: 900. },
            BadGuyMarker,
            Sprite {
                image: assest_server.load("bigfox.png"),
                ..default()
            },
        ))
        .insert(Transform::from_xyz(0., 0., 0.));
}

fn keyboard_input_system(
    health: Res<Health>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerHealth, With<BadGuyMarker>>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        info!("Hello")
    } else if keyboard.just_pressed(KeyCode::KeyH) {
        info!("health = {}", health.level);

        for player_health in player_query.iter() {
            info!("Player health = {}", player_health.level);
        }
    }
}

fn keyboad_input_change_system(
    time: Res<Time>,
    mut health: ResMut<Health>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<BadGuyMarker>>,
) {
    if keyboard.just_pressed(KeyCode::KeyJ) {
        health.level -= 10.;
    }
    for mut player_transform in player_query.iter_mut() {
        if keyboard.pressed(KeyCode::KeyD) {
            player_transform.translation.x += 100. * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            player_transform.translation.x -= 100. * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyW) {
            player_transform.translation.y += 100. * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            player_transform.translation.y -= 100. * time.delta_secs();
        }
    }
}

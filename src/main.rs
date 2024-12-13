use bevy::prelude::*;
use rand::Rng;

const WINDOW_Y: f32 = 480.;
const WINDOW_X: f32 = 854.;
const SPEED_X: f32 = 600.;

#[derive(Resource)]
pub struct Health {
    pub level: f32,
}

#[derive(Resource)]
pub struct SpawnThingTimer {
    timer: Timer,
}

#[derive(Component)]
pub struct PlayerHealth {
    pub level: f32,
}

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct RoadMarker;

#[derive(Component)]
pub struct ObstacleMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fox GT".to_string(),
                resolution: (WINDOW_X, WINDOW_Y).into(),
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Health { level: 100. })
        .insert_resource(SpawnThingTimer {
            timer: Timer::from_seconds(1., TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, keyboad_input_change_system)
        .add_systems(Update, keyboard_input_system)
        // .add_systems(Update, rotate_system)
        .run();
}

fn setup(mut commands: Commands, assest_server: Res<AssetServer>) {
    commands.spawn((Camera2d { ..default() },));

    commands
        .spawn((
            PlayerHealth { level: 900. },
            PlayerMarker,
            Sprite {
                flip_x: false,
                image: assest_server.load("car.png"),
                ..default()
            },
        ))
        .insert(Transform::from_xyz(0., -70., 0.));

    commands.spawn(Sprite {
        flip_x: false,
        image: assest_server.load("road.png"),
        ..default()
    });

    for i in 0..20 {
        commands
            .spawn((
                RoadMarker,
                Sprite {
                    flip_x: false,
                    image: assest_server.load("road-yellow-line.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(
                (i as f32) * 56. - (WINDOW_X / 2.),
                0.,
                0.,
            ));
    }
}

fn keyboard_input_system(
    health: Res<Health>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerHealth, With<PlayerMarker>>,
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
    mut commands: Commands,
    assest_server: Res<AssetServer>,
    time: Res<Time>,
    mut spawn_thing_timer: ResMut<SpawnThingTimer>,
    mut health: ResMut<Health>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut road_query: Query<(&mut Transform), With<RoadMarker>>,
    mut obstacle_query: Query<
        (Entity, &mut Transform),
        (
            With<ObstacleMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut player_query: Query<
        (&mut Transform, &mut Sprite),
        (
            With<PlayerMarker>,
            Without<RoadMarker>,
            Without<ObstacleMarker>,
        ),
    >,
) {
    if keyboard.just_pressed(KeyCode::KeyJ) {
        health.level -= 10.;
    }

    let right = keyboard.pressed(KeyCode::KeyD);
    let left = keyboard.pressed(KeyCode::KeyA);
    let up = keyboard.pressed(KeyCode::KeyW);
    let down = keyboard.pressed(KeyCode::KeyS);

    let up_just_pressed = keyboard.just_pressed(KeyCode::KeyW);
    let down_just_pressed = keyboard.just_pressed(KeyCode::KeyS);

    for mut road_transform in road_query.iter_mut() {
        if road_transform.translation.x > (WINDOW_X / 2.) + 42. {
            road_transform.translation.x = -(WINDOW_X / 2.) - 42.
        } else if road_transform.translation.x < -(WINDOW_X / 2.) - 42. {
            road_transform.translation.x = (WINDOW_X / 2.) + 42.
        }
        if right {
            road_transform.translation.x -= SPEED_X * time.delta_secs();
        }
        if left {
            road_transform.translation.x += SPEED_X * time.delta_secs();
        }
    }

    for (mut player_transform, mut player_sprite) in player_query.iter_mut() {
        if right {
            player_sprite.flip_x = true;
        }
        if left {
            player_sprite.flip_x = false;
        }
        if up_just_pressed {
            player_transform.translation.y += 140.;
        }
        if down_just_pressed {
            player_transform.translation.y -= 140.;
        }
    }

    let (mut player_transform, _) = player_query.single_mut();

    let mut allow_obstable_spawn = true;
    for (obstable_entity, mut obstable_transform) in obstacle_query.iter_mut() {
        if player_transform.translation.x < obstable_transform.translation.x + 2.
            && player_transform.translation.x > obstable_transform.translation.x - 2.
        {
            // there was a collisiion
            info!("Game over!");

            // player_transform.translation.y += 140.;
        }

        if obstable_transform.translation.x > (WINDOW_X / 2.) + 200. {
            commands.entity(obstable_entity).despawn();
        } else if obstable_transform.translation.x < -(WINDOW_X / 2.) - 200. {
            commands.entity(obstable_entity).despawn();
        }
        if right {
            obstable_transform.translation.x -= SPEED_X * time.delta_secs();
        }
        if left {
            obstable_transform.translation.x += SPEED_X * time.delta_secs();
        }

        if obstable_transform.translation.x > (WINDOW_X / 2.) + 51. - 200.
            && obstable_transform.translation.x < (WINDOW_X / 2.) + 51. + 200.
        {
            allow_obstable_spawn = false;
        }
    }

    spawn_thing_timer.timer.tick(time.delta());
    if spawn_thing_timer.timer.just_finished() {
        let y = if random_bool_one_in_n(2) { 70. } else { -70. };

        if random_bool_one_in_n(2) && allow_obstable_spawn {
            info!("Will spawn thing at this time");
            // code to spawn goes here
            commands
                .spawn((
                    ObstacleMarker,
                    Sprite {
                        image: assest_server.load("oilpuddle.png"),
                        ..default()
                    },
                ))
                .insert(Transform::from_xyz((WINDOW_X / 2.) + 51., y, 0.));
        }
    }
}

fn random_bool_one_in_n(n: u32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=n) == 1
}

fn rotate_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Sprite), With<PlayerMarker>>,
) {
    let right = keyboard.pressed(KeyCode::KeyD);
    let left = keyboard.pressed(KeyCode::KeyA);
    let up = keyboard.pressed(KeyCode::KeyW);
    let down = keyboard.pressed(KeyCode::KeyS);

    for (mut player_transform, mut player_sprite) in player_query.iter_mut() {
        // Rotate the image instead of using static sprites
        let q = player_transform.rotation.to_array();
        let rot_z = player_transform.rotation;
        let rot_z_in_degrees = f32::to_degrees(rot_z.z.asin() * 2.0);
        let unit_angle = f32::to_degrees(rot_z.w.acos() * 2.0);
        let real_angle = (rot_z_in_degrees + unit_angle) / 2.;
        let is_down = (real_angle.round() == 180. && unit_angle > 180.)
            || (real_angle.round() == 0. && unit_angle > 0.);
        let updown = if is_down { "down" } else { "up" };

        let normalized_angle = if is_down {
            if real_angle.round() == 180. {
                rot_z_in_degrees.abs() + (unit_angle.abs() - rot_z_in_degrees.abs())
            } else {
                360. - unit_angle.abs()
            }
        } else {
            rot_z_in_degrees.abs()
        };

        let rotation_degrees_per_second = 5.5;

        if up && right {
            if normalized_angle.round() != 45. {
                let direction = if (normalized_angle > 225. && normalized_angle < 360.)
                    || (normalized_angle > 0. && normalized_angle < 45.0)
                {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        } else if up && left {
            if normalized_angle.round() != 135. {
                let direction = if (normalized_angle > 315. && normalized_angle < 360.)
                    || (normalized_angle > 0. && normalized_angle < 135.0)
                {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        } else if down && left {
            if normalized_angle.round() != 225. {
                let direction = if normalized_angle > 45. && normalized_angle < 225. {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        } else if down && right {
            if normalized_angle.round() != 315. {
                let direction = if normalized_angle > 135. && normalized_angle < 315. {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        } else if up {
            if normalized_angle.round() != 90. {
                let direction = if (normalized_angle > 270. && normalized_angle < 360.)
                    || (normalized_angle > 0. && normalized_angle < 90.0)
                {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }

            // player_transform.rotation = Quat::from_rotation_z(f32::to_radians(90.));
        } else if down {
            if normalized_angle.round() != 270. {
                let direction = if (normalized_angle > 270. && normalized_angle < 360.)
                    || (normalized_angle > 0. && normalized_angle < 90.0)
                {
                    -1.
                } else {
                    1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }

            // player_transform.rotation = Quat::from_rotation_z(f32::to_radians(-90.));
            // player_transform.rotate_z(-1. * time.delta_seconds());
            // player_transform.rotation.z = f32::to_radians(180.0);
        } else if left {
            if normalized_angle.round() != 180. {
                let direction = if normalized_angle > 0. && normalized_angle < 180. {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        } else if right {
            if normalized_angle.round() != 360. {
                let direction = if normalized_angle > 180. && normalized_angle < 360. {
                    1.
                } else {
                    -1.
                };
                player_transform
                    .rotate_z(direction * rotation_degrees_per_second * time.delta_secs());
            }
        }
    }
}

use bevy::{
    math::NormedVectorSpace,
    prelude::*,
    render::{camera::ScalingMode, view::visibility},
};
use rand::Rng;

const WINDOW_Y: f32 = 480.;
const WINDOW_X: f32 = 640.;
const SPEED_X: f32 = 300.;
const LANE_HEIGHT: f32 = 70.;

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

#[derive(Component)]
pub struct Car {
    pub speed: f32,
}

#[derive(Component)]
pub struct PersonMarker;

#[derive(Component)]
pub struct PersonHighlightMarker;

#[derive(Component)]
pub struct PlayerCar {
    pub speed_coeff: f32,
    pub timer: Timer,
    pub atlas_left: (usize, usize),
    pub atlas_right: (usize, usize),
}

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
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, keyboad_input_change_system)
        .add_systems(Update, keyboard_input_system)
        .run();
}

fn setup(
    mut commands: Commands,
    assest_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Camera2d { ..default() },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: WINDOW_X,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands
        .spawn((
            PlayerHealth { level: 900. },
            PlayerMarker,
            PlayerCar {
                speed_coeff: 0.,
                timer: Timer::from_seconds(0.075, TimerMode::Repeating),
                atlas_right: (0, 2),
                atlas_left: (3, 5),
            },
            Sprite {
                flip_x: false,
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                        UVec2::new(89, 53),
                        3,
                        2,
                        None,
                        None,
                    )),
                    index: 0,
                }),
                image: assest_server.load("taxi-Sheet.png"),
                ..default()
            },
        ))
        .insert(Transform::from_xyz(0., -LANE_HEIGHT / 2., 0.));

    commands.spawn(Sprite {
        flip_x: false,
        image: assest_server.load("road.png"),
        ..default()
    });

    for i in -6..7 {
        let x = (i as f32) * 56.;
        info!(x);
        commands
            .spawn((
                RoadMarker,
                Sprite {
                    flip_x: false,
                    image: assest_server.load("road-white-line.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(x, LANE_HEIGHT, 0.));
    }

    for i in -6..7 {
        let x = (i as f32) * 56.;
        info!(x);
        commands
            .spawn((
                RoadMarker,
                Sprite {
                    flip_x: false,
                    image: assest_server.load("road-white-line.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(x, -1. * LANE_HEIGHT, 0.));
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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
    mut spawn_thing_timer: ResMut<SpawnThingTimer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut road_query: Query<(&mut Transform), With<RoadMarker>>,
    mut obstacle_query: Query<
        (Entity, &mut Transform, &Car),
        (
            With<ObstacleMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut person_query: Query<
        (Entity, &mut Transform),
        (
            With<PersonMarker>,
            Without<ObstacleMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut person_highlight_query: Query<
        (Entity, &mut GlobalTransform, &mut Visibility),
        (
            With<PersonHighlightMarker>,
            Without<PersonMarker>,
            Without<ObstacleMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut player_query: Query<
        (&mut Transform, &mut Sprite, &mut PlayerCar),
        (
            With<PlayerMarker>,
            Without<RoadMarker>,
            Without<ObstacleMarker>,
        ),
    >,
) {
    let right = keyboard.pressed(KeyCode::KeyD);
    let left = keyboard.pressed(KeyCode::KeyA);
    let gas = keyboard.pressed(KeyCode::Space);
    let up_just_pressed = keyboard.just_pressed(KeyCode::KeyW);
    let down_just_pressed = keyboard.just_pressed(KeyCode::KeyS);

    let (mut player_transform, mut player_sprite, mut animation) = player_query.single_mut();
    animation.timer.tick(time.delta());
    if animation.speed_coeff == 0.0 {
        if right {
            player_sprite.flip_x = false;
        }
        if left {
            player_sprite.flip_x = true;
        }
    }
    let player_y = player_transform.translation.y;
    let player_x = player_transform.translation.x;
    let facing_left = player_sprite.flip_x;

    let (atlas_min, atlas_max) = if facing_left {
        animation.atlas_left
    } else {
        animation.atlas_right
    };

    if gas {
        if animation.timer.just_finished() {
            if let Some(atlas) = &mut player_sprite.texture_atlas {
                if atlas.index >= atlas_max {
                    atlas.index = atlas_min;
                } else {
                    atlas.index += 1;
                }
            }
        }
        animation.speed_coeff = (animation.speed_coeff + (1. * time.delta_secs())).min(1.0);
    } else {
        if let Some(atlas) = &mut player_sprite.texture_atlas {
            atlas.index = atlas_min;
        }
        animation.speed_coeff = (animation.speed_coeff - (1. * time.delta_secs())).max(0.0);
    }

    if up_just_pressed && player_y < LANE_HEIGHT {
        player_transform.translation.y += LANE_HEIGHT;
    }
    if down_just_pressed && player_y > -LANE_HEIGHT {
        player_transform.translation.y -= LANE_HEIGHT;
    }

    for mut road_transform in road_query.iter_mut() {
        if road_transform.translation.x > (WINDOW_X / 2.) + 42. {
            road_transform.translation.x = -(WINDOW_X / 2.) - 42.
        } else if road_transform.translation.x < -(WINDOW_X / 2.) - 42. {
            road_transform.translation.x = (WINDOW_X / 2.) + 42.
        }

        if facing_left {
            road_transform.translation.x += SPEED_X * animation.speed_coeff * time.delta_secs();
        } else {
            road_transform.translation.x -= SPEED_X * animation.speed_coeff * time.delta_secs();
        }
    }

    let mut allow_obstable_spawn = true;
    for (obstable_entity, mut obstable_transform, car) in obstacle_query.iter_mut() {
        if player_transform.translation.x < obstable_transform.translation.x + 2.
            && player_transform.translation.x > obstable_transform.translation.x - 2.
        {
            // there was a collisiion
            //

            // player_transform.translation.y += 140.;
        }

        let car_speed = car.speed;

        if obstable_transform.translation.x > (WINDOW_X / 2.) + 200. {
            commands.entity(obstable_entity).despawn();
        } else if obstable_transform.translation.x < -(WINDOW_X / 2.) - 200. {
            commands.entity(obstable_entity).despawn();
        }
        if gas {
            if obstable_transform.translation.y > 0. {
                //  car is going left
                if facing_left {
                    obstable_transform.translation.x +=
                        (-0.5 + animation.speed_coeff) * car_speed * time.delta_secs();
                } else {
                    obstable_transform.translation.x -=
                        (1.0 + (2. * animation.speed_coeff)) * car_speed * time.delta_secs();
                }
            } else {
                // car is going right
                if facing_left {
                    obstable_transform.translation.x +=
                        (1.0 + (2. * animation.speed_coeff)) * car_speed * time.delta_secs();
                } else {
                    obstable_transform.translation.x -=
                        (-0.5 + animation.speed_coeff) * car_speed * time.delta_secs();
                }
            }
        } else {
            if obstable_transform.translation.y > 0. {
                // car is going left
                if facing_left {
                    obstable_transform.translation.x -=
                        car_speed * (1.0 - animation.speed_coeff) * time.delta_secs();
                } else {
                    obstable_transform.translation.x -=
                        car_speed * (1.0 + (2.0 * animation.speed_coeff)) * time.delta_secs();
                }
            } else {
                // car is going right
                if facing_left {
                    obstable_transform.translation.x +=
                        car_speed * (1.0 + (2.0 * animation.speed_coeff)) * time.delta_secs();
                } else {
                    obstable_transform.translation.x +=
                        car_speed * (1.0 - animation.speed_coeff) * time.delta_secs();
                }
            }
        }

        if obstable_transform.translation.x > (WINDOW_X / 2.) + 51. - 200.
            && obstable_transform.translation.x < (WINDOW_X / 2.) + 51. + 200.
        {
            allow_obstable_spawn = false;
        }
    }

    for (person_entity, mut person_global_transform, mut visibility) in
        person_highlight_query.iter_mut()
    {
        let global_transform = person_global_transform.compute_transform();
        let x = global_transform.translation.x;
        let y = global_transform.translation.y;

        if (facing_left && x < 10. && player_transform.translation.y > 100. && y > 0.)
            || (facing_left && x < 10. && player_transform.translation.y < -100. && y < 0.)
        {
            *visibility = Visibility::Visible;
        } else if (!facing_left && x > -10. && player_transform.translation.y < -100. && y < 0.)
            || !facing_left && x > -10. && player_transform.translation.y > 100. && y > 0.
        {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    for (person_entity, mut person_transform) in person_query.iter_mut() {
        if person_transform.translation.x > (WINDOW_X / 2.) + 200. {
            commands.entity(person_entity).despawn_recursive();
        } else if person_transform.translation.x < -(WINDOW_X / 2.) - 200. {
            commands.entity(person_entity).despawn_recursive();
        }

        if facing_left {
            person_transform.translation.x += SPEED_X * animation.speed_coeff * time.delta_secs();
        } else {
            person_transform.translation.x -= SPEED_X * animation.speed_coeff * time.delta_secs();
        }
    }

    spawn_thing_timer.timer.tick(time.delta());
    if spawn_thing_timer.timer.just_finished() {
        let mut rng = rand::thread_rng();
        let y = if random_bool_one_in_n(2) { 170. } else { -190. };
        let x = if random_bool_one_in_n(2) {
            (WINDOW_X / 2.) + 51.
        } else {
            -(WINDOW_X / 2.) - 51.
        };

        // code to spawn goes here
        commands
            .spawn((
                PersonMarker,
                Sprite {
                    flip_x: false,
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                            UVec2::new(9, 22),
                            27,
                            1,
                            None,
                            None,
                        )),
                        index: rng.gen_range(0..27),
                    }),
                    image: assest_server.load("person-Sheet.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(x, y, 0.))
            .with_children(|commands| {
                commands
                    .spawn(Sprite {
                        image: assest_server.load("player-outline.png"),
                        ..default()
                    })
                    .insert(PersonHighlightMarker)
                    .insert(Transform::from_xyz(0., 0., -1.))
                    .insert(Visibility::Hidden);
                let y = 30.;
                commands
                    .spawn(Sprite {
                        image: assest_server.load("exclaimation.png"),
                        ..default()
                    })
                    .insert(PersonHighlightMarker)
                    .insert(Transform::from_xyz(0., y, -1.))
                    .insert(Visibility::Hidden);
            });

        let y = if random_bool_one_in_n(2) {
            LANE_HEIGHT / 2.
        } else {
            LANE_HEIGHT / 2. + LANE_HEIGHT
        };
        let x = if random_bool_one_in_n(2) {
            (WINDOW_X / 2.) + 51.
        } else {
            -(WINDOW_X / 2.) - 51.
        };
        let y = if random_bool_one_in_n(2) { -1. * y } else { y };
        let flip_x = if y > 0. { true } else { false };
        let red = rng.gen_range(0.0..=1.0);
        let green = rng.gen_range(0.0..=1.0);
        let blue = rng.gen_range(0.0..=1.0);
        if random_bool_one_in_n(1) && allow_obstable_spawn {
            commands
                .spawn((
                    ObstacleMarker,
                    Car {
                        speed: rng.gen_range(200.0..290.0),
                    },
                    Sprite {
                        flip_x: flip_x,
                        color: Color::linear_rgb(red, green, blue),
                        image: assest_server.load("car_plain.png"),
                        ..default()
                    },
                ))
                .insert(Transform::from_xyz(x, y, 0.));
        }
    }
}

fn random_bool_one_in_n(n: u32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=n) == 1
}

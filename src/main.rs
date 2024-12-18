use bevy::{
    ecs::query,
    math::NormedVectorSpace,
    prelude::*,
    render::{camera::ScalingMode, view::visibility},
    text,
};
use bevy_common_assets::json::JsonAssetPlugin;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::ops::DerefMut;
use structured_dialog::Choice;

use util::window::PixelScale;

mod names;
mod structured_dialog;
mod util;

const WINDOW_Y: f32 = 480.;
const WINDOW_X: f32 = 640.;
const SPEED_X: f32 = 300.;
const LANE_HEIGHT: f32 = 70.;

#[derive(Resource, Deref, DerefMut)]
pub struct DisplayLanguage(pub &'static str);

#[derive(Component, Default)]
pub struct InteractionObjectOptions {
    // pub camera_transition_bundle: Option<camera::CameraTransitionBundle>,
    pub dialog_name: Option<String>,
    pub scene_name: Option<String>,
    pub use_npc_indication_icon: bool,
}

#[derive(Component)]
pub struct InteractionAvailable;

#[derive(Component)]
pub struct DialogTextboxChoiceMarker;

#[derive(Resource)]
pub struct InteractionRateLimit(pub Timer);

#[derive(Component)]
pub struct SelectionMarker(pub String);

#[derive(Resource, Default)]
pub struct Taxi {
    pub rides: Vec<Ride>,
    pub closest_person: Option<Entity>,
    pub current_rider: Option<Entity>,
}

pub struct Ride {
    pub who: Entity,
    pub name: String,
    pub accepted: Option<bool>,
    pub distance: f32,
    pub completed: bool,
    pub trip_cost: f32,
    pub tip: f32,
}

#[derive(Resource)]
pub struct CurrentSelection(pub String);

#[derive(Resource)]
pub struct OccuredEvents(pub Vec<String>);

#[derive(Resource)]
pub struct Posessions(pub Vec<String>);

#[derive(Resource)]
pub struct SpawnThingTimer {
    timer: Timer,
}

#[derive(Resource, Default)]
pub struct PlayerHealth {
    pub level: f32,
    pub total_earnings: f32,
    pub spent: f32,
    pub earnings: f32,
    pub time_limit_required_earnings: f32,
    pub time_limit: Timer,
    pub cycles_completed: u32,
}

#[derive(Resource, Default)]
pub struct Travel {
    distance: f32,
    traveled: f32,
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
pub struct UiElement(String);

#[derive(Component)]
pub struct PersonHighlightMarker;

#[derive(Component)]
pub struct PlayerCar {
    pub speed_coeff: f32,
    pub timer: Timer,
    pub atlas_left: (usize, usize),
    pub atlas_right: (usize, usize),
}

#[derive(Component)]
pub struct DialogDisplay(String);

#[derive(Component)]
pub struct DialogTextbox;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Taxi GT".to_string(),
                    resolution: (WINDOW_X, WINDOW_Y).into(),
                    visible: true,
                    ..default()
                }),
                ..default()
            }),
            JsonAssetPlugin::<structured_dialog::GameScript>::new(&[".json"]),
        ))
        .insert_resource(Travel::default())
        .insert_resource(PlayerHealth {
            time_limit_required_earnings: 50.,
            time_limit: Timer::from_seconds(60.0, TimerMode::Once),
            ..default()
        })
        .insert_resource(Taxi { ..default() })
        .insert_resource(CurrentSelection(String::new()))
        .insert_resource(InteractionRateLimit(Timer::from_seconds(
            0.05,
            TimerMode::Once,
        )))
        .insert_resource(OccuredEvents(vec![]))
        .insert_resource(Posessions(vec![]))
        .insert_resource(DisplayLanguage("spanish"))
        .insert_resource(structured_dialog::DialogMessage {
            selection_index: 1,
            ..default()
        })
        .insert_resource(PixelScale(1.0, 1.0))
        .insert_resource(SpawnThingTimer {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (util::window::hud_resizer, util::window::hud_scale_updater),
        )
        .add_systems(Update, (game_level_system, keyboad_input_change_system))
        .add_systems(
            Update,
            (dialog_display_system, dialog_choice_selection_system),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Camera2d { ..default() },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WINDOW_X,
                height: WINDOW_Y,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));

    let dialog = structured_dialog::DialogHandle(asset_server.load("dialog.json"));
    commands.insert_resource(dialog);

    commands
        .spawn((
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
                image: asset_server.load("taxi-Sheet.png"),
                ..default()
            },
        ))
        .insert(Transform::from_xyz(0., -LANE_HEIGHT / 2., 0.));

    commands.spawn(Sprite {
        flip_x: false,
        image: asset_server.load("road.png"),
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
                    image: asset_server.load("road-white-line.png"),
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
                    image: asset_server.load("road-white-line.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(x, -1. * LANE_HEIGHT, 0.));
    }

    commands
        .spawn((
            util::window::Scalers {
                left: Some(Val::Px(20.0)),
                // right: Some(Val::Px(75.0)),
                bottom: Some(Val::Px(75.0)),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                // display: Display::None,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    // width: Val::Px(100.),
                    position_type: PositionType::Absolute,
                    // align_items: AlignItems::Start,
                    justify_content: JustifyContent::Center,
                    padding: UiRect {
                        left: Val::Percent(1.),
                        right: Val::Percent(1.),
                        top: Val::Percent(1.),
                        bottom: Val::Percent(0.),
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                let text_style = Node {
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        right: Val::Px(15.),
                        bottom: Val::Px(15.),
                        ..default()
                    },

                    ..default()
                };
                p.spawn((text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 28.0,
                            ..default()
                        };

                        p.spawn((
                            UiElement(String::from("timer")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });
            p.spawn((
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    width: Val::Px(100.),
                    left: Val::Px(100.),
                    position_type: PositionType::Absolute,
                    // align_items: AlignItems::Start,
                    padding: UiRect {
                        left: Val::Percent(1.),
                        right: Val::Percent(1.),
                        top: Val::Percent(1.),
                        bottom: Val::Percent(0.),
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                let text_style = Node {
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        right: Val::Px(15.),
                        bottom: Val::Px(15.),
                        ..default()
                    },

                    ..default()
                };

                p.spawn((text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            UiElement(String::from("must_earn")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });

            p.spawn((
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    width: Val::Px(100.),
                    left: Val::Px(200.),
                    position_type: PositionType::Absolute,
                    // align_items: AlignItems::Start,
                    padding: UiRect {
                        left: Val::Percent(1.),
                        right: Val::Percent(1.),
                        top: Val::Percent(1.),
                        bottom: Val::Percent(0.),
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                let text_style = Node {
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        right: Val::Px(15.),
                        bottom: Val::Px(15.),
                        ..default()
                    },

                    ..default()
                };
                p.spawn((text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            UiElement(String::from("earned")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });

            p.spawn((
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    width: Val::Px(100.),
                    left: Val::Px(300.),
                    position_type: PositionType::Absolute,
                    // align_items: AlignItems::Start,
                    padding: UiRect {
                        left: Val::Percent(1.),
                        right: Val::Percent(1.),
                        top: Val::Percent(1.),
                        bottom: Val::Percent(0.),
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                let text_style = Node {
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        right: Val::Px(15.),
                        bottom: Val::Px(15.),
                        ..default()
                    },

                    ..default()
                };
                p.spawn((text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            UiElement(String::from("total_earned")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });
        });

    commands
        .spawn(Sprite {
            flip_x: false,
            image: asset_server.load("dashboard-bg.png"),
            ..default()
        })
        .insert(Transform::from_xyz(0., -320. + (0. * 75. * 1.5), 1.));
}

fn game_level_system(
    time: Res<Time>,
    mut player_data: ResMut<PlayerHealth>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
    mut ui_element_query: Query<(&UiElement, &mut TextSpan)>,
) {
    match &dialog_message.dialog {
        Some(dialog) => match dialog.choices {
            Some(_) => return,
            None => {}
        },
        None => {}
    }

    let game_script = match game_script_asset.iter().next() {
        Some(d) => d.1,
        None => &structured_dialog::GameScript::default(),
    };

    player_data.time_limit.tick(time.delta());
    for (ui_element, mut text_span) in ui_element_query.iter_mut() {
        if ui_element.0 == "timer" {
            text_span.0 = format!("{}", player_data.time_limit.remaining_secs().round());
        }
        if ui_element.0 == "must_earn" {
            text_span.0 = format!("Must earn\n{}", player_data.time_limit_required_earnings);
        }
        if ui_element.0 == "earned" {
            text_span.0 = format!("Earned\n{}", player_data.earnings);
        }
        if ui_element.0 == "total_earned" {
            text_span.0 = format!("Total earned\n{}", player_data.total_earnings);
        }
    }

    if player_data.time_limit.just_finished() {
        if player_data.earnings > player_data.time_limit_required_earnings {
            let x = player_data.cycles_completed as f32;
            // simple for now, but quickly will becomes impossible
            player_data.time_limit_required_earnings = 0.1 * x.powi(2) + 0.5 * x + 50.;
            player_data.time_limit.reset();
            player_data.earnings = 0.0;
            player_data.cycles_completed += 1;
        } else {
            let game_over = game_script
                .dialogs
                .iter()
                .map(|d| d.clone())
                .find(|d| d.id == String::from("game over"));
            dialog_message.dialog = game_over;
        }
    }
}

fn keyboad_input_change_system(
    mut commands: Commands,
    assest_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut travel: ResMut<Travel>,
    time: Res<Time>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
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

    mut taxi: ResMut<Taxi>,
    mut player_data: ResMut<PlayerHealth>,
) {
    let current_dialog_id = match &dialog_message.dialog {
        Some(dialog) => match dialog.choices {
            Some(_) => return,
            None => Some(dialog.id.clone()),
        },
        None => None,
    };

    let game_script = match game_script_asset.iter().next() {
        Some(d) => d.1,
        None => &structured_dialog::GameScript::default(),
    };

    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let gas = keyboard.pressed(KeyCode::Space);
    let up_just_pressed = keyboard.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down_just_pressed = keyboard.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);

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

    let can_drop_off = if facing_left && player_y > 100. {
        true
    } else if !facing_left && player_y < -100. {
        true
    } else {
        false
    };

    for (person_entity, mut person_global_transform, mut visibility) in
        person_highlight_query.iter_mut()
    {
        let global_transform = person_global_transform.compute_transform();
        let x = global_transform.translation.x;
        let y = global_transform.translation.y;

        let is_visible = if (facing_left && x < 50. && player_y > 100. && y > 0.)
            || (facing_left && x < 50. && player_y < -100. && y < 0.)
        {
            *visibility = Visibility::Visible;
            true
        } else if (!facing_left && x > -50. && player_y < -100. && y < 0.)
            || !facing_left && x > -50. && player_y > 100. && y > 0.
        {
            *visibility = Visibility::Visible;
            true
        } else {
            *visibility = Visibility::Hidden;
            false
        };
    }

    match taxi.current_rider {
        Some(current_rider) => {
            let info = taxi
                .rides
                .iter_mut()
                .find(|ride| ride.who == current_rider)
                .unwrap();

            if !info.completed {
                travel.traveled += SPEED_X * animation.speed_coeff * time.delta_secs() / 1000.;
                if travel.traveled > travel.distance {
                } else {
                    // info!("Traveled {} km", travel.traveled);
                }

                if travel.traveled > travel.distance {
                    let id = current_dialog_id.unwrap_or_default();
                    if id == String::from("drop off soon") {
                        // info!("Clear 1");
                        dialog_message.dialog = None
                    } else if animation.speed_coeff > 0.0 {
                        dialog_message.dialog = Some(game_script.dialogs[2].clone());
                    } else if can_drop_off
                        && animation.speed_coeff == 0.0
                        && id == String::from("here")
                    {
                        dialog_message.dialog = None;
                    } else if can_drop_off && animation.speed_coeff == 0.0 && id == String::from("")
                    {
                        // SUCCESSFUL DROP OFF
                        info!("Show bye message");
                        player_data.earnings += info.trip_cost + info.tip;
                        player_data.total_earnings += info.trip_cost + info.tip;
                        dialog_message.dialog = Some(game_script.dialogs[3].clone());
                        info.completed = true;
                    } else {
                        // taxi.current_rider = None;
                        // info!("Wait for input")
                    }
                } else if travel.traveled < travel.distance
                    && (travel.traveled / travel.distance) > 0.70
                {
                    // info!("Drop off player soon");
                    dialog_message.dialog = Some(game_script.dialogs[1].clone());
                }
            }
        }
        None => {
            let closest_persons = person_highlight_query
                .iter()
                .filter(|(_, transform, visiblility)| {
                    let global_transform = transform.compute_transform();
                    global_transform.translation.x.abs() <= 50.
                        && *visiblility == Visibility::Visible
                })
                .next();

            match closest_persons {
                Some((closest_rider_entity, _, _)) => {
                    if animation.speed_coeff == 0.0 {
                        let rider = taxi.rides.iter().find(|r| r.who == closest_rider_entity);
                        let mut rng = rand::thread_rng();
                        let show_dialog = match rider {
                            Some(rider) => match rider.accepted {
                                Some(b) => b,
                                None => true,
                            },
                            None => {
                                let d = rng.gen_range(0.25..=10.0);
                                taxi.closest_person = Some(closest_rider_entity);
                                taxi.rides.push(Ride {
                                    who: closest_rider_entity,
                                    name: names::name(),
                                    accepted: None,
                                    distance: ((d * 100.) as f32).round() / 100.,
                                    completed: false,
                                    trip_cost: ((d * 7.) as f32).ceil(),
                                    tip: 0.0,
                                });
                                true
                            }
                        };
                        if show_dialog {
                            dialog_message.dialog = Some(game_script.dialogs[0].clone());
                        }
                        // info!("Show the dialog!");
                    } else {
                        dialog_message.dialog = None;
                    }
                }
                None => {}
            }
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

pub fn dialog_display_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    display_language: ResMut<DisplayLanguage>,
    dialog_message: ResMut<structured_dialog::DialogMessage>,
    dialog_display_query: Query<(Entity, &DialogDisplay), With<DialogDisplay>>,

    // not consistent with regular dialog
    taxi: Res<Taxi>,
) {
    let dialog = match &dialog_message.dialog {
        Some(d) => d,
        None => {
            for (entity, _) in dialog_display_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            return;
        }
    };

    for (entity, dialog_display) in dialog_display_query.iter() {
        if dialog_display.0 != dialog.id {
            commands.entity(entity).despawn_recursive();
        }
    }

    if !dialog_display_query.is_empty() {
        return;
    }

    commands
        .spawn((
            DialogDisplay(dialog.id.clone()),
            util::window::Scalers {
                left: Some(Val::Px(20.0)),
                // right: Some(Val::Px(75.0)),
                top: Some(Val::Px(15.0)),
                // width: Some(Val::Px(704.)),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                // display: Display::None,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                DialogTextbox,
                BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    width: Val::Px(600.),
                    position_type: PositionType::Absolute,
                    // align_items: AlignItems::Start,
                    padding: UiRect {
                        left: Val::Percent(1.),
                        right: Val::Percent(1.),
                        top: Val::Percent(1.),
                        bottom: Val::Percent(0.),
                    },
                    ..default()
                },
            ))
            .with_children(|p| {
                let text_style = Node {
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        right: Val::Px(15.),
                        bottom: Val::Px(15.),
                        ..default()
                    },
                    ..default()
                };
                p.spawn((text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 18.0,
                            ..default()
                        };

                        let text = if display_language.0 == "english" {
                            &dialog.language.english
                        } else {
                            &dialog.language.spanish
                        };

                        let text = if let Some(current_rider) = taxi.closest_person {
                            if let Some(info) = taxi.rides.iter().find(|r| r.who == current_rider) {
                                text.replace("{person}", &info.name)
                                    .replace("{distance}", &info.distance.to_string())
                                    .replace("{price}", &info.trip_cost.to_string())
                                    .replace("{tip}", &info.tip.to_string())
                            } else {
                                text.clone()
                            }
                        } else {
                            text.clone()
                        };

                        p.spawn((text_font.clone(), TextSpan::new(text.clone())));
                        // info!("Should be displaying: {}", text);

                        match &dialog.choices {
                            Some(choices) => {
                                // text_container.insert(DialogTextboxChoiceMarker);

                                // let total_choices = choices.len();

                                for (index, choice) in choices.iter().enumerate() {
                                    // info!("Adding choice #{}", index);
                                    // let style = dialog_textbox.0.clone();
                                    let choice_id = choice.choice.clone();

                                    let text = if display_language.0 == "english" {
                                        &choice.dialog.language.english.clone()
                                    } else {
                                        &choice.dialog.language.spanish.clone()
                                    };

                                    let text_font = TextFont {
                                        font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                                        font_size: 18.0,
                                        ..default()
                                    };

                                    p.spawn((
                                        SelectionMarker(choice_id),
                                        text_font,
                                        TextSpan::new(format!("\n\n {}", text)),
                                    ));
                                }
                            }
                            None => {}
                        }
                    });
            });
        });
    return;
}

pub fn dialog_choice_selection_system(
    mut commands: Commands,
    time: Res<Time>,
    display_language: Res<DisplayLanguage>,
    mut interaction_rate_limit: ResMut<InteractionRateLimit>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // axes: Res<Axis<GamepadAxis>>,
    mut current_selection: ResMut<CurrentSelection>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    mut selections: Query<(&SelectionMarker, &mut TextSpan)>,

    // not consistent with regular dialog
    mut travel: ResMut<Travel>,
    mut taxi: ResMut<Taxi>,
) {
    let up_key_pressed = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down_key_pressed = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let enter_key_just_pressed = keyboard_input.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]);

    let dialog = match &dialog_message.dialog {
        Some(d) => d,
        None => {
            return;
        }
    };

    let choices = match &dialog.choices {
        Some(choices) => {
            if choices.is_empty() {
                return;
            }
            choices
        }
        None => return,
    };

    if enter_key_just_pressed {
        if current_selection.0 == "play again" {
            return;
        }
        dialog_message.dialog = None;
        if taxi.current_rider.is_some() {
            taxi.current_rider = None;
        } else if let Some(closest_person) = taxi.closest_person {
            if current_selection.0 == "0" {
                taxi.current_rider = taxi.closest_person;
                match taxi.rides.iter_mut().find(|r| r.who == closest_person) {
                    Some(ride) => {
                        travel.distance = ride.distance;
                        travel.traveled = 0.0;
                    }
                    None => todo!(),
                }
            } else {
                taxi.current_rider = None;
                taxi.rides
                    .iter_mut()
                    .filter(|r| r.who == closest_person)
                    .for_each(|ride| ride.accepted = Some(false));
            }
        }

        return;
    }

    interaction_rate_limit.0.tick(time.delta());
    if interaction_rate_limit.0.finished() || interaction_rate_limit.0.just_finished() {
        interaction_rate_limit.0.reset();

        let index = match choices
            .iter()
            .enumerate()
            .find(|(_, c)| c.choice == current_selection.0)
        {
            Some((index, _)) => index,
            None => 0,
        };

        let new_index = if up_key_pressed {
            if index <= 0 {
                0
            } else {
                index - 1
            }
        } else if down_key_pressed {
            if index >= choices.len() - 1 {
                choices.len() - 1
            } else {
                index + 1
            }
        } else {
            index
        };

        for (choice_index, choice) in choices.iter().enumerate() {
            let choice_id = choice.choice.clone();
            let text = if display_language.0 == "english" {
                &choice.dialog.language.english
            } else {
                &choice.dialog.language.spanish
            };

            if index == choice_index {
                for (selection, mut text_span) in selections.iter_mut() {
                    if selection.0 == choice_id {
                        *text_span = TextSpan::new(format!("\n\n {}", text.clone()));
                    }
                }
            }
            if new_index == choice_index {
                current_selection.0 = choice_id.clone();
                for (selection, mut text_span) in selections.iter_mut() {
                    if selection.0 == choice_id {
                        *text_span = TextSpan::new(format!("\n\n> {}", text.clone()));
                    }
                }
            }
        }
    }
}

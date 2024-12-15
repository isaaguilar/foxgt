use bevy::{
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
#[derive(Component)]
pub struct DialogDisplay;

#[derive(Component)]
pub struct DialogTextbox;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fox GT".to_string(),
                    resolution: (WINDOW_X, WINDOW_Y).into(),
                    visible: true,
                    ..default()
                }),
                ..default()
            }),
            JsonAssetPlugin::<structured_dialog::GameScript>::new(&[".json"]),
        ))
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
        .add_systems(Update, keyboad_input_change_system)
        .add_systems(Update, keyboard_input_system)
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

    // commands
    //     .spawn((
    //         DialogDisplay,
    //         util::window::Scalers {
    //             left: Some(Val::Px(20.0)),
    //             // right: Some(Val::Px(75.0)),
    //             top: Some(Val::Px(15.0)),
    //             // width: Some(Val::Px(704.)),
    //             ..default()
    //         },
    //         Node {
    //             position_type: PositionType::Absolute,
    //             // display: Display::None,
    //             ..default()
    //         },
    //     ))
    //     .with_child((
    //         DialogTextbox,
    //         BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
    //         Node {
    //             // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
    //             width: Val::Px(600.),
    //             position_type: PositionType::Absolute,
    //             // align_items: AlignItems::Start,
    //             padding: UiRect {
    //                 left: Val::Percent(1.),
    //                 right: Val::Percent(1.),
    //                 top: Val::Percent(1.),
    //                 bottom: Val::Percent(0.),
    //             },
    //             ..default()
    //         },
    //     ));
    // .with_children(|commands| {
    //     commands.spawn((
    //         Text::new("hello"),
    //         TextFont {
    //             font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
    //             font_size: 18.0,
    //             ..default()
    //         },
    //         Node {
    //             margin: UiRect {
    //                 left: Val::Px(15.),
    //                 top: Val::Px(15.),
    //                 right: Val::Px(15.),
    //                 bottom: Val::Px(15.),
    //                 ..default()
    //             },
    //             ..default()
    //         },
    //         // Text::from_section(
    //         //     String::new(),
    //         //     TextStyle {
    //         //         font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
    //         //         font_size: 18.0,
    //         //         ..default()
    //         //     },
    //         // )
    //         // .with_style(Style {
    //         //     margin: UiRect {
    //         //         left: Val::Px(15.),
    //         //         top: Val::Px(15.),
    //         //         right: Val::Px(15.),
    //         //         bottom: Val::Px(15.),
    //         //         ..default()
    //         //     },
    //         //     position_type: PositionType::Relative,
    //         //     ..default()
    //         // })
    //         // .with_text_justify(JustifyText::Left),
    //     ));
    // });

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
}

fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerHealth, With<PlayerMarker>>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        info!("Hello")
    } else if keyboard.just_pressed(KeyCode::KeyH) {
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
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
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
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
) {
    let right = keyboard.pressed(KeyCode::KeyD);
    let left = keyboard.pressed(KeyCode::KeyA);
    let gas = keyboard.pressed(KeyCode::Space);
    let up_just_pressed = keyboard.just_pressed(KeyCode::KeyW);
    let down_just_pressed = keyboard.just_pressed(KeyCode::KeyS);

    let game_script = match game_script_asset.iter().next() {
        Some(d) => d.1,
        None => &structured_dialog::GameScript::default(),
    };

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

    let closest_persons = person_highlight_query
        .iter()
        .filter(|(_, transform, visiblility)| {
            let global_transform = transform.compute_transform();
            global_transform.translation.x.abs() <= 50. && *visiblility == Visibility::Visible
        })
        .next();

    match closest_persons {
        Some(_) => {
            if animation.speed_coeff == 0.0 {
                dialog_message.dialog = Some(game_script.dialogs[0].clone());
                info!("Show the dialog!");
            } else {
                dialog_message.dialog = None;
            }
        }
        None => {}
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
    // mut dialog_textbox_query: Query<(Entity), With<DialogTextbox>>,
    mut dialog_display_query: Query<Entity, With<DialogDisplay>>,
) {
    let dialog = match &dialog_message.dialog {
        Some(d) => d,
        None => {
            for entity in dialog_display_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            return;
        }
    };
    if !dialog_display_query.is_empty() {
        return;
    }

    commands
        .spawn((
            DialogDisplay,
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
                        let mut rng = rand::thread_rng();
                        let text = text.replace("{0}", &names::name());
                        let text = text.replace("{1}", &rng.gen_range(1..10).to_string());
                        let text = text.replace("{2}", "");

                        p.spawn((text_font.clone(), TextSpan::new(text.clone())));
                        info!("Should be displaying: {}", text);

                        match &dialog.choices {
                            Some(choices) => {
                                // text_container.insert(DialogTextboxChoiceMarker);

                                // let total_choices = choices.len();

                                for (index, choice) in choices.iter().enumerate() {
                                    info!("Adding choice #{}", index);
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
    dialog_message: Res<structured_dialog::DialogMessage>,
    mut selections: Query<(&SelectionMarker, &mut TextSpan)>,
) {
    let up_key_pressed = keyboard_input.pressed(KeyCode::ArrowUp);
    let down_key_pressed = keyboard_input.pressed(KeyCode::ArrowDown);

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

    // for entity in main_menu_section_query.iter() {

    //     let text_box = commands.entity(entity);

    //     let min_selection_index = 1;

    //     let max_selection_index = text_sections.sections.len() - 1;
    //     if min_selection_index == max_selection_index {
    //         // this isn't really a selection, is it?
    //         warn!("A single selection was detected");
    //     }

    //     if interaction_rate_limit.0.finished() || interaction_rate_limit.0.just_finished() {
    //         let idx = if up_key_pressed {
    //             interaction_rate_limit.0.reset();
    //             if dialog_message.selection_index <= min_selection_index {
    //                 min_selection_index
    //             } else {
    //                 dialog_message.selection_index - 1
    //             }
    //         } else if down_key_pressed {
    //             interaction_rate_limit.0.reset();
    //             if dialog_message.selection_index >= max_selection_index {
    //                 max_selection_index
    //             } else {
    //                 dialog_message.selection_index + 1
    //             }
    //         } else {
    //             dialog_message.selection_index
    //         };

    //         dialog_message.selection_index = idx;
    //     }
    // }
}

// pub fn npc_dialog(
//     time: Res<Time>,
//     mut interaction_rate_limit: ResMut<InteractionRateLimit>,
//     display_language: ResMut<DisplayLanguage>,
//     mut occurred_events: ResMut<OccuredEvents>,
//     mut possessions: ResMut<Posessions>,
//     game_script_asset: Res<Assets<structured_dialog::GameScript>>,
//     mut dialog_message: ResMut<structured_dialog::DialogMessage>,
//     keyboard: Res<ButtonInput<KeyCode>>,
//     in_range_npc_query: Query<&InteractionObjectOptions, With<InteractionAvailable>>,
// ) {
//     interaction_rate_limit.0.tick(time.delta());
//     if !keyboard.just_pressed(KeyCode::KeyE) {
//         return;
//     }

//     let game_script = match game_script_asset.iter().next() {
//         Some(d) => d.1,
//         None => &structured_dialog::GameScript::default(),
//     };
//     // info!(?game_script);

//     let npc = match in_range_npc_query.get_single() {
//         Ok(npc) => npc,
//         Err(_) => {
//             dialog_message.reset();
//             return;
//         }
//     };

//     if interaction_rate_limit.0.finished() || interaction_rate_limit.0.just_finished() {
//         interaction_rate_limit.0.reset();

//         match &dialog_message.dialog {
//             Some(dialog) => {
//                 // Perform on_exit actions for the currnet dialog
//                 let actions = match &dialog.choices {
//                     Some(choices) => {
//                         let selection_index = if dialog_message.selection_index > 0 {
//                             // correct the choice selection to account for its position in text
//                             dialog_message.selection_index - 1
//                         } else {
//                             0
//                         };
//                         match &choices.get(selection_index) {
//                             Some(choice) => &choice.dialog.actions,
//                             None => {
//                                 // this is an error but we can't handle it without breaking
//                                 // hopefully this never happens or we'll be in a bad state
//                                 error!(
//                                     "Index for choices was invalid, where index={} and {:?}",
//                                     selection_index, choices
//                                 );
//                                 dialog_message.reset();
//                                 return;
//                             }
//                         }
//                     }
//                     None => &dialog.actions,
//                 };

//                 // update_events(actions, &mut occurred_events, "on_exit");
//                 // update_posessions(actions, &mut possessions, "on_exit");

//                 let next_id = &actions.next_id;

//                 info!(next_id);
//                 match game_script
//                     .dialogs
//                     .iter()
//                     .find(|npc_dialog| npc_dialog.id == *next_id)
//                 {
//                     Some(next_dialog) => {
//                         let s = if display_language.0 == "english" {
//                             next_dialog.language.english.clone()
//                         } else {
//                             next_dialog.language.spanish.clone()
//                         };

//                         dialog_message.dialog = Some(next_dialog.clone());
//                         // update_events(&next_dialog.actions, &mut occurred_events, "on_enter");
//                         // update_posessions(&next_dialog.actions, &mut possessions, "on_enter");
//                     }
//                     None => {
//                         dialog_message.reset();
//                     }
//                 }
//             }
//             None => {
//                 let npc_dialogs = game_script
//                     .dialogs
//                     .clone()
//                     .into_iter()
//                     .filter(|n| &n.name == npc.dialog_name.as_ref().unwrap())
//                     .filter(|n| {
//                         n.events.iter().all(|event| {
//                             if event.starts_with('!') {
//                                 let not_event = event.strip_prefix("!").unwrap().to_string();
//                                 !occurred_events.0.contains(&not_event)
//                             } else {
//                                 occurred_events.0.contains(event)
//                             }
//                         })
//                     })
//                     .filter(|n| {
//                         n.posessions.iter().all(|possession| {
//                             if possession.starts_with('!') {
//                                 let not_possession =
//                                     possession.strip_prefix("!").unwrap().to_string();
//                                 !possessions.0.contains(&not_possession)
//                             } else {
//                                 possessions.0.contains(possession)
//                             }
//                         })
//                     })
//                     .collect::<Vec<_>>();

//                 let dialog_ids = npc_dialogs.iter().map(|d| &d.id).collect::<Vec<_>>();
//                 info!(?dialog_ids);
//                 let dialog_with_choices = npc_dialogs
//                     .clone()
//                     .into_iter()
//                     .filter(|n| n.choices.is_some())
//                     .collect::<Vec<_>>();
//                 let mut rng = rand::thread_rng();
//                 let selected_dialog = if dialog_with_choices.is_empty() {
//                     // No choices detected
//                     npc_dialogs.choose(&mut rng)
//                 } else {
//                     dialog_with_choices.choose(&mut rng)
//                 };

//                 if let Some(npc_dialog) = selected_dialog {
//                     let s = if display_language.0 == "english" {
//                         npc_dialog.language.english.clone()
//                     } else {
//                         npc_dialog.language.spanish.clone()
//                     };

//                     dialog_message.dialog = Some(npc_dialog.clone());

//                     for event in &npc_dialog.actions.events_changed_on_enter {
//                         if event.starts_with('!') {
//                             let event_to_remove = event.strip_prefix("!").unwrap().to_string();
//                             if let Some(index) = occurred_events
//                                 .0
//                                 .iter()
//                                 .position(|item| *item == event_to_remove)
//                             {
//                                 occurred_events.0.remove(index);
//                             }
//                         } else if !occurred_events.0.contains(event) {
//                             occurred_events.0.push(event.clone());
//                         }
//                     }
//                     for posession in &npc_dialog.actions.items_changed_on_enter {
//                         if posession.starts_with('!') {
//                             let posession_to_remove =
//                                 posession.strip_prefix("!").unwrap().to_string();
//                             if let Some(index) = possessions
//                                 .0
//                                 .iter()
//                                 .position(|item| *item == posession_to_remove)
//                             {
//                                 possessions.0.remove(index);
//                             }
//                         } else if !possessions.0.contains(posession) {
//                             possessions.0.push(posession.clone());
//                         }
//                     }
//                 };
//             }
//         }
//     }
// }

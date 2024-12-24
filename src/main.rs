use bevy::{
    ecs::query,
    input::keyboard::Key,
    math::{
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
        vec2, NormedVectorSpace,
    },
    prelude::*,
    render::{camera::ScalingMode, view::visibility},
    state::commands,
    text,
    utils::hashbrown::HashMap,
};
use bevy_common_assets::json::JsonAssetPlugin;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::ops::DerefMut;
use structured_dialog::Choice;

use util::window::PixelScale;

mod menu;
mod names;
mod splash;
mod structured_dialog;
mod util;

const WINDOW_Y: f32 = 480.;
const WINDOW_X: f32 = 640.;
const SPEED_X: f32 = 300.;
const LANE_HEIGHT: f32 = 70.;
const HALF_CAR_WIDTH: f32 = 89. / 2.;
const PERSON_Y_TOP: f32 = 160.;
const PERSON_Y_BOTTOM: f32 = -160.;
const TIME_LIMIT: f32 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Splash,
    Game,
    Pause,
    Menu,
    GameOver,
}

#[derive(Resource, Deref, DerefMut)]
pub struct DisplayLanguage(pub &'static str);

#[derive(Component)]
pub struct GameState;

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

#[derive(Component, Clone)]
pub struct Passenger {
    name: String,
    sprite_index: usize,
}

#[derive(Resource)]
pub struct InteractionRateLimit(pub Timer);

#[derive(Component)]
pub struct SelectionMarker(pub String);

#[derive(Component, Default)]
struct Intersects(bool);

#[derive(Resource, Default)]
pub struct Taxi {
    pub rides: Vec<Ride>,
    pub closest_person: Option<Entity>,
    pub current_rider: Option<Entity>,
}

pub struct Ride {
    pub who: Entity,
    pub passenger: Passenger,
    pub accepted: Option<bool>,
    pub distance: f32,
    pub completed: bool,
    pub trip_cost: f32,
    pub tip_percentage: f32,
    pub tip: f32,
    pub distance_past_dropoff: f32,
    pub trip_time: f32,
}

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct CurrentSelection(pub String);

#[derive(Resource)]
pub struct OccuredEvents(pub Vec<String>);

#[derive(Resource)]
pub struct Posessions(pub Vec<String>);

#[derive(Resource)]
pub struct SpawnThingTimer {
    timer: Timer,
}

#[derive(Resource)]
pub struct PlayerHealth {
    pub level: f32,
    pub total_earnings: f32,
    pub spent: f32,
    pub earnings: f32,
    pub time_limit_required_earnings: f32,
    pub time_limit: Timer,
    pub cycles_completed: u32,
    pub distance_traveled: f32,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            time_limit_required_earnings: 50.,
            time_limit: Timer::from_seconds(TIME_LIMIT, TimerMode::Once),
            level: 0.0,
            total_earnings: 0.0,
            spent: 0.0,
            earnings: 0.0,
            cycles_completed: 0,
            distance_traveled: 0.0,
        }
    }
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
pub struct CarMarker;

#[derive(Component)]
pub struct PersonInCarMarker;

#[derive(Component, Clone)]
pub struct Car {
    pub aabb: Aabb2d,
    pub speed: f32,
    pub intersects_player: bool,
    pub intersects_npc: bool,
    pub blocks_player_movement: bool,
}

#[derive(Component)]
pub struct PersonMarker;

#[derive(Component)]
pub struct UiElement(String);

#[derive(Component)]
pub struct PersonHighlightMarker;

#[derive(Component)]
pub struct PlayerCar {
    pub aabb: Aabb2d,
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
        .add_systems(Startup, load_json)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TaxiGT".to_string(),
                    resolution: (WINDOW_X, WINDOW_Y).into(),
                    visible: true,
                    ..default()
                }),
                ..default()
            }),
            JsonAssetPlugin::<structured_dialog::GameScript>::new(&[".json"]),
            splash::SplashPlugin,
            menu::MenuPlugin,
        ))
        .init_state::<AppState>()
        .insert_resource(ResetGame(false))
        .insert_resource(ResumeGame(false))
        .insert_resource(Travel::default())
        .insert_resource(PlayerHealth::default())
        .insert_resource(Taxi { ..default() })
        .insert_resource(CurrentSelection(String::new()))
        .insert_resource(InteractionRateLimit(Timer::from_seconds(
            0.20,
            TimerMode::Once,
        )))
        .insert_resource(OccuredEvents(vec![]))
        .insert_resource(Posessions(vec![]))
        .insert_resource(DisplayLanguage("english"))
        .insert_resource(structured_dialog::DialogMessage {
            selection_index: 1,
            ..default()
        })
        .insert_resource(PixelScale(1.0, 1.0))
        .insert_resource(SpawnThingTimer {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        })
        .add_systems(OnEnter(AppState::Game), setup)
        .add_systems(
            PreUpdate,
            (util::window::hud_resizer, util::window::hud_scale_updater),
        )
        .add_systems(
            Update,
            (
                game_level_system,
                keyboad_input_change_system,
                road_line_system,
                car_intersection_system,
                reset,
                dialog_display_system,
                dialog_choice_selection_system,
            )
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(OnExit(AppState::Game), util::despawn_screen::<GameCamera>)
        .run();
}

fn load_json(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dialog = structured_dialog::DialogHandle(asset_server.load("dialog.json"));
    commands.insert_resource(dialog);
}

#[derive(Component)]
pub struct GameCamera;

fn setup(
    mut bg: ResMut<ClearColor>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resume_game: Res<ResumeGame>,
    mut last_dialog: ResMut<menu::LastDialog>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
) {
    let rba_dark_gray = 0.025;
    bg.0 = Color::linear_rgba(rba_dark_gray, rba_dark_gray, rba_dark_gray, 0.0);
    commands.spawn((
        GameState,
        GameCamera,
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

    if resume_game.0 {
        dialog_message.dialog = last_dialog.0.clone();
        last_dialog.0 = None;

        return;
    }

    let player_y_start = -LANE_HEIGHT / 2.;
    commands
        .spawn((
            GameState,
            PlayerMarker,
            Intersects::default(),
            PlayerCar {
                aabb: Aabb2d {
                    min: Vec2::new(-HALF_CAR_WIDTH, player_y_start + (-53. / 2.)),
                    max: Vec2::new(HALF_CAR_WIDTH, player_y_start + (53. / 2.)),
                },
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

    commands.spawn((
        GameState,
        Sprite {
            flip_x: false,
            image: asset_server.load("road.png"),
            ..default()
        },
    ));

    for i in -6..7 {
        let x = (i as f32) * 56.;
        // info!(x);
        commands
            .spawn((
                GameState,
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
        // info!(x);
        commands
            .spawn((
                GameState,
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
            GameState,
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
                GameState,
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
                p.spawn((GameState, text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 28.0,
                            ..default()
                        };

                        p.spawn((
                            GameState,
                            UiElement(String::from("timer")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });
            p.spawn((
                GameState,
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

                p.spawn((GameState, text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            GameState,
                            UiElement(String::from("must_earn")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });

            p.spawn((
                GameState,
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
                p.spawn((GameState, text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            GameState,
                            UiElement(String::from("earned")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });

            p.spawn((
                GameState,
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
                p.spawn((GameState, text_style.clone(), Text::default()))
                    .with_children(|p| {
                        let text_font = TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 12.0,
                            ..default()
                        };

                        p.spawn((
                            GameState,
                            UiElement(String::from("total_earned")),
                            text_font.clone(),
                            TextSpan::new(""),
                        ));
                    });
            });

            p.spawn((
                GameState,
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                Node {
                    // background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    width: Val::Px(100.),
                    left: Val::Px(500.),
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
                p.spawn((
                    GameState,
                    PersonInCarMarker,
                    Node {
                        width: Val::Px(9. * 3.),
                        height: Val::Px(22. * 3.),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    ImageNode {
                        image: asset_server.load("person-Sheet.png"),
                        texture_atlas: Some(TextureAtlas {
                            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                                UVec2::new(9, 22),
                                28,
                                1,
                                None,
                                None,
                            )),
                            index: 27,
                        }),
                        ..default()
                    },
                ));
            });
        });

    commands
        .spawn((
            GameState,
            Sprite {
                flip_x: false,
                image: asset_server.load("dashboard-bg.png"),
                ..default()
            },
        ))
        .insert(Transform::from_xyz(0., -320. + (0. * 75. * 1.5), 1.));
}

fn car_intersection_system(
    player_car_query: Query<(&Sprite, &PlayerCar)>,
    mut npc_car_query: Query<(Entity, &mut Car)>,
) {
    let (player_sprite, player_car) = player_car_query.single();
    let facing_left = player_sprite.flip_x;

    for (_, mut npc_car) in npc_car_query.iter_mut() {
        if player_car.aabb.intersects(&npc_car.aabb) {
            if npc_car.aabb.min.y > 0. {
                if npc_car.aabb.min.x > 0. {
                    npc_car.intersects_player = true;
                    if facing_left {
                        npc_car.blocks_player_movement = false;
                    } else {
                        npc_car.blocks_player_movement = true;
                    }
                } else if facing_left {
                    npc_car.intersects_player = true;
                    npc_car.blocks_player_movement = true;
                } else if !facing_left && npc_car.aabb.max.x > HALF_CAR_WIDTH / 4. {
                    npc_car.intersects_player = true;
                    npc_car.blocks_player_movement = true;
                } else {
                    npc_car.intersects_player = false;
                    npc_car.blocks_player_movement = false;
                }
            } else {
                if npc_car.aabb.max.x < 0. {
                    npc_car.intersects_player = true;
                    if !facing_left {
                        npc_car.blocks_player_movement = false;
                    } else {
                        npc_car.blocks_player_movement = true;
                    }
                } else if !facing_left {
                    npc_car.intersects_player = true;
                    npc_car.blocks_player_movement = true;
                } else if facing_left && npc_car.aabb.min.x < HALF_CAR_WIDTH / 4. {
                    npc_car.intersects_player = true;
                    npc_car.blocks_player_movement = true;
                } else {
                    npc_car.intersects_player = false;
                    npc_car.blocks_player_movement = false;
                }
            }
        } else {
            npc_car.intersects_player = false;
            npc_car.blocks_player_movement = false;
        }
    }

    let mut entitys_intersected: Vec<Entity> = vec![];
    for (e1, npc_car1) in npc_car_query.iter() {
        for (e2, npc_car2) in npc_car_query.iter() {
            if e1 == e2 {
                continue;
            }

            if npc_car1.aabb.intersects(&npc_car2.aabb) {
                entitys_intersected.push(e1);
            }
        }
    }

    npc_car_query.iter_mut().for_each(|(entity, mut npc_car)| {
        if entitys_intersected.contains(&entity) {
            npc_car.intersects_npc = true;
            // info!("NPCs intersected");
        }
    });
}

fn game_level_system(
    time: Res<Time>,
    taxi: Res<Taxi>,
    mut player_data: ResMut<PlayerHealth>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
    mut ui_element_query: Query<(&UiElement, &mut TextSpan)>,
    mut ui_person_in_car_query: Query<&mut ImageNode, With<PersonInCarMarker>>,
) {
    match &dialog_message.dialog {
        Some(dialog) => match dialog.choices {
            Some(_) => return,
            None => {}
        },
        None => {}
    }

    let current_rider = taxi.current_rider;
    let index = if let Some(info) = taxi.rides.iter().find(|ride| {
        if let Some(who) = current_rider {
            ride.who == who
        } else {
            false
        }
    }) {
        info.passenger.sprite_index
    } else {
        27
    };

    for mut ui_person_in_car in ui_person_in_car_query.iter_mut() {
        if let Some(atlas) = &mut ui_person_in_car.texture_atlas {
            atlas.index = index;
        }
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
            player_data.cycles_completed += 1;
            player_data.time_limit.reset();
            player_data.earnings = 0.0;

            let x = if player_data.cycles_completed < 10 {
                (player_data.cycles_completed as f32) * 2. / 100.
            } else if player_data.cycles_completed < 30 {
                0.2 + (player_data.cycles_completed as f32) * 2. / 1000.
            } else {
                0.258
            };

            // info!("{}", 1. + x);
            player_data.time_limit_required_earnings = (49.0_f32.powf(1. + x)).ceil();
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

fn road_line_system(
    time: Res<Time>,
    dialog_message: Res<structured_dialog::DialogMessage>,
    mut road_query: Query<(&mut Transform), With<RoadMarker>>,
    player_query: Query<(&Sprite, &PlayerCar), With<PlayerMarker>>,
) {
    if let Some(dialog) = &dialog_message.dialog {
        if dialog.choices.is_some() {
            return;
        }
    }

    let (player_sprite, player_car) = player_query.single();
    let facing_left = player_sprite.flip_x;

    for mut road_transform in road_query.iter_mut() {
        if road_transform.translation.x > (WINDOW_X / 2.) + 42. {
            road_transform.translation.x = -(WINDOW_X / 2.) - 42.
        } else if road_transform.translation.x < -(WINDOW_X / 2.) - 42. {
            road_transform.translation.x = (WINDOW_X / 2.) + 42.
        }

        if facing_left {
            road_transform.translation.x += SPEED_X * player_car.speed_coeff * time.delta_secs();
        } else {
            road_transform.translation.x -= SPEED_X * player_car.speed_coeff * time.delta_secs();
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
    mut car_query: Query<
        (Entity, &mut Transform, &mut Car),
        (With<CarMarker>, Without<RoadMarker>, Without<PlayerMarker>),
    >,
    mut person_query: Query<
        (Entity, &mut Transform, &Passenger),
        (
            With<PersonMarker>,
            Without<CarMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut person_highlight_query: Query<
        (Entity, &mut GlobalTransform, &mut Visibility, &Passenger),
        (
            With<PersonHighlightMarker>,
            Without<PersonMarker>,
            Without<CarMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
    mut player_query: Query<
        (&mut Transform, &mut Sprite, &mut PlayerCar),
        (With<PlayerMarker>, Without<RoadMarker>, Without<CarMarker>),
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

    let (mut player_transform, mut player_sprite, mut player_car) = player_query.single_mut();
    player_car.timer.tick(time.delta());
    if player_car.speed_coeff == 0.0 {
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
        player_car.atlas_left
    } else {
        player_car.atlas_right
    };

    if player_car.speed_coeff > 0.0 {
        if player_car.timer.just_finished() {
            if let Some(atlas) = &mut player_sprite.texture_atlas {
                if atlas.index >= atlas_max {
                    atlas.index = atlas_min;
                } else {
                    atlas.index += 1;
                }
            }
        }
    } else {
        if let Some(atlas) = &mut player_sprite.texture_atlas {
            atlas.index = atlas_min;
        }
    }
    if gas {
        player_car.speed_coeff = (player_car.speed_coeff + (1. * time.delta_secs())).min(1.0);
    } else {
        player_car.speed_coeff = (player_car.speed_coeff - (1. * time.delta_secs())).max(0.0);
    }

    if up_just_pressed && player_y < LANE_HEIGHT {
        player_transform.translation.y += LANE_HEIGHT;
        player_car.aabb.translate_by(vec2(0.0, LANE_HEIGHT));
    }
    if down_just_pressed && player_y > -LANE_HEIGHT {
        player_transform.translation.y -= LANE_HEIGHT;
        player_car.aabb.translate_by(vec2(0.0, -LANE_HEIGHT));
    }

    let mut allow_obstable_spawn = true;
    for (obstable_entity, mut npc_car_transform, mut npc_car) in car_query.iter_mut() {
        if npc_car_transform.translation.x > (WINDOW_X / 2.) + 200. {
            commands.entity(obstable_entity).despawn();
        } else if npc_car_transform.translation.x < -(WINDOW_X / 2.) - 200. {
            commands.entity(obstable_entity).despawn();
        }

        // TODO if player_car faces the other way it should be able to "detach" and un-intersect

        if (npc_car.intersects_npc && !npc_car.intersects_player)
            || (npc_car.intersects_player && !npc_car.blocks_player_movement)
        {
            // This looks like a car accident that brings that cars to a stop and on the road.
            // The player will pass these accidents at the same speed as the car moves along the road.
            let player_translation_speed = SPEED_X * player_car.speed_coeff * time.delta_secs();
            if facing_left {
                npc_car_transform.translation.x += player_translation_speed;
                npc_car
                    .aabb
                    .translate_by(Vec2::new(player_translation_speed, 0.0));
            } else {
                npc_car_transform.translation.x -= player_translation_speed;
                npc_car
                    .aabb
                    .translate_by(Vec2::new(-player_translation_speed, 0.0));
            }
        } else if !npc_car.intersects_player {
            let car_speed = npc_car.speed;
            if player_car.speed_coeff > 0.0 {
                let slow_down_x_translation =
                    (1. - 1.5 * player_car.speed_coeff) * car_speed * time.delta_secs();
                let speed_up_x_translation =
                    (1.0 + (1.5 * player_car.speed_coeff)) * car_speed * time.delta_secs();

                if npc_car_transform.translation.y > 0. {
                    //  car is going left
                    if facing_left {
                        npc_car_transform.translation.x -= slow_down_x_translation;
                        npc_car
                            .aabb
                            .translate_by(Vec2::new(-slow_down_x_translation, 0.0));
                    } else {
                        npc_car_transform.translation.x -= speed_up_x_translation;
                        npc_car
                            .aabb
                            .translate_by(Vec2::new(-speed_up_x_translation, 0.0));
                    }
                } else {
                    // car is going right
                    if facing_left {
                        npc_car_transform.translation.x += speed_up_x_translation;
                        npc_car
                            .aabb
                            .translate_by(Vec2::new(speed_up_x_translation, 0.0));
                    } else {
                        npc_car_transform.translation.x += slow_down_x_translation;
                        npc_car
                            .aabb
                            .translate_by(Vec2::new(slow_down_x_translation, 0.0));
                    }
                }
            } else {
                let x_translation = car_speed * time.delta_secs();
                if npc_car_transform.translation.y > 0. {
                    npc_car_transform.translation.x -= x_translation;
                    npc_car.aabb.translate_by(Vec2::new(-x_translation, 0.0));
                } else {
                    npc_car_transform.translation.x += x_translation;
                    npc_car.aabb.translate_by(Vec2::new(x_translation, 0.0));
                }
            }
        } else {
            // Speed the player_car down to a stop
            player_car.speed_coeff = (player_car.speed_coeff - (2. * time.delta_secs())).max(0.0);
        }

        if npc_car_transform.translation.x > (WINDOW_X / 2.) + 51. - 200.
            && npc_car_transform.translation.x < (WINDOW_X / 2.) + 51. + 200.
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

    for (person_entity, mut person_global_transform, mut visibility, _) in
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

    player_data.distance_traveled += SPEED_X * player_car.speed_coeff * time.delta_secs() / 1000.;

    match taxi.current_rider {
        Some(current_rider) => {
            let info = taxi
                .rides
                .iter_mut()
                .find(|ride| ride.who == current_rider)
                .unwrap();

            for (entity, _, _) in person_query
                .iter_mut()
                .filter(|(_, _, passenger)| passenger.name == info.passenger.name)
            {
                commands.entity(entity).despawn_recursive();
            }

            if !info.completed {
                travel.traveled += SPEED_X * player_car.speed_coeff * time.delta_secs() / 1000.;

                info.trip_time += time.delta_secs_f64() as f32;

                if travel.traveled > travel.distance {
                    let id = current_dialog_id.unwrap_or_default();
                    if id == String::from("drop off soon") {
                        // info!("Clear 1");
                        dialog_message.dialog = None
                    } else if player_car.speed_coeff > 0.0 {
                        dialog_message.dialog = Some(game_script.dialogs[2].clone());
                        info.distance_past_dropoff +=
                            SPEED_X * player_car.speed_coeff * time.delta_secs() / 1000.;
                        // info!("{}", info.distance_past_dropoff);
                    } else if can_drop_off
                        && player_car.speed_coeff == 0.0
                        && id == String::from("here")
                    {
                        dialog_message.dialog = None;
                    } else if can_drop_off
                        && player_car.speed_coeff == 0.0
                        && id == String::from("")
                    {
                        // SUCCESSFUL DROP OFF
                        // info!("Show bye message");
                        let distance_based_tip_adjustment = if info.distance_past_dropoff < 0.25 {
                            3.5
                        } else if info.distance_past_dropoff < 0.5 {
                            0.5
                        } else if info.distance_past_dropoff < 0.75 {
                            -2.0
                        } else if info.distance_past_dropoff < 1.75 {
                            -4.0
                        } else if info.distance_past_dropoff < 2.0 {
                            -6.0
                        } else if info.distance_past_dropoff < 3.0 {
                            -8.0
                        } else {
                            -10.
                        };

                        // Fastest time (nearly) possible
                        let fastest = info.distance * 1000. / SPEED_X;
                        let time_ratio = info.trip_time / fastest;
                        let time_past_dropoff = info.trip_time - fastest;
                        let time_ratio_based_tip_adjustment = if time_ratio < 1.1 {
                            4.0
                        } else if time_ratio < 1.2 {
                            2.5
                        } else if time_ratio < 1.3 {
                            0.5
                        } else if time_ratio < 1.4 {
                            -1.0
                        } else if time_ratio < 1.5 {
                            -3.0
                        } else if time_ratio < 1.6 {
                            -6.0
                        } else {
                            -10.
                        };

                        let time_past_dropoff_tip_adjustment = 1.0 - time_past_dropoff;

                        //

                        info.tip_percentage += distance_based_tip_adjustment
                            + time_ratio_based_tip_adjustment
                            + time_past_dropoff_tip_adjustment;

                        // info!("{}", info.tip_percentage);

                        info.tip = (info.trip_cost * (info.tip_percentage / 100.))
                            .max(0.0)
                            .floor();

                        player_data.earnings += info.trip_cost + info.tip;
                        player_data.total_earnings += info.trip_cost + info.tip;
                        dialog_message.dialog = Some(game_script.dialogs[3].clone());

                        let y = if player_y > 0. {
                            PERSON_Y_TOP
                        } else {
                            PERSON_Y_BOTTOM
                        };
                        commands
                            .spawn((
                                GameState,
                                PersonMarker,
                                info.passenger.clone(),
                                Sprite {
                                    flip_x: false,
                                    texture_atlas: Some(TextureAtlas {
                                        layout: texture_atlas_layouts.add(
                                            TextureAtlasLayout::from_grid(
                                                UVec2::new(9, 22),
                                                27,
                                                1,
                                                None,
                                                None,
                                            ),
                                        ),
                                        index: info.passenger.sprite_index,
                                    }),
                                    image: assest_server.load("person-Sheet.png"),
                                    ..default()
                                },
                            ))
                            .insert(Transform::from_xyz(player_x, y, 0.));
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
                .filter(|(_, transform, visiblility, _)| {
                    let global_transform = transform.compute_transform();
                    global_transform.translation.x.abs() <= 50.
                        && *visiblility == Visibility::Visible
                })
                .next();

            match closest_persons {
                Some((closest_rider_entity, _, _, closest_passenger)) => {
                    if player_car.speed_coeff == 0.0 {
                        let rider = taxi.rides.iter().find(|r| r.who == closest_rider_entity);
                        let mut rng = rand::thread_rng();
                        let accepted_job = match rider {
                            Some(rider) => match rider.accepted {
                                Some(b) => b,
                                None => true,
                            },
                            None => {
                                let d = rng.gen_range(0.25..=10.0);
                                taxi.closest_person = Some(closest_rider_entity);

                                taxi.rides.push(Ride {
                                    who: closest_rider_entity,
                                    passenger: closest_passenger.clone(),
                                    accepted: None,
                                    distance: ((d * 100.) as f32).round() / 100.,
                                    completed: false,
                                    trip_cost: ((d * 7.) as f32).ceil(),
                                    tip_percentage: 10.0,
                                    tip: 0.0,
                                    distance_past_dropoff: 0.0,
                                    trip_time: 0.0,
                                });
                                true
                            }
                        };
                        if accepted_job {
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

    for (person_entity, mut person_transform, _) in person_query.iter_mut() {
        if person_transform.translation.x > (WINDOW_X / 2.) + 200. {
            commands.entity(person_entity).despawn_recursive();
        } else if person_transform.translation.x < -(WINDOW_X / 2.) - 200. {
            commands.entity(person_entity).despawn_recursive();
        }

        if facing_left {
            person_transform.translation.x += SPEED_X * player_car.speed_coeff * time.delta_secs();
        } else {
            person_transform.translation.x -= SPEED_X * player_car.speed_coeff * time.delta_secs();
        }
    }

    spawn_thing_timer.timer.tick(time.delta());
    if spawn_thing_timer.timer.just_finished() {
        let mut rng = rand::thread_rng();
        let y = if random_bool_one_in_n(2) {
            PERSON_Y_TOP
        } else {
            PERSON_Y_BOTTOM
        };
        let x = if random_bool_one_in_n(2) {
            (WINDOW_X / 2.) + 51.
        } else {
            -(WINDOW_X / 2.) - 51.
        };

        let passenger_name = names::name();
        let sprite_index = rng.gen_range(0..27);
        // code to spawn goes here
        commands
            .spawn((
                GameState,
                PersonMarker,
                Passenger {
                    name: passenger_name.clone(),
                    sprite_index: sprite_index,
                },
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
                        index: sprite_index.clone(),
                    }),
                    image: assest_server.load("person-Sheet.png"),
                    ..default()
                },
            ))
            .insert(Transform::from_xyz(x, y, 0.))
            .with_children(|commands| {
                if random_bool_one_in_n(5) {
                    commands
                        .spawn((
                            GameState,
                            Sprite {
                                image: assest_server.load("player-outline.png"),
                                ..default()
                            },
                        ))
                        .insert(Passenger {
                            name: passenger_name.clone(),
                            sprite_index: sprite_index,
                        })
                        .insert(PersonHighlightMarker)
                        .insert(Transform::from_xyz(0., 0., -1.))
                        .insert(Visibility::Hidden);
                    let y = 30.;
                    commands
                        .spawn((
                            GameState,
                            Sprite {
                                image: assest_server.load("exclaimation.png"),
                                ..default()
                            },
                        ))
                        .insert(Passenger {
                            name: passenger_name.clone(),
                            sprite_index: sprite_index,
                        })
                        .insert(PersonHighlightMarker)
                        .insert(Transform::from_xyz(0., y, -1.))
                        .insert(Visibility::Hidden);
                }
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
                    GameState,
                    CarMarker,
                    Intersects::default(),
                    Car {
                        aabb: Aabb2d {
                            min: Vec2::new(x + (-89. / 2.), y + (-53. / 2.)),
                            max: Vec2::new(x + (89. / 2.), y + (53. / 2.)),
                        },
                        speed: rng.gen_range(200.0..290.0),
                        intersects_player: false,
                        intersects_npc: false,
                        blocks_player_movement: false,
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
    player_data: Res<PlayerHealth>,
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
            GameState,
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
                GameState,
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
                p.spawn((GameState, text_style.clone(), Text::default()))
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
                                text.replace("{person}", &info.passenger.name)
                                    .replace("{distance}", &info.distance.to_string())
                                    .replace("{price}", &info.trip_cost.to_string())
                                    .replace("{tip}", &info.tip.to_string())
                            } else {
                                text.clone()
                            }
                        } else {
                            text.clone()
                        };

                        let text = if dialog.id == "game over" {
                            let level = player_data.cycles_completed + 1;
                            let total_collected = player_data.total_earnings;
                            let rides_completed = taxi
                                .rides
                                .iter()
                                .filter(|r| r.completed)
                                .fold(0, |acc, _| acc + 1);
                            let total_distance =
                                (player_data.distance_traveled * 100.).round() / 100.;
                            text.replace("{level}", &level.to_string())
                                .replace("{total_collected}", &total_collected.to_string())
                                .replace("{rides_completed}", &rides_completed.to_string())
                                .replace("{total_distance}", &total_distance.to_string())
                        } else {
                            text.clone()
                        };

                        p.spawn((GameState, text_font.clone(), TextSpan::new(text.clone())));
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
                                        GameState,
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

pub fn reset(
    mut commands: Commands,
    mut reset_game: ResMut<ResetGame>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    mut travel: ResMut<Travel>,
    mut player_data: ResMut<PlayerHealth>,
    mut taxi: ResMut<Taxi>,
    mut current_selection: ResMut<CurrentSelection>,
    mut car_query: Query<
        (Entity, &mut Transform, &mut Car),
        (With<CarMarker>, Without<RoadMarker>, Without<PlayerMarker>),
    >,
    mut person_query: Query<
        (Entity, &mut Transform, &Passenger),
        (
            With<PersonMarker>,
            Without<CarMarker>,
            Without<RoadMarker>,
            Without<PlayerMarker>,
        ),
    >,
) {
    if reset_game.0 {
        for (entity, _, _) in car_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for (entity, _, _) in person_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        reset_game.0 = false;
        dialog_message.dialog = None;
        *travel = Travel::default();
        *player_data = PlayerHealth::default();
        *taxi = Taxi::default();
        current_selection.0 = String::new();
    }
}

#[derive(Resource)]
pub struct ResetGame(bool);

#[derive(Resource)]
pub struct ResumeGame(bool);

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
    mut reset_game: ResMut<ResetGame>,
    mut app_state: ResMut<NextState<AppState>>,
    mut resume_game: ResMut<ResumeGame>,
) {
    let up_key_pressed = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down_key_pressed = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let enter_key_just_pressed = keyboard_input.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]);

    let pause = keyboard_input.just_pressed(KeyCode::KeyP);
    if pause {
        resume_game.0 = true;
        app_state.set(AppState::Menu);
        return;
    }

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
            reset_game.0 = true;
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
        if up_key_pressed || down_key_pressed || enter_key_just_pressed {
            interaction_rate_limit.0.reset();
        }

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

use crate::util;
use crate::AppState;
use crate::DisplayLanguage;
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), splash_setup)
            .add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
            .add_systems(
                OnExit(AppState::Splash),
                util::despawn_screen::<OnSplashScreen>,
            );
    }
}

#[derive(Component)]
struct SplashCamera;

fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bg: ResMut<ClearColor>,
    display_language: Res<DisplayLanguage>,
) {
    bg.0 = Color::BLACK;
    info!("Splash");

    // Insert the timer as a resource
    commands.insert_resource(SplashTimer(Timer::from_seconds(2.5, TimerMode::Once)));

    commands.spawn((
        OnSplashScreen,
        SplashCamera,
        Camera2d::default(),
        RenderLayers::from_layers(&[2, 3]),
    ));

    // Display the logo
    commands
        .spawn((
            RenderLayers::layer(2),
            OnSplashScreen,
            util::window::Scalers::default(),
            Node {
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                RenderLayers::layer(2),
                OnSplashScreen,
                Node {
                    position_type: PositionType::Absolute,
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
                p.spawn((
                    RenderLayers::layer(2),
                    OnSplashScreen,
                    Node {
                        width: Val::Px(128.),
                        height: Val::Px(128.),
                        ..default()
                    },
                    ImageNode {
                        image: asset_server.load("bevylogo.png"),

                        ..default()
                    },
                ));
            });
        });

    // Display made with text
    commands
        .spawn((
            RenderLayers::layer(2),
            OnSplashScreen,
            util::window::Scalers {
                bottom: Some(Val::Px(150.)),
                ..default()
            },
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|p| {
            let text = if display_language.0 == "english" {
                "Made with Bevy"
            } else {
                "Hecho con Bevy"
            };
            p.spawn((
                RenderLayers::layer(2),
                OnSplashScreen,
                Node {
                    position_type: PositionType::Absolute,
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
                p.spawn((
                    RenderLayers::layer(2),
                    OnSplashScreen,
                    Node {
                        margin: UiRect {
                            left: Val::Px(15.),
                            top: Val::Px(15.),
                            right: Val::Px(15.),
                            bottom: Val::Px(15.),
                            ..default()
                        },

                        ..default()
                    },
                    Text::default(),
                ))
                .with_children(|p| {
                    p.spawn((
                        RenderLayers::layer(2),
                        OnSplashScreen,
                        TextFont {
                            font: asset_server.load("fonts/PressStart2P-vaV7.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextSpan::new(text),
                    ));
                });
            });
        });
}

fn countdown(
    mut app_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        app_state.set(AppState::Menu);
    }
}

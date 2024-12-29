use crate::structured_dialog;
use crate::structured_dialog::Dialog;
use crate::util;
use crate::AppState;
use crate::CurrentSelection;
use crate::DialogDisplay;
use crate::DialogTextbox;
use crate::DisplayLanguage;
use crate::InteractionRateLimit;
use crate::ResumeGame;
use crate::SelectionMarker;
use bevy::{prelude::*, render::view::RenderLayers};

#[derive(Component)]
pub struct MenuScreen;

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct LastDialog(pub Option<Dialog>);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), menu_setup)
            // .add_plugins(JsonAssetPlugin::<MenuText>::new(&[".json"]))
            .insert_resource(LastDialog(None))
            .add_systems(
                Update,
                (menu_system, menu_selection_system).run_if(in_state(AppState::Menu)),
            )
            // .add_systems(Update, debug_system.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), util::despawn_screen::<MenuScreen>);
    }
}

#[derive(Component)]
struct MenuCamera;

fn menu_setup(
    mut commands: Commands,
    mut bg: ResMut<ClearColor>,
    resume_game: Res<ResumeGame>,
    mut last_dialog: ResMut<LastDialog>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
) {
    info!("Menu");
    bg.0 = Color::srgb(0.2, 0.2, 0.2);

    commands.spawn((
        MenuScreen,
        MenuCamera,
        Camera2d::default(),
        RenderLayers::from_layers(&[2, 3]),
    ));

    let game_script = match game_script_asset.iter().next() {
        Some(d) => d.1,
        None => &structured_dialog::GameScript::default(),
    };

    let menu_id = if resume_game.resume {
        if let None = &last_dialog.0 {
            last_dialog.0 = dialog_message.dialog.clone();
        }
        "pause menu"
    } else {
        "main menu"
    };

    dialog_message.dialog = game_script
        .dialogs
        .iter()
        .filter(|d| d.id == menu_id)
        .map(|d| d.clone())
        .next();
}

pub fn menu_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    display_language: ResMut<DisplayLanguage>,
    dialog_message: ResMut<structured_dialog::DialogMessage>,
    dialog_display_query: Query<(Entity, &DialogDisplay), With<DialogDisplay>>,
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
            RenderLayers::layer(2),
            MenuScreen,
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
                RenderLayers::layer(2),
                MenuScreen,
                DialogTextbox,
                // BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
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
                p.spawn((
                    RenderLayers::layer(2),
                    MenuScreen,
                    text_style.clone(),
                    Text::default(),
                ))
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

                    p.spawn((
                        RenderLayers::layer(2),
                        MenuScreen,
                        text_font.clone(),
                        TextSpan::new(text.clone()),
                    ));
                    // info!("Should be displaying: {}", text);

                    match &dialog.choices {
                        Some(choices) => {
                            // text_container.insert(DialogTextboxChoiceMarker);

                            // let total_choices = choices.len();

                            for (_index, choice) in choices.iter().enumerate() {
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
                                    RenderLayers::layer(2),
                                    MenuScreen,
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

pub fn menu_selection_system(
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
    time: Res<Time>,
    display_language: ResMut<DisplayLanguage>,
    mut interaction_rate_limit: ResMut<InteractionRateLimit>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    resume_game: Res<ResumeGame>,
    volumes: ResMut<crate::Volumes>,
    mut current_selection: ResMut<CurrentSelection>,
    dialog_message: ResMut<structured_dialog::DialogMessage>,
    mut selections: Query<(&SelectionMarker, &mut TextSpan)>,
    app_state: ResMut<NextState<AppState>>,
) {
    let (_right, _left, gas, up, down, _pause) = match gamepads.iter().next() {
        Some(gamepad) => {
            let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
            let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap();

            (
                left_stick_x > 0.075,  //right
                left_stick_x < -0.075, //left
                gamepad.any_just_pressed([
                    GamepadButton::North,
                    GamepadButton::South,
                    GamepadButton::East,
                    GamepadButton::West,
                ]),
                left_stick_y > 0.075, //up
                left_stick_y < -0.75, //down
                gamepad.just_pressed(GamepadButton::Start),
            )
        }
        None => (false, false, false, false, false, false),
    };
    let up_key_pressed = up || keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down_key_pressed = down || keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let enter_key_just_pressed =
        gas || keyboard_input.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]);

    if enter_key_just_pressed {
        debug!(?current_selection);
        menu_options(
            current_selection.clone(),
            game_script_asset,
            dialog_message,
            app_state,
            display_language,
            volumes,
            resume_game,
        );
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

fn menu_options(
    current_selection: CurrentSelection,
    game_script_asset: Res<Assets<structured_dialog::GameScript>>,
    mut dialog_message: ResMut<structured_dialog::DialogMessage>,
    mut app_state: ResMut<NextState<AppState>>,
    mut display_language: ResMut<DisplayLanguage>,
    mut volumes: ResMut<crate::Volumes>,
    resume_game: Res<ResumeGame>,
) {
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

    if let Some(choice) = choices.iter().find(|c| c.choice == current_selection.0) {
        let game_script = match game_script_asset.iter().next() {
            Some(d) => d.1,
            None => &structured_dialog::GameScript::default(),
        };
        let next_id = &choice.dialog.actions.next_id;
        if !next_id.is_empty() {
            dialog_message.dialog = game_script
                .dialogs
                .iter()
                .filter(|d| d.id == next_id.clone())
                .map(|d| d.clone())
                .next();
        } else {
            //
            // ============ Menu Options ===============
            //
            if choice
                .dialog
                .actions
                .events_changed_on_exit
                .contains(&String::from("start_game"))
            {
                dialog_message.dialog = None;
                app_state.set(AppState::Game);
            } else if choice
                .dialog
                .actions
                .events_changed_on_exit
                .contains(&String::from("show_credits"))
            {
                dialog_message.dialog = None;
                app_state.set(AppState::Splash);
            } else {
                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("english"))
                {
                    display_language.0 = "english";
                }

                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("spanish"))
                {
                    display_language.0 = "spanish";
                }

                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("play music"))
                {
                    volumes
                        .volumes
                        .iter_mut()
                        .find(|a| a.category == "music")
                        .unwrap()
                        .volume = 0.30;
                }

                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("stop music"))
                {
                    volumes
                        .volumes
                        .iter_mut()
                        .find(|a| a.category == "music")
                        .unwrap()
                        .volume = 0.00;
                }

                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("play sfx"))
                {
                    volumes
                        .volumes
                        .iter_mut()
                        .find(|a| a.category == "sfx")
                        .unwrap()
                        .volume = 0.80;
                }

                if choice
                    .dialog
                    .actions
                    .events_changed_on_exit
                    .contains(&String::from("stop sfx"))
                {
                    volumes
                        .volumes
                        .iter_mut()
                        .find(|a| a.category == "sfx")
                        .unwrap()
                        .volume = 0.00;
                }

                let menu_id = if resume_game.resume {
                    "pause menu"
                } else {
                    "main menu"
                };

                dialog_message.dialog = game_script
                    .dialogs
                    .iter()
                    .filter(|d| d.id == menu_id)
                    .map(|d| d.clone())
                    .next();
            }
        }
    }

    return;
}

use super::*;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct GameScript {
    pub dialogs: Vec<Dialog>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Dialog {
    pub id: String,
    pub name: String,
    pub events: Vec<String>,
    pub posessions: Vec<String>,
    pub choices: Option<Vec<Choice>>,
    pub language: Language,
    pub actions: Actions,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Actions {
    pub events_changed_on_enter: Vec<String>,
    pub events_changed_on_exit: Vec<String>,
    pub items_changed_on_enter: Vec<String>,
    pub items_changed_on_exit: Vec<String>,
    pub next_id: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Language {
    pub spanish: String,
    pub english: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct Choice {
    pub choice: String,
    pub dialog: ChoiceDialog,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct ChoiceDialog {
    pub language: Language,
    pub actions: Actions,
}

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub struct DialogHandle(pub Handle<GameScript>);

#[derive(Resource, Default, Clone)]
pub struct DialogMessage {
    pub dialog: Option<Dialog>,
    pub selection_index: usize,
}

impl DialogMessage {
    pub fn reset(&mut self) {
        *self = Self {
            selection_index: 1,
            ..default()
        };
    }
}

use bevy::prelude::*;

/// Shared production contract states (Loading → MainMenu → Playing / Paused).
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
}

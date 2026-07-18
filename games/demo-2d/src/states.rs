use bevy::prelude::*;

/// Short-demo states (see docs/GAME_DOD.md).
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Victory,
    GameOver,
}

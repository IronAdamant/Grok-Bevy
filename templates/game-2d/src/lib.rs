//! Game root plugin for the 2D production template.

mod components;
mod plugins;
mod resources;
mod states;
mod systems;

use bevy::prelude::*;
use plugins::{CorePlugin, RemotePluginGate};
use states::AppState;
use systems::{
    cleanup_gameplay, cleanup_menu, gameplay_setup, loading_progress, loading_setup, menu_input,
    menu_setup, pause_input, player_movement, unpause_input,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CorePlugin, RemotePluginGate))
            .init_state::<AppState>()
            .add_systems(OnEnter(AppState::Loading), loading_setup)
            .add_systems(
                Update,
                loading_progress.run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnEnter(AppState::MainMenu), menu_setup)
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu)
            .add_systems(Update, menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::Playing), gameplay_setup)
            .add_systems(OnExit(AppState::Playing), cleanup_gameplay)
            .add_systems(
                Update,
                (player_movement, pause_input).run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, unpause_input.run_if(in_state(AppState::Paused)));
    }
}

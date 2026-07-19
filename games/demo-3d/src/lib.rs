//! Game root plugin for the 3D short-demo template (GAME_DOD).

mod components;
mod plugins;
mod resources;
mod states;
mod systems;

use bevy::prelude::*;
use plugins::{CorePlugin, RemotePluginGate};
use states::AppState;
use systems::{
    cleanup_end_screen, cleanup_gameplay, cleanup_menu, collect_pickups, end_screen_input,
    fall_off_world, game_over_setup, gameplay_setup, hazard_collision, hazard_patrol_sine,
    loading_progress, loading_setup, menu_input, menu_setup, pause_input, pause_overlay_setup,
    player_movement, unpause_input, update_hud, victory_setup,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CorePlugin, RemotePluginGate));
        #[cfg(feature = "physics")]
        {
            app.add_plugins(avian3d::PhysicsPlugins::default());
        }
        app.init_state::<AppState>()
            .add_systems(OnEnter(AppState::Loading), loading_setup)
            .add_systems(
                Update,
                loading_progress.run_if(in_state(AppState::Loading)),
            )
            .add_systems(
                OnEnter(AppState::MainMenu),
                (cleanup_gameplay, cleanup_end_screen, menu_setup).chain(),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu)
            .add_systems(Update, menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(
                OnEnter(AppState::Playing),
                (cleanup_end_screen, gameplay_setup).chain(),
            )
            .add_systems(
                Update,
                (
                    player_movement,
                    hazard_patrol_sine,
                    collect_pickups,
                    hazard_collision,
                    fall_off_world,
                    update_hud,
                    pause_input,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::Paused), pause_overlay_setup)
            .add_systems(OnExit(AppState::Paused), cleanup_end_screen)
            .add_systems(
                Update,
                (unpause_input, end_screen_input).run_if(in_state(AppState::Paused)),
            )
            .add_systems(OnEnter(AppState::Victory), victory_setup)
            .add_systems(OnEnter(AppState::GameOver), game_over_setup)
            .add_systems(
                Update,
                end_screen_input
                    .run_if(in_state(AppState::Victory).or_else(in_state(AppState::GameOver))),
            );
    }
}

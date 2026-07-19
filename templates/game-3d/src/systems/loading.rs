use bevy::prelude::*;

use crate::resources::{GameAssets, LoadingTimeout};
use crate::states::AppState;

pub fn loading_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ground_tint = asset_server.load("models/ground_tint.png");
    commands.insert_resource(GameAssets { ground_tint });
    commands.insert_resource(LoadingTimeout::default());
}

pub fn loading_progress(
    assets: Option<Res<GameAssets>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    timeout: Option<ResMut<LoadingTimeout>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(assets) = assets else {
        return;
    };
    if asset_server.is_loaded(&assets.ground_tint) {
        next.set(AppState::MainMenu);
        return;
    }
    let Some(mut timeout) = timeout else {
        return;
    };
    timeout.remaining -= time.delta_secs();
    if timeout.remaining <= 0.0 {
        bevy::log::error!(
            "Loading timed out waiting for models/ground_tint.png — advancing to MainMenu. \
             Check AssetPlugin file_path / assets/ next to the crate (debug) or binary (release)."
        );
        next.set(AppState::MainMenu);
    }
}

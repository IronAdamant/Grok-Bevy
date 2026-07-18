use bevy::prelude::*;

use crate::resources::GameAssets;
use crate::states::AppState;

pub fn loading_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ground_tint = asset_server.load("models/ground_tint.png");
    commands.insert_resource(GameAssets { ground_tint });
}

pub fn loading_progress(
    assets: Option<Res<GameAssets>>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(assets) = assets else {
        return;
    };
    if asset_server.is_loaded(&assets.ground_tint) {
        next.set(AppState::MainMenu);
    }
}

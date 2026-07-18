use bevy::prelude::*;

use crate::resources::GameAssets;
use crate::states::AppState;

pub fn loading_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player = asset_server.load("sprites/player.png");
    commands.insert_resource(GameAssets { player });
}

pub fn loading_progress(
    assets: Option<Res<GameAssets>>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(assets) = assets else {
        return;
    };
    if asset_server.is_loaded(&assets.player) {
        next.set(AppState::MainMenu);
    }
}

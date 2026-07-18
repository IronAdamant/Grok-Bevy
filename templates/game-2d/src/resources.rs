use bevy::prelude::*;

/// Handles loaded during Loading; gameplay may use them later.
#[derive(Resource, Default)]
pub struct GameAssets {
    pub player: Handle<Image>,
}

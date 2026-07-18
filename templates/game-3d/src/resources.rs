use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameAssets {
    /// Disk texture under `assets/models/ground_tint.png`.
    pub ground_tint: Handle<Image>,
}

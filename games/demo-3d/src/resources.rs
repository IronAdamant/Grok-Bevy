use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameAssets {
    pub ground_tint: Handle<Image>,
}

#[derive(Resource)]
pub struct RunProgress {
    pub collected: u32,
    pub needed: u32,
}

impl Default for RunProgress {
    fn default() -> Self {
        Self {
            collected: 0,
            needed: 3,
        }
    }
}

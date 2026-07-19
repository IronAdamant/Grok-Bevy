use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameAssets {
    pub ground_tint: Handle<Image>,
}

/// Seconds to wait for disk assets before advancing to MainMenu anyway.
pub const LOADING_TIMEOUT_SECS: f32 = 12.0;

/// Wall-clock budget for the Loading state (fail-forward if assets hang).
#[derive(Resource)]
pub struct LoadingTimeout {
    pub remaining: f32,
}

impl Default for LoadingTimeout {
    fn default() -> Self {
        Self {
            remaining: LOADING_TIMEOUT_SECS,
        }
    }
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

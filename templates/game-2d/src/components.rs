use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

/// Marker for entities despawned when leaving MainMenu.
#[derive(Component)]
pub struct MenuEntity;

/// Marker for entities despawned when leaving Playing (and Paused overlay if any).
#[derive(Component)]
pub struct GameplayEntity;

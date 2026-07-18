use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Pickup;

#[derive(Component)]
pub struct Hazard;

/// Despawned when leaving a play session (menu / restart), not on pause.
#[derive(Component)]
pub struct GameplayEntity;

#[derive(Component)]
pub struct MenuEntity;

#[derive(Component)]
pub struct EndScreenEntity;

#[derive(Component)]
pub struct HudScore;

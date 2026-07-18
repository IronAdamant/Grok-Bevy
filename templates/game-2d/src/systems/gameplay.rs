use bevy::prelude::*;

use crate::components::{GameplayEntity, Player};
use crate::resources::GameAssets;
use crate::states::AppState;

const PLAYER_SPEED: f32 = 220.0;

pub fn gameplay_setup(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((Camera2d, Name::new("MainCamera"), GameplayEntity));

    commands.spawn((
        Sprite::from_image(assets.player.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
        Name::new("Player"),
        GameplayEntity,
    ));

    // Simple visual bounds marker (procedural) so the playfield is obvious in captures.
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.14, 0.18), Vec2::new(900.0, 500.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Playfield"),
        GameplayEntity,
    ));

    commands.spawn((
        Text::new("WASD / arrows move · Esc pause"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.9, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            left: Val::Px(24.0),
            ..default()
        },
        Name::new("HudHint"),
        GameplayEntity,
    ));
}

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut tf) = q.single_mut() else {
        return;
    };
    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir != Vec3::ZERO {
        tf.translation += dir.normalize() * PLAYER_SPEED * time.delta_secs();
        // Soft bounds so the player stays on-screen for agent captures.
        tf.translation.x = tf.translation.x.clamp(-420.0, 420.0);
        tf.translation.y = tf.translation.y.clamp(-220.0, 220.0);
    }
}

pub fn pause_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Paused);
    }
}

pub fn unpause_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Playing);
    }
}

pub fn cleanup_gameplay(mut commands: Commands, q: Query<Entity, With<GameplayEntity>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

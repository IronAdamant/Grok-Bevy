use bevy::prelude::*;

use crate::components::{GameplayEntity, Hazard, HudScore, Pickup, Player};
use crate::resources::{GameAssets, RunProgress};
use crate::states::AppState;

const PLAYER_SPEED: f32 = 240.0;
const PICKUP_RADIUS: f32 = 28.0;
const HAZARD_RADIUS: f32 = 32.0;
const PLAYER_RADIUS: f32 = 22.0;

pub fn gameplay_setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    existing_player: Query<Entity, With<Player>>,
) {
    // Resume from pause: world already exists.
    if !existing_player.is_empty() {
        return;
    }

    commands.insert_resource(RunProgress::default());

    commands.spawn((Camera2d, Name::new("MainCamera"), GameplayEntity));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.10, 0.12, 0.16), Vec2::new(920.0, 520.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Playfield"),
        GameplayEntity,
    ));

    commands.spawn((
        Sprite::from_image(assets.player.clone()),
        Transform::from_xyz(0.0, -80.0, 2.0),
        Player,
        Name::new("Player"),
        GameplayEntity,
    ));

    for (i, pos) in [
        Vec3::new(-280.0, 100.0, 1.5),
        Vec3::new(0.0, 160.0, 1.5),
        Vec3::new(280.0, 80.0, 1.5),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.95, 0.75, 0.15), Vec2::splat(36.0)),
            Transform::from_translation(pos),
            Pickup,
            Name::new(format!("Pickup{i}")),
            GameplayEntity,
        ));
    }

    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.2, 0.2), Vec2::splat(40.0)),
        Transform::from_xyz(-200.0, 0.0, 1.8),
        Hazard,
        Name::new("Hazard"),
        GameplayEntity,
    ));

    commands.spawn((
        Text::new("Collect 3 orbs · avoid red · Esc pause\nOrbs: 0/3"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.93, 0.98)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        Name::new("HudScore"),
        HudScore,
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
        tf.translation.x = tf.translation.x.clamp(-420.0, 420.0);
        tf.translation.y = tf.translation.y.clamp(-220.0, 220.0);
    }
}

pub fn hazard_patrol_sine(time: Res<Time>, mut q: Query<&mut Transform, With<Hazard>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut q {
        tf.translation.x = (t * 1.4).sin() * 300.0;
        tf.translation.y = (t * 0.9).cos() * 40.0;
    }
}

pub fn collect_pickups(
    mut commands: Commands,
    mut progress: ResMut<RunProgress>,
    mut next: ResMut<NextState<AppState>>,
    player_q: Query<&Transform, With<Player>>,
    pickups: Query<(Entity, &Transform), With<Pickup>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let p = player_tf.translation.truncate();
    for (entity, tf) in &pickups {
        if p.distance(tf.translation.truncate()) < PLAYER_RADIUS + PICKUP_RADIUS {
            commands.entity(entity).despawn();
            progress.collected = progress.collected.saturating_add(1);
            if progress.collected >= progress.needed {
                next.set(AppState::Victory);
            }
        }
    }
}

pub fn hazard_collision(
    mut next: ResMut<NextState<AppState>>,
    player_q: Query<&Transform, With<Player>>,
    hazards: Query<&Transform, With<Hazard>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let p = player_tf.translation.truncate();
    for tf in &hazards {
        if p.distance(tf.translation.truncate()) < PLAYER_RADIUS + HAZARD_RADIUS {
            next.set(AppState::GameOver);
            return;
        }
    }
}

pub fn update_hud(progress: Res<RunProgress>, mut q: Query<&mut Text, With<HudScore>>) {
    for mut text in &mut q {
        **text = format!(
            "Collect 3 orbs · avoid red · Esc pause\nOrbs: {}/{}",
            progress.collected, progress.needed
        );
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

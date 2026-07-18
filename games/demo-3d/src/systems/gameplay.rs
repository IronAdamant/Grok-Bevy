use bevy::prelude::*;

use crate::components::{GameplayEntity, Hazard, HudScore, Pickup, Player};
use crate::resources::{GameAssets, RunProgress};
use crate::states::AppState;

const PLAYER_SPEED: f32 = 7.0;
const PICKUP_RADIUS: f32 = 0.9;
const HAZARD_RADIUS: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.55;

pub fn gameplay_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<GameAssets>,
    existing_player: Query<Entity, With<Player>>,
) {
    if !existing_player.is_empty() {
        return;
    }

    commands.insert_resource(RunProgress::default());

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 8.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        Name::new("MainCamera"),
        GameplayEntity,
    ));

    commands.spawn((
        PointLight {
            intensity: 2_200_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 12.0, 4.0),
        Name::new("KeyLight"),
        GameplayEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(18.0, 18.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.55, 0.4),
            base_color_texture: Some(assets.ground_tint.clone()),
            ..default()
        })),
        Transform::default(),
        Name::new("Ground"),
        GameplayEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.9))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.55, 0.95))),
        Transform::from_xyz(0.0, 1.0, 2.0),
        Player,
        Name::new("Player"),
        GameplayEntity,
    ));

    for (i, pos) in [
        Vec3::new(-4.0, 0.5, -3.0),
        Vec3::new(0.0, 0.5, -5.0),
        Vec3::new(4.0, 0.5, -2.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.7))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.9, 0.95))),
            Transform::from_translation(pos),
            Pickup,
            Name::new(format!("Pickup{i}")),
            GameplayEntity,
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.55))),
        MeshMaterial3d(materials.add(Color::srgb(0.95, 0.2, 0.2))),
        Transform::from_xyz(3.0, 0.55, 0.0),
        Hazard,
        Name::new("Hazard"),
        GameplayEntity,
    ));

    commands.spawn((
        Text::new("Collect 3 cyan cubes · avoid red · don't fall · Esc pause\nCubes: 0/3"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.94, 0.98)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
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
        dir.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if dir != Vec3::ZERO {
        tf.translation += dir.normalize() * PLAYER_SPEED * time.delta_secs();
        tf.translation.y = 1.0;
    }
}

pub fn hazard_patrol_sine(time: Res<Time>, mut q: Query<&mut Transform, With<Hazard>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut q {
        tf.translation.x = (t * 1.2).sin() * 5.0;
        tf.translation.z = (t * 0.8).cos() * 3.0;
        tf.translation.y = 0.55;
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
    let p = player_tf.translation;
    for (entity, tf) in &pickups {
        if p.distance(tf.translation) < PLAYER_RADIUS + PICKUP_RADIUS {
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
    let p = player_tf.translation;
    for tf in &hazards {
        if p.distance(tf.translation) < PLAYER_RADIUS + HAZARD_RADIUS {
            next.set(AppState::GameOver);
            return;
        }
    }
}

/// Challenge: leave the platform → game over.
pub fn fall_off_world(
    mut next: ResMut<NextState<AppState>>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(tf) = player_q.single() else {
        return;
    };
    if tf.translation.x.abs() > 9.0 || tf.translation.z.abs() > 9.0 {
        next.set(AppState::GameOver);
    }
}

pub fn update_hud(progress: Res<RunProgress>, mut q: Query<&mut Text, With<HudScore>>) {
    for mut text in &mut q {
        **text = format!(
            "Collect 3 cyan cubes · avoid red · don't fall · Esc pause\nCubes: {}/{}",
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

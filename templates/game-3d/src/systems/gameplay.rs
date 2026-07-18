use bevy::prelude::*;

use crate::components::{GameplayEntity, Player};
use crate::resources::GameAssets;
use crate::states::AppState;

const PLAYER_SPEED: f32 = 6.0;

pub fn gameplay_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<GameAssets>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 7.0, 12.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        Name::new("MainCamera"),
        GameplayEntity,
    ));

    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0),
        Name::new("KeyLight"),
        GameplayEntity,
    ));

    // Ground uses disk texture from assets/models/ground_tint.png.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.6, 0.5),
            base_color_texture: Some(assets.ground_tint.clone()),
            ..default()
        })),
        Transform::default(),
        Name::new("Ground"),
        GameplayEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.55, 0.95))),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Player,
        Name::new("Player"),
        GameplayEntity,
    ));

    commands.spawn((
        Text::new("WASD / arrows move on XZ · Esc pause"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.92, 0.95)),
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
        tf.translation.x = tf.translation.x.clamp(-10.0, 10.0);
        tf.translation.z = tf.translation.z.clamp(-10.0, 10.0);
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

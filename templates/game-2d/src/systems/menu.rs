use bevy::prelude::*;

use crate::components::MenuEntity;
use crate::states::AppState;

pub fn menu_setup(mut commands: Commands) {
    commands.spawn((Camera2d, Name::new("MainCamera"), MenuEntity));

    commands.spawn((
        Text::new("Grok-Bevy 2D — Press Enter or Space to play"),
        TextFont {
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Px(48.0),
            ..default()
        },
        Name::new("MenuText"),
        MenuEntity,
    ));
}

pub fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next.set(AppState::Playing);
    }
}

pub fn cleanup_menu(mut commands: Commands, q: Query<Entity, With<MenuEntity>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

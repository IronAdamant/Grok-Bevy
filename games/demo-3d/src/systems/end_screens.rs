use bevy::prelude::*;

use crate::components::{EndScreenEntity, GameplayEntity};
use crate::states::AppState;

pub fn victory_setup(mut commands: Commands) {
    commands.spawn((
        Text::new("VICTORY\nYou collected all cubes!\n\nEnter / Esc / Space → menu"),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        TextColor(Color::srgb(0.4, 0.95, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(140.0),
            left: Val::Px(60.0),
            ..default()
        },
        Name::new("VictoryText"),
        EndScreenEntity,
    ));
}

pub fn game_over_setup(mut commands: Commands) {
    commands.spawn((
        Text::new("GAME OVER\nHazard or fell off the platform.\n\nEnter / Esc / Space → menu"),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.45, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(140.0),
            left: Val::Px(60.0),
            ..default()
        },
        Name::new("GameOverText"),
        EndScreenEntity,
    ));
}

pub fn pause_overlay_setup(mut commands: Commands) {
    commands.spawn((
        Text::new("PAUSED\nEsc/Enter: resume · M: main menu"),
        TextFont {
            font_size: FontSize::Px(26.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(180.0),
            left: Val::Px(60.0),
            ..default()
        },
        Name::new("PauseText"),
        EndScreenEntity,
    ));
}

pub fn cleanup_end_screen(mut commands: Commands, q: Query<Entity, With<EndScreenEntity>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub fn end_screen_input(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    mut commands: Commands,
    gameplay: Query<Entity, With<GameplayEntity>>,
    end: Query<Entity, With<EndScreenEntity>>,
) {
    let to_menu = |commands: &mut Commands,
                   gameplay: &Query<Entity, With<GameplayEntity>>,
                   end: &Query<Entity, With<EndScreenEntity>>,
                   next: &mut ResMut<NextState<AppState>>| {
        for e in gameplay {
            commands.entity(e).despawn();
        }
        for e in end {
            commands.entity(e).despawn();
        }
        next.set(AppState::MainMenu);
    };

    match state.get() {
        AppState::Paused => {
            if keys.just_pressed(KeyCode::KeyM) {
                to_menu(&mut commands, &gameplay, &end, &mut next);
            }
        }
        AppState::Victory | AppState::GameOver => {
            if keys.just_pressed(KeyCode::Enter)
                || keys.just_pressed(KeyCode::Escape)
                || keys.just_pressed(KeyCode::Space)
            {
                to_menu(&mut commands, &gameplay, &end, &mut next);
            }
        }
        _ => {}
    }
}

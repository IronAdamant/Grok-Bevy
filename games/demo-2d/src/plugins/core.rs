use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "demo_2d (2D)".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }));
    }
}

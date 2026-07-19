use bevy::prelude::*;

pub struct CorePlugin;

/// Asset folder for `AssetServer` paths (`models/…`, etc.).
///
/// - Debug: crate-root `assets/` via `CARGO_MANIFEST_DIR` (works when running
///   `target/debug/*` from any cwd).
/// - Release: relative `"assets"` next to the binary / launch cwd (packaging).
/// - Override: set env `BEVY_ASSET_ROOT` to an absolute or relative path.
fn asset_root() -> String {
    if let Ok(p) = std::env::var("BEVY_ASSET_ROOT") {
        return p;
    }
    #[cfg(debug_assertions)]
    {
        format!("{}/assets", env!("CARGO_MANIFEST_DIR"))
    }
    #[cfg(not(debug_assertions))]
    {
        "assets".into()
    }
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "demo_3d (3D)".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                }),
        );
    }
}

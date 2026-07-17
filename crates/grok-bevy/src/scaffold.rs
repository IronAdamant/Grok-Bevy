//! Scaffold a BRP-enabled Bevy sample application.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn scaffold_sample_app(path: &Path, force: bool) -> Result<()> {
    if path.exists() {
        if !force {
            bail!(
                "destination {} already exists (pass --force to overwrite files)",
                path.display()
            );
        }
    } else {
        fs::create_dir_all(path)
            .with_context(|| format!("create {}", path.display()))?;
    }

    let src = path.join("src");
    fs::create_dir_all(&src)?;

    fs::write(path.join("Cargo.toml"), SAMPLE_CARGO_TOML)?;
    fs::write(src.join("main.rs"), SAMPLE_MAIN_RS)?;
    fs::write(path.join("README.md"), SAMPLE_README)?;

    println!("Scaffolded BRP-enabled Bevy sample at {}", path.display());
    println!("  Features: default (game), `remote` (BRP), `capture` (brp extras + png)");
    println!("  Run with remote+capture:");
    println!(
        "    cargo run --manifest-path {} --features remote,capture",
        path.join("Cargo.toml").display()
    );
    Ok(())
}

const SAMPLE_CARGO_TOML: &str = r#"[package]
name = "grok_bevy_sample"
version = "0.1.0"
edition = "2021"
publish = false
license = "MIT OR Apache-2.0"
description = "Grok-Bevy sample app with RemotePlugin / BrpExtras feature flags"

[dependencies]
bevy = { version = "0.19", default-features = true, features = ["png"] }

# Optional remote control stack (Bevy 0.19 track)
bevy_brp_extras = { version = "0.22.1", optional = true }

[features]
default = []
# Enable Bevy Remote Protocol HTTP server via bevy_brp_extras (adds RemotePlugin + RemoteHttpPlugin).
remote = ["dep:bevy_brp_extras", "bevy/bevy_remote"]
# Screenshot / diagnostics extras (implies remote).
capture = ["remote"]

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
"#;

const SAMPLE_MAIN_RS: &str = r#"//! Grok-Bevy sample: minimal 3D scene with optional BRP + capture.
//!
//! ```sh
//! cargo run --features remote,capture
//! ```
//!
//! Then from another terminal:
//! ```sh
//! grok-bevy brp wait --port 15702
//! grok-bevy brp query --port 15702
//! grok-bevy brp screenshot --path captures/scene.png --with-image-meta
//! ```

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Grok-Bevy Sample".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }));

    #[cfg(feature = "remote")]
    {
        // BrpExtrasPlugin enables RemotePlugin + RemoteHttpPlugin (port 15702)
        // and registers brp_extras/* methods (screenshot, diagnostics, input, …).
        app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
    }

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(8.0, 8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.35, 0.25))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Ground"),
    ));

    // Cube the agent can query / mutate
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.3))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Name::new("AgentCube"),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            intensity: 2_000_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("KeyLight"),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        Name::new("MainCamera"),
    ));
}
"#;

const SAMPLE_README: &str = r#"# Grok-Bevy sample app

Minimal Bevy 0.19 3D scene with optional **Bevy Remote Protocol** control.

## Features

| Feature   | What it enables |
|-----------|-----------------|
| (default) | Playable 3D scene only |
| `remote`  | `bevy_brp_extras` → RemotePlugin + HTTP on port **15702** |
| `capture` | Same as `remote` (screenshot / diagnostics methods) |

## Run

```bash
cargo run --features remote,capture
```

## Control from Grok-Bevy

```bash
# in another terminal (from the Grok-Bevy repo)
cargo run -p grok-bevy -- brp wait --port 15702
cargo run -p grok-bevy -- brp query --port 15702
cargo run -p grok-bevy -- brp screenshot --path captures/scene.png --with-image-meta
```

Or use the MCP server (`grok-bevy mcp`) / `bevy_brp_mcp` from Grok Build.
"#;

//! Production 3D starter — thin entry only.
//!
//! ```sh
//! cargo run --features remote,capture
//! ```
//!
//! Controls: Enter/Space from menu → play; WASD/arrows move on XZ; Esc pauses.

use bevy::prelude::*;
use demo_3d::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

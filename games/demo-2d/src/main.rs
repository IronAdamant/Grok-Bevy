//! Production 2D starter — thin entry only.
//!
//! ```sh
//! cargo run --features remote,capture
//! ```
//!
//! Controls: Enter/Space from menu → play; WASD/arrows move; Esc pauses.

use bevy::prelude::*;
use demo_2d::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

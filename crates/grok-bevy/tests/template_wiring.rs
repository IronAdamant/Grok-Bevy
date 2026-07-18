//! Structural tests: sample template enables remote/capture stack.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn sample_cargo_features_gate_remote_and_capture() {
    let toml = fs::read_to_string(repo_root().join("templates/sample-app/Cargo.toml"))
        .expect("sample Cargo.toml");
    assert!(
        toml.contains("remote") && toml.contains("bevy_brp_extras"),
        "remote feature must pull bevy_brp_extras"
    );
    assert!(
        toml.contains("capture") && toml.contains("remote"),
        "capture should imply remote"
    );
    assert!(
        toml.contains("bevy/bevy_remote") || toml.contains("bevy_remote"),
        "must enable bevy_remote feature"
    );
}

#[test]
fn sample_main_enables_brp_extras_behind_feature() {
    let main = fs::read_to_string(repo_root().join("templates/sample-app/src/main.rs"))
        .expect("sample main");
    assert!(main.contains("BrpExtrasPlugin"));
    assert!(main.contains(r#"feature = "remote""#));
    assert!(main.contains("bevy_brp_extras"));
}

#[test]
fn headless_binary_enables_remote_plugin() {
    let src = fs::read_to_string(repo_root().join("templates/sample-app/src/bin/brp_headless.rs"))
        .expect("headless");
    assert!(src.contains("RemotePlugin"));
    assert!(src.contains("RemoteHttpPlugin"));
}

#[test]
fn licenses_and_docs_present() {
    let root = repo_root();
    for rel in [
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "README.md",
        "CONTRIBUTING.md",
        "CHANGELOG.md",
        ".github/workflows/ci.yml",
        "docs/FAST_START.md",
        "docs/TROUBLESHOOTING.md",
        "docs/PRODUCTION_GAMES.md",
        "docs/ASSET_CONVENTIONS.md",
        "docs/SHIPPING.md",
        "docs/GAME_DOD.md",
        "docs/ROADMAP.md",
        "AGENTS.md",
        ".grok/skills/bevy-demo-game/SKILL.md",
    ] {
        let p = root.join(rel);
        assert!(p.is_file(), "missing {rel}");
    }
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    assert!(readme.contains("mcp_servers.grok-bevy") || readme.contains("grok mcp add"));
    assert!(readme.contains("0.19"));
    assert!(readme.contains("bevy_brp_mcp"));
    let shipping = fs::read_to_string(root.join("docs/SHIPPING.md")).unwrap();
    assert!(shipping.contains("cargo build --release"));
    let assets = fs::read_to_string(root.join("docs/ASSET_CONVENTIONS.md")).unwrap();
    assert!(assets.contains("assets/sprites"));
    assert!(assets.contains("assets/models"));
}

#[test]
fn production_2d_template_is_playable_slice() {
    let root = repo_root().join("templates/game-2d");
    assert!(root.join("assets/sprites/player.png").is_file());
    let gameplay = fs::read_to_string(root.join("src/systems/gameplay.rs")).unwrap();
    assert!(gameplay.contains("player_movement"));
    assert!(gameplay.contains("ButtonInput") || gameplay.contains("KeyCode"));
    let states = fs::read_to_string(root.join("src/states.rs")).unwrap();
    assert!(states.contains("MainMenu") && states.contains("Playing"));
    let loading = fs::read_to_string(root.join("src/systems/loading.rs")).unwrap();
    assert!(loading.contains("sprites/player.png"));
    let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("remote") && cargo.contains("capture"));
}

#[test]
fn production_3d_template_is_playable_slice() {
    let root = repo_root().join("templates/game-3d");
    assert!(root.join("assets/models/ground_tint.png").is_file());
    let gameplay = fs::read_to_string(root.join("src/systems/gameplay.rs")).unwrap();
    assert!(gameplay.contains("player_movement"));
    assert!(gameplay.contains("Camera3d"));
    let loading = fs::read_to_string(root.join("src/systems/loading.rs")).unwrap();
    assert!(loading.contains("models/ground_tint.png"));
}

/// GAME_DOD: short demos need objective, challenge, win/lose — not movement-only.
fn assert_game_dod_sources(root: &std::path::Path) {
    let states = fs::read_to_string(root.join("src/states.rs")).expect("states");
    assert!(
        states.contains("Victory") && states.contains("GameOver"),
        "{} missing Victory/GameOver states",
        root.display()
    );
    let gameplay = fs::read_to_string(root.join("src/systems/gameplay.rs")).expect("gameplay");
    assert!(
        gameplay.contains("collect_pickups") || gameplay.contains("Pickup"),
        "{} missing pickup/collect objective",
        root.display()
    );
    assert!(
        gameplay.contains("Hazard") || gameplay.contains("hazard"),
        "{} missing hazard challenge",
        root.display()
    );
    assert!(
        gameplay.contains("Player") && gameplay.contains("Name::new(\"Player\")"),
        "{} missing named Player",
        root.display()
    );
    let readme = fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.to_lowercase().contains("objective") || readme.contains("Collect"),
        "{} README must state objective",
        root.display()
    );
    assert!(
        readme.contains("WASD") || readme.contains("controls") || readme.contains("Controls"),
        "{} README must document controls",
        root.display()
    );
}

#[test]
fn game_2d_template_meets_game_dod() {
    assert_game_dod_sources(&repo_root().join("templates/game-2d"));
    let g = fs::read_to_string(repo_root().join("templates/game-2d/src/systems/gameplay.rs")).unwrap();
    assert!(g.contains("AppState::Victory") || g.contains("Victory"));
}

#[test]
fn game_3d_template_meets_game_dod() {
    assert_game_dod_sources(&repo_root().join("templates/game-3d"));
    let g = fs::read_to_string(repo_root().join("templates/game-3d/src/systems/gameplay.rs")).unwrap();
    assert!(g.contains("fall_off_world") || g.contains("GameOver"));
}

#[test]
fn dogfood_demos_exist_and_meet_game_dod() {
    let root = repo_root();
    for rel in ["games/demo-2d", "games/demo-3d"] {
        let p = root.join(rel);
        assert!(p.join("Cargo.toml").is_file(), "missing {rel}");
        assert_game_dod_sources(&p);
        assert!(p.join("assets").is_dir(), "{rel} assets");
    }
    let ws = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(ws.contains("games/demo-2d") && ws.contains("games/demo-3d"));
}

#[test]
fn package_demo_script_and_docs_exist() {
    let root = repo_root();
    let script = root.join("scripts/package-demo.sh");
    assert!(script.is_file(), "package-demo.sh missing");
    let body = fs::read_to_string(&script).unwrap();
    assert!(body.contains("cargo build --release"));
    assert!(body.contains("assets"));
    let pack = fs::read_to_string(root.join("docs/PACKAGING.md")).unwrap();
    assert!(pack.contains("package-demo.sh"));
    assert!(pack.contains("assets/"));
    let skill = fs::read_to_string(root.join(".grok/skills/bevy-package/SKILL.md")).unwrap();
    assert!(skill.contains("package-demo.sh") || skill.contains("dist/"));
}

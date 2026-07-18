//! Scaffold BRP-enabled Bevy games from in-repo templates.
//!
//! Templates under `<repo>/templates/{game-2d,game-3d,sample-app}` are the source of
//! truth. Scaffold copies the tree and substitutes package / window title tokens.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Which project template to materialize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaffoldKind {
    /// Production 2D vertical slice (`templates/game-2d`).
    TwoD,
    /// Production 3D vertical slice (`templates/game-3d`).
    ThreeD,
    /// BRP integration fixture (`templates/sample-app`) — not a production game.
    Demo,
}

impl ScaffoldKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "2d" | "two-d" | "twod" => Ok(Self::TwoD),
            "3d" | "three-d" | "threed" => Ok(Self::ThreeD),
            "demo" | "sample" | "fixture" => Ok(Self::Demo),
            other => bail!(
                "unknown scaffold kind '{other}' (expected 2d, 3d, or demo)"
            ),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TwoD => "2d",
            Self::ThreeD => "3d",
            Self::Demo => "demo",
        }
    }

    fn template_dir_name(self) -> &'static str {
        match self {
            Self::TwoD => "game-2d",
            Self::ThreeD => "game-3d",
            Self::Demo => "sample-app",
        }
    }
}

/// Resolve the templates directory (source of truth for scaffold).
///
/// Order: `GROK_BEVY_TEMPLATE_ROOT` env, then workspace `templates/` relative to
/// this crate's `CARGO_MANIFEST_DIR` (works for `cargo run` / `cargo install --path`).
pub fn template_root() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("GROK_BEVY_TEMPLATE_ROOT") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Ok(path);
        }
        bail!(
            "GROK_BEVY_TEMPLATE_ROOT={} is not a directory",
            path.display()
        );
    }
    let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    if from_manifest.is_dir() {
        return from_manifest
            .canonicalize()
            .with_context(|| format!("canonicalize {}", from_manifest.display()));
    }
    bail!(
        "could not find Grok-Bevy templates/ (looked at {}). Set GROK_BEVY_TEMPLATE_ROOT.",
        from_manifest.display()
    )
}

/// Normalize a user package name to a valid Cargo/Rust crate name (snake_case).
pub fn normalize_package_name(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '_' || ch == ' ' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        return "my_bevy_game".into();
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return format!("game_{out}");
    }
    out
}

/// Default package name from destination path or kind.
pub fn default_package_name(path: &Path, kind: ScaffoldKind) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(normalize_package_name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| match kind {
            ScaffoldKind::TwoD => "my_bevy_2d_game".into(),
            ScaffoldKind::ThreeD => "my_bevy_3d_game".into(),
            ScaffoldKind::Demo => "grok_bevy_sample".into(),
        })
}

/// Scaffold a game project into `dest`.
pub fn scaffold_app(
    dest: &Path,
    kind: ScaffoldKind,
    package_name: Option<&str>,
    force: bool,
) -> Result<()> {
    let pkg = normalize_package_name(package_name.unwrap_or(&default_package_name(dest, kind)));
    let title = match kind {
        ScaffoldKind::TwoD => format!("{pkg} (2D)"),
        ScaffoldKind::ThreeD => format!("{pkg} (3D)"),
        ScaffoldKind::Demo => format!("{pkg} (demo)"),
    };

    let templates = template_root()?;
    let src = templates.join(kind.template_dir_name());
    if !src.is_dir() {
        bail!("template missing: {}", src.display());
    }

    if dest.exists() {
        if !force {
            bail!(
                "destination {} already exists (pass --force to overwrite)",
                dest.display()
            );
        }
        if dest.is_dir() {
            fs::remove_dir_all(dest)
                .with_context(|| format!("remove existing {}", dest.display()))?;
        } else {
            bail!("{} exists and is not a directory", dest.display());
        }
    }
    fs::create_dir_all(dest).with_context(|| format!("create {}", dest.display()))?;

    copy_template(&src, dest, &pkg, &title, kind)?;

    // Ensure asset layout contract even for demo.
    ensure_asset_dirs(dest, kind)?;

    // Demo template may lack AGENTS; always write production-oriented project rules.
    write_project_agents(dest, kind, &pkg)?;

    println!(
        "Scaffolded Bevy {} project at {}",
        kind.as_str(),
        dest.display()
    );
    println!("  package: {pkg}");
    println!("  Features: `remote` (BRP), `capture` (screenshots)");
    println!(
        "  Run: cargo run --manifest-path {} --features remote,capture",
        dest.join("Cargo.toml").display()
    );
    if matches!(kind, ScaffoldKind::Demo) {
        println!("  Note: demo is a BRP fixture, not a production game template.");
    }
    Ok(())
}

fn ensure_asset_dirs(dest: &Path, kind: ScaffoldKind) -> Result<()> {
    let dirs: &[&str] = match kind {
        ScaffoldKind::TwoD => &["assets/sprites", "assets/ui", "assets/audio"],
        ScaffoldKind::ThreeD => &[
            "assets/models",
            "assets/ui",
            "assets/audio",
            "assets/sprites",
        ],
        ScaffoldKind::Demo => &["assets/sprites", "assets/models", "assets/ui", "assets/audio"],
    };
    for d in dirs {
        let p = dest.join(d);
        fs::create_dir_all(&p).with_context(|| format!("create {}", p.display()))?;
        let keep = p.join(".gitkeep");
        if !keep.exists() && fs::read_dir(&p).map(|mut i| i.next().is_none()).unwrap_or(true) {
            let _ = fs::write(&keep, b"");
        }
    }
    Ok(())
}

fn write_project_agents(dest: &Path, kind: ScaffoldKind, pkg: &str) -> Result<()> {
    let path = dest.join("AGENTS.md");
    // Prefer template AGENTS when present (already token-replaced); refresh always for contract.
    let body = match kind {
        ScaffoldKind::TwoD => format!(
            r#"# Project agent rules — 2D Bevy game (`{pkg}`)

## Pins
- Bevy **0.19**, bevy_brp_extras **0.22.1**, BRP port **15702**
- Features: `remote`, `capture`

## Skills
1. `bevy-production` + `bevy-2d-game`
2. Art → `game-asset-core` (+ specialist)
3. Live verify → `bevy-agent-loop`

## Assets
`assets/sprites/`, `assets/ui/`, `assets/audio/` (paths relative to `assets/` for AssetServer).

## States
`Loading` → `MainMenu` → `Playing` / `Paused`

## Ship
```bash
cargo build --release
```
See Grok-Bevy `docs/ASSET_CONVENTIONS.md` and `docs/SHIPPING.md` when available.
"#
        ),
        ScaffoldKind::ThreeD => format!(
            r#"# Project agent rules — 3D Bevy game (`{pkg}`)

## Pins
- Bevy **0.19**, bevy_brp_extras **0.22.1**, BRP port **15702**
- Features: `remote`, `capture`

## Skills
1. `bevy-production` + `bevy-3d-game`
2. UI/art → `game-asset-core` (+ specialist)
3. Live verify → `bevy-agent-loop`

## Assets
`assets/models/`, `assets/ui/`, `assets/audio/` (optional `sprites/`).

## States
`Loading` → `MainMenu` → `Playing` / `Paused`

## Ship
```bash
cargo build --release
```
See Grok-Bevy `docs/ASSET_CONVENTIONS.md` and `docs/SHIPPING.md` when available.
"#
        ),
        ScaffoldKind::Demo => format!(
            r#"# Project agent rules — BRP demo fixture (`{pkg}`)

This is a **demo/fixture** for BRP smoke tests, not a production game.

For a real game:
```bash
grok-bevy scaffold --kind 2d --path ./my-game
# or --kind 3d
```

Load skills: `bevy-production` + dimensional skill + `bevy-agent-loop`.

Ship (if you expand this into a game): `cargo build --release`
"#
        ),
    };
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn copy_template(
    src: &Path,
    dest: &Path,
    package_name: &str,
    window_title: &str,
    kind: ScaffoldKind,
) -> Result<()> {
    copy_dir_recursive(src, dest, package_name, window_title, kind)
}

fn copy_dir_recursive(
    src: &Path,
    dest: &Path,
    package_name: &str,
    window_title: &str,
    kind: ScaffoldKind,
) -> Result<()> {
    fs::create_dir_all(dest).with_context(|| format!("create {}", dest.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("read_dir {}", src.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        // Skip target/ and hidden cargo lock noise if any.
        if name == "target" || name == "Cargo.lock" {
            continue;
        }
        let from = entry.path();
        let to = dest.join(&name);
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to, package_name, window_title, kind)?;
        } else if file_type.is_file() {
            copy_file_substituted(&from, &to, package_name, window_title, kind)?;
        }
    }
    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "md" | "txt" | "json" | "yml" | "yaml")
    )
}

fn copy_file_substituted(
    from: &Path,
    to: &Path,
    package_name: &str,
    window_title: &str,
    kind: ScaffoldKind,
) -> Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    if is_text_file(from) {
        let mut text = fs::read_to_string(from)
            .with_context(|| format!("read {}", from.display()))?;
        text = text.replace("__PACKAGE_NAME__", package_name);
        text = text.replace("__WINDOW_TITLE__", window_title);
        // Demo template lives in the workspace; rewrite for a standalone project.
        if matches!(kind, ScaffoldKind::Demo) {
            text = text.replace("grok_bevy_sample", package_name);
            text = text.replace("Grok-Bevy Sample", window_title);
            text = text.replace("Grok-Bevy sample", window_title);
            text = text.replace(
                "bevy = { workspace = true, features = [\"png\"] }",
                "bevy = { version = \"0.19\", default-features = true, features = [\"png\"] }",
            );
            text = text.replace(
                "bevy_brp_extras = { workspace = true, optional = true }",
                "bevy_brp_extras = { version = \"0.22.1\", optional = true }",
            );
        }
        fs::write(to, text).with_context(|| format!("write {}", to.display()))?;
    } else {
        fs::copy(from, to).with_context(|| format!("copy {} → {}", from.display(), to.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_kinds() {
        assert_eq!(ScaffoldKind::parse("2d").unwrap(), ScaffoldKind::TwoD);
        assert_eq!(ScaffoldKind::parse("3D").unwrap(), ScaffoldKind::ThreeD);
        assert_eq!(ScaffoldKind::parse("demo").unwrap(), ScaffoldKind::Demo);
        assert!(ScaffoldKind::parse("4d").is_err());
    }

    #[test]
    fn package_name_normalization() {
        assert_eq!(normalize_package_name("My-Cool Game"), "my_cool_game");
        assert_eq!(normalize_package_name("123go"), "game_123go");
    }

    #[test]
    fn template_root_exists() {
        let root = template_root().expect("template root");
        assert!(root.join("game-2d").is_dir());
        assert!(root.join("game-3d").is_dir());
        assert!(root.join("sample-app").is_dir());
    }

    #[test]
    fn scaffold_2d_writes_playable_tree() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("cool_game");
        scaffold_app(&dest, ScaffoldKind::TwoD, Some("cool_game"), true).unwrap();

        assert!(dest.join("Cargo.toml").is_file());
        assert!(dest.join("src/main.rs").is_file());
        assert!(dest.join("src/lib.rs").is_file());
        assert!(dest.join("src/systems/gameplay.rs").is_file());
        assert!(dest.join("assets/sprites/player.png").is_file());
        assert!(dest.join("AGENTS.md").is_file());

        let cargo = fs::read_to_string(dest.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"cool_game\""));
        assert!(cargo.contains("remote") && cargo.contains("capture"));

        let gameplay = fs::read_to_string(dest.join("src/systems/gameplay.rs")).unwrap();
        assert!(gameplay.contains("player_movement"));
        assert!(gameplay.contains("KeyCode"));
        assert!(gameplay.contains("ButtonInput"));

        let states = fs::read_to_string(dest.join("src/states.rs")).unwrap();
        assert!(states.contains("MainMenu") && states.contains("Playing"));

        let loading = fs::read_to_string(dest.join("src/systems/loading.rs")).unwrap();
        assert!(loading.contains("sprites/player.png"));
        assert!(loading.contains("AssetServer"));

        let main = fs::read_to_string(dest.join("src/main.rs")).unwrap();
        assert!(main.contains("cool_game::GamePlugin"));
        assert!(!main.contains("__PACKAGE_NAME__"));
    }

    #[test]
    fn scaffold_3d_writes_playable_tree() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("space_run");
        scaffold_app(&dest, ScaffoldKind::ThreeD, Some("space_run"), true).unwrap();

        assert!(dest.join("assets/models/ground_tint.png").is_file());
        let gameplay = fs::read_to_string(dest.join("src/systems/gameplay.rs")).unwrap();
        assert!(gameplay.contains("player_movement"));
        assert!(gameplay.contains("Camera3d"));
        let loading = fs::read_to_string(dest.join("src/systems/loading.rs")).unwrap();
        assert!(loading.contains("models/ground_tint.png"));
        let agents = fs::read_to_string(dest.join("AGENTS.md")).unwrap();
        assert!(agents.contains("cargo build --release"));
        assert!(agents.contains("assets/models"));
    }

    #[test]
    fn scaffold_demo_rewrites_package() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("fixture_app");
        scaffold_app(&dest, ScaffoldKind::Demo, Some("fixture_app"), true).unwrap();
        let cargo = fs::read_to_string(dest.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"fixture_app\""));
        assert!(dest.join("src/main.rs").is_file());
        assert!(dest.join("AGENTS.md").is_file());
        assert!(dest.join("assets/sprites").is_dir());
    }
}

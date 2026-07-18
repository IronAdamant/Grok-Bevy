# __WINDOW_TITLE__

Production **2D** Bevy 0.19 vertical slice scaffolded by Grok-Bevy.

## Play

```bash
cargo run --features remote,capture
```

| Input | Action |
|-------|--------|
| Enter / Space | Start from menu |
| WASD / arrows | Move player |
| Esc | Pause / unpause |

## Features

| Feature | Enables |
|---------|---------|
| `remote` | BRP HTTP on port **15702** (`bevy_brp_extras`) |
| `capture` | Same as `remote` (screenshots / diagnostics) |

## Asset paths

Paths are relative to the `assets/` folder (Bevy `AssetServer`):

| Path | Use |
|------|-----|
| `assets/sprites/` | Characters, props (`sprites/player.png`) |
| `assets/ui/` | HUD / menu chrome |
| `assets/audio/` | SFX / music |

See project `AGENTS.md` and upstream Grok-Bevy docs: asset conventions + shipping.

## Ship

```bash
cargo build --release
```

Binary: `target/release/__PACKAGE_NAME__`. See shipping notes in the Grok-Bevy repo (`docs/SHIPPING.md`) when developing from the monorepo.

## Agent control

```bash
grok-bevy brp wait --port 15702
grok-bevy brp query --port 15702
grok-bevy brp screenshot --path captures/scene.png
```

Or use MCP: `bevy_launch_app`, `bevy_capture_viewport`, …

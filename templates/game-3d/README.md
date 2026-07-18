# __WINDOW_TITLE__

Production **3D** Bevy 0.19 vertical slice scaffolded by Grok-Bevy.

## Play

```bash
cargo run --features remote,capture
```

| Input | Action |
|-------|--------|
| Enter / Space | Start from menu |
| WASD / arrows | Move player on the XZ plane |
| Esc | Pause / unpause |

## Features

| Feature | Enables |
|---------|---------|
| `remote` | BRP HTTP on port **15702** |
| `capture` | Screenshots / diagnostics (implies remote) |

## Asset paths

| Path | Use |
|------|-----|
| `assets/models/` | Meshes, glTF, material textures (`models/ground_tint.png`) |
| `assets/ui/` | HUD / menu chrome |
| `assets/audio/` | SFX / music |
| `assets/sprites/` | Optional 2D overlays |

## Ship

```bash
cargo build --release
```

Binary: `target/release/__PACKAGE_NAME__`. See Grok-Bevy `docs/SHIPPING.md` for packaging notes.

## Agent control

```bash
grok-bevy brp wait --port 15702
grok-bevy brp query --port 15702
grok-bevy brp screenshot --path captures/scene.png
```

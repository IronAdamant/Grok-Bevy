# Grok-Bevy — agent project rules

## Mission

Help users build **production Bevy games** (playable, modular, shippable), not one-file demos. Use this repo’s skills, pins, and MCP tools.

## Version pins (do not freestyle)

| Component | Version |
|-----------|---------|
| Bevy | **0.19** |
| bevy_brp_mcp / bevy_brp_extras | **0.22.1** |
| Default BRP port | **15702** |
| Features | `remote`, `capture` (`capture` implies `remote`) |

## Skill routing (load these)

| User intent | Load first |
|-------------|------------|
| Build / structure / ship a Bevy game | `bevy-production` |
| 2D (sprites, tiles, top-down, platformer) | `bevy-production` + `bevy-2d-game` |
| 3D (meshes, glTF, lighting) | `bevy-production` + `bevy-3d-game` |
| Live run, BRP, screenshot, MCP iterate | `bevy-agent-loop` |
| Generate game art | Grok `game-asset-core` + matching specialist (`game-animation-frames`, `game-character-consistency`, `game-tilesets`, `game-ui-icons`) |

Skills live under `.grok/skills/`. See [docs/PRODUCTION_GAMES.md](docs/PRODUCTION_GAMES.md).

## Layout contract for real games

Prefer (or create) this structure over a single `main.rs` scene:

- `src/main.rs` — thin entry only  
- `src/lib.rs` + `plugins/`, `states.rs`, `systems/`, `components.rs`, `resources.rs`  
- `assets/{sprites|models,ui,audio}/`  
- App states: `Loading` → `MainMenu` → `Playing` (+ `Paused`)  
- Feature-gated remote: `BrpExtrasPlugin` under `feature = "remote"`

## Anti-demo rules

For any request to **build a game** (not “smoke-test BRP”):

- **Do not** stop at the static cube sample (`templates/sample-app`).  
- **Do** implement input + `Update` systems and state transitions.  
- **Do** use at least one disk asset under `assets/` when art exists.  
- **Do** verify with capture when the MCP server is available (`bevy-agent-loop`).

The sample app is an **integration fixture** for doctor/BRP/capture CI — not a product template.

## MCP tools (this repo)

`bevy_env_check`, `bevy_launch_app`, `bevy_brp_*`, `bevy_capture_viewport`. Prefer full **`bevy_brp_mcp`** for hierarchy, watches, and input injection when installed.

## Docs

- Production: [docs/PRODUCTION_GAMES.md](docs/PRODUCTION_GAMES.md)  
- Fast start: [docs/FAST_START.md](docs/FAST_START.md)  
- Troubleshooting: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)  

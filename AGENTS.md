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
| Complete short **demo** (objective, win/lose) | `bevy-demo-game` + dimensional skill + `bevy-production` |
| 2D (sprites, tiles, top-down, platformer) | `bevy-production` + `bevy-2d-game` (+ `bevy-demo-game` for full demos) |
| 3D (meshes, glTF, lighting) | `bevy-production` + `bevy-3d-game` (+ `bevy-demo-game` for full demos) |
| Live run, BRP, screenshot, MCP iterate | `bevy-agent-loop` |
| Generate game art | Grok `game-asset-core` + matching specialist (`game-animation-frames`, `game-character-consistency`, `game-tilesets`, `game-ui-icons`) |

Skills live under `.grok/skills/`. See [docs/PRODUCTION_GAMES.md](docs/PRODUCTION_GAMES.md), [docs/GAME_DOD.md](docs/GAME_DOD.md), [docs/ROADMAP.md](docs/ROADMAP.md).
## Layout contract for real games

Prefer (or create) this structure over a single `main.rs` scene:

- `src/main.rs` — thin entry only  
- `src/lib.rs` + `plugins/`, `states.rs`, `systems/`, `components.rs`, `resources.rs`  
- `assets/{sprites|models,ui,audio}/`  
- App states: `Loading` → `MainMenu` → `Playing` (+ `Paused`, `Victory` / `GameOver` for demos)  
- Short demos must meet **docs/GAME_DOD.md** (not movement-only)  
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
- Agent eyesight baseline (`/goal`): [docs/AGENT_EYESIGHT_PLAN.md](docs/AGENT_EYESIGHT_PLAN.md) — V0–V6 “eyes open”, **not** an editor  
- Agent eyesight **20/20 acuity** (`/goal`): [docs/AGENT_EYESIGHT_20_20_PLAN.md](docs/AGENT_EYESIGHT_20_20_PLAN.md) — fovea, multi-view, subjects, motion; **taste/design human-owned**  
- Agent sight **next** (`/goal`): [docs/AGENT_SIGHT_NEXT_PLAN.md](docs/AGENT_SIGHT_NEXT_PLAN.md) — MCP-first ranking/profiles; dogfood CD + IF; **no** video/editor/taste scorer  
- Agent sight **2D + 3D** (shipped): [docs/AGENT_SIGHT_2D3D_PLAN.md](docs/AGENT_SIGHT_2D3D_PLAN.md) — dim-specific sight; packs `hud`/`env_2d`; IF height terrain; findings [docs/AGENT_SIGHT_2D3D_FINDINGS_2026-07-22.md](docs/AGENT_SIGHT_2D3D_FINDINGS_2026-07-22.md)  
- Agent sight **debt** (shipped): [docs/AGENT_SIGHT_DEBT_PLAN.md](docs/AGENT_SIGHT_DEBT_PLAN.md) — residual acuity; CD+IF full asset/env improve; findings [docs/AGENT_SIGHT_DEBT_FINDINGS_2026-07-23.md](docs/AGENT_SIGHT_DEBT_FINDINGS_2026-07-23.md)  
- Agent sight **hardening** (shipped): [docs/AGENT_SIGHT_HARDENING_PLAN.md](docs/AGENT_SIGHT_HARDENING_PLAN.md) — Tier 1–3 gates; CD PulseMine; IF LoadingBay + heightfield; findings [docs/AGENT_SIGHT_HARDENING_FINDINGS_2026-07-23.md](docs/AGENT_SIGHT_HARDENING_FINDINGS_2026-07-23.md)  
- Agent sight **fidelity** (`/goal`): [docs/AGENT_SIGHT_FIDELITY_PLAN.md](docs/AGENT_SIGHT_FIDELITY_PLAN.md) — Tier 1–3; CD/IF dogfood (1 new feature each; improve all existing; complex silhouettes; transparent 2D BG; randomized IF height)  

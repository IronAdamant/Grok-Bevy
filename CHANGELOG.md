# Changelog

All notable changes to Grok-Bevy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Game factory roadmap (v0.3 / alpha):** agent path from short demos → package → later Steam
  - `docs/GAME_DOD.md` — definition of done (objective, challenge, win/lose)
  - `docs/ROADMAP.md` — phases G1–G6
  - Skill `bevy-demo-game`; production/2d/3d/agent-loop skills reference DoD
  - MCP prompts `build_demo_2d`, `build_demo_3d`, `package_demo`; workflow goals `complete_demo_2d|3d`, `package_demo`

- **Production games (v0.2):** skill pack, templates, multi-kind scaffold, ship docs, MCP routing
  - `.grok/skills/bevy-production` / `bevy-2d-game` / `bevy-3d-game` / `bevy-agent-loop`
  - `templates/game-2d` and `templates/game-3d` playable vertical slices (states, movement, disk assets)
  - `grok-bevy scaffold --kind 2d|3d|demo` copies templates (source of truth) + project `AGENTS.md`
  - `docs/ASSET_CONVENTIONS.md`, `docs/SHIPPING.md`, `docs/PRODUCTION_GAMES.md`
  - MCP `initialize.instructions` covers production skills and scaffold kinds
- MCP **prompts**: `start_2d_game`, `start_3d_game`, `iterate_scene`, `prepare_ship` (`prompts/list` + `prompts/get`)
- MCP tool **`bevy_workflow`**: goal → ordered skills + tools/CLI steps (`new_2d`, `new_3d`, `verify_scene`, `ship`, `add_sprite`)
- Root **`AGENTS.md`**; README production section; PROGRESS v0.2 checklist

### Changed

- Default scaffold kind is **`2d`** (production); `demo` is the BRP cube fixture (`templates/sample-app`)
- Positioning: `templates/sample-app` is BRP **integration fixture**, not a production game template

## [0.1.0] — 2026-07-17

### Added

- **`grok-bevy` CLI** with:
  - `doctor` / `env-check` — cross-platform Bevy readiness (Rust/Cargo, OS guidance, optional compile probe)
  - `mcp` — stdio MCP server for Grok Build agents
  - `mcp --delegate-brp-mcp` — exec into installed `bevy_brp_mcp`
  - `scaffold` — generate a BRP-enabled Bevy sample
  - `brp` — discover / query / mutate / screenshot / wait helpers
  - `compat` and `mcp-config` — version matrix and Grok registration snippets
- **`grok-bevy-env`** — injectable, unit-tested environment detection library
- **`grok-bevy-brp`** — BRP HTTP client, named targets, PNG capture → MCP image adapter
- **`templates/sample-app`** — Bevy 0.19 scene with `remote` / `capture` features and headless BRP smoke binary
- Docs: README fast-start, CONTRIBUTING, troubleshooting, multi-platform CI
- Dual MIT / Apache-2.0 licensing (matching Bevy)

### Compatibility

| Bevy | bevy_brp_mcp | bevy_brp_extras |
|------|--------------|-----------------|
| 0.19 | 0.22.1       | 0.22.1          |

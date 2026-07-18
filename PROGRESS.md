# Grok-Bevy progress checklist

## v0.1 — public-ready runtime control (done)

Track first public-ready (v0.1) delivery. Check items as they land.

### Research
- [x] Pin Bevy / `bevy_brp_mcp` / `bevy_brp_extras` versions (Bevy **0.19**, BRP stack **0.22.1**)
- [x] Confirm BRP methods, screenshot path, MCP integration model

### Scaffold
- [x] Workspace layout (`grok-bevy-env`, `grok-bevy-brp`, `grok-bevy`, `templates/sample-app`)
- [x] Dual MIT/Apache-2.0 licenses, gitignore, modular crates

### Environment readiness
- [x] Cross-platform detection (Windows / Linux / macOS)
- [x] Rust/Cargo readiness + OS-specific install guidance
- [x] Optional Bevy create+compile probe
- [x] Unit tests for detection + guidance

### MCP / BRP
- [x] BRP HTTP client (query / mutate / generic call)
- [x] MCP stdio server with agent tools
- [x] Integration path with `bevy_brp_mcp` (install/delegate)
- [x] Sample template with RemotePlugin / BrpExtras + feature flags

### Visual capture
- [x] Screenshot via `brp_extras/screenshot`
- [x] Image return adapter (PNG → MCP image content)
- [x] Fixture-based unit tests for image adapter
- [x] Live capture verified (2560×1440 PNG)

### Docs & OSS hygiene
- [x] README fast-start + Grok Build MCP snippets
- [x] CONTRIBUTING, CHANGELOG, multi-platform CI
- [x] Troubleshooting + compatibility matrix

### Verification
- [x] CLI env-check evidence
- [x] BRP control evidence (query + mutate)
- [x] Capture evidence (live PNG)
- [x] Template build (`remote,capture`)

---

## v0.2 — production games (done)

Skills define HOW; scaffold defines WHERE; MCP verifies WHAT.
See [docs/PRODUCTION_GAMES.md](docs/PRODUCTION_GAMES.md).

### Positioning
- [x] README production section + demo vs game distinction
- [x] `docs/PRODUCTION_GAMES.md` skill map and layout contract
- [x] PROGRESS v0.2 checklist

### Skill pack
- [x] Root `AGENTS.md` (pins, skill routing, anti-demo rules)
- [x] `.grok/skills/bevy-production`
- [x] `.grok/skills/bevy-2d-game`
- [x] `.grok/skills/bevy-3d-game`
- [x] `.grok/skills/bevy-agent-loop`

### Production templates
- [x] `templates/game-2d` playable vertical slice (states, movement, disk asset)
- [x] `templates/game-3d` playable vertical slice (states, movement, disk asset)
- [x] Keep `templates/sample-app` as BRP integration fixture only

### Scaffold
- [x] `scaffold --kind 2d|3d|demo` copies templates (source of truth)
- [x] Scaffold writes project `AGENTS.md` + asset dirs
- [x] Integration test: scaffold tree (+ cargo check in verification)

### Phase 4 production depth
- [x] `docs/ASSET_CONVENTIONS.md` (sprites/models/ui/audio)
- [x] `docs/SHIPPING.md` (`cargo build --release`, packaging notes)
- [x] Scaffolded project README/AGENTS surface ship + asset paths

### MCP skill routing
- [x] Richer `initialize.instructions` (skills + scaffold kinds)
- [x] MCP prompts: start_2d_game, start_3d_game, iterate_scene, prepare_ship
- [x] `bevy_workflow` tool (goal → steps + skills + tools)

---

## v0.3 — game factory (alpha): demos that are real games

See [docs/ROADMAP.md](docs/ROADMAP.md) and [docs/GAME_DOD.md](docs/GAME_DOD.md).  
Steam is **later** (G5); non-Steam package is G4.

### G1 — DoD + agent contract
- [x] `docs/GAME_DOD.md` + `docs/ROADMAP.md`
- [x] Skill `bevy-demo-game`
- [x] Production / 2d / 3d / agent-loop skills reference DoD
- [x] MCP prompts + `bevy_workflow` goals for complete demos / package

### G2 — Templates are short games
- [ ] `templates/game-2d` meets GAME_DOD (objective, win/lose, challenge, HUD)
- [ ] `templates/game-3d` meets GAME_DOD
- [ ] Structural tests + cargo check with `remote,capture`

### G3 — In-repo dogfood
- [ ] `games/demo-2d` workspace member
- [ ] `games/demo-3d` workspace member
- [ ] README default path: run dogfood demos

### G4 — Package (non-Steam)
- [ ] Package script/docs (binary + assets → dist/zip)
- [ ] Skill `bevy-package` + workflow `package_demo`
- [ ] CI check demos; optional release artifact

### G5 — Steam path (later)
- [ ] `docs/STEAM_PATH.md` + skill checklist
- [ ] Optional steam feature stub

### G6 — Install ergonomics
- [ ] Reliable templates after install / embedded templates

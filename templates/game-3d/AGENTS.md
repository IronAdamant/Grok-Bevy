# Project agent rules — 3D Bevy game

## Pins

- Bevy **0.19**, bevy_brp_extras **0.22.1**, BRP port **15702**
- Features: `remote`, `capture`

## Skills to load

1. `bevy-production` + `bevy-3d-game`
2. UI/art → `game-asset-core` (+ specialist)
3. Live verify → `bevy-agent-loop`

## Layout

Thin `main.rs`; logic in plugins/systems.  
Assets: `assets/models`, `assets/ui`, `assets/audio` (optional `sprites/`).

## States

`Loading` → `MainMenu` → `Playing` / `Paused`.

## Ship

`cargo build --release`. Ensure lighting keeps the player visible in captures.

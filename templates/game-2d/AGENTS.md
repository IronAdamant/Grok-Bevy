# Project agent rules — 2D Bevy game

## Pins

- Bevy **0.19**, bevy_brp_extras **0.22.1**, BRP port **15702**
- Features: `remote`, `capture`

## Skills to load

1. `bevy-production` + `bevy-2d-game`
2. Art → `game-asset-core` (+ specialist)
3. Live verify → `bevy-agent-loop`

## Layout

Keep `main.rs` thin; game logic in `lib` / `plugins` / `systems`.  
Assets: `assets/sprites`, `assets/ui`, `assets/audio`.

## States

`Loading` → `MainMenu` → `Playing` / `Paused`.

## Ship

`cargo build --release`. Do not ship without menu→play and movement working.

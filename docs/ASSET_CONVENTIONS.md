# Asset path conventions

Bevy loads files **relative to the project `assets/` directory**. Grok-Bevy production templates and skills use these roots:

| Directory | Purpose | Example `AssetServer` path |
|-----------|---------|----------------------------|
| `assets/sprites/` | 2D characters, props, tiles, VFX sheets | `"sprites/player.png"` |
| `assets/models/` | 3D meshes, glTF, material textures | `"models/player.glb"`, `"models/ground_tint.png"` |
| `assets/ui/` | HUD chrome, buttons, panels, icons | `"ui/panel.png"` |
| `assets/audio/` | SFX and music | `"audio/jump.ogg"` |

## Rules for agents

1. **Never invent absolute paths** for runtime loads — always paths under `assets/`.
2. **Put art from `game-asset-*` skills** into the matching root; keep filenames stable when re-exporting.
3. **Prefer engine-ready assets**: keyable backgrounds for sprites (see `game-asset-core`); no baked UI text when the game localizes.
4. **Name entities** after important visual assets (`Name::new("Player")`) so BRP queries match captures.
5. **Loading state**: wait until handles are loaded (`AssetServer::is_loaded`) before transitioning MainMenu → Playing when art is required.

## Scaffolded layout

`grok-bevy scaffold --kind 2d|3d` creates these directories (and ships at least one placeholder disk asset for the vertical slice).

## Related

- [SHIPPING.md](SHIPPING.md) — release builds and packaging
- [PRODUCTION_GAMES.md](PRODUCTION_GAMES.md) — overall production loop

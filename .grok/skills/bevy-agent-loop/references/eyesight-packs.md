# Eyesight capture packs (20/20 + 2D/3D)

Not an editor. Not taste. Schema: `grok-bevy.eyesight/v1` with `acuity: "20/20-candidate"`.

## Hard rules

1. Open every PNG `abs_path` in the packet.  
2. Observation claims cite paths. Design taste is human-owned.  
3. Env claims need **Playing** (Iron Feud: `IRON_FEUD_AUTO_PLAY=1` + `require_playing`).  
4. Prefer **`bevy_see_verify` / `grok-bevy see verify --profile …` first**, then packs.  
5. Rebuild MCP before dogfood pass evidence:  
   `cargo install --path crates/grok-bevy --force`  
6. Stills + short strips only — **no** livestream / 60 FPS video.  
7. New Named gameplay entities need GAMEPLAY_NAME_HINTS / DOGFOOD_NAME_STEMS score >0 so they survive `gameplay_prefer` (unit-tested onboarding table).  
8. After asset/env change: `save_baseline` then `compare_baseline` once.  
9. Landscape alt uses **side XZ nudge** for top-down 3D; if hashes match, heed `views_similar`.

## Tools / CLI

| Tool | CLI |
|------|-----|
| `bevy_see_scene` | `grok-bevy see scene` |
| `bevy_see_verify` | `grok-bevy see verify --profile crystal-drift\|iron-feud` |
| `bevy_see_entity` | `grok-bevy see entity --name …` |
| `bevy_see_region` | `grok-bevy see region --x --y --w --h` |
| `bevy_see_motion` | `grok-bevy see motion --frames 8` |
| `bevy_see_diff` | `grok-bevy see diff --baseline …` |
| `bevy_see_pack` | `grok-bevy see pack <pack>` |

## Packs

| Pack | Dimension | Views / notes |
|------|-----------|----------------|
| `entity_craft` | shared | scene + fovea on ranked primary |
| `landscape` | shared / 3D height | game + surface/horizon crop + alt (camera nudge); **height_bands** warning when TerrainFlat/Hill/Peak present |
| `water` | 3D | game + center crop + alt |
| `hud` | **2D** | full + top-left + top-bar region presets |
| `env_2d` | **2D** | full + horizon band + center (station/debris/craft composition) |
| `physics_jump` | shared | motion strip |
| `lighting` | shared | lit still (unlit not automatic) |
| `diagnostic` | shared | full + bounds on ranked/allowlist primary (not Player-only) |

## Region presets (pure geometry for `see_region`)

| Preset | Use |
|--------|-----|
| `hud_top_left` / `hud` | Score/fuel chrome |
| `hud_top_bar` | Full top HUD strip |
| `horizon_band` | Sky / upper third |
| `ground_band` | Lower surface third |
| `center_half` | Center craft / station crop |

## Profiles (preferred)

| Profile | Sets |
|---------|------|
| `crystal-drift` / `2d` / `ortho2d` | ortho2d, half 640×360, wait Player |
| `iron-feud` / `3d` / `topdown3d` | topdown3d, half 20×20, **require_playing**, wait StrategyCamera/WaterBody/Ground |

`primary_subject` is **ranked** (Player / StrategyCamera / WaterBody / Ground over Crystal / OreCrystal).  
Child mesh parts (WatchPostLegs, OreSiloBody, …) are demoted so they do not crowd the filter.

## Projection

- `ortho2d` — Crystal Drift (world XY)  
- `topdown3d` — Iron Feud StrategyCamera (world XZ)  
- `--visible-half-w/h` scale world→screen (profiles set defaults)

## Height terrain dogfood (Iron Feud)

- World must expose **three distinct height bands** (flat → hill → peak), measurable in code and visible in landscape packs.  
- Recommended Names: `TerrainFlat`, `TerrainHill_*`, `TerrainPeak_*` (or `HeightTerrain`).  
- Keep factory start cells flat / playable; place hills/peaks outside the start pocket.  
- Landscape pack should show relief; packet may warn `height_bands present…`.

## Examples

```bash
# Crystal Drift (2D)
grok-bevy see verify --profile crystal-drift --out-dir . --save-baseline captures/eyesight/baseline_scene.png
grok-bevy see pack env_2d --profile crystal-drift --out-dir .
grok-bevy see pack hud --profile crystal-drift --out-dir .
grok-bevy see pack landscape --profile crystal-drift --out-dir .

# Iron Feud Playing (3D)
IRON_FEUD_AUTO_PLAY=1 cargo run --features remote,capture
grok-bevy see verify --profile iron-feud --out-dir .
grok-bevy see pack landscape --profile iron-feud --out-dir .
grok-bevy see pack water --profile iron-feud --out-dir .
```

**MCP first:** rebuild/reload grok-bevy after sight code changes before dogfood captures.

## Dual instances

One app per BRP port (default 15702). Second game: different port + `bevy_register_target`. Sequential dogfood only on a single port.

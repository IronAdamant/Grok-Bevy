# Eyesight capture packs (20/20 acuity)

Not an editor. Not taste. Schema: `grok-bevy.eyesight/v1` with `acuity: "20/20-candidate"`.

## Hard rules

1. Open every PNG `abs_path` in the packet.  
2. Observation claims cite paths. Design taste is human-owned.  
3. Env claims need **Playing** (Iron Feud: `IRON_FEUD_AUTO_PLAY=1` + `require_playing`).  

## Tools / CLI

| Tool | CLI |
|------|-----|
| `bevy_see_scene` | `grok-bevy see scene` |
| `bevy_see_entity` | `grok-bevy see entity --name …` |
| `bevy_see_region` | `grok-bevy see region --x --y --w --h` |
| `bevy_see_motion` | `grok-bevy see motion --frames 8` |
| `bevy_see_diff` | `grok-bevy see diff --baseline …` |
| `bevy_see_pack` | `grok-bevy see pack landscape\|water\|…` |

## Packs

| Pack | Views / notes |
|------|----------------|
| `entity_craft` | scene + fovea |
| `landscape` | game + alt (camera nudge) |
| `water` | game + alt |
| `physics_jump` | motion strip |
| `lighting` | lit still |
| `diagnostic` | full + bounds outline on fovea |

## Projection

- `ortho2d` — Crystal Drift (world XY)  
- `topdown3d` — Iron Feud StrategyCamera (world XZ)  
- `--visible-half-w/h` scale world→screen  

## Examples

```bash
# Crystal Drift
grok-bevy see scene --out-dir . --wait-for Player --save-baseline captures/eyesight/baseline.png
grok-bevy see entity --name Player --half 120
grok-bevy see scene --compare-baseline captures/eyesight/baseline.png

# Iron Feud Playing
IRON_FEUD_AUTO_PLAY=1 cargo run --features remote,capture
grok-bevy see scene --require-playing --wait-for WaterBody --wait-for Ground \
  --projection topdown3d --visible-half-w 20 --visible-half-h 20
grok-bevy see entity --name WaterBody --projection topdown3d --visible-half-w 20 --visible-half-h 20
grok-bevy see pack water --projection topdown3d
```

## Dual instances

One app per BRP port (default 15702). Second game: different port + `bevy_register_target`.

# Eyesight capture packs (agent senses)

Not an editor. Use with `bevy_see_*` / `grok-bevy see *`. Schema: `grok-bevy.eyesight/v1`.

## Hard rules

1. Open every PNG `abs_path` in the packet.  
2. Aesthetic / physics-feel claims must cite paths.  
3. Prefer packs over one-off screenshots when refining.

## Packs (`bevy_see_pack`)

| Pack | Use when |
|------|----------|
| `entity_craft` | Sprite/mesh looks blocky or wrong scale |
| `landscape` | Terrain / ground / horizon composition |
| `water` | Water body color, edge, motion |
| `physics_jump` | Jump/slide/fall feel (temporal strip) |
| `lighting` | Muddy lighting (unlit needs game-side help) |

## CLI

```bash
grok-bevy see scene --out-dir .
grok-bevy see entity --name Player --half 120 --out-dir .
grok-bevy see region --x 0 --y 0 --w 400 --h 300 --label hud --out-dir .
grok-bevy see motion --frames 6 --interval-ms 80 --out-dir .
grok-bevy see diff --baseline captures/eyesight/scene_full.png --out-dir .
grok-bevy see pack landscape --out-dir .
```

## Style + art loop (V5)

1. `bevy_see_entity` / pack with `style_intent`  
2. Judge (blocky, muddy, off-scale)  
3. `game-asset-core` regen  
4. `bevy_see_diff` baseline vs after  

## Physics (V3)

Use `bevy_see_motion` after stimulus (`keys` or mutate). Judge contact, arc, jitter from the strip — not only Transform numbers.

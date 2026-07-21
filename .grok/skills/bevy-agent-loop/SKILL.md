---
name: bevy-agent-loop
description: >
  Live Bevy iteration with Grok-Bevy MCP and BRP: doctor, launch, query, mutate,
  viewport capture, and fix loops. Use when debugging a running Bevy app, taking
  screenshots, using BRP, MCP tools, bevy_capture_viewport, or /bevy-agent-loop.
metadata:
  short-description: "MCP/BRP live verify loop for Bevy"
---

# Bevy agent loop (MCP + BRP)

You control a **running** Bevy app. Prefer evidence from **agent eyesight** (pixels you open and judge) over guessing.  
This is **not a Bevy editor** — eyesight is a sensory channel for the agent brain.

Plan: `docs/AGENT_EYESIGHT_PLAN.md` (schema `grok-bevy.eyesight/v1`).

## Pins

- BRP default port **15702**  
- App must enable features **`remote,capture`** and add `BrpExtrasPlugin` when `remote` is on  
- Prefer full **`bevy_brp_mcp`** when installed for hierarchy/watches/input  

## Eyesight rules (hard — V0 discipline)

1. **Pixels are primary.** Query without opening captures is not eyesight.  
2. **Open every PNG** returned (`abs_path` if chat truncates image bytes).  
3. **Aesthetic / physics-feel claims must cite capture paths** (and preferably what you saw).  
4. **Refine with compare:** baseline → change → `bevy_see_diff` or second `bevy_see_scene`.  
5. **Subject classes:** entity, landscape, water, fx, lighting, ui, physics_motion — pick a pack when needed.  
6. **Style intent:** when refining art, pass `style_intent` and chain to `game-asset-core` if the look is wrong.  
7. **Black-frame warning** in packets: check minimized window, lights, camera, Loading state — do not invent beauty.

## Standard loop (hard requirements)

1. **`bevy_env_check`** (or CLI `grok-bevy doctor`) if the host is unknown.  
2. **Launch — never block MCP on cold Bevy compile**  
   - **Cold first compile (no `target/debug/<package>`):** shell/background  
     `cargo run --features remote,capture` (or `cargo build` then run).  
   - **Warm binary exists:** MCP `bevy_launch_app` with `manifest_path` +  
     `features: "remote,capture"` and **`wait_secs: 0`** (default; returns immediately; may spawn the debug binary).  
   - **Always** next: step 3. Do **not** set high `wait_secs` on launch (host tool timeouts ~120s).  
3. **Wait** with MCP **`bevy_wait_brp`** (`timeout_secs` **180** cold / **30** warm) or CLI `grok-bevy brp wait --timeout-secs 180`.  
4. **`bevy_see_scene`** (or CLI `grok-bevy see scene --out-dir .`) — open packet captures + subjects.  
5. **Fovea / motion as needed:** `bevy_see_entity`, `bevy_see_region`, `bevy_see_motion`, `bevy_see_pack`.  
6. **Mutate** if needed (`bevy_brp_mutate`) then **re-see** (`bevy_see_diff` with baseline).  
7. **Fix code or assets** → rebuild/restart if needed → eyesight again.  
8. Stop when the glance test would pass for a non-author human.

### Dual launch / one BRP port

Default BRP port is **15702**. Only **one** app should bind that port. Dual `cargo run` is fine for “start A, stop A, start B” smoke tests — not two simultaneous instances on the same port. For two live apps, register distinct ports/targets (`bevy_register_target`) and change `BrpExtrasPlugin` / env port on the second.

## Tool map (grok-bevy MCP)

| Tool | Use |
|------|-----|
| `bevy_env_check` | Host ready? |
| `bevy_launch_app` | Spawn game (`wait_secs=0` default; non-blocking) |
| `bevy_wait_brp` | Poll until BRP ready (use after launch) |
| `bevy_register_target` / `bevy_list_targets` | Multi-instance |
| `bevy_brp_discover` | Method list (`rpc.discover`) |
| `bevy_brp_query` | Read components |
| `bevy_brp_mutate` | Write field |
| `bevy_brp_call` | Arbitrary BRP / `brp_extras/*` |
| `bevy_capture_viewport` | Raw PNG as MCP image (`brp_extras/screenshot`) |
| `bevy_see_scene` | E0 eyesight packet (full + subjects + abs_path) |
| `bevy_see_entity` | E1 fovea crop for named entity |
| `bevy_see_region` | E1 pixel-rect crop (landscape / water / HUD) |
| `bevy_see_motion` | E2 temporal strip (+ optional keys) |
| `bevy_see_diff` | E3 baseline vs after (+ abs-diff) |
| `bevy_see_pack` | Multi-view: entity_craft / landscape / water / physics_jump / lighting |
| `bevy_brp_mcp_status` | Upstream MCP install help |

CLI mirrors: `grok-bevy see scene|entity|region|motion|diff|pack`.

## Exact BRP method names (do not invent)

| Method | Use |
|--------|-----|
| `rpc.discover` | List methods |
| `world.query` | Query |
| `world.mutate_components` | Mutate |
| `brp_extras/screenshot` | Screenshot path (**not** `bevy_brp_extras/…`) |

## Capture quality rules

- Window must be **visible** (not minimized); black frames are often occlusion, not “empty scene”.  
- Ensure lights + non-black clear color in 3D.  
- After art changes, confirm texture path and that the handle finished loading (Loading state).  
- Compare before/after captures when iterating.

## Demo DoD captures

When finishing a short demo (`docs/GAME_DOD.md` / skill `bevy-demo-game`), capture at least:

1. **Main menu**  
2. **Mid-play** with objective/HUD visible  
3. **Victory or GameOver** end screen  

Movement-only scenes are not complete demos.
## BRP hygiene

- **Aliases** (preferred): `Name`, `Transform`, `GlobalTransform` — MCP expands to Bevy 0.19 FQNs.  
- Or pass FQNs with `::` (e.g. `bevy_ecs::name::Name`). Default query is Name + Transform.  
- Prefer entities with **`Name`** for human-readable queries.  
- Mutate small fields first (e.g. `component: "Transform"`, `path: "translation"`) to prove the pipe.  
- Custom components need Reflect + BRP registration to be mutable remotely.

### Optional: movement via keys (after BRP ready)

1. Query Player Transform (alias `Transform`).  
2. `bevy_brp_discover` / check `brp_extras/send_keys` params.  
3. `bevy_brp_call` method `brp_extras/send_keys` (discover-first params).  
4. Re-query Transform.  
5. If send_keys fails: `bevy_brp_mutate` translation instead (portfolio closed-loop still works).

## When to use bevy_brp_mcp

Install/register when you need:

- Entity hierarchy browsing  
- Watches / richer world tools  
- Keyboard/mouse injection  
- Advanced launch discovery  

Same port; complementary to grok-bevy’s focused tools.

## Failure playbook

| Symptom | Check |
|---------|--------|
| Connection refused | App not running / missing `remote` / wrong port; call `bevy_wait_brp` |
| Launch MCP timeout | `wait_secs=0` + `bevy_wait_brp`; cold = shell `cargo run`; warm = debug binary |
| Black screenshot | Minimized window; no lights; camera wrong way |
| Empty query / still Loading | Wrong type path; asset root (see ASSET_CONVENTIONS); wait for fail-forward timeout |
| Mutate no-op | Path wrong; type not Reflect; wrong entity id |
| B0001 panic | Overlapping mut queries — ParamSet / Without / split systems |
| Compile forever | First Bevy build; shell cargo run; doctor guidance; dev opt profiles |

## Chain with other skills

- Structure: **`bevy-production`**  
- Slice: **`bevy-2d-game`** / **`bevy-3d-game`**  
- Art defects in capture: regenerate with **`game-asset-core`** (keyable bg, scale)

## References

- `references/loop-checklist.md`  

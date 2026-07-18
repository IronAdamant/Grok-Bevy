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

You control a **running** Bevy app. Prefer evidence from **viewport capture** over guessing.

## Pins

- BRP default port **15702**  
- App must enable features **`remote,capture`** and add `BrpExtrasPlugin` when `remote` is on  
- Prefer full **`bevy_brp_mcp`** when installed for hierarchy/watches/input  

## Standard loop

1. **`bevy_env_check`** (or CLI `grok-bevy doctor`) if the host is unknown.  
2. **Launch**  
   - MCP: `bevy_launch_app` with `manifest_path` + `features: "remote,capture"`  
   - Or user terminal: `cargo run --features remote,capture`  
3. **Wait** until BRP answers (`bevy_brp_discover` or CLI `brp wait`).  
4. **Query** named entities / transforms (`bevy_brp_query`).  
5. **Mutate** if needed (`bevy_brp_mutate` — fully-qualified Reflect type paths).  
6. **`bevy_capture_viewport`** — **look at the image**; describe defects honestly.  
7. **Fix code or assets** → rebuild/restart if needed → capture again.  
8. Stop when the capture matches the acceptance criteria.

## Tool map (grok-bevy MCP)

| Tool | Use |
|------|-----|
| `bevy_env_check` | Host ready? |
| `bevy_launch_app` | Start game binary |
| `bevy_register_target` / `bevy_list_targets` | Multi-instance |
| `bevy_brp_discover` | Method list |
| `bevy_brp_query` | Read components |
| `bevy_brp_mutate` | Write field |
| `bevy_brp_call` | Arbitrary BRP / `brp_extras/*` |
| `bevy_capture_viewport` | PNG as MCP image |
| `bevy_brp_mcp_status` | Upstream MCP install help |

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

- Use **fully-qualified** component type paths from `rpc.discover` / list components when unsure.  
- Prefer entities with **`Name`** for human-readable queries.  
- Mutate small fields first (e.g. translation) to prove the pipe before complex edits.  
- Custom components need Reflect + BRP registration to be mutable remotely.

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
| Connection refused | App not running / missing `remote` / wrong port |
| Black screenshot | Minimized window; no lights; camera wrong way |
| Empty query | Wrong type path; entity not spawned yet (still Loading) |
| Mutate no-op | Path wrong; type not Reflect; wrong entity id |
| Compile forever | First Bevy build; use doctor guidance; dev opt profiles |

## Chain with other skills

- Structure: **`bevy-production`**  
- Slice: **`bevy-2d-game`** / **`bevy-3d-game`**  
- Art defects in capture: regenerate with **`game-asset-core`** (keyable bg, scale)

## References

- `references/loop-checklist.md`  

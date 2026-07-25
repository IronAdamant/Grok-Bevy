# Agent Sight Soft Edges — findings (2026-07-25)

## Status

**E0–E7 complete.** Soft edges S1–S5 investigated; platform fixes landed where BRP allows; CI matrix documented; full `agent-sight-dogfood` mode=full is the closeout bar.

## S1 Hierarchy

| | |
|--|--|
| **Probe** | Live IF: `ChildOf`-only BRP query returns ~103 entities with bare parent `u64`. `Name+Transform+ChildOf` multi-component query returns **0** (AND filter). |
| **Decision** | **A-partial** — merge parents via second query. |
| **Fix** | `parent_map_from_childof_query` + `merge_childof_parents` in `query_all_subjects`. |
| **Evidence** | `{SCRATCH}/hierarchy-brp.json`; unit `parent_map_from_childof_bare_u64`. |

## S2 Multi-view

| | |
|--|--|
| **Probe** | `BrpClient::spawn_entity` exists; switching **active** render camera via BRP is unreliable for viewport capture. |
| **Decision** | Keep StrategyCamera nudge chain (alt → side-orbit → dedicated-side). |
| **Fix** | `find_dedicated_view_camera` for optional `AgentSightSideCamera` / `SideCamera` Names if games place one later. |
| **Evidence** | Unit `find_dedicated_view_camera_prefers_named`; pack still restores after nudge. |

## S3 Fovea craft readability

| | |
|--|--|
| **Probe** | Aim can be correct (OreCrusher ~1949,801 topdown3d) while crop is deep shadow. |
| **Fix** | After fovea crop, `path_nonblack_fraction` vs `FOVEA_CROP_NONBLACK_MIN` (0.05) → warning `fovea_dark: … do not claim craft identity from this crop alone`. |
| **Evidence** | Unit `fovea_crop_nonblack_min_is_strict`; live via workflow entity step. |

## S4 CD env_2d

| | |
|--|--|
| **Probe** | Prior workflows: black_frame / HUD-only env_2d fulls (timing). |
| **Fix** | Wait for Nebula/WarpGate/Station/Player (up to 4s); if full nonblack &lt; `FULL_FRAME_NONBLACK_MIN`, sleep 400ms and **recapture once**; warn `env_2d_dark` if still dark. |
| **Evidence** | Live workflow env_2d packs after fix. |

## S5 CI / non-silent

| | |
|--|--|
| **Doc** | [AGENT_SIGHT_CI_MATRIX.md](AGENT_SIGHT_CI_MATRIX.md) |
| **Policy** | Headless: always `cargo test -p grok-bevy -p grok-bevy-brp`. Live bar: **`/agent-sight-dogfood` mode=full**. No fake headless screenshot CI. |

## E6 Full workflow

**agent-sight-dogfood-4** — overall **`passed: true`**, skeptic **green** (~6m).

| Phase | Result |
|-------|--------|
| Gates | PASS · 56+ brp tests · mcp_rebuilt=false |
| CD | Player · CargoPod · magenta 0 · env_2d_dark warned honestly when space-black |
| IF | StrategyCamera · OreCrusher @ (1949,801) topdown3d · packs [mutated][restored] · multi-view differs |
| Skeptic | green · failures [] |

Evidence: `{SCRATCH}/workflow-soft-full-report.md`, `workflow-soft-full.log`, `gbr-tests.log`, `mcp-surface.log`, `hierarchy-brp.json`, `soft-baseline.md`.

## Residual

- Games without BRP ChildOf still use string/co-located demotion (merge path no-ops).
- True multi-view still needs game-side secondary active camera for free parallax without strategy-cam nudge.
- Dark fovea under shadow — **warned** (`fovea_dark`), not claimed as craft-ok.
- CD env_2d may still warn env_2d_dark on pure-black space (gate vs soft nebulas); composition not overclaimed.

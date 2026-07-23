# Roadmap — agent-native game factory (alpha)

**Status:** alpha public demo.  
**North star:** Grok-Bevy helps agents **create real short Bevy games**, package them for sharing, and later structure them for Steam—without Steam blocking the first demos.

Design, art direction, and design iteration are **user-owned**. Grok-Bevy owns **structure, systems kit, agent skills, MCP verify, packaging**.

## Horizons

| Horizon | Meaning |
|---------|---------|
| **D — Demo** | 2D + 3D short games meeting [GAME_DOD.md](GAME_DOD.md) |
| **P — Package** | Zip/folder: binary + `assets/`, non-Steam |
| **S — Steam path** | Checklists + optional stubs; upload is a later process |

## Decisions

- Dogfood **in-repo** (`games/demo-2d`, `games/demo-3d`) **and** scaffold for external projects — **same layout contract**  
- **Both 2D and 3D**  
- **Non-Steam first**, Steam-ready structure later  

## Phases

| Phase | Name | Outcome |
|-------|------|---------|
| **G1** | DoD + skills | Agents know when a demo is “done” |
| **G2** | Templates → short games | **Done** — kits meet GAME_DOD |
| **G3** | In-repo dogfood | **Done** — `games/demo-2d`, `games/demo-3d` |
| **G4** | Packaging | **Done** — `scripts/package-demo.sh` + docs/skill |
| **G5** | Steam path | Docs/skill only until packaging works |
| **G6** | Install ergonomics | **Done** — templates embedded in CLI; `GROK_BEVY_TEMPLATE_ROOT` optional override |

```text
G1 ──► G2a (2D kit) ──► G3a (demo-2d)
    └► G2b (3D kit) ──► G3b (demo-3d)
G2 ──► G4 package ──► G5 steam (thin)
G6 parallel after G2
```

## Agentic loop

Skills (HOW) → kit/templates (WHERE) → dogfood or scaffold (BUILD) → **agent eyesight** (see / judge / re-see) → package (SHIP DEMO) → later Steam checklist.

MCP capture is the optic nerve; **agent eyesight** (entity, landscape, water, physics-as-motion) baseline is [AGENT_EYESIGHT_PLAN.md](AGENT_EYESIGHT_PLAN.md) (V0–V6 shipped). **Acuity 20/20** (true fovea, multi-view, clean subjects, temporal reliability) is [AGENT_EYESIGHT_20_20_PLAN.md](AGENT_EYESIGHT_20_20_PLAN.md) — still agent senses, **not** a Bevy editor; taste/design stay human-owned. Execute with `/goal` against those docs.

## Tracking

Checklist: [PROGRESS.md](../PROGRESS.md) (v0.3+).  
Eyesight baseline: [AGENT_EYESIGHT_PLAN.md](AGENT_EYESIGHT_PLAN.md) (V0–V6).  
Eyesight 20/20: [AGENT_EYESIGHT_20_20_PLAN.md](AGENT_EYESIGHT_20_20_PLAN.md) (A0–A8).  
Agent sight next (profiles, ranking, dogfood): [AGENT_SIGHT_NEXT_PLAN.md](AGENT_SIGHT_NEXT_PLAN.md) (S0–S4 shipped).  
Agent sight **2D + 3D** (profiles/packs, CD+IF height terrain): [AGENT_SIGHT_2D3D_PLAN.md](AGENT_SIGHT_2D3D_PLAN.md) (D0–D5 shipped; findings [AGENT_SIGHT_2D3D_FINDINGS_2026-07-22.md](AGENT_SIGHT_2D3D_FINDINGS_2026-07-22.md)).  
Agent sight **debt** (next `/goal`): [AGENT_SIGHT_DEBT_PLAN.md](AGENT_SIGHT_DEBT_PLAN.md) — residual filter/multi-view/fovea; full CD+IF asset+env improve pass.  
Session plan detail: maintained in agent sessions; this file is the **durable product roadmap**.

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
| **G6** | Install ergonomics | Templates/binary easy outside monorepo |

```text
G1 ──► G2a (2D kit) ──► G3a (demo-2d)
    └► G2b (3D kit) ──► G3b (demo-3d)
G2 ──► G4 package ──► G5 steam (thin)
G6 parallel after G2
```

## Agentic loop

Skills (HOW) → kit/templates (WHERE) → dogfood or scaffold (BUILD) → MCP capture (VERIFY) → package (SHIP DEMO) → later Steam checklist.

## Tracking

Checklist: [PROGRESS.md](../PROGRESS.md) (v0.3+).  
Session plan detail: maintained in agent sessions; this file is the **durable product roadmap**.

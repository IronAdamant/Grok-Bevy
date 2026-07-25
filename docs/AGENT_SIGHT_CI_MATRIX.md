# Agent sight — CI / regression matrix (S5)

## Always (headless, no GPU window required)

```bash
cargo test -p grok-bevy -p grok-bevy-brp
```

Covers: ranking, structural demotion, fovea projection, pack restore helpers, multi-view placement pure math, magenta/nonblack gates.

Wire this in CI for every PR.

## Live bar (GPU / window required) — release & soft-edge closeout

```text
/agent-sight-dogfood
# args must include mode=full (not gates-only)
```

| Check | Expected |
|-------|----------|
| Gates | cargo test PASS |
| Crystal Drift | primary Player; dogfood Name present; no magenta plates |
| Iron Feud | Playing; primary ≠ OreCrystal*; OreCrusher in subjects; entity `--profile iron-feud` topdown aim |
| Multi-view | game≠alt or honest `views_similar` |
| Pack restore | notes `[restored]` / camera back to game pose |
| Skeptic | green preferred |

## Not claimed

- Headless fake “screenshot CI” without a real Bevy window  
- Parallel dual games on BRP 15702  

## Policy

Unit tests prevent silent ranking/projection rot. Full workflow is the **required** live sight bar for soft-edge plans and recommended for releases after eyesight changes.

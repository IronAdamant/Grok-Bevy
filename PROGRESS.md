# Grok-Bevy progress checklist

Track first public-ready (v0.1) delivery. Check items as they land.

## Research
- [x] Pin Bevy / `bevy_brp_mcp` / `bevy_brp_extras` versions (Bevy **0.19**, BRP stack **0.22.1**)
- [x] Confirm BRP methods, screenshot path, MCP integration model

## Scaffold
- [x] Workspace layout (`grok-bevy-env`, `grok-bevy-brp`, `grok-bevy`, `templates/sample-app`)
- [x] Dual MIT/Apache-2.0 licenses, gitignore, modular crates

## Environment readiness
- [x] Cross-platform detection (Windows / Linux / macOS)
- [x] Rust/Cargo readiness + OS-specific install guidance
- [x] Optional Bevy create+compile probe
- [x] Unit tests for detection + guidance

## MCP / BRP
- [x] BRP HTTP client (query / mutate / generic call)
- [x] MCP stdio server with agent tools
- [x] Integration path with `bevy_brp_mcp` (install/delegate)
- [x] Sample template with RemotePlugin / BrpExtras + feature flags

## Visual capture
- [x] Screenshot via `brp_extras/screenshot`
- [x] Image return adapter (PNG → MCP image content)
- [x] Fixture-based unit tests for image adapter
- [x] Live capture verified (2560×1440 PNG)

## Docs & OSS hygiene
- [x] README fast-start + Grok Build MCP snippets
- [x] CONTRIBUTING, CHANGELOG, multi-platform CI
- [x] Troubleshooting + compatibility matrix

## Verification
- [x] CLI env-check evidence
- [x] BRP control evidence (query + mutate)
- [x] Capture evidence (live PNG)
- [x] Template build (`remote,capture`)

# Changelog

All notable changes to Grok-Bevy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-07-17

### Added

- **`grok-bevy` CLI** with:
  - `doctor` / `env-check` — cross-platform Bevy readiness (Rust/Cargo, OS guidance, optional compile probe)
  - `mcp` — stdio MCP server for Grok Build agents
  - `mcp --delegate-brp-mcp` — exec into installed `bevy_brp_mcp`
  - `scaffold` — generate a BRP-enabled Bevy sample
  - `brp` — discover / query / mutate / screenshot / wait helpers
  - `compat` and `mcp-config` — version matrix and Grok registration snippets
- **`grok-bevy-env`** — injectable, unit-tested environment detection library
- **`grok-bevy-brp`** — BRP HTTP client, named targets, PNG capture → MCP image adapter
- **`templates/sample-app`** — Bevy 0.19 scene with `remote` / `capture` features and headless BRP smoke binary
- Docs: README fast-start, CONTRIBUTING, troubleshooting, multi-platform CI
- Dual MIT / Apache-2.0 licensing (matching Bevy)

### Compatibility

| Bevy | bevy_brp_mcp | bevy_brp_extras |
|------|--------------|-----------------|
| 0.19 | 0.22.1       | 0.22.1          |

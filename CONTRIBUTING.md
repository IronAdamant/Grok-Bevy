# Contributing to Grok-Bevy

Thanks for helping agents drive Bevy at runtime.

## Principles

1. **Prefer composition over reinvention.** Extend or integrate [`bevy_brp_mcp`](https://github.com/natepiano/bevy_brp) / [`bevy_brp_extras`](https://crates.io/crates/bevy_brp_extras) rather than duplicating BRP or MCP surfaces.
2. **Keep v0 focused.** High-quality readiness checks, a reliable MCP path, capture, templates, and docs beat feature sprawl.
3. **Test the shipped path.** Unit tests should call real modules (inject I/O at boundaries). Avoid mocks that reimplement production logic.

## Workspace layout

| Path | Role |
|------|------|
| `crates/grok-bevy-env` | OS / Rust / Bevy readiness detection |
| `crates/grok-bevy-brp` | BRP HTTP client + image capture helpers |
| `crates/grok-bevy` | CLI + MCP server |
| `templates/sample-app` | Feature-gated RemotePlugin / capture sample |

## Development setup

```bash
# Toolchain
rustup default stable

# Library + CLI tests (fast)
cargo test -p grok-bevy-env -p grok-bevy-brp -p grok-bevy

# Doctor
cargo run -p grok-bevy -- doctor

# Sample with BRP (needs GPU/window for full scene + screenshots)
cargo run -p grok_bevy_sample --features remote,capture
```

## Extending

- **New OS detection signals** → `grok-bevy-env` (`CommandRunner` stays injectable).
- **New BRP helpers** → `grok-bevy-brp` client methods; wire MCP tools in `crates/grok-bevy/src/mcp.rs`.
- **Richer agent tools** → prefer upstream `bevy_brp_mcp` PRs; wrap here only for Grok-specific UX (env, image return shaping, docs).

## Pull requests

1. Run `cargo test -p grok-bevy-env -p grok-bevy-brp -p grok-bevy` and `cargo build -p grok-bevy`.
2. Update `CHANGELOG.md` under `[Unreleased]` or the next version section.
3. Keep docs (README compatibility matrix) in sync when bumping Bevy / BRP crate versions.
4. Dual-license all new source as MIT OR Apache-2.0.

## Code style

- Rust 2021, `rustfmt` defaults.
- Prefer small public APIs with `serde` types for CLI/MCP boundaries.
- Logging on MCP servers must use **stderr** only (stdout is the protocol).

## License

By contributing, you agree that your contributions are dual-licensed under MIT OR Apache-2.0, the same as Bevy.

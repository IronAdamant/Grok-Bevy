# Grok-Bevy

**Runtime control of the [Bevy](https://bevy.org) game engine for [Grok Build](https://x.ai) and other MCP-compatible AI coding agents.**

Grok-Bevy prepares a machine for Bevy development, scaffolds BRP-enabled apps, and exposes an MCP server so agents can **launch, query, mutate, and screenshot** live Bevy scenes. It builds on the mature [`bevy_brp_mcp`](https://github.com/natepiano/bevy_brp) / [`bevy_brp_extras`](https://crates.io/crates/bevy_brp_extras) stack instead of reimplementing the Bevy Remote Protocol.

| Layer | Role |
|-------|------|
| `grok-bevy` CLI | Doctor, scaffold, BRP helpers, MCP entrypoint |
| `grok-bevy` MCP | Focused agent tools (env, query/mutate, capture-as-image) |
| `bevy_brp_mcp` | Full BRP tool surface (optional, recommended) |
| `templates/sample-app` | Feature-gated RemotePlugin + capture sample |

## Compatibility

| Bevy | bevy_brp_mcp | bevy_brp_extras | Grok-Bevy |
|------|--------------|-----------------|-----------|
| **0.19** | **0.22.1** | **0.22.1** | 0.1.x |

Bevy is a **Cargo dependency**, not a global binary. Readiness means: can this host compile and run Bevy apps?

## Fast start (a few minutes)

### 1. Check your environment

```bash
cargo install --path crates/grok-bevy
# or from a clone:
cargo run -p grok-bevy -- doctor
```

Example healthy output includes your **OS family**, `rustc` / `cargo` versions, and `READY`. If something is missing, the report prints **OS-specific install steps** (Windows MSVC / Linux packages / macOS Xcode CLT).

Optional deeper probe (downloads and compiles a tiny Bevy crate — slow):

```bash
grok-bevy doctor --compile-probe
```

### 2. Register the MCP server with Grok Build

**Option A — config.toml** (`~/.grok/config.toml` or project `.grok/config.toml`):

```toml
[mcp_servers.grok-bevy]
command = "grok-bevy"          # or absolute path from `cargo build -p grok-bevy`
args = ["mcp"]
enabled = true
startup_timeout_sec = 30

# Recommended: full BRP tool surface from upstream
[mcp_servers.bevy-brp]
command = "bevy_brp_mcp"
args = []
enabled = true
```

**Option B — CLI:**

```bash
# After: cargo install --path crates/grok-bevy
grok mcp add grok-bevy -- grok-bevy mcp

# Full-featured upstream server
cargo install bevy_brp_mcp --locked
grok mcp add bevy-brp -- bevy_brp_mcp
```

Print snippets for your built binary:

```bash
grok-bevy mcp-config
```

### 3. Run a BRP-enabled sample

```bash
cargo run -p grok_bevy_sample --features remote,capture
```

In another terminal:

```bash
grok-bevy brp wait --port 15702
grok-bevy brp query --port 15702
grok-bevy brp screenshot --path captures/scene.png --with-image-meta
```

Or from a Grok Build session: use MCP tools `bevy_brp_query`, `bevy_brp_mutate`, `bevy_capture_viewport`, or the richer `bevy_brp_mcp` tools (`brp_launch`, `world_query`, `brp_extras_screenshot`, …).

### 4. Scaffold your own app

```bash
grok-bevy scaffold --path ./my-bevy-game
cd my-bevy-game
cargo run --features remote,capture
```

## Architecture

```
┌─────────────────┐     MCP stdio      ┌──────────────────┐
│  Grok Build /   │ ◄────────────────► │  grok-bevy mcp   │
│  other agents   │                    │  (+ bevy_brp_mcp)│
└─────────────────┘                    └────────┬─────────┘
                                                │ HTTP JSON-RPC (BRP)
                                                ▼
                                       ┌──────────────────┐
                                       │ Bevy app +       │
                                       │ RemotePlugin /   │
                                       │ BrpExtrasPlugin  │
                                       └──────────────────┘
```

| Crate | Description |
|-------|-------------|
| [`grok-bevy-env`](crates/grok-bevy-env) | Cross-platform readiness checks (testable `CommandRunner`) |
| [`grok-bevy-brp`](crates/grok-bevy-brp) | BRP HTTP client, named targets, PNG → MCP image adapter |
| [`grok-bevy`](crates/grok-bevy) | CLI + MCP server |
| [`templates/sample-app`](templates/sample-app) | 3D scene + `remote` / `capture` features + headless BRP binary |

## MCP tools (grok-bevy)

| Tool | Purpose |
|------|---------|
| `bevy_env_check` | OS + Rust/Cargo readiness report |
| `bevy_register_target` / `bevy_list_targets` | Named multi-instance targets |
| `bevy_brp_discover` | `rpc.discover` |
| `bevy_brp_query` | `world.query` |
| `bevy_brp_mutate` | `world.mutate_components` |
| `bevy_brp_call` | Any BRP method (including `brp_extras/*`) |
| `bevy_capture_viewport` | Screenshot → **MCP image** content |
| `bevy_launch_app` | `cargo run` a manifest with features |
| `bevy_brp_mcp_status` | Detect/install guidance for upstream MCP |

For hierarchy ops, diagnostics, watches, keyboard/mouse, and rich launch discovery, use **`bevy_brp_mcp`** (same BRP port).

## Feature flags (sample app)

```toml
[features]
remote  = ["dep:bevy_brp_extras", "bevy/bevy_remote"]
capture = ["remote"]
```

```rust
#[cfg(feature = "remote")]
app.add_plugins(bevy_brp_extras::BrpExtrasPlugin::default());
```

`BrpExtrasPlugin` wires `RemotePlugin` + `RemoteHttpPlugin` (default port **15702**) and methods such as `brp_extras/screenshot` and `brp_extras/get_diagnostics`.

## CLI reference

```text
grok-bevy doctor [--compile-probe] [--json]
grok-bevy mcp [--delegate-brp-mcp]
grok-bevy scaffold --path DIR [--force]
grok-bevy brp discover|query|mutate|call|screenshot|wait
grok-bevy compat
grok-bevy mcp-config
```

## Troubleshooting

See [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) for common issues:

- Rust/Cargo missing → rustup + OS build tools  
- BRP connection refused → app not run with `remote` / wrong port  
- Black screenshots → minimized/occluded window (known platform limitation)  
- Slow Bevy compiles → use `opt-level` workspace profiles  

## License

Dual-licensed under **MIT** OR **Apache-2.0**, at your option — matching [Bevy](https://github.com/bevyengine/bevy).

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)

## Credits

- [Bevy](https://bevy.org) — engine and Remote Protocol  
- [natepiano/bevy_brp](https://github.com/natepiano/bevy_brp) — `bevy_brp_mcp` + `bevy_brp_extras`  

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please keep the first public surface focused and well-tested.

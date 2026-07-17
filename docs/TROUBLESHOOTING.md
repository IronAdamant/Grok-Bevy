# Troubleshooting

## Environment / `grok-bevy doctor`

### `rustc` or `cargo` missing

Install via [rustup](https://rustup.rs):

| OS | Notes |
|----|--------|
| **Windows** | rustup + Visual Studio Build Tools (“Desktop development with C++”) |
| **Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` then distro `build-essential` / `gcc` |
| **macOS** | rustup + `xcode-select --install` |

Open a **new** shell after installing so `PATH` includes `~/.cargo/bin`.

### Compile probe fails

`doctor --compile-probe` creates a temporary Bevy dependency and runs `cargo build`. Failures are often:

- Missing C linker / MSVC tools  
- Linux: missing X11/Wayland/ALSA/udev headers (see doctor’s install guidance)  
- Network blocked from crates.io  

You can still use Grok-Bevy without the probe if `rustc` and `cargo` report ready.

### Bevy is not a global binary

Grok-Bevy correctly does **not** search for a `bevy` executable. Bevy is pulled as a Cargo crate per project.

## BRP connection

### Connection refused on port 15702

1. Run the app with BRP enabled:
   ```bash
   cargo run -p grok_bevy_sample --features remote,capture
   ```
2. Confirm nothing else is bound: change port with `BrpExtrasPlugin::with_port(…)` or `BRP_EXTRAS_PORT`.
3. Wait for first compile + window startup:
   ```bash
   grok-bevy brp wait --port 15702 --timeout-secs 180
   ```

### Query returns empty / type not found

Use fully-qualified reflect paths, e.g.:

```text
bevy_transform::components::transform::Transform
```

Discover methods with `grok-bevy brp discover` or MCP `bevy_brp_discover`. For schema help, prefer `bevy_brp_mcp`’s type guide tools.

### Mutate fails

- Entity IDs are Bevy `u64` canonical IDs from query results.  
- `path` uses Bevy `GetPath` syntax (e.g. `translation.x`).  
- Component must be `Reflect` and registered.

## Screenshots / visual capture

### `brp_extras/screenshot` method not found

The running app needs `bevy_brp_extras` (`capture` / `remote` features on the sample). Plain `RemotePlugin` alone does not register screenshot.

Also enable Bevy’s `png` feature.

### Black or empty PNG

Known limitation: minimized, hidden, or fully occluded primary windows may produce black frames on some platforms. Success means the PNG is fully written — not that pixels are nonuniform.

Mitigations:

- Keep the window visible and focused  
- Capture a specific camera entity when supported  
- Use headless/offscreen targets in advanced apps  

### MCP image not showing in Grok Build

`bevy_capture_viewport` returns MCP `image` content (base64 PNG). If the client truncates large payloads, lower resolution or use the path returned in the text block and `read` the file.

## MCP server

### Grok Build cannot start `grok-bevy`

- Use an absolute `command` path in `config.toml`  
- Ensure the binary is built: `cargo build -p grok-bevy --release`  
- Logs belong on stderr; do not redirect stdout  

### Want the full Bevy agent tool list

```bash
cargo install bevy_brp_mcp --locked
grok mcp add bevy-brp -- bevy_brp_mcp
```

Or: `grok-bevy mcp --delegate-brp-mcp` (requires `bevy_brp_mcp` on `PATH`).

## Slow compiles

Bevy debug builds are heavy. The sample uses elevated `opt-level` for dependencies. For day-to-day work consider:

- `cargo run --release` when iterating on BRP only  
- Dynamic linking setups from Bevy docs (advanced)  

## Version skew

Keep Bevy, `bevy_brp_extras`, and `bevy_brp_mcp` on the same track (see README matrix). Mixing 0.18 apps with 0.22 MCP crates will fail in subtle ways.

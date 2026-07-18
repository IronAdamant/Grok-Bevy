# Grok-Bevy sample app (BRP **demo fixture**)

Minimal Bevy 0.19 3D scene for **Remote Protocol / capture smoke tests**.

> This is **not** a production game template. For playable starters:
>
> ```bash
> grok-bevy scaffold --kind 2d --path ./my-2d-game
> grok-bevy scaffold --kind 3d --path ./my-3d-game
> ```

## Features

| Feature   | What it enables |
|-----------|-----------------|
| (default) | Static 3D scene only |
| `remote`  | `bevy_brp_extras` → RemotePlugin + HTTP on port **15702** |
| `capture` | Same as `remote` (screenshot / diagnostics methods) |

## Run (workspace)

```bash
cargo run -p grok_bevy_sample --features remote,capture
```

## Control from Grok-Bevy

```bash
cargo run -p grok-bevy -- brp wait --port 15702
cargo run -p grok-bevy -- brp query --port 15702
cargo run -p grok-bevy -- brp screenshot --path captures/scene.png --with-image-meta
```

Or use the MCP server (`grok-bevy mcp`) / `bevy_brp_mcp` from Grok Build.

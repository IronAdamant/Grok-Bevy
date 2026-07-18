# Grok-Bevy

**Build and control [Bevy](https://bevy.org) games with AI coding agents** — especially [Grok Build](https://x.ai), and any tool that speaks **MCP (Model Context Protocol)**.

Grok-Bevy is an open-source companion for people who want help from an AI assistant while making **2D or 3D games in Rust** with the Bevy game engine. It checks that your computer is ready, can start a small playable game for you, and gives your AI agent **live eyes and hands** on a running game: launch the app, inspect the scene, change things, and take screenshots.

> **Public demo.** This repo is a working demonstration of agent-native Bevy tooling. Use it to learn the workflow, dogfood it, and adapt it for your own projects.

---

## Who this is for

| You might be… | Grok-Bevy helps you… |
|---------------|----------------------|
| New to Bevy, using **Grok Build** or another AI coding assistant | Get a ready machine + a real starter game without memorizing every setup step |
| Building a game with an **LLM** (Grok, Claude, GPT, local models, etc.) | Give the agent tools so it can *see* and *control* the live game, not only edit code |
| Looking for an **MCP server for Bevy** | Plug into Grok Build, Cursor, Claude Code, Continue, Cline, Windsurf, or other MCP clients |
| Prototyping a **2D platformer / top-down game** or a small **3D** scene | Scaffold a playable vertical slice (menu → play → move) with remote control hooks |

You do **not** need to be a Rust expert to start. You will need a normal developer setup (Rust installed, ability to run terminal commands). The `doctor` command explains what’s missing in plain language.

---

## What you get in plain English

1. **“Is my PC ready for Bevy?”** — `grok-bevy doctor` checks Rust and your OS, then suggests fixes.  
2. **“Start a game project for me.”** — Scaffold a **2D** or **3D** starter (or a simple demo cube for testing).  
3. **“Let my AI play with the running game.”** — An **MCP server** so the agent can query the world, tweak values, and **capture the viewport as an image**.  
4. **“How should the AI build a real game?”** — Built-in **Grok skills** (playbooks) for production structure, not one-file demos.

**Skills define how to build. Scaffold defines where files go. MCP verifies what is on screen.**

---

## Why MCP + Bevy matters for AI game development

Most AI coding chats only see source files. Game work also needs:

- Does it **compile and run** on this machine?  
- What does the **player actually see**?  
- Can the agent **change a transform** and re-check without guessing?

Grok-Bevy wires that loop using the **Bevy Remote Protocol (BRP)** and optional [bevy_brp_mcp](https://github.com/natepiano/bevy_brp) / [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras). Your agent (Grok Build or another MCP-compatible LLM client) can treat a live Bevy app like a controllable environment: **launch → query → mutate → screenshot → fix → repeat**.

That makes this project useful if you are searching for phrases like:

- *Bevy MCP server* / *MCP for Bevy game engine*  
- *AI coding agent for game development*  
- *Grok Build Bevy* / *LLM-assisted Rust game dev*  
- *screenshot the Bevy window for an AI assistant*  
- *scaffold a 2D or 3D Bevy game with remote control*

---

## Quick start (about 10 minutes)

### Prerequisites

- A computer that can run games (a normal GPU + window is best for screenshots)  
- [Rust](https://rustup.rs) (`rustc` + `cargo`)  
- Optional but recommended: [Grok Build](https://x.ai) or any **MCP client** for AI agents  

### 1. Install the CLI

```bash
git clone https://github.com/IronAdamant/Grok-Bevy.git
cd Grok-Bevy
cargo install --path crates/grok-bevy
```

### 2. Check your machine

```bash
grok-bevy doctor
```

You want a **READY** report. If something is missing, follow the printed install tips for Windows, macOS, or Linux.

### 3. Connect the MCP server (so your AI can use it)

**Grok Build** — add to `~/.grok/config.toml` (or run `grok mcp add …`):

```toml
[mcp_servers.grok-bevy]
command = "grok-bevy"   # or the full path from `which grok-bevy`
args = ["mcp"]
enabled = true
startup_timeout_sec = 30
```

Optional richer Bevy control (hierarchy, watches, input injection):

```bash
cargo install bevy_brp_mcp --locked
```

```toml
[mcp_servers.bevy-brp]
command = "bevy_brp_mcp"
args = []
enabled = true
```

Print ready-made config:

```bash
grok-bevy mcp-config
```

**Other MCP clients** (Cursor, Claude Code, Continue, Cline, etc.): register a stdio MCP server that runs `grok-bevy mcp` the same way you register any local MCP tool.

Restart or reload MCP so the agent sees tools like `bevy_env_check`, `bevy_launch_app`, `bevy_capture_viewport`, and `bevy_workflow`.

### 4. Play the in-repo short demos (dogfood)

These are **real short games** (objective, hazard, win/lose) — the default dogfood path:

```bash
# 2D: collect 3 orbs, avoid the red hazard
cargo run -p demo_2d --features remote,capture

# 3D: collect 3 cubes, avoid hazard / don't fall off
cargo run -p demo_3d --features remote,capture
```

Package a playable folder/zip (binary + `assets/`):

```bash
./scripts/package-demo.sh demo_2d games/demo-2d
./scripts/package-demo.sh demo_3d games/demo-3d
```

See [docs/GAME_DOD.md](docs/GAME_DOD.md), [docs/PACKAGING.md](docs/PACKAGING.md), [docs/ROADMAP.md](docs/ROADMAP.md).

### 5. Or scaffold your own game

```bash
grok-bevy scaffold --kind 2d --path ./my-first-game
cd my-first-game
cargo run --features remote,capture
```

- **2D / 3D** — GAME_DOD short demos (menu, collect, hazard, victory/game over).  
- **`demo`** — static cube for BRP plumbing only (not a product game).

Controls and objectives are in each game’s README.
### 6. Let the agent look at the game

With the game running (`remote,capture` features) and MCP connected, ask your assistant something like:

> Check Bevy readiness, wait for the game on port 15702, query the scene, and capture a screenshot of the window.

Or: use the MCP prompt **`start_2d_game`** / **`start_3d_game`**, or the **`bevy_workflow`** tool with goal `new_2d`, `new_3d`, `verify_scene`, or `ship`.

More detail: [docs/FAST_START.md](docs/FAST_START.md) · [docs/PRODUCTION_GAMES.md](docs/PRODUCTION_GAMES.md)

---

## What the AI can do (MCP tools, human labels)

| Tool (name) | Everyday meaning |
|-------------|------------------|
| `bevy_env_check` | Is this computer ready to build Bevy games? |
| `bevy_launch_app` | Start my game with remote control turned on |
| `bevy_brp_query` | What entities/components exist right now? |
| `bevy_brp_mutate` | Change a value on a live object (e.g. position) |
| `bevy_capture_viewport` | **Screenshot the game window** for the agent to “see” |
| `bevy_workflow` | Step-by-step plan: which skills + tools for a goal |
| `bevy_brp_mcp_status` | Is the optional full Bevy BRP MCP installed? |

**MCP prompts** (shortcuts for agents): `start_2d_game`, `start_3d_game`, `iterate_scene`, `prepare_ship`.

---

## Skills (playbooks for building real games)

These live in `.grok/skills/` so Grok Build (and similar) can load them automatically:

| Skill | When it helps |
|-------|----------------|
| `bevy-production` | Project layout, states, shipping mindset |
| `bevy-2d-game` | 2D camera, sprites, movement |
| `bevy-3d-game` | 3D camera, lights, meshes |
| `bevy-agent-loop` | Live run → screenshot → fix loop |

For **game art**, pair with Grok’s image skills (e.g. `game-asset-core`) and drop files under `assets/sprites`, `assets/models`, `assets/ui`, or `assets/audio`. See [docs/ASSET_CONVENTIONS.md](docs/ASSET_CONVENTIONS.md) and [docs/SHIPPING.md](docs/SHIPPING.md).

---

## Compatibility

| Piece | Version |
|-------|---------|
| Bevy game engine | **0.19** |
| bevy_brp_mcp / bevy_brp_extras | **0.22.1** |
| Grok-Bevy | **0.1.x** (v0.2 features on `main`) |

Bevy is a **Cargo** dependency of your game project—not a separate installer. “Ready” means this machine can **compile and run** Bevy apps.

---

## Project map (for the curious)

```
Your AI (Grok Build / Claude / Cursor / …)
        │  MCP (stdio)
        ▼
   grok-bevy mcp  (+ optional bevy_brp_mcp)
        │  Bevy Remote Protocol (HTTP)
        ▼
   Your Bevy game (features: remote, capture)
```

| Piece | Role |
|-------|------|
| `grok-bevy` CLI | doctor, scaffold, BRP helpers, MCP entry |
| MCP server | agent tools + prompts + workflow router |
| `templates/game-2d`, `game-3d` | playable starters |
| `templates/sample-app` | remote-control test fixture (cube) |
| `.grok/skills/` | production playbooks |

---

## CLI cheat sheet

```text
grok-bevy doctor [--compile-probe] [--json]
grok-bevy mcp
grok-bevy scaffold --kind 2d|3d|demo --path DIR [--name NAME]
grok-bevy brp discover|query|mutate|call|screenshot|wait
grok-bevy mcp-config
grok-bevy compat
```

---

## Troubleshooting (short list)

Full guide: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

- **Doctor not READY** — install Rust via rustup; follow OS tips (Xcode CLT / MSVC / Linux packages).  
- **Agent can’t connect** — game must run with `--features remote,capture`; default port **15702**.  
- **Black screenshot** — keep the game window visible (not minimized).  
- **First compile is slow** — normal for Bevy; later builds are faster.  

---

## License

**MIT OR Apache-2.0** (same dual license style as Bevy).

- [LICENSE-MIT](LICENSE-MIT)  
- [LICENSE-APACHE](LICENSE-APACHE)  

## Credits

- [Bevy](https://bevy.org) — game engine and Remote Protocol  
- [natepiano/bevy_brp](https://github.com/natepiano/bevy_brp) — `bevy_brp_mcp` and `bevy_brp_extras`  
- [Grok Build](https://x.ai) / xAI — agent-first coding environment this project targets first  

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This public demo favors a **small, tested surface**: readiness, scaffold, MCP/BRP control, and clear agent docs.

---

### Keywords for discovery (humans and agents)

Grok-Bevy · Grok Build · MCP server · Model Context Protocol · Bevy game engine · Bevy Remote Protocol · BRP · AI game development · LLM coding agent · Rust game engine · 2D game scaffold · 3D game scaffold · agent screenshots · viewport capture · open source Bevy tooling  

If you found this while looking for an **AI-friendly Bevy workflow**, start with `doctor` → register `grok-bevy mcp` → `scaffold --kind 2d` → ask your agent to capture the window.

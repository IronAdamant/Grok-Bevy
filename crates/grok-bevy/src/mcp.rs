//! Minimal MCP stdio server for Grok Build agents.
//!
//! Implements the core MCP surface (initialize, tools/list, tools/call) over
//! newline-delimited JSON-RPC on stdin/stdout. Designed to complement
//! `bevy_brp_mcp` (full BRP tool surface) with Grok-Bevy-specific tools:
//! environment readiness, BRP query/mutate, and viewport capture as images.

use anyhow::{anyhow, Context, Result};
use grok_bevy_brp::{
    capture_viewport_image, BrpClient, BrpTarget, CapturedImage, TargetRegistry, DEFAULT_PORT,
};
use grok_bevy_env::{check_readiness, DoctorOptions, SystemCommandRunner};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

struct ServerState {
    targets: TargetRegistry,
    /// Child process of a launched sample / app (if any).
    child: Option<tokio::process::Child>,
}

impl ServerState {
    fn new() -> Self {
        let mut targets = TargetRegistry::new();
        targets.register(BrpTarget::default_local());
        Self {
            targets,
            child: None,
        }
    }
}

pub async fn run_stdio_server() -> Result<()> {
    // MCP logs must not go to stdout.
    tracing::info!("Grok-Bevy MCP server starting on stdio");
    let state = Arc::new(Mutex::new(ServerState::new()));
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("invalid JSON on stdin: {e}");
                continue;
            }
        };

        // Notifications have no id and no response.
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        if id.is_none() {
            // notification
            if method == "notifications/initialized" {
                tracing::info!("client initialized");
            }
            continue;
        }

        let result = handle_request(method, params, state.clone()).await;
        let response = match result {
            Ok(r) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": r,
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32000,
                    "message": e.to_string(),
                }
            }),
        };
        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }
    Ok(())
}

async fn handle_request(method: &str, params: Value, state: Arc<Mutex<ServerState>>) -> Result<Value> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "grok-bevy",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": mcp_instructions()
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_defs() })),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| anyhow!("missing tool name"))?;
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            call_tool(name, args, state).await
        }
        "resources/list" => Ok(json!({ "resources": [] })),
        "prompts/list" => Ok(json!({ "prompts": prompts_list_json() })),
        "prompts/get" => {
            let name = params
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| anyhow!("missing prompt name"))?;
            prompts_get_json(name)
        }
        other => Err(anyhow!("method not found: {other}")),
    }
}

/// High-signal agent instructions returned on MCP `initialize`.
pub fn mcp_instructions() -> String {
    concat!(
        "Grok-Bevy MCP: environment readiness, BRP query/mutate/call, viewport capture, and app launch. ",
        "Production games (not demos): load Grok skills bevy-production plus bevy-2d-game or bevy-3d-game; ",
        "for art load game-asset-core (+ specialist); for live verify load bevy-agent-loop. ",
        "Scaffold: `grok-bevy scaffold --kind 2d|3d|demo --path DIR` (2d/3d = playable production slices; demo = BRP fixture only). ",
        "MCP prompts: start_2d_game, start_3d_game, iterate_scene, prepare_ship. ",
        "Workflow router tool: bevy_workflow with goal new_2d|new_3d|verify_scene|ship|add_sprite. ",
        "Asset roots: assets/sprites, assets/models, assets/ui, assets/audio. Ship: cargo build --release. ",
        "Typical loop: bevy_env_check → bevy_launch_app (features remote,capture) → bevy_brp_query/mutate → bevy_capture_viewport. ",
        "BRP default port 15702. For full BRP hierarchy/watches/input, also install and register bevy_brp_mcp ",
        "(cargo install bevy_brp_mcp --locked)."
    )
    .to_string()
}

/// One MCP production prompt (list + get share this catalog).
#[derive(Debug, Clone, Copy)]
pub struct PromptDef {
    pub name: &'static str,
    pub description: &'static str,
    pub body: &'static str,
}

/// Production entry prompts advertised on `prompts/list`.
pub fn prompt_catalog() -> &'static [PromptDef] {
    &[
        PromptDef {
            name: "start_2d_game",
            description:
                "Scaffold and build a production 2D Bevy game (skills, layout, vertical slice).",
            body: concat!(
                "You are building a production 2D Bevy 0.19 game with Grok-Bevy — not a cube demo.\n\n",
                "1. Load skills: bevy-production + bevy-2d-game. For art, also game-asset-core (+ specialist).\n",
                "2. Scaffold: `grok-bevy scaffold --kind 2d --path <dir>` (or extend an existing production layout).\n",
                "3. Keep layout: thin main.rs, plugins, states Loading|MainMenu|Playing|Paused, assets/sprites|ui|audio.\n",
                "4. Ensure menu→play, WASD/arrow movement, and at least one disk asset via AssetServer.\n",
                "5. Run with --features remote,capture. Live verify with bevy-agent-loop: ",
                "bevy_env_check → bevy_launch_app → bevy_brp_query → bevy_capture_viewport.\n",
                "6. Optional: call bevy_workflow with goal \"new_2d\" for an ordered checklist.\n",
            ),
        },
        PromptDef {
            name: "start_3d_game",
            description:
                "Scaffold and build a production 3D Bevy game (skills, lighting, vertical slice).",
            body: concat!(
                "You are building a production 3D Bevy 0.19 game with Grok-Bevy — not a static BRP fixture.\n\n",
                "1. Load skills: bevy-production + bevy-3d-game. UI/art overlays: game-asset-core (+ specialist).\n",
                "2. Scaffold: `grok-bevy scaffold --kind 3d --path <dir>`.\n",
                "3. Keep layout: plugins, states Loading|MainMenu|Playing|Paused, assets/models|ui|audio.\n",
                "4. Ensure menu→play, XZ (or genre) movement, lit scene, disk texture/glTF path under assets/models.\n",
                "5. Run with --features remote,capture. Verify via bevy-agent-loop and bevy_capture_viewport ",
                "(avoid black captures: lights, visible window).\n",
                "6. Optional: bevy_workflow goal \"new_3d\".\n",
            ),
        },
        PromptDef {
            name: "iterate_scene",
            description:
                "Live BRP/MCP loop: launch, query/mutate, capture viewport, fix, recapture.",
            body: concat!(
                "Iterate on a running Bevy app with evidence from viewport captures.\n\n",
                "1. Load skill: bevy-agent-loop (and bevy-production if changing structure).\n",
                "2. Ensure the app uses features remote,capture and BRP port 15702 (or registered target).\n",
                "3. MCP loop: bevy_env_check → bevy_launch_app (if needed) → bevy_brp_discover/query → ",
                "optional bevy_brp_mutate → bevy_capture_viewport → describe defects → patch code/art → recapture.\n",
                "4. Prefer fully-qualified Reflect type paths; Name entities for readable queries.\n",
                "5. For full hierarchy/watches/input injection use bevy_brp_mcp when installed.\n",
                "6. Optional: bevy_workflow goal \"verify_scene\".\n",
            ),
        },
        PromptDef {
            name: "prepare_ship",
            description:
                "Ship checklist: release build, assets next to binary, production readiness.",
            body: concat!(
                "Prepare a Bevy game for engineering release (not store certification).\n\n",
                "1. Load skill: bevy-production (ship checklist). Confirm states and playability.\n",
                "2. Assets under assets/sprites|models|ui|audio load via AssetServer; see docs/ASSET_CONVENTIONS.md.\n",
                "3. Run `cargo build --release`; binary at target/release/<package>. Ship assets/ beside the binary or document CWD.\n",
                "4. Verify menu→play→pause, release textures not missing, README controls/features.\n",
                "5. Optional live capture review with bevy_capture_viewport before calling done.\n",
                "6. Optional: bevy_workflow goal \"ship\". See docs/SHIPPING.md.\n",
            ),
        },
    ]
}

pub fn prompts_list_json() -> Value {
    Value::Array(
        prompt_catalog()
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "description": p.description,
                })
            })
            .collect(),
    )
}

pub fn prompts_get_json(name: &str) -> Result<Value> {
    let prompt = prompt_catalog()
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("unknown prompt: {name}"))?;
    Ok(json!({
        "description": prompt.description,
        "messages": [{
            "role": "user",
            "content": {
                "type": "text",
                "text": prompt.body
            }
        }]
    }))
}

/// Production goals accepted by the `bevy_workflow` tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowGoal {
    New2d,
    New3d,
    VerifyScene,
    Ship,
    AddSprite,
}

impl WorkflowGoal {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "new_2d" | "2d" | "start_2d" | "start_2d_game" => Ok(Self::New2d),
            "new_3d" | "3d" | "start_3d" | "start_3d_game" => Ok(Self::New3d),
            "verify_scene" | "iterate" | "iterate_scene" | "capture" => Ok(Self::VerifyScene),
            "ship" | "prepare_ship" | "release" => Ok(Self::Ship),
            "add_sprite" | "art" | "sprite" => Ok(Self::AddSprite),
            other => bail_unknown_goal(other),
        }
    }

}

fn bail_unknown_goal(other: &str) -> Result<WorkflowGoal> {
    Err(anyhow!(
        "unknown workflow goal '{other}' (expected new_2d, new_3d, verify_scene, ship, add_sprite)"
    ))
}

/// Ordered production plan for a workflow goal (skills + tools/CLI).
pub fn workflow_plan(goal: WorkflowGoal) -> String {
    match goal {
        WorkflowGoal::New2d => concat!(
            "Goal: new_2d — production 2D Bevy game\n",
            "Skills to load:\n",
            "  1. bevy-production\n",
            "  2. bevy-2d-game\n",
            "  3. game-asset-core (+ specialist when generating art)\n",
            "  4. bevy-agent-loop (when verifying live)\n",
            "Steps:\n",
            "  1. bevy_env_check (MCP) or `grok-bevy doctor`\n",
            "  2. CLI: `grok-bevy scaffold --kind 2d --path <game-dir>`\n",
            "  3. Implement/extend vertical slice (MainMenu→Playing, movement, assets/sprites)\n",
            "  4. MCP: bevy_launch_app with features remote,capture\n",
            "  5. MCP: bevy_brp_query / bevy_capture_viewport; fix until capture matches acceptance\n",
            "  6. Optional prompt: start_2d_game\n",
        )
        .to_string(),
        WorkflowGoal::New3d => concat!(
            "Goal: new_3d — production 3D Bevy game\n",
            "Skills to load:\n",
            "  1. bevy-production\n",
            "  2. bevy-3d-game\n",
            "  3. game-asset-core (+ specialist for UI/art)\n",
            "  4. bevy-agent-loop (when verifying live)\n",
            "Steps:\n",
            "  1. bevy_env_check or `grok-bevy doctor`\n",
            "  2. CLI: `grok-bevy scaffold --kind 3d --path <game-dir>`\n",
            "  3. Implement/extend slice (menu→play, movement, lighting, assets/models)\n",
            "  4. MCP: bevy_launch_app (remote,capture)\n",
            "  5. MCP: bevy_brp_query / bevy_capture_viewport; ensure lit non-black scene\n",
            "  6. Optional prompt: start_3d_game\n",
        )
        .to_string(),
        WorkflowGoal::VerifyScene => concat!(
            "Goal: verify_scene — live iterate with capture evidence\n",
            "Skills to load:\n",
            "  1. bevy-agent-loop\n",
            "  2. bevy-production (if restructuring)\n",
            "Steps:\n",
            "  1. Confirm app features remote,capture; BRP port 15702\n",
            "  2. MCP: bevy_launch_app if not running\n",
            "  3. MCP: bevy_brp_discover → bevy_brp_query\n",
            "  4. Optional: bevy_brp_mutate for quick transform checks\n",
            "  5. MCP: bevy_capture_viewport — inspect image, list defects\n",
            "  6. Patch code or assets; recapture until acceptance\n",
            "  7. Optional prompt: iterate_scene\n",
        )
        .to_string(),
        WorkflowGoal::Ship => concat!(
            "Goal: ship — release readiness\n",
            "Skills to load:\n",
            "  1. bevy-production (ship checklist)\n",
            "  2. bevy-agent-loop (optional final capture)\n",
            "Steps:\n",
            "  1. Confirm menu→play→pause and disk assets under assets/\n",
            "  2. CLI: `cargo build --release` in the game project\n",
            "  3. Place/document assets/ next to the release binary\n",
            "  4. Update README controls/features; skim docs/SHIPPING.md + ASSET_CONVENTIONS.md\n",
            "  5. Optional: bevy_capture_viewport final visual check\n",
            "  6. Optional prompt: prepare_ship\n",
        )
        .to_string(),
        WorkflowGoal::AddSprite => concat!(
            "Goal: add_sprite — engine-ready art into a Bevy game\n",
            "Skills to load:\n",
            "  1. game-asset-core (+ game-character-consistency / game-animation-frames / game-tilesets / game-ui-icons as needed)\n",
            "  2. bevy-2d-game or bevy-3d-game depending on project\n",
            "  3. bevy-agent-loop for capture verify\n",
            "Steps:\n",
            "  1. Generate engine-ready art (keyable background for sprites)\n",
            "  2. Write under assets/sprites (or ui/models as appropriate)\n",
            "  3. Load via AssetServer path relative to assets/\n",
            "  4. MCP: bevy_launch_app + bevy_capture_viewport to confirm scale/pivot\n",
        )
        .to_string(),
    }
}

fn tool_defs() -> Value {
    json!([
        {
            "name": "bevy_env_check",
            "description": "Check whether this machine can build and run Bevy apps (OS, Rust/Cargo, optional compile probe). Returns a structured readiness report with OS-specific install guidance.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "compile_probe": {
                        "type": "boolean",
                        "description": "If true, create and compile a minimal Bevy probe (slow).",
                        "default": false
                    }
                }
            }
        },
        {
            "name": "bevy_register_target",
            "description": "Register a named BRP target (host/port) for multi-instance control.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "port": { "type": "integer", "default": 15702 },
                    "host": { "type": "string", "default": "127.0.0.1" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "bevy_list_targets",
            "description": "List named BRP targets known to this MCP server.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "bevy_brp_discover",
            "description": "Call rpc.discover on a running Bevy app's BRP HTTP port.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": 15702 },
                    "target": { "type": "string", "description": "Named target (optional)" }
                }
            }
        },
        {
            "name": "bevy_brp_query",
            "description": "Query entities/components via world.query on a running BRP-enabled Bevy app.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": 15702 },
                    "target": { "type": "string" },
                    "components": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Fully-qualified component type paths"
                    }
                }
            }
        },
        {
            "name": "bevy_brp_mutate",
            "description": "Mutate a component field via world.mutate_components.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": 15702 },
                    "target": { "type": "string" },
                    "entity": { "type": "integer" },
                    "component": { "type": "string" },
                    "path": { "type": "string" },
                    "value": {}
                },
                "required": ["entity", "component", "path", "value"]
            }
        },
        {
            "name": "bevy_brp_call",
            "description": "Generic BRP JSON-RPC method call (any registered method including brp_extras/*).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": 15702 },
                    "target": { "type": "string" },
                    "method": { "type": "string" },
                    "params": {}
                },
                "required": ["method"]
            }
        },
        {
            "name": "bevy_capture_viewport",
            "description": "Capture the Bevy primary window (or camera if supported by extras) via brp_extras/screenshot and return PNG image content for the agent to see the 3D scene.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "port": { "type": "integer", "default": 15702 },
                    "target": { "type": "string" },
                    "path": {
                        "type": "string",
                        "description": "Filesystem path for the PNG",
                        "default": "captures/grok-bevy-scene.png"
                    }
                }
            }
        },
        {
            "name": "bevy_launch_app",
            "description": "Launch a Bevy app (cargo run) with optional features. Prefer apps that enable remote+capture features. Logs go to a temp file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "manifest_path": {
                        "type": "string",
                        "description": "Path to Cargo.toml of the Bevy app"
                    },
                    "features": {
                        "type": "string",
                        "description": "Cargo features, e.g. remote,capture",
                        "default": "remote,capture"
                    },
                    "port": {
                        "type": "integer",
                        "description": "Expected BRP port after launch",
                        "default": 15702
                    },
                    "name": {
                        "type": "string",
                        "description": "Named target to register",
                        "default": "launched"
                    }
                },
                "required": ["manifest_path"]
            }
        },
        {
            "name": "bevy_brp_mcp_status",
            "description": "Check whether bevy_brp_mcp is installed (full BRP MCP tool surface) and print install guidance.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "bevy_workflow",
            "description": "Production workflow router: given a goal (new_2d, new_3d, verify_scene, ship, add_sprite), return ordered steps naming which Grok skills to load and which grok-bevy MCP tools/CLI actions to use.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "goal": {
                        "type": "string",
                        "description": "One of: new_2d, new_3d, verify_scene, ship, add_sprite (aliases: 2d, 3d, iterate_scene, prepare_ship, sprite)"
                    }
                },
                "required": ["goal"]
            }
        }
    ])
}

async fn call_tool(name: &str, args: Value, state: Arc<Mutex<ServerState>>) -> Result<Value> {
    match name {
        "bevy_env_check" => {
            let compile_probe = args
                .get("compile_probe")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let report = tokio::task::spawn_blocking(move || {
                check_readiness(
                    &SystemCommandRunner,
                    &DoctorOptions {
                        compile_probe,
                        ..DoctorOptions::default()
                    },
                )
            })
            .await?;
            text_result(serde_json::to_string_pretty(&report)?)
        }
        "bevy_register_target" => {
            let name = args["name"].as_str().ok_or_else(|| anyhow!("name required"))?;
            let port = args.get("port").and_then(|p| p.as_u64()).unwrap_or(DEFAULT_PORT as u64)
                as u16;
            let host = args
                .get("host")
                .and_then(|h| h.as_str())
                .unwrap_or("127.0.0.1")
                .to_string();
            let mut t = BrpTarget::new(name, port);
            t.host = host;
            let mut st = state.lock().await;
            st.targets.register(t.clone());
            text_result(format!("registered target {} -> {}", t.name, t.base_url()))
        }
        "bevy_list_targets" => {
            let st = state.lock().await;
            text_result(serde_json::to_string_pretty(st.targets.list())?)
        }
        "bevy_brp_discover" => {
            let client = client_from_args(&args, &state).await?;
            let result = tokio::task::spawn_blocking(move || {
                client.call("rpc.discover", None)?.into_result()
            })
            .await??;
            text_result(serde_json::to_string_pretty(&result)?)
        }
        "bevy_brp_query" => {
            let client = client_from_args(&args, &state).await?;
            let comps: Vec<String> = args
                .get("components")
                .and_then(|c| c.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| {
                    vec!["bevy_transform::components::transform::Transform".into()]
                });
            let result = tokio::task::spawn_blocking(move || {
                let refs: Vec<&str> = comps.iter().map(|s| s.as_str()).collect();
                client.query(&refs)
            })
            .await??;
            text_result(serde_json::to_string_pretty(&result)?)
        }
        "bevy_brp_mutate" => {
            let client = client_from_args(&args, &state).await?;
            let entity = args["entity"]
                .as_u64()
                .ok_or_else(|| anyhow!("entity required"))?;
            let component = args["component"]
                .as_str()
                .ok_or_else(|| anyhow!("component required"))?
                .to_string();
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path required"))?
                .to_string();
            let value = args
                .get("value")
                .cloned()
                .ok_or_else(|| anyhow!("value required"))?;
            let result = tokio::task::spawn_blocking(move || {
                client.mutate_components(entity, &component, &path, value)
            })
            .await??;
            text_result(serde_json::to_string_pretty(&json!({
                "ok": true,
                "result": result
            }))?)
        }
        "bevy_brp_call" => {
            let client = client_from_args(&args, &state).await?;
            let method = args["method"]
                .as_str()
                .ok_or_else(|| anyhow!("method required"))?
                .to_string();
            let params = args.get("params").cloned();
            let result = tokio::task::spawn_blocking(move || {
                client.call(&method, params)?.into_result()
            })
            .await??;
            text_result(serde_json::to_string_pretty(&result)?)
        }
        "bevy_capture_viewport" => {
            let client = client_from_args(&args, &state).await?;
            let path = args
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("captures/grok-bevy-scene.png")
                .to_string();
            let img: CapturedImage = tokio::task::spawn_blocking(move || {
                capture_viewport_image(&client, PathBuf::from(path))
            })
            .await??;
            Ok(json!({
                "content": img.to_mcp_content_blocks(),
                "isError": false
            }))
        }
        "bevy_launch_app" => {
            let manifest = args["manifest_path"]
                .as_str()
                .ok_or_else(|| anyhow!("manifest_path required"))?
                .to_string();
            let features = args
                .get("features")
                .and_then(|f| f.as_str())
                .unwrap_or("remote,capture")
                .to_string();
            let port = args.get("port").and_then(|p| p.as_u64()).unwrap_or(DEFAULT_PORT as u64)
                as u16;
            let name = args
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("launched")
                .to_string();

            let log_path = std::env::temp_dir().join(format!(
                "grok-bevy-launch-{}-{}.log",
                name,
                std::process::id()
            ));
            let log_file = std::fs::File::create(&log_path)
                .with_context(|| format!("create log {}", log_path.display()))?;
            let log_err = log_file.try_clone()?;

            let mut cmd = Command::new("cargo");
            cmd.arg("run")
                .arg("--manifest-path")
                .arg(&manifest)
                .arg("--features")
                .arg(&features)
                .stdin(Stdio::null())
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(log_err))
                .kill_on_drop(true);

            let child = cmd.spawn().context("spawn cargo run")?;
            {
                let mut st = state.lock().await;
                st.child = Some(child);
                st.targets.register(BrpTarget::new(&name, port));
            }

            // Wait briefly for BRP.
            let client = BrpClient::with_port(port);
            let wait = tokio::task::spawn_blocking(move || {
                client.wait_until_ready(std::time::Duration::from_secs(120))
            })
            .await?;

            match wait {
                Ok(_) => text_result(format!(
                    "Launched {manifest} (features={features}). BRP ready on port {port} as target '{name}'. Logs: {}",
                    log_path.display()
                )),
                Err(e) => text_result(format!(
                    "Launched process for {manifest}, but BRP not ready yet on port {port}: {e}. Logs: {}. The app may still be compiling.",
                    log_path.display()
                )),
            }
        }
        "bevy_brp_mcp_status" => {
            let status = match which::which("bevy_brp_mcp") {
                Ok(p) => format!(
                    "bevy_brp_mcp found at {}\nRegister it in Grok Build for the full BRP tool surface.\nRecommended version: {}",
                    p.display(),
                    grok_bevy_env::compat::BEVY_BRP_MCP
                ),
                Err(_) => format!(
                    "bevy_brp_mcp not on PATH.\nInstall:\n  cargo install bevy_brp_mcp --locked\nThen:\n  grok mcp add bevy-brp -- bevy_brp_mcp\nPinned version: {}",
                    grok_bevy_env::compat::BEVY_BRP_MCP
                ),
            };
            text_result(status)
        }
        "bevy_workflow" => {
            let goal_str = args
                .get("goal")
                .and_then(|g| g.as_str())
                .ok_or_else(|| anyhow!("goal required (new_2d|new_3d|verify_scene|ship|add_sprite)"))?;
            let goal = WorkflowGoal::parse(goal_str)?;
            text_result(workflow_plan(goal))
        }
        other => Err(anyhow!("unknown tool: {other}")),
    }
}

async fn client_from_args(args: &Value, state: &Arc<Mutex<ServerState>>) -> Result<BrpClient> {
    let port = args.get("port").and_then(|p| p.as_u64()).map(|p| p as u16);
    let name = args.get("target").and_then(|t| t.as_str());
    let st = state.lock().await;
    let target = st.targets.resolve(name, port);
    Ok(BrpClient::new(target))
}

fn text_result(text: impl Into<String>) -> Result<Value> {
    Ok(json!({
        "content": [{ "type": "text", "text": text.into() }],
        "isError": false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_instructions_cover_production_and_scaffold() {
        let s = mcp_instructions();
        assert!(!s.is_empty());
        assert!(s.contains("bevy-production"));
        assert!(s.contains("bevy-2d-game") || s.contains("2d"));
        assert!(s.contains("scaffold"));
        assert!(s.contains("2d") && s.contains("3d") && s.contains("demo"));
        assert!(s.contains("game-asset-core") || s.contains("assets/sprites"));
        assert!(s.contains("cargo build --release") || s.contains("Ship"));
        assert!(s.contains("start_2d_game") && s.contains("bevy_workflow"));
    }

    #[test]
    fn prompt_catalog_has_four_production_entry_points() {
        let names: Vec<&str> = prompt_catalog().iter().map(|p| p.name).collect();
        for expected in [
            "start_2d_game",
            "start_3d_game",
            "iterate_scene",
            "prepare_ship",
        ] {
            assert!(names.contains(&expected), "missing prompt {expected}");
        }
        assert_eq!(prompt_catalog().len(), 4);
        for p in prompt_catalog() {
            assert!(!p.description.is_empty(), "{} empty description", p.name);
            assert!(!p.body.is_empty(), "{} empty body", p.name);
        }
    }

    #[test]
    fn prompts_list_and_get_drive_catalog() {
        let list = prompts_list_json();
        let arr = list.as_array().expect("array");
        assert_eq!(arr.len(), 4);
        for item in arr {
            assert!(item.get("name").and_then(|n| n.as_str()).is_some());
            assert!(item
                .get("description")
                .and_then(|d| d.as_str())
                .is_some_and(|s| !s.is_empty()));
        }

        let two_d = prompts_get_json("start_2d_game").unwrap();
        let text = two_d["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("bevy-2d-game"));
        assert!(text.contains("scaffold") && text.contains("2d"));

        let three_d = prompts_get_json("start_3d_game").unwrap();
        let text = three_d["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("bevy-3d-game"));

        let iterate = prompts_get_json("iterate_scene").unwrap();
        let text = iterate["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();
        assert!(text.contains("bevy-agent-loop") || text.contains("bevy_capture_viewport"));

        let ship = prompts_get_json("prepare_ship").unwrap();
        let text = ship["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("cargo build --release") || text.contains("release"));

        assert!(prompts_get_json("no_such_prompt").is_err());
    }

    #[test]
    fn workflow_plan_lists_skills_and_tools_for_goals() {
        let two_d = workflow_plan(WorkflowGoal::parse("new_2d").unwrap());
        assert!(two_d.contains("bevy-production"));
        assert!(two_d.contains("bevy-2d-game"));
        assert!(two_d.contains("scaffold --kind 2d") || two_d.contains("--kind 2d"));
        assert!(two_d.contains("bevy_launch_app") || two_d.contains("bevy_capture_viewport"));

        let ship = workflow_plan(WorkflowGoal::parse("ship").unwrap());
        assert!(ship.contains("bevy-production"));
        assert!(ship.contains("cargo build --release"));

        let iterate = workflow_plan(WorkflowGoal::parse("verify_scene").unwrap());
        assert!(iterate.contains("bevy-agent-loop"));
        assert!(iterate.contains("bevy_capture_viewport"));

        assert!(WorkflowGoal::parse("not_a_goal").is_err());
    }

    #[test]
    fn tool_defs_include_bevy_workflow() {
        let tools = tool_defs();
        let arr = tools.as_array().unwrap();
        let names: Vec<&str> = arr
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"bevy_workflow"));
        assert!(names.contains(&"bevy_capture_viewport"));
    }
}

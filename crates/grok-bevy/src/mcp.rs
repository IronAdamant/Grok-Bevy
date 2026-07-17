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
                "tools": {}
            },
            "serverInfo": {
                "name": "grok-bevy",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": "Grok-Bevy MCP: env readiness, BRP query/mutate, viewport capture. For the full Bevy BRP tool surface, also install and register bevy_brp_mcp (cargo install bevy_brp_mcp --locked)."
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
        "prompts/list" => Ok(json!({ "prompts": [] })),
        other => Err(anyhow!("method not found: {other}")),
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

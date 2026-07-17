//! Grok-Bevy CLI — environment doctor, BRP helpers, scaffold, and MCP server.

mod mcp;
mod scaffold;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use grok_bevy_brp::{capture_viewport_image, BrpClient, BrpTarget, DEFAULT_PORT};
use grok_bevy_env::{
    check_readiness, format_report_text, DoctorOptions, SystemCommandRunner,
};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "grok-bevy",
    version,
    about = "Grok-Bevy: environment checks, scaffolding, and MCP/BRP control for Bevy",
    long_about = "Companion tooling for AI coding agents (Grok Build and other MCP clients)\n\
                  to prepare a machine for Bevy and control running Bevy apps via BRP.\n\
                  Full-featured MCP control integrates with bevy_brp_mcp; this binary also\n\
                  ships a focused MCP server and thin BRP client."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check whether this machine can build and run Bevy apps.
    #[command(alias = "env-check", alias = "check")]
    Doctor {
        /// Also create and compile a minimal Bevy probe project (slow).
        #[arg(long)]
        compile_probe: bool,
        /// Emit JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
        /// Bevy version for the compile probe.
        #[arg(long, default_value = "0.19")]
        bevy_version: String,
    },
    /// Run the Grok-Bevy MCP server on stdio (for Grok Build / MCP clients).
    Mcp {
        /// Instead of the Grok-Bevy server, exec `bevy_brp_mcp` (full BRP tool surface).
        #[arg(long)]
        delegate_brp_mcp: bool,
    },
    /// Scaffold a BRP-enabled Bevy sample app.
    Scaffold {
        /// Destination directory.
        #[arg(long, default_value = "sample-bevy-app")]
        path: PathBuf,
        /// Overwrite if the directory exists.
        #[arg(long)]
        force: bool,
    },
    /// Call a BRP JSON-RPC method on a running Bevy app.
    Brp {
        #[command(subcommand)]
        cmd: BrpCommands,
    },
    /// Print compatibility matrix and install tips for bevy_brp_mcp.
    Compat,
    /// Print Grok Build MCP registration snippets.
    McpConfig {
        /// Path to the grok-bevy binary to embed (default: this executable).
        #[arg(long)]
        bin: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum BrpCommands {
    /// Discover available BRP methods (`rpc.discover`).
    Discover {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        #[arg(long)]
        host: Option<String>,
    },
    /// Query entities for components.
    Query {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        /// Fully-qualified component type paths (repeatable).
        #[arg(long = "component", default_values_t = vec![
            "bevy_transform::components::transform::Transform".to_string()
        ])]
        components: Vec<String>,
    },
    /// Mutate a component field on an entity.
    Mutate {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        #[arg(long)]
        entity: u64,
        #[arg(long)]
        component: String,
        #[arg(long)]
        path: String,
        /// JSON value to write.
        #[arg(long)]
        value: String,
    },
    /// Generic BRP method call.
    Call {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        method: String,
        /// Optional JSON params object/array.
        #[arg(long)]
        params: Option<String>,
    },
    /// Capture a screenshot via brp_extras/screenshot and print path + metadata.
    Screenshot {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        #[arg(long, default_value = "captures/scene.png")]
        path: PathBuf,
        /// Also print base64 length (for MCP image path validation).
        #[arg(long)]
        with_image_meta: bool,
    },
    /// Wait until BRP responds on the port.
    Wait {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        #[arg(long, default_value_t = 30)]
        timeout_secs: u64,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Doctor {
            compile_probe,
            json,
            bevy_version,
        } => cmd_doctor(compile_probe, json, bevy_version),
        Commands::Mcp { delegate_brp_mcp } => {
            if delegate_brp_mcp {
                delegate_to_bevy_brp_mcp()
            } else {
                // MCP must own stdin/stdout; run async server.
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                rt.block_on(mcp::run_stdio_server())
            }
        }
        Commands::Scaffold { path, force } => scaffold::scaffold_sample_app(&path, force),
        Commands::Brp { cmd } => cmd_brp(cmd),
        Commands::Compat => {
            print_compat();
            Ok(())
        }
        Commands::McpConfig { bin } => {
            print_mcp_config(bin)?;
            Ok(())
        }
    }
}

fn cmd_doctor(compile_probe: bool, json: bool, bevy_version: String) -> Result<()> {
    let options = DoctorOptions {
        compile_probe,
        bevy_version,
        ..DoctorOptions::default()
    };
    let runner = SystemCommandRunner;
    let report = check_readiness(&runner, &options);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", format_report_text(&report));
    }
    if report.ready {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn cmd_brp(cmd: BrpCommands) -> Result<()> {
    match cmd {
        BrpCommands::Discover { port, host } => {
            let mut target = BrpTarget::new("default", port);
            if let Some(h) = host {
                target.host = h;
            }
            let client = BrpClient::new(target);
            let result = client.call("rpc.discover", None)?.into_result()?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        BrpCommands::Query { port, components } => {
            let client = BrpClient::with_port(port);
            let comps: Vec<&str> = components.iter().map(|s| s.as_str()).collect();
            let result = client.query(&comps)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        BrpCommands::Mutate {
            port,
            entity,
            component,
            path,
            value,
        } => {
            let client = BrpClient::with_port(port);
            let v: serde_json::Value = serde_json::from_str(&value).context("parse --value JSON")?;
            let result = client.mutate_components(entity, &component, &path, v)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        BrpCommands::Call {
            port,
            method,
            params,
        } => {
            let client = BrpClient::with_port(port);
            let p = match params {
                Some(s) => Some(serde_json::from_str(&s).context("parse --params JSON")?),
                None => None,
            };
            let result = client.call(&method, p)?.into_result()?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        BrpCommands::Screenshot {
            port,
            path,
            with_image_meta,
        } => {
            let client = BrpClient::with_port(port);
            if with_image_meta {
                let img = capture_viewport_image(&client, &path)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "path": img.path,
                        "byte_len": img.byte_len,
                        "mime_type": img.mime_type,
                        "width": img.width_hint,
                        "height": img.height_hint,
                        "base64_len": img.png_base64.len(),
                    }))?
                );
            } else {
                let published = client.screenshot(&path)?;
                println!("{}", published.display());
            }
        }
        BrpCommands::Wait { port, timeout_secs } => {
            let client = BrpClient::with_port(port);
            let result = client.wait_until_ready(Duration::from_secs(timeout_secs))?;
            println!(
                "BRP ready on port {port}: {}",
                serde_json::to_string(&result)?
                    .chars()
                    .take(200)
                    .collect::<String>()
            );
        }
    }
    Ok(())
}

fn delegate_to_bevy_brp_mcp() -> Result<()> {
    let bin = which::which("bevy_brp_mcp").context(
        "bevy_brp_mcp not found on PATH. Install with:\n  cargo install bevy_brp_mcp --locked\n\
         Or run without --delegate-brp-mcp to use the built-in Grok-Bevy MCP server.",
    )?;
    tracing::info!("delegating MCP stdio to {}", bin.display());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(&bin)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .exec();
        bail!("failed to exec {}: {err}", bin.display());
    }

    #[cfg(not(unix))]
    {
        let status = Command::new(&bin)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("spawn {}", bin.display()))?;
        if status.success() {
            Ok(())
        } else {
            bail!("bevy_brp_mcp exited with {status}");
        }
    }
}

fn print_compat() {
    use grok_bevy_env::compat;
    println!("Grok-Bevy compatibility matrix");
    println!("==============================");
    println!("Bevy              : {}", compat::BEVY);
    println!("bevy_brp_mcp      : {}", compat::BEVY_BRP_MCP);
    println!("bevy_brp_extras   : {}", compat::BEVY_BRP_EXTRAS);
    println!("Default BRP port  : {}", compat::DEFAULT_BRP_PORT);
    println!();
    println!("Install full BRP MCP (recommended alongside grok-bevy):");
    println!("  cargo install bevy_brp_mcp --locked");
    println!();
    println!("In your Bevy app Cargo.toml:");
    println!("  bevy = {{ version = \"{}\", features = [\"bevy_remote\", \"png\"] }}", compat::BEVY);
    println!("  bevy_brp_extras = \"{}\"", compat::BEVY_BRP_EXTRAS);
}

fn print_mcp_config(bin: Option<PathBuf>) -> Result<()> {
    let exe = bin.unwrap_or(std::env::current_exe()?);
    let exe_str = exe.display().to_string();
    println!("# Grok Build — ~/.grok/config.toml (or project .grok/config.toml)\n");
    println!("[mcp_servers.grok-bevy]");
    println!("command = {exe_str:?}");
    println!("args = [\"mcp\"]");
    println!("enabled = true");
    println!("startup_timeout_sec = 30");
    println!();
    println!("# Optional: full bevy_brp_mcp tool surface (after cargo install bevy_brp_mcp)");
    println!("[mcp_servers.bevy-brp]");
    println!("command = \"bevy_brp_mcp\"");
    println!("args = []");
    println!("enabled = true");
    println!();
    println!("# CLI equivalent:");
    println!("#   grok mcp add grok-bevy -- {exe_str} mcp");
    println!("#   grok mcp add bevy-brp -- bevy_brp_mcp");
    Ok(())
}

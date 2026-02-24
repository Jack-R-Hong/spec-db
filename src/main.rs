use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use rmcp::ServiceExt;
use spec_db_causal::FjallStore;
use spec_db_core::{SpecDbConfig, load_config};
use spec_db_ingest::{GitSync, StorePaths};
use spec_db_mcp::SpecDbMcpServer;

mod telemetry;

const HELLO_WORLD_SPEC: &str = r#"---
id: "spec::example::hello-world"
title: "Hello World"
version: 1
tags: ["example"]
depends_on: []
created: "2026-01-01"
---
# Hello World

Welcome to lattice! This is an example specification.
"#;

const GETTING_STARTED_SPEC: &str = r#"---
id: "spec::example::getting-started"
title: "Getting Started"
version: 1
tags: ["example", "guide"]
depends_on: ["spec::example::hello-world"]
owner: "team"
created: "2026-01-01"
---
# Getting Started

This spec depends on the hello-world spec, demonstrating the `depends_on` relationship.
"#;

#[derive(Parser)]
#[command(name = "lattice", about = "A causal specification database for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Serve,
    Sync {
        #[arg(long)]
        full: bool,
    },
    Rebuild,
    Status,
    Edge {
        #[command(subcommand)]
        action: EdgeAction,
    },
}

#[derive(Subcommand)]
enum EdgeAction {
    Promote {
        source: String,
        target: String,
        #[arg(long, default_value = "depends_on")]
        r#type: String,
    },
    Reject {
        source: String,
        target: String,
        #[arg(long, default_value = "depends_on")]
        r#type: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run_command(cli.command).await
}

async fn run_command(command: Commands) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    match command {
        Commands::Init => run_init(&cwd),
        Commands::Serve => {
            let cfg = load_project_config(&cwd)?;
            telemetry::init_observability(&cfg.telemetry)?;
            run_serve(&cwd, &cfg).await
        }
        Commands::Sync { full } => {
            let cfg = load_project_config(&cwd)?;
            run_sync(&cwd, &cfg, full)
        }
        Commands::Rebuild => {
            let cfg = load_project_config(&cwd)?;
            run_sync(&cwd, &cfg, true)
        }
        Commands::Status => {
            let cfg = load_project_config(&cwd)?;
            run_status(&cwd, &cfg)
        }
        Commands::Edge { action } => {
            let cfg = load_project_config(&cwd)?;
            run_edge_action(&cwd, &cfg, action)
        }
    }
}

fn run_init(cwd: &Path) -> anyhow::Result<()> {
    let config_dir = cwd.join(".lattice");
    let config_path = config_dir.join("config.yaml");

    if config_path.exists() {
        println!("Warning: .lattice/config.yaml already exists. Skipping initialization.");
        return Ok(());
    }

    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(cwd.join("specs/example"))?;
    std::fs::create_dir_all(cwd.join("data/tantivy"))?;
    std::fs::create_dir_all(cwd.join("data/fjall"))?;

    let config = spec_db_core::SpecDbConfig::default();
    let yaml = serde_yml::to_string(&config)?;
    std::fs::write(&config_path, yaml)?;

    std::fs::write(cwd.join("specs/example/hello-world.md"), HELLO_WORLD_SPEC)?;
    std::fs::write(cwd.join("specs/example/getting-started.md"), GETTING_STARTED_SPEC)?;

    let gitignore_path = cwd.join(".gitignore");
    let needs_entry = if gitignore_path.exists() {
        let content = std::fs::read_to_string(&gitignore_path)?;
        !content.lines().any(|line| line.trim() == "data/" || line.trim() == "data")
    } else {
        true
    };
    if needs_entry {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&gitignore_path)?;
        writeln!(f)?;
        writeln!(f, "# Lattice runtime data (indexes, databases)")?;
        writeln!(f, "data/")?;
    }

    println!("Initialized lattice project:");
    println!("  specs/example/hello-world.md");
    println!("  specs/example/getting-started.md");
    println!("  .lattice/config.yaml");
    println!();
    println!("Next steps:");
    println!("  lattice sync    - Build search index and causal graph");
    println!("  lattice serve   - Start MCP server");
    println!("  lattice status  - Check project status");

    Ok(())
}

fn load_project_config(cwd: &Path) -> anyhow::Result<SpecDbConfig> {
    let path = cwd.join(".lattice/config.yaml");
    load_config(&path)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("failed to load project config at {}", path.display()))
}

#[derive(Clone)]
struct AppLayout {
    repo_path: PathBuf,
    specs_root: String,
    tantivy_dir: PathBuf,
    fjall_dir: PathBuf,
}

fn app_layout(cwd: &Path, cfg: &SpecDbConfig) -> AppLayout {
    let data_dir = cwd.join(&cfg.data_dir);
    AppLayout {
        repo_path: cwd.to_path_buf(),
        specs_root: cfg.specs_dir.clone(),
        tantivy_dir: data_dir.join("tantivy"),
        fjall_dir: data_dir.join("fjall"),
    }
}

async fn run_serve(cwd: &Path, cfg: &SpecDbConfig) -> anyhow::Result<()> {
    let layout = app_layout(cwd, cfg);
    std::fs::create_dir_all(&layout.tantivy_dir)?;
    std::fs::create_dir_all(&layout.fjall_dir)?;

    let store = FjallStore::open(&layout.fjall_dir)?;
    if store.last_sync_sha()?.is_none() {
        let report = run_git_sync(&layout, true)?;
        println!("initial sync completed: {} ({} specs)", report.head_sha, report.specs_ingested);
    }

    let (consistent, stored_count, actual_count) = consistency_state(&layout)?;
    println!(
        "consistency check: {} (stored_doc_count={}, actual_nodes={})",
        if consistent { "consistent" } else { "drifted" },
        stored_count,
        actual_count
    );
    if !consistent {
        anyhow::bail!("stores are drifted; run `lattice rebuild` before serving");
    }

    if cfg.transport.http.is_some() {
        println!("http transport configuration detected but deferred; serving stdio only");
    }

    let server = SpecDbMcpServer::new(
        layout.repo_path.clone(),
        layout.specs_root.clone(),
        layout.tantivy_dir.clone(),
        layout.fjall_dir.clone(),
        cfg.ai.default_trust,
    );

    if cfg.web.enabled {
        let web_state =
            spec_db_web::state::AppState::new(layout.tantivy_dir, layout.fjall_dir, cfg.clone());

        let web_config = spec_db_web::WebConfig {
            host: cfg.web.host.clone(),
            port: cfg.web.port,
            auth_token: cfg.web.auth_token.clone(),
        };

        let router = spec_db_web::build_router(web_state, &web_config);
        let web_host = cfg.web.host.clone();
        let web_port = cfg.web.port;

        let web_handle = tokio::spawn(async move {
            if let Err(e) = spec_db_web::start_web_server(router, &web_host, web_port).await {
                tracing::error!("web server error: {e}");
            }
        });

        let service = server.serve(rmcp::transport::io::stdio()).await?;

        tokio::select! {
            result = service.waiting() => { let _ = result?; }
            _ = web_handle => {}
        }
    } else {
        let service = server.serve(rmcp::transport::io::stdio()).await?;
        let _ = service.waiting().await?;
    }

    Ok(())
}

fn run_sync(cwd: &Path, cfg: &SpecDbConfig, full: bool) -> anyhow::Result<()> {
    let layout = app_layout(cwd, cfg);
    let report = run_git_sync(&layout, full)?;
    println!("status: ok");
    println!("message: sync completed");
    println!(
        "details: mode={}, specs_ingested={}, head_sha={}",
        if full { "full" } else { "incremental" },
        report.specs_ingested,
        report.head_sha
    );
    Ok(())
}

fn run_status(cwd: &Path, cfg: &SpecDbConfig) -> anyhow::Result<()> {
    let layout = app_layout(cwd, cfg);
    let store = FjallStore::open(&layout.fjall_dir)?;
    let actual_nodes = store.iter_nodes()?.len();
    let stored_doc_count = store.doc_count()?.unwrap_or(actual_nodes);
    let last_sync_sha = store.last_sync_sha()?.unwrap_or_else(|| "unknown".to_owned());
    let consistency = if stored_doc_count == actual_nodes { "consistent" } else { "drifted" };

    println!("doc_count: {actual_nodes}");
    println!("last_sync_sha: {last_sync_sha}");
    println!("consistency: {consistency}");
    Ok(())
}

fn run_edge_action(cwd: &Path, cfg: &SpecDbConfig, action: EdgeAction) -> anyhow::Result<()> {
    let layout = app_layout(cwd, cfg);
    let handler = spec_db_mcp::ToolHandler {
        repo_path: layout.repo_path,
        specs_root: layout.specs_root,
        tantivy_dir: layout.tantivy_dir,
        fjall_dir: layout.fjall_dir,
        ai_default_trust: cfg.ai.default_trust,
    };

    let (tool_name, input) = match action {
        EdgeAction::Promote { source, target, r#type } => {
            ("promote", spec_db_mcp::EdgeActionInput { source, target, edge_type: Some(r#type) })
        }
        EdgeAction::Reject { source, target, r#type } => {
            ("reject", spec_db_mcp::EdgeActionInput { source, target, edge_type: Some(r#type) })
        }
    };

    let result = if tool_name == "promote" {
        handler.promote_edge(input)
    } else {
        handler.reject_edge(input)
    };

    match result {
        Ok(value) => {
            if let Some(msg) = value.get("message").and_then(|v| v.as_str()) {
                println!("{msg}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn run_git_sync(
    layout: &AppLayout,
    full: bool,
) -> Result<spec_db_ingest::SyncReport, anyhow::Error> {
    let sync = GitSync::new(
        layout.repo_path.clone(),
        layout.specs_root.clone(),
        StorePaths { tantivy_dir: layout.tantivy_dir.clone(), fjall_dir: layout.fjall_dir.clone() },
    );

    if full { Ok(sync.full_rebuild()?) } else { Ok(sync.incremental_sync()?) }
}

fn consistency_state(layout: &AppLayout) -> Result<(bool, usize, usize), anyhow::Error> {
    let store = FjallStore::open(&layout.fjall_dir)?;
    let actual = store.iter_nodes()?.len();
    let stored = store.doc_count()?.unwrap_or(actual);
    Ok((stored == actual, stored, actual))
}

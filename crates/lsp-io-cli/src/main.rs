use anyhow::Result;
use clap::{Parser, Subcommand};
use lsp_io_core::config::ProjectConfig;
use lsp_io_core::language::{ScanProfile, scan_languages_with_profile};
use lsp_io_core::progress::NoopProgress;
use lsp_io_core::sdl_mcp::{SdlMcpExportOptions, build_sdl_mcp_export, write_sdl_mcp_config};
use lsp_io_core::server::{
    ServerOptions, all_status_with_options, cache_dir, install_server_with_options,
    remove_server_with_options,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "lsp-io",
    version,
    about = "Manage recommended language server installs"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Detect project languages from manifests, directory markers, and source files.
    Detect {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        all_evidence: bool,
    },
    /// Export configuration for another tool.
    Export {
        #[command(subcommand)]
        target: ExportCommand,
    },
    /// Print language server installation status.
    Status,
    /// Install one managed language server by id.
    Install { id: String },
    /// Remove one app-managed language server by id.
    Remove { id: String },
    /// Print the managed server cache directory.
    CacheDir,
}

#[derive(Debug, Subcommand)]
enum ExportCommand {
    /// Export SDL-MCP semanticEnrichment.providers.lsp.servers config.
    SdlMcp {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        write_config: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        include_missing: bool,
        #[arg(long)]
        validate_launch: bool,
        #[arg(long)]
        enable_semantic_enrichment: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Some(Command::Detect { path, all_evidence }) => {
            let profile = if all_evidence {
                ScanProfile::AllEvidence
            } else {
                ScanProfile::Recommendation
            };
            let languages = scan_languages_with_profile(&path, profile)?;
            if languages.is_empty() {
                println!("No supported languages detected");
            } else {
                for language in languages {
                    println!(
                        "{}\t{}\t{}\t{}",
                        language.display_name(),
                        language.category.label(),
                        language.confidence.label(),
                        language.evidence
                    );
                }
            }
        }
        Some(Command::Export { target }) => match target {
            ExportCommand::SdlMcp {
                path,
                write_config,
                output,
                include_missing,
                validate_launch,
                enable_semantic_enrichment,
            } => {
                let config = ProjectConfig::load(&path)?;
                let options = SdlMcpExportOptions {
                    include_missing,
                    validate_launch,
                };
                let result = if let Some(config_path) = &write_config {
                    write_sdl_mcp_config(
                        &path,
                        config_path,
                        &config,
                        options,
                        enable_semantic_enrichment,
                    )?
                } else {
                    build_sdl_mcp_export(&path, &config, options)?
                };

                for diagnostic in &result.diagnostics {
                    eprintln!(
                        "{} skipped/warned: {}",
                        diagnostic.server_id, diagnostic.reason
                    );
                }

                let fragment = serde_json::to_string_pretty(&result.fragment)?;
                if let Some(output_path) = &output {
                    std::fs::write(output_path, format!("{fragment}\n"))?;
                }

                if write_config.is_none() && output.is_none() {
                    println!("{fragment}");
                } else {
                    println!(
                        "Exported {} SDL-MCP LSP server config(s)",
                        result.server_count()
                    );
                }
            }
        },
        Some(Command::Status) => {
            let (root, config) = load_config()?;
            let options = ServerOptions::from_config(&root, &config);
            for server in all_status_with_options(&options) {
                println!(
                    "{:<34} {:<22} {:<14} {}",
                    server.id, server.language_display, server.install_state, server.install_method
                );
            }
        }
        Some(Command::Install { id }) => {
            let (root, config) = load_config()?;
            let options = ServerOptions::from_config(&root, &config);
            let outcome = install_server_with_options(&id, &options, &NoopProgress).await?;
            println!("{}", outcome.message);
            if let Some(path) = outcome.path {
                println!("{}", path);
            }
        }
        Some(Command::Remove { id }) => {
            let (root, config) = load_config()?;
            let options = ServerOptions::from_config(&root, &config);
            let outcome = remove_server_with_options(&id, &options, &NoopProgress)?;
            println!("{}", outcome.message);
        }
        Some(Command::CacheDir) => {
            let (root, config) = load_config()?;
            if config.cache_dir.is_some() {
                let options = ServerOptions::from_config(&root, &config);
                println!("{}", options.cache_dir.display());
            } else {
                println!("{}", cache_dir().display());
            }
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}

fn load_config() -> Result<(PathBuf, ProjectConfig)> {
    let root = std::env::current_dir()?;
    let config = ProjectConfig::load(&root)?;
    Ok((root, config))
}

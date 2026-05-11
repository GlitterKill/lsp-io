use anyhow::Result;
use clap::{Parser, Subcommand};
use lsp_io_core::config::ProjectConfig;
use lsp_io_core::language::scan_languages;
use lsp_io_core::progress::NoopProgress;
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
        Some(Command::Detect { path }) => {
            let languages = scan_languages(&path)?;
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

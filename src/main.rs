//! A basic example of a preprocessor that does nothing.

use anyhow::Result;
use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_blox::BloxProcessor;
use mdbook_blox::config::Config;
use semver::{Version, VersionReq};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

/// mdbook preprocessor to add support for admonition-like blocks
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check whether a renderer is supported by this preprocessor
    Supports { renderer: String },
    /// Generate css
    Css {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        log::error!("Fatal error: {}", error);
        for error in error.chain() {
            log::error!("  - {}", error);
        }
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        None => handle_preprocessing(),
        Some(Commands::Supports { renderer }) => {
            handle_supports(renderer);
        }
        Some(Commands::Css { dir }) => handle_css(dir.unwrap_or_else(|| PathBuf::from("."))),
    }
}

fn handle_preprocessing() -> Result<()> {
    log::debug!("Start preprocessing blox");
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook,\
             but we're being called from version {}",
            mdbook_blox::PREPROCESSOR_NAME,
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = BloxProcessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: String) -> ! {
    if BloxProcessor.supports_renderer(&renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn handle_css(dir: PathBuf) -> anyhow::Result<()> {
    let book_toml = dir.join("book.toml");
    log::info!("Reading configuration file '{}'", book_toml.display());

    let config = Config::from_file(&book_toml)?;
    let css = mdbook_blox::css::css_from_config(&config)?;

    let output = dir.join(config.css);
    log::info!("Writing custom CSS file '{}'", output.display());
    fs::write(output, css)?;

    Ok(())
}

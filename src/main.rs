//! A preprocessor that handles blox

use anyhow::Result;
use clap::{Parser, Subcommand};
use mdbook_blox::{BloxPreProcessor, Config};
use mdbook_preprocessor::{MDBOOK_VERSION, Preprocessor, parse_input};
use semver::{Version, VersionReq};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;
use tracing::debug_span;
use tracing::{debug, info};

/// mdbook preprocessor to add support for admonition-like blocks
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
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
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_env_var("MDBOOK_LOG")
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        tracing::error!("Fatal error: {}", error);
        for error in error.chain() {
            tracing::error!("  - {}", error);
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
    let (ctx, book) = parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook,\
             but we're being called from version {}",
            BloxPreProcessor.name(),
            MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = BloxPreProcessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: String) -> ! {
    let supported = BloxPreProcessor
        .supports_renderer(&renderer)
        .unwrap_or(false);

    if supported {
        debug!("blox supports {}", &renderer);
        process::exit(0);
    } else {
        info!("blox does not support {}", &renderer);
        process::exit(1);
    }
}

fn handle_css(dir: PathBuf) -> anyhow::Result<()> {
    let book_toml = dir.join("book.toml");

    let span = debug_span!("config").entered();
    let config = Config::from_file(&book_toml)?;
    span.exit();

    let span = debug_span!("writing css").entered();
    let css = config.css().css_string(&config);
    let output = dir.join(config.css().file());
    debug!("Writing CSS file to '{}'", output.display());
    fs::write(output, css)?;
    span.exit();

    Ok(())
}

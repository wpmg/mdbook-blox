//! A basic example of a preprocessor that does nothing.

use clap::{Parser, Subcommand};
use mdbook::errors::Result;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_blox::Nop;
use semver::{Version, VersionReq};
use std::io;
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
    }
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            mdbook_blox::PREPROCESSOR_NAME,
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = Nop.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: String) -> ! {
    if Nop.supports_renderer(&renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

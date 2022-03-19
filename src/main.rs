use anyhow::Result;
use build::watch_build;
use clap::StructOpt;
use serve::run_serve;

mod build;
mod code_blocks;
mod data;
mod engine;
mod entity;
mod feed;
mod helpers;
mod serve;

pub use self::engine::Render;
pub use self::engine::ZineEngine;
pub use self::entity::Entity;

pub static ZINE_FILE: &str = "zine.toml";

/// The temporal build dir, mainly for `zine serve` command.
pub static TEMP_ZINE_BUILD_DIR: &str = "__zine_build";

#[derive(Debug, clap::Parser)]
#[clap(name = "zine")]
#[clap(about = "A simple and opinionated tool to build your own magazine.", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Build zine site.
    #[clap(arg_required_else_help = true)]
    Build {
        /// The source directory of zine site.
        source: String,
        /// The destination directory. Default dest dir is `build`.
        dest: Option<String>,
        /// Enable watching.
        #[clap(short, long)]
        watch: bool,
    },
    /// Serve the zine site.
    #[clap(arg_required_else_help = true)]
    Serve {
        /// The source directory of zine site.
        source: String,
        /// The listen port.
        #[clap(short, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            source,
            dest,
            watch,
        } => {
            let dest = dest.unwrap_or_else(|| "build".into());
            watch_build(&source, &dest, watch).await?;
        }
        Commands::Serve { source, port } => {
            run_serve(source, port).await?;
        }
    }

    Ok(())
}

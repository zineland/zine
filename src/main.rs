use anyhow::Result;
use build::watch_build;
use clap::StructOpt;
use new::new_zine_project;
use serve::run_serve;

mod build;
mod code_blocks;
mod data;
mod engine;
mod entity;
mod feed;
mod helpers;
mod new;
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
    /// Build Zine site.
    #[clap(arg_required_else_help = true)]
    Build {
        /// The source directory of zine site.
        source: Option<String>,
        /// The destination directory. Default dest dir is `build`.
        dest: Option<String>,
        /// Enable watching.
        #[clap(short, long)]
        watch: bool,
    },
    /// Serve the Zine site.
    #[clap(arg_required_else_help = true)]
    Serve {
        /// The source directory of zine site.
        source: Option<String>,
        /// The listen port.
        #[clap(short, default_value_t = 3000)]
        port: u16,
    },
    /// New a Zine project.
    New {
        /// The project name.
        name: Option<String>,
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
            watch_build(&source.unwrap_or_else(|| ".".into()), &dest, watch).await?;
            println!("Build success! The build directory is `{}`.", dest);
        }
        Commands::Serve { source, port } => {
            run_serve(source.unwrap_or_else(|| ".".into()), port).await?;
        }
        Commands::New { name } => new_zine_project(name)?,
    }

    Ok(())
}

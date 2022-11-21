use anyhow::Result;
use build::watch_build;
use clap::{Parser, Subcommand};
use new::new_zine_project;
use parking_lot::RwLock;
use serve::run_serve;

mod build;
mod code_blocks;
mod data;
mod engine;
mod entity;
mod error;
mod feed;
mod helpers;
mod html;
mod i18n;
mod locales;
mod markdown;
mod new;
mod serve;

pub use self::engine::ZineEngine;
pub use self::entity::Entity;

pub static ZINE_FILE: &str = "zine.toml";
pub static ZINE_BANNER: &str = r"

███████╗██╗███╗   ██╗███████╗
╚══███╔╝██║████╗  ██║██╔════╝
  ███╔╝ ██║██╔██╗ ██║█████╗  
 ███╔╝  ██║██║╚██╗██║██╔══╝  
███████╗██║██║ ╚████║███████╗
╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝
                             
";

static MODE: RwLock<Mode> = parking_lot::const_rwlock(Mode::Unknown);

#[derive(Copy, Clone)]
pub enum Mode {
    Build,
    Serve,
    Unknown,
}

/// Get current run mode.
pub fn current_mode() -> Mode {
    *MODE.read()
}

fn set_current_mode(mode: Mode) {
    *MODE.write() = mode;
}

#[derive(Debug, Parser)]
#[command(name = "zine")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Build Zine site.
    Build {
        /// The source directory of zine site.
        source: Option<String>,
        /// The destination directory. Default dest dir is `build`.
        dest: Option<String>,
        /// Enable watching.
        #[arg(short, long)]
        watch: bool,
    },
    /// Serve the Zine site.
    Serve {
        /// The source directory of zine site.
        source: Option<String>,
        /// The listen port.
        #[arg(short, default_value_t = 3000)]
        port: u16,
    },
    /// New a Zine project.
    New {
        /// The project name.
        name: Option<String>,
    },
    /// Prints the app version.
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            source,
            dest,
            watch,
        } => {
            set_current_mode(Mode::Build);
            let dest = dest.unwrap_or_else(|| "build".into());
            watch_build(&source.unwrap_or_else(|| ".".into()), &dest, watch).await?;
            println!("Build success! The build directory is `{}`.", dest);
        }
        Commands::Serve { source, port } => {
            set_current_mode(Mode::Serve);
            run_serve(source.unwrap_or_else(|| ".".into()), port).await?;
        }
        Commands::New { name } => new_zine_project(name)?,
        Commands::Version => {
            let version =
                option_env!("CARGO_PKG_VERSION").unwrap_or("(Unknown Cargo package version)");
            let date = option_env!("LAST_COMMIT_DATE").unwrap_or("");
            let build_info = env!("BUILD_INFO");
            println!("{}", ZINE_BANNER);
            println!("Zine version {} {}", version, date);
            println!("({})", build_info);
        }
    }
    Ok(())
}

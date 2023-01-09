use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use build::watch_build;
use clap::{Parser, Subcommand};
use entity::Zine;
use error::ZineError;
use new::{new_zine_issue, new_zine_project};
use parking_lot::RwLock;
use serve::run_serve;
use walkdir::WalkDir;

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
mod lint;
mod locales;
mod markdown;
mod new;
mod serve;

pub use self::engine::ZineEngine;
pub use self::entity::Entity;

/// The convention name of zine config file.
pub static ZINE_FILE: &str = "zine.toml";
/// The convention name of zine markdown directory.
pub static ZINE_CONTENT_DIR: &str = "content";
/// The convention name of introduction file for zine issue.
pub static ZINE_INTRO_FILE: &str = "intro.md";
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
        /// New issue.
        #[arg(short)]
        issue: bool,
    },
    /// Lint Zine project.
    Lint {
        /// The source directory of zine site.
        source: Option<String>,
        /// Enable CI mode. If lint failed will reture a non-zero code.
        #[arg(long)]
        ci: bool,
    },
    /// Prints the app version.
    Version,
}

/// Find the root zine file in current dir and try to parse it
fn parse_root_zine_file<P: AsRef<Path>>(path: P) -> Result<Option<Zine>> {
    // Find the name in current dir
    if WalkDir::new(&path).max_depth(1).into_iter().any(|entry| {
        let entry = entry.as_ref().unwrap();
        entry.file_name() == crate::ZINE_FILE
    }) {
        // Try to parse the root zine.toml as Zine instance
        return Ok(Some(Zine::parse_from_toml(path)?));
    }

    Ok(None)
}

/// Locate folder contains the root `zine.toml`, and return path info and Zine instance.
pub fn locate_root_zine_folder<P: AsRef<Path>>(path: P) -> Result<Option<(PathBuf, Zine)>> {
    match parse_root_zine_file(&path) {
        Ok(Some(zine)) => return Ok(Some((path.as_ref().to_path_buf(), zine))),
        Err(err) => match err.downcast::<ZineError>() {
            // Found a root zine.toml, but it has invalid format
            Ok(inner_err @ ZineError::InvalidRootTomlFile(_)) => return Err(anyhow!(inner_err)),
            // Found a zine.toml, but it isn't a root zine.toml
            Ok(ZineError::NotRootTomlFile) => {}
            // No zine.toml file found
            _ => {}
        },
        _ => {}
    }

    match path.as_ref().parent() {
        Some(parent_path) => locate_root_zine_folder(parent_path),
        None => Ok(None),
    }
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
            watch_build(&source.unwrap_or_else(|| ".".into()), &dest, watch, None).await?;
            println!("Build success! The build directory is `{}`.", dest);
        }
        Commands::Serve { source, port } => {
            set_current_mode(Mode::Serve);
            run_serve(source.unwrap_or_else(|| ".".into()), port).await?;
        }
        Commands::New { name, issue } => {
            if issue {
                new_zine_issue()?;
            } else {
                new_zine_project(name)?
            }
        }
        Commands::Lint { source, ci } => {
            let success = lint::lint_zine_project(source.unwrap_or_else(|| ".".into())).await?;
            if ci && !success {
                std::process::exit(1);
            }
        }
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

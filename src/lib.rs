use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use entity::Zine;
use error::ZineError;
use parking_lot::RwLock;
use walkdir::WalkDir;

pub mod build;
mod code_blocks;
mod data;
mod engine;
mod entity;
mod error;
mod feed;
pub mod helpers;
mod html;
mod i18n;
pub mod lint;
mod locales;
mod markdown;
pub mod new;
pub mod serve;

pub use self::engine::ZineEngine;
pub use self::entity::Entity;
pub use self::entity::{SiteBuilder, Site, Issue, MetaArticle, Article};
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

pub fn set_current_mode(mode: Mode) {
    *MODE.write() = mode;
}

// Find the root zine file in current dir and try to parse it
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

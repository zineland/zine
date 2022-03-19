mod code_blocks;
pub mod data;
mod engine;
mod entity;
mod feed;
mod helpers;

pub use self::engine::Render;
pub use self::engine::ZineEngine;
pub use self::entity::Entity;

pub(crate) static ZINE_FILE: &str = "zine.toml";

/// The temporal build dir, mainly for `zine serve` command.
pub static TEMP_ZINE_BUILD_DIR: &str = "__zine_build";

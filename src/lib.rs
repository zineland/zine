mod entity;
mod zine;

pub use self::entity::Entity;
pub use self::zine::Render;
pub use self::zine::ZineEngine;

pub(crate) static ZINE_FILE: &str = "zine.toml";

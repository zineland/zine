mod entity;
mod zine;

pub use entity::Entity;
pub use zine::Render;
pub use zine::ZineEngine;

pub(crate) static ZINE_FILE: &str = "zine.toml";

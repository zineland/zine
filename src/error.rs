use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZineError {
    #[error("Invalid format of root `zine.toml`: {0}")]
    InvalidRootTomlFile(#[from] toml::de::Error),
    #[error("Not a root `zine.toml`, maybe it a `zine.toml` for issue?")]
    NotRootTomlFile,
}

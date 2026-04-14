use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("pf error: {0}")]
    Pf(#[from] pfctl::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml (de) error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("toml (ser) error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("invalid rule: {0}")]
    InvalidRule(String),

    #[error("rule #{0} not found")]
    RuleNotFound(u32),

    #[error("pf not enabled — run `mufw enable` first")]
    NotEnabled,

    #[error("permission denied — mufw must run as root (try `sudo`)")]
    PermissionDenied,
}

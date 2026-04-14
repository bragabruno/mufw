//! mufw-core — ufw-like rule model and pf translation for macOS.
//!
//! This crate exposes a small, serializable rule model (`Rule`) and a store
//! backed by a TOML file on disk. Rules are translated to `pfctl` filter rules
//! and applied to a dedicated anchor (`mufw`) so we never clobber Apple's own
//! pf configuration.
//!
//! The CLI lives in the `mufw-cli` crate; this crate is intentionally free of
//! I/O beyond the rules file so it can be reused by GUIs, TUIs, or tests.

pub mod anchor;
pub mod error;
pub mod pf;
pub mod rule;
pub mod store;

pub use error::{Error, Result};
pub use rule::{Action, PortSpec, Proto, Rule, Target};
pub use store::Store;

/// Default anchor name used by mufw.
pub const ANCHOR: &str = "mufw";

/// Default on-disk rules path (root-owned, mode 0600).
pub const RULES_PATH: &str = "/etc/mufw/rules.toml";

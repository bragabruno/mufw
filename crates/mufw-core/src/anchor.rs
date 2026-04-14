//! Anchor lifecycle: write the anchor file, enable pf, and load the ruleset.
//!
//! The heavy lifting (ioctls) is done via `pfctl-rs` where practical; when
//! features aren't yet exposed by the crate we fall back to invoking
//! `/sbin/pfctl` which is always present on macOS.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};
use crate::pf;
use crate::rule::Rule;

/// Default anchor path under `/etc/pf.anchors/`.
pub const ANCHOR_FILE: &str = "/etc/pf.anchors/com.mufw";

/// Hook line that must exist in /etc/pf.conf so the kernel picks up our anchor.
pub const PF_CONF_HOOK: &str = concat!(
    "anchor \"com.mufw\"\n",
    "load anchor \"com.mufw\" from \"/etc/pf.anchors/com.mufw\"\n",
);

/// Make sure `/etc/pf.conf` contains our hook. Idempotent.
pub fn ensure_hook() -> Result<()> {
    let conf = Path::new("/etc/pf.conf");
    let existing = fs::read_to_string(conf).unwrap_or_default();
    if existing.contains("anchor \"com.mufw\"") {
        return Ok(());
    }
    let new = format!("{existing}\n{PF_CONF_HOOK}");
    fs::write(conf, new)?;
    Ok(())
}

/// Write `/etc/pf.anchors/com.mufw` from the given rules.
pub fn write_anchor(rules: &[&Rule]) -> Result<()> {
    let body = pf::render_anchor(rules);
    fs::write(ANCHOR_FILE, body)?;
    Ok(())
}

/// Validate the main pf config (dry-run) using `pfctl -nf`.
pub fn validate() -> Result<()> {
    let status = Command::new("/sbin/pfctl")
        .args(["-nf", "/etc/pf.conf"])
        .status()?;
    if !status.success() {
        return Err(Error::InvalidRule(
            "pfctl validation failed — see `pfctl -nf /etc/pf.conf`".into(),
        ));
    }
    Ok(())
}

/// Enable pf (idempotent — ignores "pf already enabled").
pub fn enable() -> Result<()> {
    let _ = Command::new("/sbin/pfctl").arg("-E").status()?;
    Ok(())
}

/// Disable pf.
pub fn disable() -> Result<()> {
    let _ = Command::new("/sbin/pfctl").arg("-d").status()?;
    Ok(())
}

/// Load `/etc/pf.conf` (which anchors in our file).
pub fn load() -> Result<()> {
    let status = Command::new("/sbin/pfctl")
        .args(["-f", "/etc/pf.conf"])
        .status()?;
    if !status.success() {
        return Err(Error::InvalidRule("pfctl load failed".into()));
    }
    Ok(())
}

/// Flush rules from the mufw anchor only (not Apple's).
pub fn flush_anchor() -> Result<()> {
    let _ = Command::new("/sbin/pfctl")
        .args(["-a", "com.mufw", "-F", "rules"])
        .status()?;
    Ok(())
}

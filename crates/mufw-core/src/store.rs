use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::rule::{Action, Rule};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub next_id: u32,
    /// Path rules are persisted to (not serialized).
    #[serde(skip)]
    path: PathBuf,
}

impl Store {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self { path, ..Default::default() });
        }
        let raw = fs::read_to_string(&path)?;
        let mut s: Self = toml::from_str(&raw)?;
        s.path = path;
        Ok(s)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn add(&mut self, mut rule: Rule) -> Result<u32> {
        rule.validate()?;
        self.next_id = self.next_id.max(1);
        rule.id = self.next_id;
        self.next_id += 1;
        self.rules.push(rule);
        self.save()?;
        Ok(self.rules.last().unwrap().id)
    }

    pub fn delete(&mut self, id: u32) -> Result<()> {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        if self.rules.len() == before { return Err(Error::RuleNotFound(id)); }
        self.save()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.rules.clear();
        self.next_id = 1;
        self.save()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(|r| r.enabled)
    }

    /// Return rules ordered for pf application: denies before allows, then limits last.
    pub fn ordered(&self) -> Vec<&Rule> {
        let mut out: Vec<&Rule> = self.iter().collect();
        out.sort_by_key(|r| match r.action {
            Action::Deny => 0,
            Action::Allow => 1,
            Action::Limit => 2,
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Proto, PortSpec};
    use tempfile::tempdir;

    #[test]
    fn roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rules.toml");
        let mut s = Store::load(&path).unwrap();
        let id = s.add(Rule {
            id: 0, action: Action::Allow, proto: Proto::Tcp,
            src: None, dst: None, port: Some(PortSpec::single(22)),
            interface: None, comment: Some("ssh".into()), enabled: true,
        }).unwrap();
        assert_eq!(id, 1);
        let loaded = Store::load(&path).unwrap();
        assert_eq!(loaded.rules.len(), 1);
        assert_eq!(loaded.rules[0].id, 1);
    }
}

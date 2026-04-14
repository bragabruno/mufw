use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Deny,
    /// Rate-limit new connections (pf `max-src-conn-rate` + overload table).
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Proto {
    Tcp,
    Udp,
    Any,
}

impl Proto {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "tcp" => Ok(Self::Tcp),
            "udp" => Ok(Self::Udp),
            "" | "any" => Ok(Self::Any),
            other => Err(Error::InvalidRule(format!("unknown proto `{other}`"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Any,
    Cidr(ipnet::IpNet),
    Host(IpAddr),
}

impl Target {
    pub fn parse(s: &str) -> Result<Self> {
        if s.eq_ignore_ascii_case("any") {
            return Ok(Self::Any);
        }
        if let Ok(net) = s.parse::<ipnet::IpNet>() {
            return Ok(Self::Cidr(net));
        }
        if let Ok(ip) = s.parse::<IpAddr>() {
            return Ok(Self::Host(ip));
        }
        Err(Error::InvalidRule(format!("invalid target `{s}`")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortSpec {
    pub from: u16,
    pub to: u16,
}

impl PortSpec {
    pub fn single(p: u16) -> Self { Self { from: p, to: p } }

    pub fn parse(s: &str) -> Result<Self> {
        if let Some((a, b)) = s.split_once(':') {
            let from: u16 = a.parse().map_err(|_| Error::InvalidRule(format!("bad port `{a}`")))?;
            let to: u16 = b.parse().map_err(|_| Error::InvalidRule(format!("bad port `{b}`")))?;
            if from > to { return Err(Error::InvalidRule("port range reversed".into())); }
            Ok(Self { from, to })
        } else {
            let p: u16 = s.parse().map_err(|_| Error::InvalidRule(format!("bad port `{s}`")))?;
            Ok(Self::single(p))
        }
    }
}

impl fmt::Display for PortSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.from == self.to {
            write!(f, "{}", self.from)
        } else {
            write!(f, "{}:{}", self.from, self.to)
        }
    }
}

/// A single user-facing firewall rule. This is the canonical on-disk model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Stable ID — assigned by the store on insertion.
    pub id: u32,
    pub action: Action,
    #[serde(default = "default_proto")]
    pub proto: Proto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<Target>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst: Option<Target>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<PortSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
}

fn default_proto() -> Proto { Proto::Any }
fn yes() -> bool { true }

impl Rule {
    pub fn validate(&self) -> Result<()> {
        if matches!(self.action, Action::Limit) && self.port.is_none() {
            return Err(Error::InvalidRule("limit rules require a port".into()));
        }
        if matches!(self.proto, Proto::Any) && self.port.is_some() {
            // pf requires proto when filtering by port; we'll default to tcp when rendering.
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ports() {
        assert_eq!(PortSpec::parse("22").unwrap(), PortSpec::single(22));
        assert_eq!(PortSpec::parse("8000:9000").unwrap(), PortSpec { from: 8000, to: 9000 });
        assert!(PortSpec::parse("abc").is_err());
    }

    #[test]
    fn parses_targets() {
        assert!(matches!(Target::parse("any").unwrap(), Target::Any));
        assert!(matches!(Target::parse("10.0.0.0/8").unwrap(), Target::Cidr(_)));
        assert!(matches!(Target::parse("1.2.3.4").unwrap(), Target::Host(_)));
        assert!(Target::parse("bogus").is_err());
    }
}

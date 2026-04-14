use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use mufw_core::{
    anchor,
    rule::{Action, PortSpec, Proto, Rule, Target},
    Store, RULES_PATH,
};
use tabled::{settings::Style, Table, Tabled};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "mufw",
    version,
    about = "ufw-like command-line firewall for macOS (pf)",
    long_about = None,
)]
struct Cli {
    /// Emit JSON instead of human-readable output where applicable.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Enable pf and load mufw rules.
    Enable,
    /// Disable pf.
    Disable,
    /// Reload rules from disk.
    Reload,
    /// Remove all mufw rules.
    Reset,
    /// Show rule list / pf status.
    Status {
        /// Show numbered rules (same IDs as on disk).
        #[arg(long)]
        numbered: bool,
        /// Verbose pf info.
        #[arg(long)]
        verbose: bool,
    },
    /// Allow traffic. Examples:
    ///   mufw allow 22
    ///   mufw allow 443/tcp
    ///   mufw allow from 192.168.1.0/24 to any port 5432
    Allow(RuleArgs),
    /// Deny traffic. Mirror of `allow`.
    Deny(RuleArgs),
    /// Rate-limit inbound (brute-force throttle) on a port.
    Limit {
        /// Port (tcp assumed).
        port: u16,
    },
    /// Delete a rule by its numeric ID.
    Delete {
        /// Rule ID (see `mufw status --numbered`).
        id: u32,
    },
    /// List rules (machine-friendly, same as `status --numbered --json`).
    List,
}

#[derive(clap::Args, Debug, Default)]
struct RuleArgs {
    /// Shorthand: `<port>[/tcp|/udp]`. Mutually exclusive with `--from/--to/--port`.
    target: Option<String>,
    /// Source (IP, CIDR, or `any`).
    #[arg(long)]
    from: Option<String>,
    /// Destination (IP, CIDR, or `any`).
    #[arg(long)]
    to: Option<String>,
    /// Destination port or `from:to` range.
    #[arg(long)]
    port: Option<String>,
    /// Protocol: tcp, udp, any.
    #[arg(long, default_value = "any")]
    proto: String,
    /// Interface to bind the rule to (e.g. en0).
    #[arg(long)]
    interface: Option<String>,
    /// Free-form comment.
    #[arg(long)]
    comment: Option<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    if !is_root() && needs_root(&cli.cmd) {
        bail!("mufw must run as root — try: sudo mufw …");
    }

    let mut store = Store::load(RULES_PATH).context("loading rules store")?;

    match cli.cmd {
        Cmd::Enable => {
            anchor::ensure_hook()?;
            apply(&store)?;
            anchor::enable()?;
            println!("mufw enabled ({} rules)", store.rules.len());
        }
        Cmd::Disable => {
            anchor::disable()?;
            println!("mufw disabled");
        }
        Cmd::Reload => {
            apply(&store)?;
            println!("reloaded ({} rules)", store.rules.len());
        }
        Cmd::Reset => {
            store.reset()?;
            apply(&store)?;
            println!("all mufw rules cleared");
        }
        Cmd::Status { numbered, verbose } => {
            status(&store, numbered, verbose, cli.json)?;
        }
        Cmd::List => {
            status(&store, true, false, true)?;
        }
        Cmd::Allow(a) => {
            let rule = build_rule(Action::Allow, a)?;
            let id = store.add(rule)?;
            apply(&store)?;
            println!("added rule #{id}");
        }
        Cmd::Deny(a) => {
            let rule = build_rule(Action::Deny, a)?;
            let id = store.add(rule)?;
            apply(&store)?;
            println!("added rule #{id}");
        }
        Cmd::Limit { port } => {
            let rule = Rule {
                id: 0,
                action: Action::Limit,
                proto: Proto::Tcp,
                src: None,
                dst: None,
                port: Some(PortSpec::single(port)),
                interface: None,
                comment: Some(format!("rate-limit :{port}")),
                enabled: true,
            };
            let id = store.add(rule)?;
            apply(&store)?;
            println!("added limit rule #{id} on port {port}");
        }
        Cmd::Delete { id } => {
            store.delete(id)?;
            apply(&store)?;
            println!("deleted rule #{id}");
        }
    }
    Ok(())
}

fn needs_root(cmd: &Cmd) -> bool {
    !matches!(cmd, Cmd::Status { .. } | Cmd::List)
}

fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

fn build_rule(action: Action, a: RuleArgs) -> Result<Rule> {
    let mut proto = Proto::parse(&a.proto).map_err(anyhow::Error::from)?;
    let mut port: Option<PortSpec> = None;
    let mut src: Option<Target> = None;
    let mut dst: Option<Target> = None;

    if let Some(shorthand) = a.target {
        let (p, proto_s) = match shorthand.split_once('/') {
            Some((p, s)) => (p, Some(s.to_string())),
            None => (shorthand.as_str(), None),
        };
        port = Some(PortSpec::parse(p).map_err(anyhow::Error::from)?);
        if let Some(s) = proto_s {
            proto = Proto::parse(&s).map_err(anyhow::Error::from)?;
        }
    }
    if let Some(p) = a.port {
        port = Some(PortSpec::parse(&p).map_err(anyhow::Error::from)?);
    }
    if let Some(s) = a.from {
        src = Some(Target::parse(&s).map_err(anyhow::Error::from)?);
    }
    if let Some(s) = a.to {
        dst = Some(Target::parse(&s).map_err(anyhow::Error::from)?);
    }

    Ok(Rule {
        id: 0,
        action,
        proto,
        src,
        dst,
        port,
        interface: a.interface,
        comment: a.comment,
        enabled: true,
    })
}

fn apply(store: &Store) -> Result<()> {
    let ordered = store.ordered();
    anchor::write_anchor(&ordered)?;
    anchor::validate()?;
    anchor::load()?;
    Ok(())
}

#[derive(Tabled, serde::Serialize)]
struct Row {
    id: u32,
    action: String,
    proto: String,
    src: String,
    dst: String,
    port: String,
    comment: String,
}

fn status(store: &Store, numbered: bool, verbose: bool, json: bool) -> Result<()> {
    let rows: Vec<Row> = store
        .rules
        .iter()
        .map(|r| Row {
            id: r.id,
            action: format!("{:?}", r.action).to_lowercase(),
            proto: format!("{:?}", r.proto).to_lowercase(),
            src: r.src.as_ref().map(fmt_target).unwrap_or_else(|| "any".into()),
            dst: r.dst.as_ref().map(fmt_target).unwrap_or_else(|| "any".into()),
            port: r.port.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
            comment: r.comment.clone().unwrap_or_default(),
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("no rules configured. try: sudo mufw allow 22");
    } else if numbered {
        let mut t = Table::new(&rows);
        t.with(Style::rounded());
        println!("{t}");
    } else {
        for r in &rows {
            println!("{} {} {} {} {}", r.action, r.proto, r.src, r.port, r.comment);
        }
    }

    if verbose {
        let _ = std::process::Command::new("/sbin/pfctl").args(["-s", "info"]).status();
    }
    Ok(())
}

fn fmt_target(t: &Target) -> String {
    match t {
        Target::Any => "any".into(),
        Target::Host(ip) => ip.to_string(),
        Target::Cidr(n) => n.to_string(),
    }
}

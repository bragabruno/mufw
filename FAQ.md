# Frequently Asked Questions

Quick answers to the things people ask most. If yours isn't here, open a [discussion](https://github.com/bragdev/mufw/discussions).

## General

### What is mufw?
A small Rust CLI that gives macOS a **ufw-like experience** on top of the built-in `pf` firewall. Commands like `sudo mufw allow 22` manage a dedicated pf anchor (`com.mufw`) so your rules never collide with Apple's.

### Why not just use `pfctl`?
`pfctl` is powerful but verbose. Writing, validating, and reloading rules by hand is error-prone, and a typo in `/etc/pf.conf` can break system services (mDNS, AirDrop, Screen Sharing). mufw keeps its rules in an **isolated anchor**, so the worst case is "my custom rules don't apply" rather than "my Mac can't resolve DNS."

### Why not Little Snitch / LuLu / appFirewall?
Those are **per-application outbound** firewalls built on Apple's NetworkExtension framework. mufw is a **packet-level** firewall (layer 3/4, inbound-focused by default). They solve different problems and work well **together**.

| | mufw | LuLu / Little Snitch / appFirewall |
|---|---|---|
| Layer | L3/L4 (IP + port) | L7 (per-process) |
| Direction | Inbound by default | Outbound by default |
| CLI-first | Yes | No / limited |
| Server / headless use | Yes | Mostly GUI |
| Blocks based on | IPs, CIDRs, ports, proto | App identity, domains |

### Is mufw production-ready?
Pre-1.0. Ergonomics and the CLI surface may change between minor versions. The underlying behavior (writing an anchor, validating, loading) is conservative and reversible.

---

## Install & requirements

### Do I need Rust to use mufw?
**No**, if you install via Homebrew or a prebuilt binary from GitHub Releases. **Yes**, if you `cargo install mufw` or build from source.

### Which macOS versions are supported?
macOS 11 (Big Sur) and later. pf ships on macOS since 10.7 but the `pfctl` interface has been stable since Big Sur.

### Does it work on Apple Silicon?
Yes. Releases ship both `aarch64-apple-darwin` and `x86_64-apple-darwin`.

### Does it replace the built-in macOS Application Firewall?
No. `socketfilterfw` (System Settings → Network → Firewall) operates at the application layer and runs alongside mufw. You can use both.

### Does mufw need root?
Yes — pf ioctls require it. Anything that mutates rules or toggles pf will prompt you to rerun with `sudo`. Read-only commands (`status`, `list`) don't.

---

## Rules & behavior

### What's the default policy?
**Default deny inbound, allow outbound**, with `set skip on lo0` (loopback unfiltered). Same opinion as ufw.

### Where are rules stored?
`/etc/mufw/rules.toml` (root-owned, mode `0600`). This is the source of truth; the pf anchor at `/etc/pf.anchors/com.mufw` is regenerated from it.

### What happens to my rules after a reboot?
If you installed the LaunchDaemon (`scripts/install.sh` or Homebrew's `brew services`), pf + your rules are re-applied at boot. Without it, macOS boots pf with Apple's default anchor only.

### Does mufw touch `/etc/pf.conf`?
Once, to add a two-line load directive for our anchor:
```
anchor "com.mufw"
load anchor "com.mufw" from "/etc/pf.anchors/com.mufw"
```
That's it. Apple's anchors and existing rules are left alone.

### Does it support IPv6?
Today: IPv4 first-class; IPv6 works for `any`/host targets but CIDR support for IPv6 is listed for **v0.2** on the roadmap.

### Does it work with VPNs (WireGuard, Tailscale, OpenVPN)?
Yes — but you may need to skip the VPN interface so the firewall doesn't filter intra-tunnel traffic:
```bash
# the anchor header currently skips lo0; add utun if needed by editing
# crates/mufw-core/src/pf.rs anchor_header(), or open an issue / PR.
```
Plan for v0.2: `mufw skip add utun0` as a first-class command.

### Can I edit the anchor file by hand?
Don't — it's regenerated on every `mufw` mutation. Edit `/etc/mufw/rules.toml` directly if you need something mufw doesn't yet express, then `sudo mufw reload`.

### How is `mufw limit` different from `allow`?
`limit` applies pf's `max-src-conn-rate` with an **overload table** (`<mufw_bruteforce>`), throttling a source IP that opens too many connections too fast. Great for SSH.

---

## Safety & recovery

### What if a rule locks me out of my own machine?
Three safety nets:

1. **Validation** — mufw runs `pfctl -nf /etc/pf.conf` before loading. Syntax errors never hit the kernel.
2. **Anchor isolation** — Apple anchors always load; `com.mufw` loads last. `sudo pfctl -a com.mufw -F rules` wipes just mufw's rules.
3. **Manual override** — `sudo pfctl -d` disables pf entirely without uninstalling mufw.

A `safe-apply` mode with an auto-rollback timer is on the v0.2 roadmap.

### How do I completely remove mufw and everything it touched?
```bash
sudo ./scripts/uninstall.sh
# then, if you want, manually remove:
sudo rm -rf /etc/mufw
# and the two lines added to /etc/pf.conf
```

---

## Development

### How do I run the tests?
```bash
cargo test                         # unit tests, no root required
sudo -E cargo test -- --ignored    # integration tests that touch pf
```

### Can I use the library without the CLI?
Yes — `mufw-core` is a standalone crate. Use it to build GUIs, TUIs, or integrations.

### Will there be a Linux version?
No. Linux already has `ufw`/`nftables`/`iptables`. mufw exists specifically because macOS doesn't.

### Will there be a TUI?
Planned for v0.3 (ratatui).

### Can I help?
Please! See [CONTRIBUTING.md](CONTRIBUTING.md). Good first issues are tagged `good-first-issue` on GitHub.

---

## Licensing & trust

### What's the license?
Dual **MIT OR Apache-2.0**, at your option. Rust community norm; maximally permissive.

### Does mufw phone home?
No network calls. Ever. The only thing mufw talks to is `/dev/pf` and `/sbin/pfctl` locally.

### How do I report a security issue?
Email **contact@bragdev.com** — please don't open a public issue. See [SECURITY.md](SECURITY.md).

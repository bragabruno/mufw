# mufw

> **mufw** — a ufw-like command-line firewall for macOS, built in Rust on top of `pf`.

[![CI](https://github.com/bragdev/mufw/actions/workflows/ci.yml/badge.svg)](https://github.com/bragdev/mufw/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mufw.svg)](https://crates.io/crates/mufw)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

macOS ships with the powerful **pf** packet filter — but its syntax is verbose and there's no equivalent to Linux's `ufw`. **mufw** closes that gap: a tiny, fast CLI that manages a dedicated pf anchor so you get one-liners like `ufw allow 22` without ever touching Apple's own pf config.

```console
$ sudo mufw enable
$ sudo mufw allow 22
$ sudo mufw allow 443/tcp --comment "web"
$ sudo mufw allow --from 192.168.1.0/24 --to any --port 5432
$ sudo mufw deny --from 203.0.113.66
$ sudo mufw limit 22
$ sudo mufw status --numbered
```

## Features

- **ufw-style CLI** — `allow`, `deny`, `limit`, `delete`, `status`, `enable/disable/reload/reset`
- **Built on `pfctl-rs`** — no shell-parsing fragility
- **Isolated anchor** (`com.mufw`) — your rules never collide with Apple's
- **Brute-force throttle** (`limit`) via `max-src-conn-rate` + overload table
- **VPN-friendly defaults** — `set skip on lo0`, easy to add utunN
- **JSON output** (`--json`) for scripting / dashboards
- **LaunchDaemon** for persistence across reboots
- **Tests**: unit tests in core, CLI integration tests via `assert_cmd`
- **CI**: fmt, clippy, tests on `macos-latest`

## Install

### Homebrew (recommended once the tap is published)

```bash
brew install bragdev/tap/mufw
```

### From source

```bash
git clone https://github.com/bragdev/mufw
cd mufw
cargo install --path crates/mufw-cli
```

### Persistence (LaunchDaemon)

```bash
sudo ./scripts/install.sh
```

## Usage

| ufw | mufw |
|---|---|
| `ufw enable` | `sudo mufw enable` |
| `ufw allow 22` | `sudo mufw allow 22` |
| `ufw allow 80/tcp` | `sudo mufw allow 80/tcp` |
| `ufw limit 22` | `sudo mufw limit 22` |
| `ufw deny from 1.2.3.4` | `sudo mufw deny --from 1.2.3.4` |
| `ufw status numbered` | `sudo mufw status --numbered` |
| `ufw delete 3` | `sudo mufw delete 3` |
| `ufw reset` | `sudo mufw reset` |

See `mufw --help` for the full surface, or the [man page](man/mufw.1).

Got questions? Check the [FAQ](FAQ.md).

## How it works

- mufw stores rules in `/etc/mufw/rules.toml` (root-owned, 0600).
- It renders them into a pf anchor at `/etc/pf.anchors/com.mufw`.
- It adds (once) a load line to `/etc/pf.conf`:
  ```
  anchor "com.mufw"
  load anchor "com.mufw" from "/etc/pf.anchors/com.mufw"
  ```
- On every mutation it validates the ruleset with `pfctl -nf` before loading.

## Roadmap

- [ ] v0.1 — MVP CLI (this scaffold)
- [ ] v0.2 — safe-apply with auto-rollback timer, profiles, IPv6 parity
- [ ] v0.3 — TUI (`mufw tui`) with ratatui
- [ ] v1.0 — stable CLI contract, Homebrew core submission

## Contributing

PRs welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Report security issues privately per [SECURITY.md](SECURITY.md).

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Prior art & thanks

- [Mullvad pfctl-rs](https://github.com/mullvad/pfctl-rs) — the ioctl bridge that makes this practical
- [ufw](https://launchpad.net/ufw) — the ergonomics we're imitating
- [hjuutilainen/pf-conf](https://github.com/hjuutilainen/pf-conf), [stefancaspersz/pf-setup](https://github.com/stefancaspersz/pf-setup), [essandess/macOS-Fortress](https://github.com/essandess/macOS-Fortress) — prior art in the macOS pf space

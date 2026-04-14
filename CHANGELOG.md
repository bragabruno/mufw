# Changelog

All notable changes to this project will be documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial scaffold: `mufw-core` + `mufw-cli` workspace.
- CLI verbs: `enable`, `disable`, `reload`, `reset`, `status`, `list`,
  `allow`, `deny`, `limit`, `delete`.
- TOML-backed rule store at `/etc/mufw/rules.toml`.
- Dedicated pf anchor `com.mufw` (never touches Apple anchors).
- LaunchDaemon plist + install/uninstall scripts.
- GitHub Actions CI (fmt, clippy, test) on `macos-latest`.
- Dual MIT/Apache-2.0 license, CoC, SECURITY policy.

[Unreleased]: https://github.com/bragdev/mufw/compare/v0.0.0...HEAD

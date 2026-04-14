# Contributing to mufw

Thanks for your interest! mufw is a small, focused project — contributions of all shapes are welcome.

## Ground rules

- Be kind. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Open an issue before large changes so we can align on scope.
- Security issues → read [SECURITY.md](SECURITY.md) — **don't** open a public issue.

## Development setup

```bash
git clone https://github.com/bragdev/mufw
cd mufw
rustup toolchain install stable
cargo build
cargo test
```

Integration tests that touch pf are gated behind `--ignored` and require `sudo`:

```bash
sudo -E cargo test -- --ignored
```

## Style

- `cargo fmt` — must pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — must pass.
- Commit style: [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:` …).
- Keep PRs focused; unrelated refactors in separate PRs.

## Testing matrix

| Layer | Tool |
|---|---|
| Unit | `cargo test` |
| CLI integration | `assert_cmd` |
| pf apply (manual) | tests gated behind `--ignored` |

## Adding a subcommand

1. Add a variant to `Cmd` in `crates/mufw-cli/src/main.rs`.
2. Thread it into `match cli.cmd`.
3. If it mutates rules, route through `Store` and `apply()`.
4. Add at least one integration test.
5. Update README and the man page.

## Release

Maintainers only — tag `vX.Y.Z`, push. `release.yml` publishes to crates.io and bumps the Homebrew tap.

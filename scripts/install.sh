#!/usr/bin/env bash
# install.sh — build mufw from source and install the LaunchDaemon.
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2; exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> building mufw (release)"
cargo build --release -p mufw

echo "==> installing binary to /usr/local/bin/mufw"
install -m 0755 target/release/mufw /usr/local/bin/mufw

echo "==> creating /etc/mufw"
install -d -m 0700 /etc/mufw

echo "==> installing LaunchDaemon"
install -m 0644 -o root -g wheel \
  packaging/launchd/com.mufw.plist /Library/LaunchDaemons/com.mufw.plist
launchctl unload /Library/LaunchDaemons/com.mufw.plist 2>/dev/null || true
launchctl load   -w /Library/LaunchDaemons/com.mufw.plist

echo "==> done. try: sudo mufw status"

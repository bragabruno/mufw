#!/usr/bin/env bash
set -euo pipefail
if [[ $EUID -ne 0 ]]; then echo "run with sudo" >&2; exit 1; fi

launchctl unload -w /Library/LaunchDaemons/com.mufw.plist 2>/dev/null || true
rm -f /Library/LaunchDaemons/com.mufw.plist
rm -f /usr/local/bin/mufw
pfctl -a com.mufw -F rules 2>/dev/null || true
rm -f /etc/pf.anchors/com.mufw
echo "mufw uninstalled. (Rules file at /etc/mufw/rules.toml preserved — delete manually if desired.)"

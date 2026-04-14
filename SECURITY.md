# Security Policy

## Supported Versions

While mufw is pre-1.0, only the latest minor release is supported.

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Email **contact@bragdev.com** with:

- A clear description of the issue
- Steps to reproduce (proof-of-concept if possible)
- The version of mufw and macOS you observed it on
- Your disclosure timeline preference

You'll receive an acknowledgement within **72 hours** and a remediation plan within **7 days** for confirmed issues. Coordinated disclosure is preferred; we'll credit you in the release notes unless you'd rather remain anonymous.

## Scope

In scope: the `mufw` binary, the `mufw-core` library, install scripts, LaunchDaemon plist, Homebrew formula.

Out of scope: vulnerabilities in upstream `pfctl-rs`, the macOS `pf` kernel module, or other dependencies — report those to the appropriate upstream maintainers.

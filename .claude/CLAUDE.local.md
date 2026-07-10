# Development Environment

## Environment Split

Engine development (M0-M9), networking (M10), and the card pipeline (M12) are pure Rust
with zero GUI dependencies. All of this work happens on the **Debian VM** over SSH.

Tauri UI work (M11+) requires a display server and platform webview libraries. This work
happens on the **Windows PC** with the same repo. Push from one machine, pull on the other.

This split doesn't need to be solved until M11.

## Global Installs (Debian VM — one-time setup)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
apt install sqlite3 git
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
```

## Project-Scoped Version Pinning

- `rust-toolchain.toml` pins Rust stable
- `.nvmrc` pins Node.js 22
- `rusqlite` uses `bundled` feature (no system libsqlite3-dev needed)

After cloning:
```bash
nvm use && cargo build && cargo test --all
```

## Windows PC Setup (M11+ only)

```powershell
winget install Rustlang.Rustup
# nvm-windows: https://github.com/coreybutler/nvm-windows
# nvm install 22 && nvm use 22
# Tauri CLI: cargo install tauri-cli
```

## CI: GitHub Actions

Deliberately minimal — `.github/workflows/ci.yml`, one job, kept cheap until the
engine is closer to playable alpha.

- Runs on: Ubuntu only (`ubuntu-latest`), 45-minute timeout
- Triggers: push to `main`, PRs targeting `main`, and `workflow_dispatch`
- Runs: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all`
- Caching: `Swatinem/rust-cache@v2`; concurrency group cancels superseded runs
- Deferred to M10/M11: OS matrix (Windows, macOS), nightly benchmark regression
  alerts, cross-platform Tauri builds via `tauri-action`

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

- Runs on: Ubuntu, Windows, macOS
- Runs: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all`
- Nightly: performance benchmarks with regression alerts
- Tauri builds: cross-platform via `tauri-action` (configured in M11)

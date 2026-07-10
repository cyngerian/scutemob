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

- `rust-toolchain.toml` pins Rust to an **exact stable version** (currently `1.95.0`),
  not the floating `stable` channel (SR-11). It is the single source of truth for the
  toolchain: `rustup` honors it for every cargo/clippy/rustfmt call in the repo and
  auto-installs that version with `[rustfmt, clippy]` if missing, and CI reads the same
  `channel` value out of the file (see below). CI and every dev box therefore run the
  same rustc/clippy by construction — a local `cargo clippy -- -D warnings` is now an
  authoritative preview of the CI clippy gate. **The local gate is only as authoritative
  as this pin**: if the file is bumped, `rustup update` / `rustup toolchain install
  <version>` locally before trusting a green local run.
- `.nvmrc` pins Node.js 22
- `rusqlite` uses `bundled` feature (no system libsqlite3-dev needed)

To bump the Rust version: edit `channel` in `rust-toolchain.toml` (one place), install
it locally, re-run the full gate (fmt + `clippy --all-targets -- -D warnings` + `test
--all` + `build --workspace`) to surface any new lints, and commit. Do it on a
deliberate schedule, not incidentally — a floating channel is exactly what SR-11 removed.

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
  `cargo build --workspace` (SR-3 seal gate), `cargo test --all`
- Toolchain (SR-11): a "Read pinned toolchain" step greps `channel` out of
  `rust-toolchain.toml` and feeds it to `dtolnay/rust-toolchain@master`
  (`toolchain:` input), then a "Verify toolchain matches the pin" step fails the
  run if the installed `rustc` version differs. This replaced `@stable`, which
  floated to the newest release and could redden CI on new lints with no commit.
  The pin lives in `rust-toolchain.toml`, not the workflow — bump it there.
- Caching: `Swatinem/rust-cache@v2`; concurrency group cancels superseded runs
- Deferred to M10/M11: OS matrix (Windows, macOS), nightly benchmark regression
  alerts, cross-platform Tauri builds via `tauri-action`

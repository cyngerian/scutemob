//! M11-local play server — one human seat, three simulator bots, over HTTP.
//!
//! The first-playable surface: a browser plays a real Commander game against
//! `HeuristicBot`s driven by the same `LocalGame` the fuzzer uses. This is the
//! only crate in M11-local with async or IO (`memory/m11-session-plan.md` §3);
//! `crates/engine`, `crates/simulator` and `crates/view-model` stay pure
//! (Architecture Invariant 1).
//!
//! Usage:
//!   play-server --port 3040 --players 4 --bot heuristic --seed 0

mod api;
mod session;
mod view;

use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use clap::{Parser, ValueEnum};
use mtg_simulator::BotKind;
use tower_http::services::ServeDir;

use session::{AppState, NewGameDefaults, SharedState};

/// M11-local play server.
#[derive(Parser, Debug)]
#[command(
    name = "play-server",
    about = "MTG Commander play server (1 human + bots)"
)]
struct Cli {
    /// Port to bind the HTTP server to. 3040 rather than 3030 so this can run
    /// alongside the replay viewer without a collision.
    #[arg(long, default_value = "3040")]
    port: u16,

    /// Host address to bind to. Default is localhost-only. Use 0.0.0.0 to expose on the local network.
    /// MR-M9.5-06: default is 127.0.0.1, not 0.0.0.0, to avoid unintended network exposure.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Number of seats at the table. Seat 1 is the human (CR 903.1 — Commander
    /// is a multiplayer format; the default table is four).
    #[arg(long, default_value = "4")]
    players: u32,

    /// Which bot fills the non-human seats. `HeuristicBot` is the web client's
    /// default because `RandomBot` makes nonsense plays that read as engine bugs
    /// to a human (plan §8 R5); `RandomBot` remains the fuzzer's default.
    #[arg(long, value_enum, default_value_t = BotArg::Heuristic)]
    bot: BotArg,

    /// Seed for the deterministic pregame build. Fixed at 0 by default — a play
    /// session should be reproducible from its command line, and a bug report
    /// that says "seed 0, four players, heuristic" is replayable. Pass a
    /// different value for a different table; `POST /api/game` can override it
    /// per game.
    #[arg(long, default_value = "0")]
    seed: u64,
}

/// clap mirror of `mtg_simulator::BotKind` — the simulator type is not a clap
/// `ValueEnum` and teaching it to be one would put a CLI dependency in a library
/// crate.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum BotArg {
    Random,
    Heuristic,
}

impl From<BotArg> for BotKind {
    fn from(arg: BotArg) -> Self {
        match arg {
            BotArg::Random => BotKind::Random,
            BotArg::Heuristic => BotKind::Heuristic,
        }
    }
}

fn main() -> Result<()> {
    // Build a custom tokio runtime with 8 MB worker thread stacks.
    //
    // The MTG rules engine uses deep call chains when resolving triggered abilities
    // (prowess, ward, ETB cascades). In debug builds these chains exceed the default
    // tokio worker thread stack (2 MB), causing stack overflows. 8 MB matches the OS
    // default for regular threads (used by `cargo test`). Same reason, and the same
    // numbers, as `tools/replay-viewer/src/main.rs`.
    //
    // The MULTI-THREAD flavor is additionally load-bearing here, not just a
    // performance choice: every handler wraps its synchronous engine work in
    // `tokio::task::block_in_place`, which PANICS on a current-thread runtime.
    // (That is also why the inline HTTP tests must use
    // `#[tokio::test(flavor = "multi_thread")]`.)
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_stack_size(8 * 1024 * 1024) // 8 MB
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    runtime.block_on(async_main())
}

/// Binds a listener and serves. **Only reached from `main`** — the inline tests
/// drive [`build_router`] through `tower::ServiceExt::oneshot` and never open a
/// socket (session plan §7 constraint 1: an agent context that starts this
/// binary gets SIGKILL/137).
async fn async_main() -> Result<()> {
    let cli = Cli::parse();

    let defaults = NewGameDefaults {
        players: cli.players,
        bot: cli.bot.into(),
        seed: cli.seed,
    };
    // Fail fast on an unusable table size rather than 400-ing every request.
    session::config_for(defaults).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let state: SharedState = AppState::new(defaults);

    // Same resolution order as the replay viewer: next to the executable first
    // (an installed build), then the crate's own dist/ (a `cargo run` from the
    // workspace root), then a bare dist/.
    let dist_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dist");
    let dist_dir = if dist_dir.exists() {
        dist_dir
    } else {
        let cwd_dist = PathBuf::from("tools/play-server/dist");
        if cwd_dist.exists() {
            cwd_dist
        } else {
            PathBuf::from("dist")
        }
    };

    let router = build_router(state, &dist_dir);

    let addr = format!("{}:{}", cli.host, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    println!("Play server running at http://{}:{}/", cli.host, cli.port);
    println!("API: http://{}:{}/api/", cli.host, cli.port);
    if dist_dir.exists() {
        println!("Frontend: serving from {}", dist_dir.display());
    } else {
        println!("Frontend: dist/ not found — run `npm run build` in tools/play-server/frontend/");
    }

    axum::serve(listener, router).await?;
    Ok(())
}

/// Build the axum router with all API routes and static file serving.
///
/// A **free function**, not a method or a closure over a bound listener, so a
/// test can construct the whole HTTP surface and drive it with
/// `tower::ServiceExt::oneshot` without binding a port.
fn build_router(state: SharedState, dist_dir: &PathBuf) -> Router {
    let api_router = Router::new()
        .route("/game", get(api::get_game).post(api::post_game))
        .route("/game/action", post(api::post_action))
        .route("/game/mulligan", post(api::post_mulligan))
        .route("/healthz", get(api::get_healthz))
        .with_state(state);

    let router = Router::new().nest("/api", api_router);

    // Serve the Svelte frontend from dist/ if it exists (Session 6 builds it).
    if dist_dir.exists() {
        router.fallback_service(ServeDir::new(dist_dir).append_index_html_on_directories(true))
    } else {
        router
    }
}

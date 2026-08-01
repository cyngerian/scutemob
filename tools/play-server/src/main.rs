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

// ── Integration tests ─────────────────────────────────────────────────────────

/// Inline HTTP tests, in the shape of `tools/replay-viewer/src/main.rs`'s.
///
/// # No test in this module binds a port
///
/// Session plan §7 constraint 1. Every request is driven through
/// [`build_router`] with `tower::ServiceExt::oneshot`; nothing here reaches
/// [`async_main`], `tokio::net::TcpListener` or `axum::serve`. (An agent context
/// that starts the real binary gets SIGKILL/137 — the replay-viewer note in
/// `memory/gotchas-infra.md`.) The symbols `TcpListener`, `axum::serve`,
/// `bind` and `async_main` must never appear below this line.
///
/// # Seed-pinned
///
/// Every fixture is built at [`SEED`], so the pregame is byte-identical run to
/// run (`mtg_simulator::setup::build_initial_state` is deterministic from the
/// config alone). Every card name and every count asserted below was *read off a
/// real run* at that seed, not reasoned to — the PB-DX3 lesson.
///
/// # `flavor = "multi_thread"`
///
/// Mandatory on every async test here: the handlers wrap their engine work in
/// `tokio::task::block_in_place`, which **panics** on the current-thread runtime
/// that a plain `#[tokio::test]` builds.
#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{BTreeSet, HashSet};

    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use http_body_util::BodyExt;
    use mtg_view_model::StateViewModel;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    /// The pin. Changing it invalidates every card name and count below.
    const SEED: u64 = 0;
    /// CR 903.1: the default Commander table.
    const PLAYERS: u32 = 4;
    /// `session::HUMAN_SEAT` rendered by `setup::seat_name`.
    const HUMAN: &str = "Human-1";

    // ── Harness ───────────────────────────────────────────────────────────────

    fn shared_state() -> SharedState {
        AppState::new(NewGameDefaults {
            players: PLAYERS,
            bot: BotKind::Heuristic,
            seed: SEED,
        })
    }

    /// `dist/` deliberately does not exist, so no `ServeDir` fallback is mounted
    /// and a 404 in these tests really is "the API said 404".
    fn app(state: SharedState) -> Router {
        build_router(state, &PathBuf::from("nonexistent_dist"))
    }

    async fn body_string(body: Body) -> String {
        let bytes = body.collect().await.expect("body collects").to_bytes();
        String::from_utf8(bytes.to_vec()).expect("body is UTF-8")
    }

    /// `GET`, returning the status and the **raw** body text. Test 7 asserts over
    /// this string rather than over parsed fields on purpose.
    async fn get_raw(state: &SharedState, uri: &str) -> (StatusCode, String) {
        let resp = app(state.clone())
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .expect("router is infallible");
        let status = resp.status();
        (status, body_string(resp.into_body()).await)
    }

    async fn get_json(state: &SharedState, uri: &str) -> (StatusCode, Value) {
        let (status, text) = get_raw(state, uri).await;
        (status, serde_json::from_str(&text).expect("body is JSON"))
    }

    async fn post_json(state: &SharedState, uri: &str, body: Value) -> (StatusCode, Value) {
        let resp = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .expect("router is infallible");
        let status = resp.status();
        let text = body_string(resp.into_body()).await;
        (status, serde_json::from_str(&text).expect("body is JSON"))
    }

    /// `POST /api/game` with the CLI defaults, asserting it worked.
    async fn new_game(state: &SharedState) -> Value {
        let (status, view) = post_json(state, "/api/game", json!({})).await;
        assert_eq!(status, StatusCode::OK, "POST /api/game failed: {view}");
        view
    }

    fn decision(view: &Value) -> &Value {
        let d = &view["decision"];
        assert!(!d.is_null(), "expected a pending decision, got {view}");
        d
    }

    fn seq(view: &Value) -> u64 {
        decision(view)["seq"].as_u64().expect("seq is a number")
    }

    fn command_count(view: &Value) -> u64 {
        view["summary"]["command_count"]
            .as_u64()
            .expect("command_count is a number")
    }

    fn action_indices(view: &Value, kind: &str) -> Vec<u64> {
        decision(view)["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .filter(|a| a["kind"] == kind)
            .map(|a| a["index"].as_u64().expect("index is a number"))
            .collect()
    }

    fn action_index_by_label(view: &Value, label: &str) -> Option<u64> {
        decision(view)["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["label"] == label)
            .map(|a| a["index"].as_u64().expect("index is a number"))
    }

    fn event_texts(view: &Value) -> Vec<String> {
        view["events"]
            .as_array()
            .expect("events is an array")
            .iter()
            .map(|e| e["text"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    /// A card name as it appears inside the serialized JSON body (apostrophes and
    /// the like survive, but this keeps the search honest for any name serde
    /// would escape).
    fn as_serialized(name: &str) -> String {
        let quoted = serde_json::to_string(name).expect("a string serializes");
        quoted[1..quoted.len() - 1].to_string()
    }

    /// Drive the seed-pinned opening until the human is offered a **targeted**
    /// spell.
    ///
    /// At `SEED` this is: pass in upkeep, pass in draw, play Island in
    /// precombat main — after which `Cast Dispel` (CR 601.2c: "counter target
    /// spell") is affordable and offered. Observed, not assumed; the panic below
    /// prints the whole action list if the opening ever changes.
    async fn drive_to_targeted_spell(state: &SharedState) -> Value {
        let mut view = new_game(state).await;
        for _ in 0..8 {
            if action_index_by_label(&view, "Cast Dispel").is_some() {
                return view;
            }
            let index = action_indices(&view, "PlayLand")
                .first()
                .copied()
                .unwrap_or(0);
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({ "seq": seq(&view), "action_index": index }),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "driving failed: {next}");
            view = next;
        }
        panic!(
            "seed {SEED} no longer offers a targeted spell in the opening; last decision was {}",
            decision(&view)
        );
    }

    // ── 1 ─────────────────────────────────────────────────────────────────────

    /// `POST /api/game` builds a session where there was none and answers with
    /// the human's first decision already computed (the handler calls
    /// `advance()` before returning).
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_game_creates_session_and_returns_decision() {
        let state = shared_state();

        // Non-vacuity: there is provably no session before the POST, so the
        // assertions below cannot be describing a pre-existing game.
        let (status, before) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(before["kind"], "no_session");

        let view = new_game(&state).await;

        assert_eq!(view["summary"]["players"], PLAYERS);
        assert_eq!(view["summary"]["human"], 1);
        assert_eq!(view["summary"]["seed"], SEED);
        assert_eq!(view["summary"]["bot"], "Heuristic");
        // CR 103: nothing has been done yet, so the game is still pregame.
        assert_eq!(command_count(&view), 0);
        assert_eq!(view["summary"]["pregame"], true);

        let d = decision(&view);
        assert_eq!(d["seq"], 1, "the first decision is seq 1");
        assert_eq!(d["kind"], "Priority");
        assert_eq!(d["player"], 1, "the decision belongs to the human seat");
        let actions = d["actions"].as_array().expect("actions is an array");
        assert!(
            !actions.is_empty(),
            "a decision with no actions is a deadlock"
        );
        // Seed-pinned: turn 1 upkeep, no mana, nothing but a pass.
        assert_eq!(actions[0]["kind"], "PassPriority");
        assert_eq!(actions[0]["label"], "Pass priority");

        // The session really was built: a real table exists behind the decision.
        assert_eq!(
            view["state"]["zones"]["hand"][HUMAN]
                .as_array()
                .expect("the human has a hand")
                .len(),
            7
        );
    }

    // ── 2 ─────────────────────────────────────────────────────────────────────

    /// `GET /api/game` returns this seat's view, with a real CR 103.5 / 402.1
    /// opening hand of seven **named** cards.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_game_returns_seat_view_with_seven_card_hand() {
        let state = shared_state();
        new_game(&state).await;

        let (status, view) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);

        let hands = view["state"]["zones"]["hand"]
            .as_object()
            .expect("hand is keyed by player name");
        assert_eq!(hands.len(), PLAYERS as usize, "every seat has a hand");

        let own = hands[HUMAN].as_array().expect("the human has a hand");
        assert_eq!(own.len(), 7, "CR 103.5 opening hand of seven");

        // Non-vacuous: every entry is a *named*, unredacted card, not a
        // placeholder — a seat view that had redacted its own hand would still
        // have length 7.
        let own_names: Vec<&str> = own
            .iter()
            .map(|c| c["name"].as_str().expect("a name"))
            .collect();
        for (card, name) in own.iter().zip(&own_names) {
            assert_eq!(
                card["hidden"], false,
                "{name} should be visible to its owner"
            );
            assert!(
                !name.is_empty(),
                "an empty name is a redaction leak-through"
            );
            assert_ne!(*name, "Hidden card");
        }
        // Seed-pinned exact hand, read off a real run at SEED.
        assert_eq!(
            own_names,
            vec![
                "Island",
                "Mist-Syndicate Naga",
                "Mist-Cloaked Herald",
                "Obelisk of Urd",
                "Accorder's Shield",
                "Hermes, Overseer of Elpis",
                "Swiftfoot Boots",
            ]
        );

        // The other three seats hold seven cards each too (CR 103.5 applies to
        // everyone) — but only as counts, which is Invariant 7's business and is
        // proven properly by `test_seat_view_over_http_contains_no_other_hand_card_names`.
        for seat in ["Bot-2", "Bot-3", "Bot-4"] {
            assert_eq!(hands[seat].as_array().expect("a hand").len(), 7);
        }
    }

    // ── 3 ─────────────────────────────────────────────────────────────────────

    /// `POST /api/game/action` with a pass both **advances the game** and lets the
    /// **bot seats act inside the same request** — the property that makes a push
    /// channel unnecessary in M11-local (item 8).
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_action_pass_priority_advances_and_bots_act() {
        let state = shared_state();
        let view = new_game(&state).await;
        assert_eq!(command_count(&view), 0);
        let pass = action_indices(&view, "PassPriority")[0];

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&view), "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        // ── half one: the game advanced ──
        // Four commands, not one: the human's pass plus one per bot seat. Read
        // off a real run at SEED; a 2-player table would show 2.
        assert_eq!(command_count(&after), 4, "one pass per seat, applied");
        assert_eq!(seq(&after), 2, "a new decision was minted");
        assert_eq!(after["summary"]["pregame"], false);
        assert_eq!(
            after["state"]["turn"]["step"], "Draw",
            "CR 500.1: the round of passes ended the upkeep step"
        );

        // ── half two: the bots acted ──
        let texts = event_texts(&after);
        for expected in [
            "Human-1 passes",
            "Bot-2 passes",
            "Bot-3 passes",
            "Bot-4 passes",
            "All players passed",
            "Beginning — Draw",
            // CR 504.1, seed-pinned: the human's draw for the turn.
            "Human-1 draws Dispel",
        ] {
            assert!(
                texts.iter().any(|t| t == expected),
                "expected event {expected:?}; got {texts:?}"
            );
        }
        // Not merely "some events": every bot seat is individually represented,
        // so this cannot pass on the human's own command alone.
        assert!(
            texts.iter().filter(|t| t.ends_with(" passes")).count() >= 4,
            "got {texts:?}"
        );
    }

    // ── 4 ─────────────────────────────────────────────────────────────────────

    /// A `seq` that does not match the outstanding decision is **409**, and the
    /// game state is left exactly as it was.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_action_stale_seq_returns_409() {
        let state = shared_state();
        let view = new_game(&state).await;
        assert_eq!(seq(&view), 1);
        let pass = action_indices(&view, "PassPriority")[0];

        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": 999, "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(err["kind"], "stale_decision");
        let message = err["error"].as_str().expect("an error message");
        assert!(
            message.contains("expected 1") && message.contains("got 999"),
            "the client must be able to resync from the message: {message:?}"
        );

        // Non-vacuous, twice over.
        // (a) The rejection changed nothing: same seq, same command count.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(seq(&still), 1, "the decision was not invalidated");
        assert_eq!(command_count(&still), 0, "no command was applied");
        // (b) The very same action index, at the right seq, is accepted — so the
        // 409 was about `seq` and nothing else.
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": 1, "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
        assert_eq!(command_count(&ok), 4);
    }

    // ── 5 ─────────────────────────────────────────────────────────────────────

    /// An `action_index` outside the list the server just sent is **400**: the
    /// request is malformed on its face, not refused by the engine.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_action_unknown_index_returns_400() {
        let state = shared_state();
        let view = new_game(&state).await;
        let count = decision(&view)["actions"]
            .as_array()
            .expect("actions is an array")
            .len();
        let out_of_range = count as u64 + 98;

        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&view), "action_index": out_of_range }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(err["kind"], "unknown_action");
        assert!(
            err["error"]
                .as_str()
                .expect("an error message")
                .contains(&out_of_range.to_string()),
            "the message must name the offending index: {err}"
        );

        // Non-vacuous: the decision survived, and an in-range index on it is
        // accepted — so the 400 was about the index, not about the request shape.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(seq(&still), seq(&view));
        assert_eq!(command_count(&still), 0);
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&view), "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
    }

    // ── 6 ─────────────────────────────────────────────────────────────────────

    /// A target the **engine** refuses is **422**, not 400.
    ///
    /// The distinction is the whole point of the test. `BadParams` -> 400 fires
    /// when the `LegalAction` variant has no channel for the supplied param
    /// (`ParamError::UnsupportedParam`) and never reaches the engine;
    /// `Rejected` -> 422 means the command was built, handed to
    /// `process_command`, and refused there. This test drives the second path
    /// and demonstrates the first alongside it, at the same `seq`, so the two are
    /// told apart by observation rather than by argument.
    ///
    /// Dispel is "counter target spell" (CR 601.2c); a player is not a spell, so
    /// `handle_cast_spell`'s target validation refuses it with
    /// `GameStateError::InvalidTarget`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_action_illegal_target_returns_422() {
        let state = shared_state();
        let view = drive_to_targeted_spell(&state).await;
        let at_seq = seq(&view);
        let before = command_count(&view);
        let cast = action_index_by_label(&view, "Cast Dispel").expect("driven to the cast");

        // Control, same decision: `targets` on a `PassPriority` has no channel at
        // all, so it never reaches the engine. 400, kind `bad_params`.
        let pass = action_indices(&view, "PassPriority")[0];
        let (status, control) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": at_seq,
                "action_index": pass,
                "params": { "targets": [{ "Player": 2 }] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "an unsupported param is a client error: {control}"
        );
        assert_eq!(control["kind"], "bad_params");

        // Subject: the same illegal target on an action that *does* carry
        // targets. The command is built and the engine refuses it.
        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": at_seq,
                "action_index": cast,
                "params": { "targets": [{ "Player": 2 }] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "an engine rejection is 422, not 400: {err}"
        );
        assert_eq!(err["kind"], "rejected");
        let message = err["error"].as_str().expect("an error message");
        assert!(
            message.contains("invalid target"),
            "the GameStateError must be rendered as text: {message:?}"
        );

        // Non-vacuous: neither refusal touched the game.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(seq(&still), at_seq, "the decision is still outstanding");
        assert_eq!(command_count(&still), before, "no command was applied");
        // And the decision is still answerable, so the 422 was about the target
        // and not about the game having become unplayable.
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": at_seq, "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
    }

    // ── 7 ─────────────────────────────────────────────────────────────────────

    /// **Architecture Invariant 7 at the HTTP boundary.**
    ///
    /// The omniscient truth is read separately, straight off the `PlaySession`'s
    /// `GameState` via `StateViewModel::from_game_state` (the developer path), to
    /// learn what the other seats are *actually* holding. Every one of those card
    /// names is then searched for in the **raw response text** of
    /// `GET /api/game` — not in the parsed `zones.hand` field.
    ///
    /// Searching the whole body is deliberate and is the S4 review HIGH applied
    /// forward: redaction follows the *rendering site*, not the zone, so a leak
    /// through an action label, an event line, a stack item, an attacker name or
    /// a boolean-driven annotation would slip past a field-scoped check.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_seat_view_over_http_contains_no_other_hand_card_names() {
        let state = shared_state();
        new_game(&state).await;

        // ── omniscient truth, obtained out of band ──
        let omniscient = {
            let guard = state.session.lock().expect("the lock is not poisoned");
            let play = guard.as_ref().expect("a session exists");
            StateViewModel::from_game_state(play.game.state(), &play.names)
        };

        // Names the human is legitimately entitled to read: their own hand, plus
        // everything in a public zone (CR 903.6 puts every commander in the
        // command zone, which is open information).
        let mut allowed: HashSet<String> = HashSet::new();
        for permanents in omniscient.zones.battlefield.values() {
            for p in permanents {
                allowed.insert(p.name.clone());
            }
        }
        for zone in [&omniscient.zones.graveyard, &omniscient.zones.command_zone] {
            for cards in zone.values() {
                for c in cards {
                    allowed.insert(c.name.clone());
                }
            }
        }
        for c in &omniscient.zones.exile {
            allowed.insert(c.name.clone());
        }
        for item in &omniscient.zones.stack {
            allowed.insert(item.source_name.clone());
        }
        let own_hand: Vec<String> = omniscient.zones.hand[HUMAN]
            .iter()
            .map(|c| c.name.clone())
            .collect();
        assert_eq!(own_hand.len(), 7);
        allowed.extend(own_hand.iter().cloned());

        // Every card name held in another seat's hand that is NOT excused by the
        // set above. A name the human legitimately holds a copy of is excluded
        // (and there is nothing to conclude from its presence either way).
        let mut secrets: BTreeSet<String> = BTreeSet::new();
        for (seat, cards) in &omniscient.zones.hand {
            if seat == HUMAN {
                continue;
            }
            for c in cards {
                if !allowed.contains(&c.name) {
                    secrets.insert(c.name.clone());
                }
            }
        }
        // Seed-pinned: 21 cards across the three bot hands collapse to 20
        // distinct names the human has no entitlement to. Asserted exactly, so a
        // future change that quietly empties this set fails here rather than
        // turning the search below into a no-op.
        assert_eq!(
            secrets.len(),
            20,
            "guard against a vacuous pass: {secrets:?}"
        );

        // ── the payload the browser would actually receive ──
        let (status, body) = get_raw(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);

        for name in &secrets {
            let needle = as_serialized(name);
            assert!(
                !body.contains(&needle),
                "seat view leaked another player's hand card {name:?} (CR 402.1)"
            );
        }

        // Guard against the other vacuity: a wholly empty payload would pass
        // every assertion above. The human's own hand must be named in full.
        for name in &own_hand {
            let needle = as_serialized(name);
            assert!(
                body.contains(&needle),
                "the seat view must still show the human their own hand: {name:?} missing"
            );
        }
        // And the other seats appear as counted placeholders, not as absences.
        let parsed: Value = serde_json::from_str(&body).expect("body is JSON");
        for seat in ["Bot-2", "Bot-3", "Bot-4"] {
            let hand = parsed["state"]["zones"]["hand"][seat]
                .as_array()
                .expect("a hand");
            assert_eq!(hand.len(), 7);
            for card in hand {
                assert_eq!(card["hidden"], true);
            }
        }
    }

    // ── 8 ─────────────────────────────────────────────────────────────────────

    /// `GET /api/healthz` answers without a session and without touching the
    /// session lock, so it stays live while a long resolution holds it.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_healthz_ok() {
        let state = shared_state();

        // No game has been created — proof the handler does not depend on one.
        let (status, missing) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(missing["kind"], "no_session");

        let (status, health) = get_json(&state, "/api/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(health["status"], "ok");
        assert_eq!(health["service"], "play-server");
    }
}

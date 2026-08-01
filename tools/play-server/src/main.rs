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
    /// session should be reproducible from its command line. Pass a different
    /// value for a different table; `POST /api/game` can override it per game.
    ///
    /// A bug report needs the **mulligan count** as well as the seed (S5 review
    /// LOW 7): a redeal builds from `setup::redeal_seed(seed, seat, count)` and
    /// leaves this value untouched, so "seed 0, four players, heuristic" alone
    /// stops describing the table the moment a mulligan is taken.
    /// `GameSummary.mulligan_count` carries the missing term.
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
/// `bind` and `async_main` must never appear below the `#[cfg(test)]` attribute
/// on the next line.
///
/// That is **machine-enforced** (S5 review LOW 8), not a promise:
/// `test_no_socket_symbol_appears_in_the_test_region` reads **every `.rs` file
/// in the crate** and fails on any of the four inside a `#[cfg(test)]` region.
///
/// The paragraph above deliberately names all four symbols and also writes the
/// attribute inline. That is safe because the gate anchors on a *line-anchored*
/// occurrence of the attribute — a line whose first non-whitespace text is the
/// attribute itself — so a doc-comment line, which always starts with `///`, can
/// never be mistaken for it. The previous gate used a bare `find`, which located
/// **this paragraph** rather than the attribute below, and passed only because
/// all four symbols happened to be typed to the left of the marker in that one
/// sentence (S5 re-review MEDIUM 2).
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

    /// Test-only: the sentinel `seed` that makes `POST /api/game` fail its rebuild
    /// *inside* the session lock.
    ///
    /// Carried by the request, not by process state, so parallel tests in this binary
    /// cannot steal it from one another — see `api::post_game`'s injection point for the
    /// global-flag version that could, and did.
    ///
    /// Deliberately declared INSIDE this module rather than at file scope: a top-level
    /// `#[cfg(test)]` item moves the boundary `test_region` computes, which swept the real
    /// serving entry point — and the forbidden symbols it uses — into the "test region"
    /// and reddened `test_no_socket_symbol_appears_in_the_test_region`. The gate was right
    /// three times over: it then caught this very doc comment naming two of those symbols
    /// below the cut, on two successive attempts to describe why it had fired. Describe
    /// them periphrastically here, as the helper below already has to.
    pub(crate) const REBUILD_FAILURE_SEED: u64 = 0xDEAD_BEEF_F00D_u64;

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

    /// `POST` of a **raw** body, so a test can send something `serde_json`
    /// would never produce: a malformed document, or no body and no
    /// `Content-Type` at all. `content_type: None` omits the header.
    async fn post_raw(
        state: &SharedState,
        uri: &str,
        content_type: Option<&str>,
        body: &'static str,
    ) -> (StatusCode, String) {
        let mut builder = Request::builder().method("POST").uri(uri);
        if let Some(ct) = content_type {
            builder = builder.header(header::CONTENT_TYPE, ct);
        }
        let resp = app(state.clone())
            .oneshot(builder.body(Body::from(body)).unwrap())
            .await
            .expect("router is infallible");
        let status = resp.status();
        (status, body_string(resp.into_body()).await)
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

    /// The targeted spell `drive_to_targeted_spell` drives toward, and how many
    /// decisions it takes to get there.
    ///
    /// **Seed-pinned, and pinned to the `Complete`-def pool** — like the exact-hand and
    /// secrets-count assertions elsewhere in this module, a completeness flip in any
    /// card-def batch re-deals every seeded deck and this fixture moves with it.
    /// Re-observe both values off a real run when they do; do not guess them.
    ///
    /// PB-DX4 (2026-08-01, `scutemob-168`) re-derived them: this was `"Cast Dispel"` in 8
    /// steps, and the batch's four completeness demotions (one of them
    /// `thrasios_triton_hero`, a legendary creature, which shifted the commander draw for
    /// every seat) dealt the human a hand with no Dispel in it at all. The current opening
    /// is two Swamps into `Cast Drown in Ichor` ({1}{B}, CR 601.2c "target creature gets
    /// -X/-X"), which is not reachable inside 8 decisions because it needs a SECOND land
    /// drop — hence the larger bound.
    const TARGETED_SPELL: &str = "Cast Drown in Ichor";
    const TARGETED_SPELL_STEPS: usize = 48;

    /// Drive the seed-pinned opening until the human is offered a **targeted**
    /// spell.
    ///
    /// Observed, not assumed; the panic below prints the whole action list if the
    /// opening ever changes.
    ///
    /// The caller feeds the found action a `Player` target, which must be ILLEGAL for
    /// [`TARGETED_SPELL`] — that is the whole point of the test. Drown in Ichor targets a
    /// creature, so it is. If a future re-pin picks a spell that legally targets a player,
    /// the caller's `422` assertion fails loudly rather than passing for the wrong reason.
    async fn drive_to_targeted_spell(state: &SharedState) -> Value {
        let mut view = new_game(state).await;
        for _ in 0..TARGETED_SPELL_STEPS {
            if action_index_by_label(&view, TARGETED_SPELL).is_some() {
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
            "seed {SEED} no longer offers {TARGETED_SPELL} within {TARGETED_SPELL_STEPS} \
             steps; last decision was {}",
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
                "Regrowth",
                "Gemrazer",
                "Beast Whisperer",
                "Drown in Ichor",
                "Swamp",
                "Swamp",
                "Grave Pact",
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
            // CR 504.1, seed-pinned: the human's draw for the turn. Like the exact-hand
            // and secrets-count pins, this is a function of the `Complete`-def pool and
            // re-deals whenever a card-def batch flips a marker — re-read it off a real
            // run. (PB-DX4, 2026-08-01: was "Dispel".)
            "Human-1 draws In Garruk's Wake",
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

    // ── 4b ────────────────────────────────────────────────────────────────────

    /// **S5 review MEDIUM 1**: a `seq` from a game that has been *replaced* is
    /// stale, and the 409 must fire.
    ///
    /// The whole anti-stale-tab guarantee rested on `seq` being unique to a
    /// decision, and it was not: `LocalGame::start` restarts `decision_seq` at 0,
    /// so game B's first decision reused game A's `seq: 1` and `submit` matched
    /// it. Pre-fix, this exact sequence answered **200** and moved the new game's
    /// `command_count` from 0 to 4 — read off a real run, not reasoned to.
    ///
    /// Session 6 ships `POST /api/game` as a "New Game" button, so a tab
    /// rendering the old action list while someone restarts is an ordinary
    /// accident, not a contrived one.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_seq_from_a_replaced_game_is_stale() {
        let state = shared_state();
        let a = new_game(&state).await;
        let stale = seq(&a);
        assert_eq!(stale, 1);

        // Someone hits "New Game" while the first tab still shows game A.
        let b = new_game(&state).await;
        assert_eq!(command_count(&b), 0, "a fresh game has applied nothing");
        assert!(
            seq(&b) > stale,
            "the wire seq must not restart: game A issued {stale}, game B issued {}",
            seq(&b)
        );

        // The stale tab acts on its old render.
        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": stale, "action_index": 0 }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "a superseded seq must be a 409: {err}"
        );
        assert_eq!(err["kind"], "stale_decision");
        // Truthful `expected`/`got`, so the client can resync in one round trip.
        let message = err["error"].as_str().expect("an error message");
        assert!(
            message.contains(&format!("expected {}", seq(&b)))
                && message.contains(&format!("got {stale}")),
            "the message must name the CURRENT seq and the one sent: {message:?}"
        );

        // The point of the finding: game B was not acted on.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            command_count(&still),
            0,
            "the stale post must not have applied a command to the new game"
        );
        assert_eq!(
            seq(&still),
            seq(&b),
            "game B's decision is still outstanding"
        );

        // Non-vacuous: the *current* seq, same index, is still accepted — so the
        // 409 was about the seq belonging to a dead game and nothing else.
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&b), "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
        assert_eq!(command_count(&ok), 4);
    }

    /// The same guarantee across the **mulligan** rebuild (S5 review MEDIUM 1).
    ///
    /// `PlaySession::mulligan` calls `LocalGame::start` too, so it had the
    /// identical collision: pre-fix the redealt table's first decision was again
    /// `seq: 1` and the pre-mulligan tab's post was accepted with **200**,
    /// applying a command to a table it had never seen.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_seq_from_before_a_mulligan_is_stale() {
        let state = shared_state();
        let before = new_game(&state).await;
        let stale = seq(&before);
        let first_hand = before["state"]["zones"]["hand"][HUMAN].clone();

        let (status, after) =
            post_json(&state, "/api/game/mulligan", json!({ "take": true })).await;
        assert_eq!(status, StatusCode::OK, "{after}");
        assert_eq!(after["summary"]["mulligan_count"], 1);
        // Non-vacuous: the redeal really did rebuild the table (CR 103.5).
        assert_ne!(
            after["state"]["zones"]["hand"][HUMAN], first_hand,
            "a redeal that returns the same hand would make this test meaningless"
        );
        assert!(seq(&after) > stale, "the wire seq must not restart");

        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": stale, "action_index": 0 }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "a pre-mulligan seq must be a 409: {err}"
        );
        assert_eq!(err["kind"], "stale_decision");

        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            command_count(&still),
            0,
            "the stale post must not have applied a command to the redealt table"
        );
        assert_eq!(
            still["summary"]["pregame"], true,
            "and the game must still be mulliganable"
        );
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

    // ── 5b ────────────────────────────────────────────────────────────────────

    /// **S5 review MEDIUM 2**: a body axum itself could not deserialize is a
    /// **400** in this crate's JSON envelope — and explicitly **not** a 422.
    ///
    /// The collision is the finding. `JsonDataError`'s own status is 422, which
    /// this crate documents as "the *engine* refused the command"; and axum
    /// answers it directly, with a `text/plain` body carrying no `kind`. A
    /// client posting `"target"` for `"targets"` — a typo, never seen by the
    /// engine — read `kind: undefined` and a status its 422 branch would report
    /// to the user as an engine rejection.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_malformed_action_body_returns_400_not_422() {
        let state = shared_state();
        let view = new_game(&state).await;
        let at_seq = seq(&view);

        // The finding's own example: `target` for `targets`, under
        // `deny_unknown_fields`.
        let (status, text) = post_raw(
            &state,
            "/api/game/action",
            Some("application/json"),
            r#"{"seq":1,"action_index":0,"params":{"target":[]}}"#,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a client-side typo never reaches the engine: {text}"
        );
        assert_ne!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "422 is reserved for an ENGINE rejection; reusing it here is the collision \
             this test exists to prevent"
        );
        let err: Value = serde_json::from_str(&text).expect("the envelope is JSON, not text/plain");
        assert_eq!(
            err["kind"], "invalid_body",
            "the client must be able to branch without parsing prose"
        );
        assert!(err["error"].as_str().is_some_and(|s| !s.is_empty()));

        // A missing required field is the same class.
        let (status, text) = post_raw(
            &state,
            "/api/game/action",
            Some("application/json"),
            r#"{"action_index":0}"#,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{text}");
        let err: Value = serde_json::from_str(&text).expect("the envelope is JSON");
        assert_eq!(err["kind"], "invalid_body");

        // Syntactically broken JSON keeps axum's 400 but gains a `kind`.
        let (status, text) = post_raw(
            &state,
            "/api/game/action",
            Some("application/json"),
            r#"{"seq":1,"#,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{text}");
        let err: Value = serde_json::from_str(&text).expect("the envelope is JSON");
        assert_eq!(err["kind"], "malformed_json");

        // Non-vacuous: none of the three refusals touched the game, and a
        // well-formed body at the same seq is still accepted.
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": at_seq, "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
    }

    /// **S5 review LOW 5**: `POST /api/game` with an unknown field is a 400 and
    /// starts **no game** — it used to answer 200 with a default four-player one.
    ///
    /// `Option<T>`'s `FromRequest` impl is `T::from_request(..).ok()`, so it
    /// mapped every rejection to `None` and `None` meant "use the CLI defaults".
    /// The absent-body case still has to work, so both are asserted here.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_game_rejects_a_malformed_body_but_accepts_an_absent_one() {
        let state = shared_state();

        let (status, text) = post_raw(
            &state,
            "/api/game",
            Some("application/json"),
            r#"{"playerz":9}"#,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a misspelled field must not silently yield a default game: {text}"
        );
        let err: Value = serde_json::from_str(&text).expect("the envelope is JSON");
        assert_eq!(err["kind"], "invalid_body");

        // The sharp half: no session was created by the rejected request.
        let (status, missing) = get_json(&state, "/api/game").await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "the rejected POST must not have started a game"
        );
        assert_eq!(missing["kind"], "no_session");

        // An ABSENT body is deliberate and supported: no body, no content type.
        let (status, text) = post_raw(&state, "/api/game", None, "").await;
        assert_eq!(
            status,
            StatusCode::OK,
            "an omitted body still means 'use the CLI defaults': {text}"
        );
        let view: Value = serde_json::from_str(&text).expect("a seat view");
        assert_eq!(view["summary"]["players"], PLAYERS);
        assert_eq!(view["summary"]["seed"], SEED);
        assert_eq!(view["summary"]["bot"], "Heuristic");
    }

    // ── 5c ────────────────────────────────────────────────────────────────────

    /// **S5 review LOW 9**: `NoPendingDecision` -> **409**, the one error
    /// semantic plan item 6 names that had no test.
    ///
    /// Reached the only way the HTTP surface can reach it, by playing a real game
    /// to its end: `LocalGame::advance` clears `pending` on `AdvanceOutcome::
    /// GameOver`, so the next `POST /api/game/action` takes that arm. Nothing here
    /// reaches into `PlaySession` to manufacture the state — a two-seat table
    /// (CR 104.2a: the last player standing wins) where the human answers every
    /// decision with a pass is an ordinary game, just a short-lived human.
    ///
    /// **This test costs ~3 s**, which is why it is the only one that plays a
    /// whole game. The alternatives were all worse: `HaltReason::MaxTurns` needs
    /// 200 turns rather than ~110, and `LegalAction::Concede` is deliberately
    /// never offered by the provider (`legal_actions.rs`: "bots should never
    /// auto-concede"), so there is no shortcut that is still the same arm.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_action_after_game_over_returns_409() {
        let state = shared_state();
        let (status, mut view) = post_json(&state, "/api/game", json!({ "players": 2 })).await;
        assert_eq!(status, StatusCode::OK, "{view}");

        // Answer every decision with a pass until someone wins. The cap is ~5x
        // the observed 1,016 actions; it exists so a rules change that makes the
        // game unwinnable fails loudly instead of hanging.
        let mut steps = 0u32;
        while view["game_over"].is_null() {
            steps += 1;
            assert!(
                steps <= 5_000,
                "no game over after {steps} actions; last view {}",
                view["summary"]
            );
            let index = action_indices(&view, "PassPriority")
                .first()
                .copied()
                .unwrap_or(0);
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({ "seq": seq(&view), "action_index": index }),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "action {steps} failed: {next}");
            view = next;
        }
        // CR 104.2a — and non-vacuity: a *halted* game would also fill
        // `game_over`, and that is a different arm of `seat_view`.
        assert_eq!(
            view["game_over"]["halted"], false,
            "this must be a real conclusion, not a safety valve: {}",
            view["game_over"]
        );
        assert!(view["game_over"]["winner"].is_string());
        assert!(
            view["decision"].is_null(),
            "a concluded game must offer no decision"
        );

        // The subject: an action posted against a game that has nothing left to
        // decide. `seq` is irrelevant here — the pending decision is gone, so the
        // `seq` check is never even reached.
        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": 1, "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{err}");
        assert_eq!(err["kind"], "no_pending_decision");
        assert!(err["error"]
            .as_str()
            .expect("an error message")
            .contains("no decision"));

        // And `GET` still answers, so the 409 is about the decision and not about
        // the session having become unreadable.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(still["game_over"]["halted"], false);
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
        let cast = action_index_by_label(&view, TARGETED_SPELL).expect("driven to the cast");

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
        // Seed-pinned: the cards across the three bot hands collapse to 18
        // distinct names the human has no entitlement to. Asserted exactly, so a
        // future change that quietly empties this set fails here rather than
        // turning the search below into a no-op. (This pin, like the exact-hand
        // pin above, is a function of the Complete-def pool: a completeness
        // flip in any card-def batch re-deals every seed-pinned deck — re-read
        // the value off a real run when it moves.)
        assert_eq!(
            secrets.len(),
            14,
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

    // ── 7b ────────────────────────────────────────────────────────────────────

    /// **S5 review LOW 6**: `POST /api/game` — and only it — recovers from a
    /// poisoned session mutex.
    ///
    /// Before this, one panic inside a handler cost a process restart, on the one
    /// surface in the project that runs with `check_invariants: true` and live
    /// debug assertions specifically so that engine panics surface. The route
    /// that replaces the session wholesale is exactly the route that can recover:
    /// it overwrites the `Option` and reads no game out of the poisoned value.
    ///
    /// Every other route **that takes the lock** keeps its 500, which is the half
    /// this test pins hardest — a blanket `into_inner()` would be a silent "carry
    /// on with a half-mutated game". (`GET /api/healthz` never locks and is
    /// unaffected; the pre-lock 400 paths keep their 400. S5 re-review LOW 12.)
    ///
    /// The recovery's *atomicity* is a separate property, pinned by
    /// `test_poison_recovery_is_atomic_when_the_rebuild_fails`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_post_game_recovers_from_a_poisoned_lock() {
        let state = shared_state();
        let first = new_game(&state).await;
        let first_seq = seq(&first);

        // The only way a `std::sync::Mutex` becomes poisoned: a panic while the
        // guard is held. The panic message and its stderr trace are EXPECTED
        // output of this test, not a failure.
        let handle = {
            let session = state.session.clone();
            std::thread::spawn(move || {
                let _guard = session.lock().expect("not poisoned yet");
                panic!("deliberate: poisoning the session lock for the recovery test");
            })
        };
        assert!(
            handle.join().is_err(),
            "the helper thread must have panicked"
        );
        assert!(state.session.is_poisoned(), "the lock must now be poisoned");

        // Every reading route is 500 — asserted BEFORE the recovery, because the
        // recovery clears the flag.
        let (status, err) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err["kind"], "session_poisoned");
        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": first_seq, "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{err}");
        assert_eq!(err["kind"], "session_poisoned");
        let (status, err) = post_json(&state, "/api/game/mulligan", json!({ "take": false })).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{err}");
        assert_eq!(err["kind"], "session_poisoned");

        // The subject: a new game is still startable.
        let (status, fresh) = post_json(&state, "/api/game", json!({})).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the one route that overwrites the session must recover: {fresh}"
        );
        assert_eq!(command_count(&fresh), 0);
        // MEDIUM 1's guarantee survives the recovery: `next_seq_base()` — which
        // reads the single `u64` high-water mark, and nothing else (re-review
        // LOW 9) — is the one thing read out of the poisoned session, precisely
        // so a stale tab cannot be revived by a panic.
        assert!(
            seq(&fresh) > first_seq,
            "the wire seq must still be monotonic across a poisoned rebuild"
        );

        // And the whole surface is usable again — the flag was cleared, not
        // merely bypassed for one request.
        assert!(!state.session.is_poisoned());
        let (status, view) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        assert_eq!(seq(&view), seq(&fresh));
    }

    // ── 7c ────────────────────────────────────────────────────────────────────

    /// **S5 re-review MEDIUM 1**: the poison recovery is **atomic**.
    ///
    /// The first fix cycle cleared the poison flag *before* `session::new_game`,
    /// and `new_game` is fallible on a **client-supplied seed**: `deck::
    /// basics_for_colors` pads a colourless commander's deck with Forests, whose
    /// green identity violates CR 903.5c, so `validate_deck` refuses the seat and
    /// `SetupError::InvalidDeck` `?`s out of the handler. `*guard = Some(play)`
    /// never ran, so the half-mutated session survived **with the flag cleared** —
    /// the next `GET /api/game` answered 200 off a session the crate itself calls
    /// untrustworthy, where before the "fix" it answered 500.
    ///
    /// Observed, not reasoned to. Against the pre-fix ordering this test's final
    /// `GET` returned **200 OK** with `summary.command_count == 0` and a live
    /// `decision`; the seed sweep that found the failing seeds reported 7 failures
    /// in 180 `(players, seed)` pairs (`players=2, seed=17` among them).
    ///
    /// The fix is structural rather than an ordering convention: the recovery arm
    /// `take()`s the corrupt session in the same straight-line block that clears
    /// the flag, with no fallible operation between, so "the flag is clear" and
    /// "there is no untrustworthy session left to read" cannot come apart.
    ///
    /// # Maintenance: this test is coupled to `OOS-M11-6`
    ///
    /// The **only** way this test makes `session::new_game` fail from client
    /// input is the CR 903.5c Forest padding filed on this branch as
    /// `OOS-M11-6` (`docs/audits/decision-point-audit.md` §8.1). Closing that
    /// seed may make `new_game` infallible from a client-supplied seed and leave
    /// this test with no trigger — at which point the first assertion below goes
    /// red rather than the test rotting into a vacuous pass, which is why this is
    /// a note and not a blocker. Whoever closes `OOS-M11-6` needs a new way to
    /// fail a rebuild inside the lock; there is no other one today.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_poison_recovery_is_atomic_when_the_rebuild_fails() {
        let state = shared_state();
        new_game(&state).await;

        let handle = {
            let session = state.session.clone();
            std::thread::spawn(move || {
                let _guard = session.lock().expect("not poisoned yet");
                panic!("deliberate: poisoning the session lock for the atomicity test");
            })
        };
        assert!(handle.join().is_err());
        assert!(state.session.is_poisoned());

        // A rebuild that fails *inside the lock*, after the recovery has run.
        //
        // PB-DX4 (2026-08-01): this used to rely on `players: 2, seed: 17` drawing a
        // colourless commander so `validate_deck` refused the padded Forests — the
        // OOS-M11-6 bug the maintenance note above warned this test was coupled to. That
        // bug is fixed and nothing a client can send fails a rebuild any more, so the
        // trigger is explicit now. See `api::post_game`'s injection point.
        let (status, err) = post_json(
            &state,
            "/api/game",
            json!({ "players": 2, "seed": REBUILD_FAILURE_SEED }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "the rebuild must fail here, or this test proves nothing: {err}"
        );
        assert_eq!(err["kind"], "setup_failed");

        // The subject. The corrupt session must be gone, not merely unflagged.
        let (status, after) = get_json(&state, "/api/game").await;
        assert_ne!(
            status,
            StatusCode::OK,
            "a failed rebuild must not leave the poisoned session readable: {after}"
        );
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(after["kind"], "no_session");

        // Non-vacuous, and the property the recovery exists for: the surface is
        // still usable, and a *successful* rebuild still works.
        assert!(!state.session.is_poisoned());
        let (status, fresh) = post_json(&state, "/api/game", json!({})).await;
        assert_eq!(status, StatusCode::OK, "{fresh}");
        assert_eq!(command_count(&fresh), 0);
    }

    // ── 7d ────────────────────────────────────────────────────────────────────

    /// **S5 third audit LOW 4**: the *healthy-path* half of the same property —
    /// a rebuild that fails must leave a **running** game exactly as it was.
    ///
    /// `post_game`'s healthy arm only *peeks* at the seq counter (`as_ref`,
    /// never `take`), so `session::new_game`'s `?` cannot destroy a live game on
    /// its way out. That was true in the code and asserted nowhere, and the
    /// round-2 finding was a claim-vs-code gap on this same arm — so the prose
    /// is now backed by a run.
    ///
    /// Same `OOS-M11-6` coupling as the test above: `players: 2, seed: 17` is
    /// the only client-reachable way to make the rebuild fail.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_a_failed_rebuild_leaves_a_running_game_untouched() {
        let state = shared_state();
        let view = new_game(&state).await;
        let pass = action_indices(&view, "PassPriority")[0];

        // Play one action first, so "unchanged" is distinguishable from "wiped
        // and silently rebuilt at the defaults" — a fresh session would report
        // `command_count == 0`, which is what a brand-new game reports too.
        let (status, live) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&view), "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{live}");
        let (live_seq, live_commands) = (seq(&live), command_count(&live));
        assert_eq!(live_commands, 4, "the game really is in progress");

        // The failing rebuild, on a HEALTHY lock this time. Explicit trigger since
        // PB-DX4 closed OOS-M11-6 — see the sibling test above.
        let (status, err) = post_json(
            &state,
            "/api/game",
            json!({ "players": 2, "seed": REBUILD_FAILURE_SEED }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "the rebuild must fail here, or this test proves nothing: {err}"
        );
        assert_eq!(err["kind"], "setup_failed");

        // The subject: the running game survived the failed rebuild intact.
        let (status, after) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{after}");
        assert_eq!(
            seq(&after),
            live_seq,
            "the outstanding decision is the same"
        );
        assert_eq!(
            command_count(&after),
            live_commands,
            "no play was discarded"
        );
        assert_eq!(after["summary"]["players"], PLAYERS, "still the same table");
        assert_eq!(after["summary"]["seed"], SEED, "not rebuilt at seed 17");

        // Non-vacuous: it is still the same *playable* game, not just the same
        // numbers — the decision it is holding is still answerable.
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": live_seq, "action_index": 0 }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{ok}");
        assert!(command_count(&ok) > live_commands);
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

    // ── 9 ─────────────────────────────────────────────────────────────────────

    /// **S5 review LOW 8**, widened by the re-review (**MEDIUM 2 / LOW 5 /
    /// LOW 6**): the no-socket rule, machine-enforced across the whole crate.
    ///
    /// Session plan §7 constraint 1 says no test in this crate may open a
    /// listening socket — an agent context that starts the real server binary is
    /// SIGKILLed (the replay-viewer 137 note in `memory/gotchas-infra.md`). Until
    /// LOW 8 that rule was prose in a doc comment, held by review alone, in a
    /// project that machine-enforces its invariants everywhere else (SR-2, SR-3,
    /// SR-5, SR-6, SR-9a…). Sessions 6 and 7 add tests to this crate.
    ///
    /// # Three things the first version got wrong
    ///
    /// 1. **The cut landed in prose (MEDIUM 2).** It was a bare
    ///    `source.find(marker)`, and the module doc comment above spells the
    ///    attribute out — so the "test region" began at that *paragraph*, not at
    ///    the attribute. It passed only because all four symbols are typed to the
    ///    left of the marker inside that one sentence; rewording it would have
    ///    turned the gate red against its own documentation, and the failure
    ///    message would have made deleting the gate look like the fix. The cut is
    ///    now **line-anchored** — see [`test_region`] — so a `///` line cannot be
    ///    it.
    /// 2. **It read one file (LOW 5).** The rule is crate-wide and the README
    ///    called it crate-wide, but the gate read `main.rs` alone: a
    ///    `#[cfg(test)] mod tests` in `api.rs`, `session.rs` or `view.rs`, or any
    ///    file under `tests/`, was unchecked. It now walks **every `.rs` file**
    ///    under the crate's `src/` and `tests/`, rooted at `CARGO_MANIFEST_DIR`
    ///    (a compile-time constant, so the walk does not depend on the process's
    ///    working directory) and recursively, so a file Session 6 or 7 adds is
    ///    covered without editing this test. A file under `tests/` is checked
    ///    **in full** — an integration test carries no `#[cfg(test)]` attribute
    ///    because the whole file is already test code.
    /// 3. **Its non-vacuity guard was itself satisfiable by prose (LOW 6).** It
    ///    asserted each needle appeared *above* the cut — but "above" includes
    ///    the paragraph naming all four, so renaming the serving entry point
    ///    everywhere except that paragraph left the guard green while the needle
    ///    matched nothing real. Guard (c) now searches **comment-stripped** code.
    ///    (Note the phrasing of that sentence: this doc comment sits *below* the
    ///    cut, so it may not spell any of the four out. The widened gate caught
    ///    a draft of it that did — which is the first thing it ever found.)
    ///
    /// # What the third audit found (MEDIUM 1): a silent skip
    ///
    /// [`test_region`] returning `""` used to mean "no test code here", and the
    /// loop below `continue`d on it **with no signal**. But `""` really means
    /// "no spelling of the attribute that this function recognises", which is a
    /// strictly larger set. Two forms fell into the gap and were **observed to
    /// pass a forbidden symbol through**, not reasoned about: a `src/` file that
    /// is the body of a `#[cfg(test)] mod tests;` split (the file itself carries
    /// no attribute), and `#[cfg(all(test, feature = "…"))]`. Both were given a
    /// `TcpLis`+`tener` call and the gate stayed green.
    ///
    /// Two repairs, and the claim about coverage is now narrower:
    ///
    /// * [`test_region`] recognises the `#[cfg(all(test` prefix as well.
    /// * A `src/` file whose **code** (not prose — see [`code_only`]) is
    ///   test-shaped but whose region is empty is now a **failure**, not a skip.
    ///   So a spelling this function does not understand is loud.
    ///
    /// The honest statement of coverage is therefore: a file Session 6 or 7 adds
    /// is either checked, or the gate goes red naming it. It is *not* "every
    /// arrangement is silently understood".
    ///
    /// # Self-reference
    ///
    /// The gate lives *inside* a region it checks, so every needle would match
    /// its own source if written plainly. Each is therefore assembled from two
    /// halves with `concat!`, which produces the whole symbol at compile time
    /// while the file itself contains only the halves. Keep any needle you add in
    /// that form — a plainly-written one turns this test red against itself, and
    /// the obvious "fix" is to delete the gate.
    #[test]
    fn test_no_socket_symbol_appears_in_the_test_region() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_rs_files(&root.join("src"), &mut sources);
        // `tests/` is separate because an **integration** test file carries no
        // `#[cfg(test)]` attribute — the whole file is test code. Observed, not
        // assumed: a `tests/tmp_probe.rs` containing a forbidden symbol was run
        // against a version of this gate that looked for the attribute in every
        // file, and it stayed green. Hence `whole_file`.
        let mut integration: Vec<(String, String)> = Vec::new();
        collect_rs_files(&root.join("tests"), &mut integration);

        let forbidden = [
            concat!("TcpLis", "tener"),
            concat!("axum::", "serve"),
            concat!("asyn", "c_main"),
            concat!("bi", "nd"),
        ];

        // MEDIUM 1: an unrecognised spelling must be loud, not skipped. A `src/`
        // file whose CODE is test-shaped — an attribute, a `fn test_`, a test
        // module — but whose region is empty is a file this gate does not
        // understand, and silently skipping it is exactly how a forbidden symbol
        // gets in. Prose does not count: `api.rs`'s module doc names
        // `#[tokio::test]` and has no tests at all, which is why the search runs
        // over [`code_only`] output.
        for (path, source) in &sources {
            if !test_region(source).is_empty() {
                continue;
            }
            let code = code_only(source);
            let shaped = ["#[test]", "#[tokio::test", "fn test_", "mod tests"]
                .into_iter()
                .find(|marker| code.contains(marker));
            assert!(
                shaped.is_none(),
                "{path} contains test-shaped code ({:?}) but no test region this gate \
                 recognises, so it would be skipped in silence. Either give it a \
                 line-anchored `#[cfg` attribute this gate's `test_region` understands, \
                 or teach `test_region` the spelling. Session plan §7 constraint 1 \
                 applies to every test in this crate, however it is arranged.",
                shaped.unwrap_or_default()
            );
        }

        let regions = sources
            .iter()
            .map(|(path, source)| (path, test_region(source)))
            .chain(
                integration
                    .iter()
                    .map(|(path, source)| (path, source.as_str())),
            );

        let mut files_with_a_region = 0;
        for (path, region) in regions {
            if region.is_empty() {
                continue;
            }
            files_with_a_region += 1;
            for needle in forbidden {
                assert!(
                    !region.contains(needle),
                    "session plan §7 constraint 1: {needle:?} must not appear below a \
                     test-module attribute, and it does in {path}. Every HTTP test drives \
                     `build_router` through `tower::ServiceExt::oneshot`; an agent context \
                     that starts the real server executable is SIGKILLed."
                );
            }
        }

        // ── non-vacuity, three directions ──
        // (a) The walk saw the files it is supposed to see. A path that silently
        //     resolved to nothing would otherwise be a gate over zero bytes.
        let seen: BTreeSet<&str> = sources
            .iter()
            .filter_map(|(p, _)| p.rsplit('/').next())
            .collect();
        for expected in ["main.rs", "api.rs", "session.rs", "view.rs"] {
            assert!(
                seen.contains(expected),
                "the source walk missed {expected}; it saw {seen:?}"
            );
        }
        assert!(
            files_with_a_region >= 1,
            "no file in the crate has a test-module attribute — the gate checked nothing"
        );

        // (b) The cut in THIS file landed on the attribute, not on the paragraph
        //     above that spells it out. Asserted directly, because that mistake
        //     is exactly what the re-review found and it is invisible in a green
        //     run otherwise.
        let main_src = sources
            .iter()
            .find(|(p, _)| p.ends_with("main.rs"))
            .map(|(_, s)| s.as_str())
            .expect("main.rs is in the walk");
        let marker = concat!("#[cfg", "(test)]");
        let region = test_region(main_src);
        assert!(
            region.starts_with(marker),
            "the cut must land on the attribute itself, not inside prose"
        );
        let cut = main_src.len() - region.len();
        let above = &main_src[..cut];
        assert!(
            above.contains(marker),
            "this file's doc comment spells the attribute out above the cut; if that \
             stops being true the line-anchored cut is no longer being exercised and \
             this gate has stopped proving anything about MEDIUM 2"
        );
        assert!(
            region.len() > main_src.len() / 4,
            "the checked region of main.rs is implausibly small: {} of {} bytes",
            region.len(),
            main_src.len()
        );

        // (c) Every needle occurs above the cut in real CODE — `main`'s own
        //     serving path uses all four. Comments AND string literals are
        //     blanked first (see [`code_only`]): the doc comment above names all
        //     four, so an un-stripped search would pass on prose alone (LOW 6);
        //     and the serving entry point's `format!("Failed to bi"+"nd to
        //     {addr}")` is a *code* line whose string body satisfies the fourth
        //     needle by itself, so a line-comment-only stripper would let that
        //     guard pass on a diagnostic message rather than on the call (third
        //     audit LOW 2 — demonstrated both ways: with the real call removed
        //     and the message kept, the old stripper ran green and this one runs
        //     red). A misspelled needle, or a `concat!` producing a symbol no
        //     longer in the codebase, fails here.
        let code_above = code_only(above);
        for needle in forbidden {
            assert!(
                code_above.contains(needle),
                "{needle:?} occurs in no CODE line above the cut — the gate would be \
                 vacuous (prose mentioning it does not count)"
            );
        }
    }

    /// [`code_only`]'s three branches that no file in this crate exercises.
    ///
    /// Written because the alternative was a doc comment claiming block
    /// comments, raw strings and char literals are handled, with nothing
    /// checking it — the exact shape of defect this fix cycle exists to remove.
    /// Each case is asserted in **both** directions: the body disappears, and
    /// the code around it survives.
    #[test]
    fn test_code_only_blanks_comments_and_string_bodies() {
        // Block comment, nested — a `/* */` form of the module doc would
        // otherwise restore the vacuity guard (c) exists to close.
        let src = "let a = 1; /* secret /* deeper */ still */ let b = 2;";
        let out = code_only(src);
        assert!(!out.contains("secret"), "{out:?}");
        assert!(!out.contains("deeper"), "{out:?}");
        assert!(
            out.contains("let a = 1;") && out.contains("let b = 2;"),
            "{out:?}"
        );

        // Raw string at a non-zero hash count, containing a quote.
        let src = "let s = r#\"secret \"quoted\" text\"#; let c = 3;";
        let out = code_only(src);
        assert!(!out.contains("secret"), "{out:?}");
        assert!(out.contains("let c = 3;"), "{out:?}");

        // Char literals that a naive scanner desynchronises on: a quote and a
        // backslash. If either were mishandled the rest of the line would be
        // swallowed as string body, so asserting the tail survives is the test.
        let src = "if c == '\"' { keep_me(); } if d == '\\\\' { keep_me_too(); }";
        let out = code_only(src);
        assert!(out.contains("keep_me();"), "{out:?}");
        assert!(out.contains("keep_me_too();"), "{out:?}");

        // A lifetime is NOT a char literal — an unbounded "scan to the next
        // quote" would eat everything between two lifetimes on one line.
        let src = "fn f<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { secret_ident }";
        let out = code_only(src);
        assert!(out.contains("secret_ident"), "{out:?}");
        assert!(out.contains("&'a str"), "{out:?}");

        // Line comment anywhere on a line, not only at its start.
        let src = "let x = 1; // secret\nlet y = 2;";
        let out = code_only(src);
        assert!(!out.contains("secret"), "{out:?}");
        assert!(
            out.contains("let x = 1;") && out.contains("let y = 2;"),
            "{out:?}"
        );

        // Newlines survive, so a failure message's line structure is intact.
        assert_eq!(code_only("a\n/* x\ny */\nb").lines().count(), 4);
    }

    /// Every `.rs` file under `dir`, recursively, as `(path, contents)`.
    ///
    /// A directory that does not exist contributes nothing: `tests/` is absent
    /// today and Sessions 6/7 may add it, and the gate must cover it the moment
    /// it appears without needing to be edited. Paths are sorted so a failure
    /// message is reproducible.
    fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{} is unreadable: {e}", path.display()));
                out.push((path.display().to_string(), text));
            }
        }
    }

    /// The slice of `source` at and below its first **line-anchored** test
    /// `cfg` attribute, or `""` when the file has none.
    ///
    /// "Line-anchored" means the attribute is the first non-whitespace text on
    /// its line, which is true of a real attribute and false of every mention
    /// inside a `///`, `//!` or `//` comment. That distinction is the whole
    /// repair for re-review MEDIUM 2. `starts_with` rather than equality so the
    /// one-line `#[cfg(test)] mod tests {` form is caught too.
    ///
    /// Two spellings are recognised: the plain attribute, and the `#[cfg(all(test`
    /// prefix that covers `#[cfg(all(test, feature = "…"))]` (third audit
    /// MEDIUM 1). Anything else still yields `""` — but `""` is no longer a
    /// silent skip: the caller fails on a file whose code is test-shaped and
    /// whose region is empty, so an unrecognised spelling is loud.
    fn test_region(source: &str) -> &str {
        let markers = [concat!("#[cfg", "(test)]"), concat!("#[cfg", "(all(test")];
        let mut offset = 0usize;
        for line in source.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if markers.iter().any(|m| trimmed.starts_with(m)) {
                return &source[offset..];
            }
            offset += line.len();
        }
        ""
    }

    /// `source` with every comment body and every string-literal body blanked to
    /// spaces, leaving code. Newlines survive, so line structure is preserved.
    ///
    /// Both non-vacuity searches above look for a *symbol*, and neither must be
    /// satisfiable by text that merely spells it. Two ways that happens here:
    /// prose (this file's own module doc names all four forbidden symbols), and
    /// string literals — the serving entry point's failure message contains the
    /// fourth needle as message text. Blanking both is what makes guard (c) a
    /// claim about the serving path rather than about a sentence.
    ///
    /// (Both mentions above are deliberately periphrastic. This function sits
    /// below the cut, so it may not spell any of the four symbols out — and an
    /// earlier draft of this very doc comment did, which the gate caught. That
    /// is now the second time it has found a fault in its own documentation.)
    ///
    /// Handles `//` line comments anywhere on a line, **nested** `/* */` block
    /// comments, `"…"` strings with `\` escapes, raw strings at any hash count
    /// (`r"…"`, `r#"…"#`), and char literals — including `'"'` and `'\\'`, which
    /// a naive scanner would let desynchronise the string tracking (lifetimes
    /// are told apart from char literals by looking for the closing quote).
    ///
    /// **Which of those the crate's own source actually exercises, stated
    /// because the distinction is this fix cycle's whole subject.** The slice
    /// guard (c) feeds in is `main.rs` above the cut, and the files the
    /// test-shaped check feeds in are `api.rs` / `session.rs` / `view.rs`.
    /// Between them those contain line comments and ordinary strings and
    /// **nothing else** — no block comment, no raw string, no char literal. So
    /// three of the branches above are defensive against source this crate does
    /// not yet contain, and would otherwise be an untested claim in a doc
    /// comment. They are pinned directly by
    /// [`test_code_only_blanks_comments_and_string_bodies`] instead.
    ///
    /// **Residual, stated rather than glossed.** This is a lint over source
    /// text, not a Rust lexer. Text produced by a macro is invisible to it by
    /// construction, and so is anything pulled in by `include_str!`. Neither
    /// occurs in this crate; if one ever does, the guard weakens silently, which
    /// is the standing hazard of every source-text gate in this project.
    fn code_only(source: &str) -> String {
        let chars: Vec<char> = source.chars().collect();
        let n = chars.len();
        let mut out = String::with_capacity(source.len());
        let mut i = 0usize;
        // Blank `count` characters starting at `i`, keeping any newline among
        // them so line structure survives.
        let blank = |out: &mut String, from: usize, to: usize| {
            for &c in &chars[from..to] {
                out.push(if c == '\n' { '\n' } else { ' ' });
            }
        };
        while i < n {
            let c = chars[i];
            let next = chars.get(i + 1).copied();
            // Line comment: blank to (not including) the newline.
            if c == '/' && next == Some('/') {
                let start = i;
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                blank(&mut out, start, i);
                continue;
            }
            // Block comment, nested per the Rust grammar.
            if c == '/' && next == Some('*') {
                let start = i;
                let mut depth = 1usize;
                i += 2;
                while i < n && depth > 0 {
                    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                blank(&mut out, start, i);
                continue;
            }
            // Raw string: `r`, any number of `#`, then `"`.
            if c == 'r' {
                let mut j = i + 1;
                while j < n && chars[j] == '#' {
                    j += 1;
                }
                if j < n && chars[j] == '"' {
                    let hashes = j - i - 1;
                    let start = i;
                    i = j + 1;
                    while i < n {
                        if chars[i] == '"'
                            && chars[i + 1..].iter().take(hashes).all(|&h| h == '#')
                            && i + 1 + hashes <= n
                        {
                            i += 1 + hashes;
                            break;
                        }
                        i += 1;
                    }
                    blank(&mut out, start, i.min(n));
                    continue;
                }
            }
            // Ordinary string literal.
            if c == '"' {
                let start = i;
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                blank(&mut out, start, i.min(n));
                continue;
            }
            // Char literal vs lifetime. A char literal is a quote, then either
            // ONE character or an escape sequence, then a closing quote — so the
            // scan is tightly bounded. `'a` in `&'a str` is a lifetime and falls
            // through to be emitted as code; an unbounded "scan to the next
            // quote" would swallow it and everything up to the next lifetime on
            // the line.
            if c == '\'' {
                let close = if chars.get(i + 1) == Some(&'\\') {
                    // `'\n'`, `'\\'`, `'\u{1F600}'` — an escape body is short.
                    (i + 2..(i + 12).min(n)).find(|&j| chars[j] == '\'')
                } else if chars.get(i + 2) == Some(&'\'') {
                    Some(i + 2)
                } else {
                    None
                };
                if let Some(j) = close {
                    blank(&mut out, i, j + 1);
                    i = j + 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }
}

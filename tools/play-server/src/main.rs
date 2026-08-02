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
        .route("/game/report", get(api::get_report))
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

    use std::collections::{BTreeSet, HashMap, HashSet};

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
    use mtg_view_model::{StateViewModel, Viewer};
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
    /// every seat) dealt the human a hand with no Dispel in it at all. It then became
    /// `Cast Drown in Ichor` at [`SEED`] within 48 steps.
    ///
    /// CARDS-2 (2026-08-02, `scutemob-181`) re-derived it again, and had to move the SEED
    /// as well as the spell: after the printed-field repairs, a fresh sweep of
    /// `seed` ∈ 0..24 × `develop` ∈ {false, true} found that **seed 0 reaches no targeted
    /// cast at all within 300 decisions**, so no step budget would have rescued the old
    /// pin. The fixture now rides [`TARGET_SEED`], which the same sweep chose for the
    /// X-value test, so one observation serves both. Dispatch is `{W}` "tap target
    /// creature" (CR 601.2c) — a player is not a creature, which is the property the caller's
    /// `422` depends on.
    ///
    /// Re-observed **four times** in this one batch, and the fourth is the instructive one. The
    /// first three followed from card-def repairs moving the deal. The fourth followed from
    /// merging `main`: **SIM-1** (`scutemob-175`) taught the provider to offer commander casts,
    /// which changes what every seeded game does from turn one, and the third-pass reviewer
    /// caught it by building the merge tree and running it — the branch was green on its own
    /// and red on the merge. So the rule has a second half: these pins are a function of the
    /// whole corpus **and of the provider**, and a branch that re-derives them must re-derive
    /// them again after any merge that touches `legal_actions.rs`.
    ///
    /// Choosing a seed was not free, and the reason is worth recording. Several otherwise
    /// usable seeds reach a targeted cast, are **offered** it, and then have the engine refuse
    /// it with "player does not have enough mana to pay the cost" once five sources are tapped
    /// — the provider's colour-blind affordability shortcut offering what the engine rejects,
    /// i.e. playtest finding **F4** / **OOS-CARDS2-9** reproducing on several independent
    /// seeds. Another (Flame Jab, "any target") would have made the `422` assertion pass for
    /// the wrong reason, since a player IS a legal target for it. So the sweep checks that the
    /// engine actually ACCEPTS the cast, and the spell must be one a player cannot be a legal
    /// target of. Dispatch is `{W}` "Tap target creature" (CR 601.2c).
    const TARGETED_SPELL: &str = "Cast Dispatch";

    /// Drive the seed-pinned opening until the human is offered a **targeted**
    /// spell.
    ///
    /// Observed, not assumed; the panic below prints the whole action list if the
    /// opening ever changes.
    ///
    /// The caller feeds the found action a `Player` target, which must be ILLEGAL for
    /// [`TARGETED_SPELL`] — that is the whole point of the test. It is derived from the
    /// constant, not restated here: whatever [`TARGETED_SPELL`] names must be a spell for
    /// which a player is not a legal target, and the sweep that chose it enforced that. If a future re-pin picks a spell that legally targets a player,
    /// the caller's `422` assertion fails loudly rather than passing for the wrong reason.
    async fn drive_to_targeted_spell(state: &SharedState) -> Value {
        // Delegates to `drive_until` rather than re-implementing the walk: CARDS-2 moved
        // this fixture onto TARGET_SEED (see [`TARGETED_SPELL`]), and the two fixtures now
        // share one observation and one driver. `drive_until` panics with the last decision
        // if the spell never appears, which is the same loud failure the hand-rolled loop
        // gave.
        drive_until(state, TARGET_SEED, false, |v| {
            action_index_by_label(v, TARGETED_SPELL).is_some()
        })
        .await
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
        // NOT `summary.seed` — review MR-M11-01 removed it from the seat payload
        // (it reconstructs every other seat's hidden zones). See
        // `test_mr_m11_01_seat_payload_carries_no_reconstruction_key`.
        assert!(view["summary"]["seed"].is_null());
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
        //
        // CARDS-2 (2026-08-02, `scutemob-181`): re-derived. This batch flipped ZERO
        // completeness markers, and every seeded deck still re-dealt — so the guard these
        // pins were written under ("re-read when a card-def batch flips a marker") was too
        // narrow. `deck.rs::random_deck` draws its commander from the cards that are
        // `Complete` **and Legendary and a Creature**, and then fills the deck by *colour
        // identity*, which is computed from the mana cost. A printed-field repair moves both
        // inputs without touching a marker: correcting three type lines moved the commander
        // pool 91 -> 90 (+Akroma, Angel of Fury, which really is Legendary; -Overlord of the
        // Hauntwoods and -Prosperous Innkeeper, neither of which is), and correcting mana
        // costs moved colour identities (Braided Net {2} -> {2}{U} is no longer colourless).
        // A shorter index into `rng.random_range(0..commanders.len())` re-picks every seat.
        // The durable form of the rule: **these pins are a function of the corpus, not of
        // the completeness markers** — re-observe after any card-def batch.
        //
        // The same batch then demonstrated the rule twice over: a later pass demoted
        // `cyber_conversion` and `exalted_angel` (two `Complete` defs implementing oracle text
        // their cards do not have), and that re-dealt every seat AGAIN — this time through the
        // channel the old comment did anticipate. Both channels are real; neither is the whole
        // rule.
        assert_eq!(
            own_names,
            vec![
                "Helm of the Host",
                "Solemn Simulacrum",
                "Simic Signet",
                "Fierce Empath",
                "Master Biomancer",
                "Cankerbloom",
                "Momentous Fall",
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
            // and secrets-count pins, this is a function of the whole card corpus — the
            // commander pool and every colour identity — and re-deals whenever a card-def
            // batch moves any of that, marker flip or not. Re-read it off a real run.
            // (PB-DX4, 2026-08-01: was "Dispel". CARDS-2, 2026-08-02: was "In Garruk's
            // Wake"; see the exact-hand pin above for why a batch that flipped no marker
            // still moved it.)
            "Human-1 draws Basilisk Collar",
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
        assert!(view["summary"]["seed"].is_null(), "MR-M11-01");
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
    /// [`TARGETED_SPELL`] is Dispatch, "tap target creature" (CR 601.2c); a player is
    /// not a creature, so `handle_cast_spell`'s target validation refuses it with
    /// `GameStateError::InvalidTarget`. (This paragraph named Dispel for two batches after
    /// the constant had moved on — it is derived from the constant now, not restated.)
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
        // Seed-pinned: the cards across the three bot hands collapse to this many
        // distinct names the human has no entitlement to. Asserted exactly, so a
        // future change that quietly empties this set fails here rather than
        // turning the search below into a no-op. (This pin, like the exact-hand
        // pin above, is a function of the whole card corpus — commander pool and
        // colour identities, not just the completeness markers — so ANY card-def
        // batch can re-deal it. CARDS-2, 2026-08-02: 14 -> 18, re-read off a real
        // run; see the exact-hand pin for the mechanism.)
        assert_eq!(
            secrets.len(),
            18,
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
        // "not rebuilt at the sentinel seed" is now read off `command_count` and the
        // live decision below rather than off `summary.seed`, which MR-M11-01 removed
        // from the seat payload.
        assert!(after["summary"]["seed"].is_null(), "MR-M11-01");

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

    // ── S7: targeting, combat and choice ──────────────────────────────────────

    /// The S7 fixtures, **observed rather than chosen**.
    ///
    /// Every one of these numbers was read off a real playthrough by a temporary
    /// `#[ignore]`d probe driven through `oneshot` (no port bound), which swept
    /// `players` ∈ {2, 4} × `seed` ∈ 0..12 and reported, per game, whether the
    /// human seat was ever offered a `DeclareAttackers`, a `DeclareBlockers`, or
    /// a `CastSpell` whose target query returned a non-empty candidate list. The
    /// probe was then deleted. Do not guess replacements for these: like the
    /// exact-hand and [`TARGETED_SPELL`] pins above, a completeness flip in any
    /// card-def batch re-deals every seeded deck and moves them. Re-observe.
    ///
    /// Seed 6 at four players is the only swept pair that reaches **both** halves
    /// of combat (attackers at turn 5, blockers at turn 6); seed 9 reaches a
    /// targeted removal spell with a real, legal creature candidate.
    const COMBAT_SEED: u64 = 6;
    // CARDS-2 (2026-08-02, `scutemob-181`): 9 -> 20, re-observed by the sweep described
    // above (extended to `seed` ∈ 0..24 because 0..12 no longer contained a usable pair).
    // Seed 9 still stops `option_with_targets` on *something* — Deserted Temple's "untap
    // target land" — which is why two of the three tests below stayed green while the
    // X-value one failed: that predicate matches any action with candidates, and the deal
    // had quietly retargeted this fixture from a spell onto an activated ability.
    //
    // Final value 1, from a POST-MERGE sweep of `seed` ∈ 0..24 that checked THREE properties
    // at once,
    // because the fixture serves three tests: a slot with ≥2 candidates (the order-perturbation
    // test), a targeted CastSpell reachable with five untapped sources, and — measured, not
    // assumed — the engine actually ACCEPTING that cast afterwards. Only five seeds (1, 5, 13,
    // 21, 23) satisfied all three; five more reached a cast the engine then refused for want of
    // mana, which is OOS-CARDS2-9/F4 and not a property to build a fixture on. Post-merge the
    // qualifying set is seeds 1, 5, 13 and 21; Dispatch was chosen over Reanimate and Cyclonic
    // Rift because "tap target creature" is the property the `422` caller actually depends on
    // (Reanimate targets a card in a graveyard, Cyclonic Rift a nonland permanent — both would
    // still 422 on a player, but for a reason the test does not name).
    // SIM-2 (2026-08-02, `scutemob-176`): 1 -> 13, and this is the *second* re-derivation
    // in two days, by the second half of the rule stated above -- SIM-2 changes
    // `legal_actions.rs::can_afford` (it now asks the residual solver one question instead
    // of a pool shortcut OR a whole-cost solve) and `mana_solver.rs` (production counted in
    // mana, layer-resolved sources, unactivatable abilities excluded), so every seeded game
    // diverges from turn one. Swept `seed` in 0..24 by running these four tests against each:
    // **only seed 13 passed all four**, which is a stricter check than the property sweep it
    // replaces because it asserts the fixtures themselves rather than their preconditions.
    //
    // Seed 1 does not merely miss the fixture now -- it drives the engine into an i32
    // **overflow panic** at `layers.rs`'s `ModifyPower` (`Devilish Valet`, whose Alliance
    // trigger doubles its own power until end of turn; observed at power = delta =
    // 1_073_741_824 = 2^30). That is a pre-existing engine fragility on an unbounded
    // doubling, not a SIM-2 defect and not reachable through anything this batch wrote:
    // filed as **OOS-SIM2-5**. It is recorded here because "seed 1 panics" would otherwise
    // look like a property of this fixture rather than of the engine.
    const TARGET_SEED: u64 = 13;

    /// How many decisions the drivers below will answer before giving up. Chosen
    /// well above the observed cost of the slowest fixture (the X-value one needs
    /// ~150 decisions to reach five untapped mana sources) so a small shift in
    /// the deal does not turn a moved fixture into a timeout.
    const S7_MAX_STEPS: u32 = 700;

    /// Drive the human seat until `stop` accepts the payload.
    ///
    /// The policy is the dullest one that makes progress: take a land drop if
    /// offered; else, when `develop` is set, cast the first spell that announces
    /// nothing (no targets, no `{X}`, no modes) so the board actually fills;
    /// else pass; else answer whatever is first, which covers the blocking
    /// decisions a priority-shaped policy cannot answer at all (e.g.
    /// `DiscardToHandSize` at cleanup). It **never declares attackers or
    /// blockers**, so a combat fixture is reached by the game arriving at it
    /// rather than by this driver steering into it.
    ///
    /// `develop` is a parameter and not a constant because the two fixture
    /// families were observed under different policies and pinning each to the
    /// one it was actually seen under is cheaper than re-sweeping:
    ///
    /// * **`true`** for the combat fixtures — with no creatures cast, the human
    ///   seat is never offered a `DeclareAttackers` at all
    ///   (`legal_actions.rs` emits it only when `eligible` is non-empty), and
    ///   seed 6 ran 700 decisions without one.
    /// * **`false`** for the targeting and X fixtures, which need a *stable*
    ///   board late enough to have five untapped mana sources; casting along the
    ///   way spends the mana and removes the creature the removal spell targets.
    ///
    /// Panics with the last decision if the fixture is not reached, so a moved
    /// deal is a loud failure naming what it did offer.
    async fn drive_until(
        state: &SharedState,
        seed: u64,
        develop: bool,
        stop: impl Fn(&Value) -> bool,
    ) -> Value {
        let (status, mut view) = post_json(
            state,
            "/api/game",
            json!({ "players": PLAYERS, "seed": seed }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "POST /api/game failed: {view}");

        for _ in 0..S7_MAX_STEPS {
            if view["decision"].is_null() {
                break;
            }
            if stop(&view) {
                return view;
            }
            let actions = decision(&view)["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            // Candidates in policy order, not one pick: the FIRST choice may be an action the
            // provider offers and the engine then refuses, and the driver must be able to fall
            // through to the next rather than abort the whole fixture.
            //
            // This is not hypothetical and not a test smell. An **Aura** carries its target
            // requirement in `KeywordAbility::Enchant(...)`, which `casting.rs` special-cases
            // (CR 303.4a, "Aura spells require exactly one target"); the *provider* does not
            // read that keyword, so the offer reports `target_min: 0` — "announces nothing" —
            // and the engine rejects the cast with a 422. The develop policy below selects on
            // `target_min == 0`, so it walks straight into it: CARDS-2 (2026-08-02) re-dealt
            // seed 6 and the driver died on "Cast Hyena Umbra". That is a live browser-client
            // defect (a human clicking any Aura gets a 422) of the same family as playtest
            // findings F4 and F9, filed as **OOS-CARDS2-4**, and it is the provider's bug to
            // fix — not something a fixture should be reshaped around.
            //
            // A rejection is skipped, never silently swallowed: if every candidate is refused
            // the loop makes no progress and the panic below reports the fixture unreached with
            // the last payload, which is the same loud failure as before.
            let candidates: Vec<Value> = actions
                .iter()
                .filter(|a| a["kind"] == "PlayLand")
                .chain(actions.iter().filter(|a| {
                    develop
                        && a["kind"] == "CastSpell"
                        && a["target_min"] == 0
                        && a["needs_x"] == false
                        && a["modes"].as_array().is_some_and(|m| m.is_empty())
                }))
                .chain(actions.iter().filter(|a| a["kind"] == "PassPriority"))
                .chain(actions.iter().take(1))
                .cloned()
                .collect();

            let mut advanced = false;
            for pick in &candidates {
                let (status, next) = post_json(
                    state,
                    "/api/game/action",
                    json!({ "seq": seq(&view), "action_index": pick["index"] }),
                )
                .await;
                if status == StatusCode::OK {
                    view = next;
                    advanced = true;
                    break;
                }
                // Only the ONE documented false offer is skipped. Anything else is a NEW
                // provider/engine disagreement — the same SR-38 class — and tolerating it
                // would make this driver absorb exactly the bug it exists downstream of.
                // A blanket skip would be near-unfailable, because `PassPriority` is in the
                // candidate chain and essentially always succeeds.
                // Named, filed defects only. Each is a case of the provider offering an
                // action `process_command` then refuses — SR-38's "never offer what the engine
                // rejects". Anything NOT on this list fails loudly: a fixture driver that
                // tolerates arbitrary refusals absorbs exactly the class it sits downstream of.
                //
                // The last three entries are ONE defect with three symptoms, not three
                // defects — **OOS-CARDS2-9**: the provider's affordability check counts mana
                // abilities it could not legally activate. It checks that a source is untapped
                // and nothing else, so an unmet `activation_condition` (CR 602.5b), a
                // summoning-sick creature (CR 302.6) and — per the previously filed **SG-1** —
                // a `life_cost` it cannot pay all inflate the pool it believes in. Playtest
                // finding **F4** is the fourth symptom (Sol Ring credited as one mana). The fix
                // is one place: make the solver ask whether the ability is *activatable*, not
                // whether its source is untapped.
                //
                // **SIM-2 (`scutemob-176`) CLOSED symptoms 1 and 2 and DELETED their entries**,
                // adopting the policy the sibling list in
                // `crates/simulator/tests/local_game_playthrough.rs` states and enforces: *"an
                // excusal list is a debt register with a maturity date — delete the entry the
                // moment it stops firing"*. A dead entry is not inert here: this list has no
                // staleness assertion (the sibling's does), so carrying
                // `"mana ability activation condition not met"` and `"summoning sickness and
                // cannot tap for mana"` after the fix would silently drive a future REGRESSION
                // of either past instead of failing this driver. SIM-2's first pass argued for
                // keeping them as a record; the record belongs in the seed row (`OOS-CARDS2-9`),
                // not in a live allowlist.
                const KNOWN_FALSE_OFFERS: &[&str] = &[
                    // OOS-CARDS2-4: an Aura's target requirement lives in
                    // `KeywordAbility::Enchant(...)`, which `casting.rs` special-cases (CR
                    // 303.4a) and the provider never reads, so the offer says `target_min: 0`.
                    "Aura spells require exactly one target",
                ];
                let reason = next["error"].as_str().unwrap_or_default();
                assert!(
                    KNOWN_FALSE_OFFERS.iter().any(|k| reason.contains(k)),
                    "driving seed {seed}: the engine refused {} with an UNEXPECTED reason \
                     {reason:?}. Only the shapes in KNOWN_FALSE_OFFERS are filed; a new refusal \
                     means the provider is offering something else the engine rejects, and that \
                     is a finding, not something to drive past.",
                    pick["label"]
                );
            }
            assert!(
                advanced,
                "driving seed {seed}: the engine refused every action the provider offered at \
                 this decision — {}",
                decision(&view)
            );
        }
        panic!(
            "seed {seed} did not reach the fixture within {S7_MAX_STEPS} decisions; \
             last payload was {view}"
        );
    }

    /// Every action of the current decision, as a `Vec<Value>`.
    fn options(view: &Value) -> Vec<Value> {
        decision(view)["actions"]
            .as_array()
            .expect("actions is an array")
            .clone()
    }

    /// The first action with a target slot holding at least `least` candidates.
    ///
    /// `least` is a parameter rather than a bare "non-empty" test because of a
    /// perturbation check that would otherwise have been reported as passing:
    /// reversing the candidate order inside `action_option_view` left
    /// [`test_action_option_target_slots_match_engine_query`] **green**, because
    /// the first fixture it stopped on had a single-candidate slot and reversing
    /// a one-element list changes nothing. That test now asks for `least = 2`, so
    /// its per-slot order assertion is actually exercised.
    fn option_with_targets(view: &Value, least: usize) -> Option<Value> {
        options(view).into_iter().find(|a| {
            a["target_slots"].as_array().is_some_and(|slots| {
                slots
                    .iter()
                    .any(|s| s["candidates"].as_array().unwrap().len() >= least)
            })
        })
    }

    /// The first action carrying the named combat payload (`"attack"` / `"block"`).
    fn option_with_combat(view: &Value, field: &str) -> Option<Value> {
        options(view).into_iter().find(|a| !a[field].is_null())
    }

    /// The candidate ids of one wire slot, in order.
    fn slot_ids(slot: &Value) -> Vec<u64> {
        slot["candidates"]
            .as_array()
            .expect("a slot has a candidates array")
            .iter()
            .map(|t| t["id"].as_u64().expect("id is a number"))
            .collect()
    }

    // ── 10 ────────────────────────────────────────────────────────────────────

    /// **Plan item 1**: `ActionOptionView.target_slots` is the engine's own
    /// answer, not a client-side re-derivation.
    ///
    /// The wire payload is compared against a *second, independent* call to
    /// `mtg_engine::spell_target_requirements` + `legal_targets_per_slot`, made
    /// here against the same `GameState` the server rendered from and reached
    /// through the session directly rather than through HTTP. Slot count, order
    /// within each slot, and the `(min, max)` range must all agree exactly.
    ///
    /// That is a real check rather than a tautology because the two sides differ
    /// in everything except the query: the wire side went through
    /// `decision_view` -> `action_option_view` -> `target_options` -> serde ->
    /// HTTP -> `serde_json::Value`, and reads `TargetOptionView::id`, while this
    /// side reads `mtg_engine::Target` straight out of the engine. A rendering
    /// bug that dropped, reordered or duplicated a slot shows up as an
    /// inequality; a `modes_chosen`/`alt_cost` argument that did not match what
    /// `params.rs` builds the `Command` with shows up as a different candidate
    /// set.
    ///
    /// CR 601.2c.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_action_option_target_slots_match_engine_query() {
        let state = shared_state();
        let view = drive_until(&state, TARGET_SEED, false, |v| {
            option_with_targets(v, 2).is_some()
        })
        .await;
        let option = option_with_targets(&view, 2).expect("the driver stopped on one");
        let index = option["index"].as_u64().expect("index is a number") as usize;

        let wire_slots = option["target_slots"]
            .as_array()
            .expect("target_slots is an array")
            .clone();

        // Non-vacuity, stated first so a fixture that has drifted to an
        // all-empty slot list cannot make the comparison below pass trivially.
        assert!(
            wire_slots
                .iter()
                .any(|s| !s["candidates"].as_array().unwrap().is_empty()),
            "fixture drift: no slot has a candidate. Action: {option}"
        );

        let guard = state.session.lock().expect("session lock");
        let play = guard.as_ref().expect("a session exists");
        let game_state = play.game.state();
        let pending = play.pending.as_ref().expect("a decision is outstanding");
        let action = pending
            .actions
            .get(index)
            .expect("the wire index addresses a real action");

        let (requirements, source) = match action {
            mtg_simulator::LegalAction::CastSpell { card, .. } => (
                mtg_engine::spell_target_requirements(game_state, *card, &[], None),
                *card,
            ),
            mtg_simulator::LegalAction::ActivateAbility {
                source,
                ability_index,
                ..
            } => (
                mtg_engine::ability_target_requirements(game_state, *source, *ability_index),
                *source,
            ),
            other => panic!("fixture drift: expected a targeting action, got {other:?}"),
        };
        let expected =
            mtg_engine::legal_targets_per_slot(game_state, pending.player, source, &requirements);

        assert_eq!(
            wire_slots.len(),
            expected.len(),
            "slot count disagrees with the engine query"
        );
        for (i, (wire, engine)) in wire_slots.iter().zip(expected.iter()).enumerate() {
            let engine_ids: Vec<u64> = engine
                .iter()
                .map(|t| match t {
                    mtg_engine::Target::Object(id) => id.0,
                    mtg_engine::Target::Player(p) => p.0,
                })
                .collect();
            assert_eq!(
                slot_ids(wire),
                engine_ids,
                "slot {i} disagrees with the engine query (order matters: the client \
                 submits `targets` in slot order)"
            );

            // CR 601.2c per slot. `UpToN { count }` is one requirement worth up
            // to `count` targets, so a client holding only the collective range
            // cannot tell which slot the slack belongs to — which is why
            // `TargetSlotView` carries its own. Checked against the same engine
            // function over a one-element slice.
            let (slot_min, slot_max) =
                mtg_engine::target_count_range(std::slice::from_ref(&requirements[i]));
            assert_eq!(
                wire["min"].as_u64(),
                Some(slot_min as u64),
                "slot {i} min disagrees with the engine"
            );
            assert_eq!(
                wire["max"].as_u64(),
                Some(slot_max as u64),
                "slot {i} max disagrees with the engine"
            );
        }

        let (min, max) = mtg_engine::target_count_range(&requirements);
        assert_eq!(option["target_min"].as_u64(), Some(min as u64));
        assert_eq!(option["target_max"].as_u64(), Some(max as u64));

        // The per-slot ranges must sum to the collective one — the property a
        // client relies on when it validates a pick locally before submitting.
        let (sum_min, sum_max) = wire_slots.iter().fold((0u64, 0u64), |(lo, hi), s| {
            (
                lo + s["min"].as_u64().expect("min is a number"),
                hi + s["max"].as_u64().expect("max is a number"),
            )
        });
        assert_eq!((sum_min, sum_max), (min as u64, max as u64));
    }

    // ── CARDS-1 (OOS-M11-10) ────────────────────────────────────────────────────

    /// Build a minimal `GameState`: Skullclamp on the battlefield under `p1`, plus
    /// one creature `p1` controls and one `p2` controls, `p1` holding priority
    /// with exactly the {1} generic mana Skullclamp's Equip costs.
    ///
    /// Mirrors `crates/engine/tests/primitives/cards1_equip_target_repair.rs`'s
    /// `setup_skullclamp_scenario` (same card, same shape) — duplicated rather
    /// than shared, because `tools/play-server` and `crates/engine/tests` are
    /// separate compilation units with no shared test-support crate (exactly the
    /// reasoning that file's own T7 gives for not reusing `crates/engine/tests/core`
    /// logic across its own two test binaries). The layer-resolved +1/-1 bonus is
    /// not wired here (no `register_static_continuous_effects` call): this
    /// fixture only needs the equip ability to be legally *targetable*, not to
    /// resolve — that half is already proven by the engine test file's T3.
    fn setup_skullclamp_view_scenario() -> (
        mtg_engine::GameState,
        mtg_engine::ObjectId,
        mtg_engine::ObjectId,
        mtg_engine::ObjectId,
        mtg_engine::PlayerId,
        mtg_engine::PlayerId,
    ) {
        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let defs: HashMap<String, mtg_engine::CardDefinition> = mtg_engine::all_cards()
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect();

        let skullclamp = mtg_engine::enrich_spec_from_def(
            mtg_engine::ObjectSpec::card(p1, "Skullclamp")
                .in_zone(mtg_engine::ZoneId::Battlefield)
                .with_card_id(mtg_engine::card_name_to_id("Skullclamp")),
            &defs,
        );
        let p1_creature = mtg_engine::ObjectSpec::creature(p1, "P1 Bear", 2, 2);
        let p2_creature = mtg_engine::ObjectSpec::creature(p2, "P2 Bear", 2, 2);

        let mut state = mtg_engine::GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(mtg_engine::CardRegistry::new(mtg_engine::all_cards()))
            .object(skullclamp)
            .object(p1_creature)
            .object(p2_creature)
            .active_player(p1)
            .at_step(mtg_engine::Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(mtg_engine::ManaColor::Colorless, 1);
        state.turn_mut().priority_holder = Some(p1);

        let find = |name: &str, controller: mtg_engine::PlayerId| -> mtg_engine::ObjectId {
            state
                .objects()
                .iter()
                .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
                .map(|(id, _)| *id)
                .unwrap_or_else(|| panic!("object '{name}' controlled by {controller:?} not found"))
        };
        let skullclamp_id = find("Skullclamp", p1);
        let p1_creature_id = find("P1 Bear", p1);
        let p2_creature_id = find("P2 Bear", p2);

        (state, skullclamp_id, p1_creature_id, p2_creature_id, p1, p2)
    }

    /// **CARDS-1 (OOS-M11-10), browser-path half.** Engine coverage already
    /// proves `mtg_engine::ability_target_requirements` reports Skullclamp's
    /// equip slot once its def declares it
    /// (`crates/engine/tests/primitives/cards1_equip_target_repair.rs` T5). This
    /// test proves the same thing survives the view/wire layer this crate
    /// renders for the Svelte client — the layer at which the original defect
    /// was actually observed: `ActionOptionView.target_slots` was empty, so the
    /// browser picker never asked for a target, the activation validated with
    /// zero declared targets, and the attach silently fizzled with the cost
    /// already paid.
    ///
    /// Driven entirely in-process, without HTTP: `mtg_simulator::StubProvider`
    /// (the same provider `LocalGame` uses) computes the real `LegalAction` list
    /// off the built `GameState`, and `view::decision_view` — the exact function
    /// `api::seat_view` calls to build the payload a real session sends — renders
    /// it. No test in this crate opens a listening socket (see
    /// `test_no_socket_symbol_appears_in_the_test_region`); this one does not
    /// even use `tower::ServiceExt::oneshot`, since there is no HTTP surface
    /// between "a `GameState`" and "the wire `DecisionView`" worth crossing here.
    ///
    /// CR 702.6a ("Attach this permanent to target creature you control") / CR
    /// 602.2b (activating an ability announces targets the same way CR 601.2c
    /// requires for casting a spell).
    #[test]
    fn test_cards1_skullclamp_activate_ability_view_carries_target_slot() {
        use mtg_simulator::LegalActionProvider as _;

        let (state, skullclamp_id, p1_creature_id, p2_creature_id, p1, p2) =
            setup_skullclamp_view_scenario();

        let actions = mtg_simulator::StubProvider.legal_actions(&state, p1);
        let (index, _) = actions
            .iter()
            .enumerate()
            .find(|(_, a)| {
                matches!(
                    a,
                    mtg_simulator::LegalAction::ActivateAbility { source, .. }
                        if *source == skullclamp_id
                )
            })
            .expect(
                "StubProvider must offer Skullclamp's equip ability as an ActivateAbility \
                 action -- p1 has priority, {1} generic mana in pool, and a legal target \
                 creature on the battlefield",
            );

        let pending = mtg_simulator::PendingDecision {
            seq: 0,
            player: p1,
            kind: mtg_simulator::DecisionKind::Priority,
            actions,
        };

        let player_names: HashMap<mtg_engine::PlayerId, String> =
            [(p1, "Human-1".to_string()), (p2, "Bot-2".to_string())]
                .into_iter()
                .collect();

        let state_view =
            StateViewModel::from_game_state_for(&state, &player_names, Viewer::Seat(p1));
        let names = view::NameIndex::from_view(&state_view);
        let decision = view::decision_view(&pending, 0, &state, &names, &player_names);

        // Serialized to a raw `serde_json::Value`, not inspected as the Rust
        // struct — the wire shape is what the Svelte picker actually reads, and
        // that is the layer this test is proving something about.
        let wire = serde_json::to_value(&decision).expect("DecisionView serializes");
        let action = wire["actions"]
            .as_array()
            .expect("actions is an array")
            .get(index)
            .expect("the found index addresses a real wire action")
            .clone();

        assert_eq!(action["kind"], "ActivateAbility");
        assert_eq!(action["object_id"].as_u64(), Some(skullclamp_id.0));

        // Regression floor for OOS-M11-10 itself: this is the assertion that
        // would have caught the original defect at the layer the playtest
        // actually observed it. With the pre-fix `targets: vec![]`,
        // `target_slots` here is empty and the picker never asks.
        let target_slots = action["target_slots"]
            .as_array()
            .expect("target_slots is an array");
        assert_eq!(
            target_slots.len(),
            1,
            "OOS-M11-10: Skullclamp's ActivateAbility option must carry exactly one target \
             slot once the def declares its TargetRequirement -- an empty target_slots is \
             exactly the wire shape of the silent-fizzle defect (the picker never asks, the \
             activation validates with zero declared targets, and the attach never happens). \
             Wire action: {action}"
        );

        let candidates = target_slots[0]["candidates"]
            .as_array()
            .expect("candidates is an array");

        // Non-vacuity floor, stated first so the scoping checks below cannot pass
        // trivially against an empty list.
        assert!(
            !candidates.is_empty(),
            "OOS-M11-10: the slot's candidate list must be non-empty -- Skullclamp's own \
             controller (p1) controls a legal creature target"
        );

        let candidate_ids: Vec<u64> = candidates
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();

        // CR 702.6a "target creature you control": scoped to the activating seat.
        assert!(
            candidate_ids.contains(&p1_creature_id.0),
            "the activating player's own creature must be an offered candidate; got \
             {candidate_ids:?}"
        );
        assert!(
            !candidate_ids.contains(&p2_creature_id.0),
            "an opponent's creature must NOT be an offered candidate (CR 702.6a 'you \
             control') -- this is exactly the assertion that would have caught the browser \
             picker never asking for a target at all; got {candidate_ids:?}"
        );
    }

    // ── 11 ────────────────────────────────────────────────────────────────────

    /// **Plan item 2, first half**: a human can attack through the API, and the
    /// engine really registers it.
    ///
    /// The declaration is built entirely out of what the payload offered — the
    /// first `attack.eligible` entry and the first `attack.targets` entry,
    /// echoing that target's `value` verbatim rather than reconstructing the
    /// `AttackTarget` encoding — which is exactly what the frontend picker does.
    ///
    /// The assertion is on `GameEvent::AttackersDeclared` reaching the client as
    /// an `EventView`, not merely on a 200: a 200 is also what an empty
    /// declaration returns (CR 508.1 makes attacking with nothing legal), so a
    /// status-only test would pass against precisely the S6 bug this session
    /// closes.
    ///
    /// CR 508.1 / CR 508.1a.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_declare_attackers_through_api_emits_attackers_declared() {
        let state = shared_state();
        let view = drive_until(&state, COMBAT_SEED, true, |v| {
            option_with_combat(v, "attack").is_some()
        })
        .await;
        let option = option_with_combat(&view, "attack").expect("the driver stopped on one");
        let attack = &option["attack"];

        let eligible = attack["eligible"].as_array().expect("eligible is an array");
        let targets = attack["targets"].as_array().expect("targets is an array");
        assert!(
            !eligible.is_empty() && !targets.is_empty(),
            "fixture drift: `legal_actions.rs` only emits this action when both are \
             non-empty, so an empty one here means the payload is wrong: {attack}"
        );
        // Every combatant is labelled from the seat-redacted view, never an id.
        for c in eligible {
            assert!(
                c["label"].as_str().is_some_and(|l| !l.is_empty()),
                "an eligible attacker has no label: {c}"
            );
        }

        let attacker = eligible[0]["id"].clone();
        let target = targets[0]["value"].clone();
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": { "attackers": [[attacker, target]] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the declaration was refused: {after}"
        );

        let kinds: Vec<String> = after["events"]
            .as_array()
            .expect("events is an array")
            .iter()
            .map(|e| e["kind"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(
            kinds.iter().any(|k| k == "AttackersDeclared"),
            "no AttackersDeclared reached the client; events were {kinds:?}"
        );
    }

    // ── 12 ────────────────────────────────────────────────────────────────────

    /// **Plan item 2, second half**: a blocker the decision never offered is a
    /// **400**, refused by this crate before the engine sees it.
    ///
    /// Two halves, and the second is what stops the first being satisfiable by a
    /// validator that rejects everything:
    ///
    /// 1. An id outside `block.eligible` is refused with `bad_params`, the
    ///    message names CR 509.1a, and — checked afterwards — the decision is
    ///    still outstanding with no command applied, so the refusal cost the
    ///    human nothing.
    /// 2. The pairing the payload actually offered is **not** refused as
    ///    `bad_params`. Deliberately not asserted to be a 200: `eligible` is the
    ///    provider's own `can_block` list and the engine may still refuse the
    ///    pair on a rule the provider does not model (flying, menace, a
    ///    restriction), which would be a legitimate 422. What matters is that it
    ///    got past this crate's own gate and reached engine code.
    ///
    /// CR 509.1 / CR 509.1a.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_declare_blockers_rejects_ineligible_blocker() {
        let state = shared_state();
        let view = drive_until(&state, COMBAT_SEED, true, |v| {
            option_with_combat(v, "block").is_some()
        })
        .await;
        let option = option_with_combat(&view, "block").expect("the driver stopped on one");
        let block = &option["block"];
        let eligible = block["eligible"].as_array().expect("eligible is an array");
        let attackers = block["attackers"]
            .as_array()
            .expect("attackers is an array");
        assert!(
            !eligible.is_empty() && !attackers.is_empty(),
            "fixture drift: this action is only emitted with both non-empty: {block}"
        );

        let at_seq = seq(&view);
        let before = command_count(&view);
        let attacker = attackers[0]["id"].clone();

        // An id no object in this game has. Derived from the payload rather than
        // hardcoded, so it cannot collide with a real object however the deal moves.
        let offered: Vec<u64> = eligible
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        let bogus = offered.iter().copied().max().unwrap_or(0) + 100_000;

        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": at_seq,
                "action_index": option["index"],
                "params": { "blockers": [[bogus, attacker]] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "an ineligible blocker is a client error, not an engine rejection: {err}"
        );
        assert_eq!(err["kind"], "bad_params");
        let message = err["error"].as_str().expect("an error message");
        assert!(
            message.contains("509.1a") && message.contains(&bogus.to_string()),
            "the message must name the rule and the offending object: {message:?}"
        );

        // The refusal touched nothing.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(seq(&still), at_seq, "the decision is still outstanding");
        assert_eq!(command_count(&still), before, "no command was applied");

        // Control: the offered pairing is not stopped by *this* gate.
        let (status, out) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": at_seq,
                "action_index": option["index"],
                "params": { "blockers": [[eligible[0]["id"], attacker]] },
            }),
        )
        .await;
        assert_ne!(
            status,
            StatusCode::BAD_REQUEST,
            "the validator rejected a pairing the server itself offered: {out}"
        );
    }

    // ── 12b (UI-3, `scutemob-180`) ────────────────────────────────────────────

    /// The seed whose first `DeclareAttackers` offer has **more than one**
    /// eligible attacker, so an attack can genuinely be split across two
    /// different defending players.
    ///
    /// Not [`COMBAT_SEED`], and the difference is the whole point of having a
    /// second constant. Observed by a throwaway probe over `seed` ∈ 0..24 at
    /// [`PLAYERS`] seats, driving each to its first attack offer and recording
    /// the offer's shape: **every** seed offers 3 player targets (the three
    /// opponents, which is just CR 506.2), and **only seed 21 offers 2 eligible
    /// attackers** — every other seed offers exactly 1, because at the turn the
    /// first attack becomes available the boards hold a single creature. The
    /// probe was then deleted.
    ///
    /// With one attacker, "attacker → defender" degenerates to "there is a
    /// defender", and a mapping bug that *swapped two attackers' defenders*
    /// would pass. Re-observe rather than guess if this stops splitting: like
    /// [`COMBAT_SEED`] and [`TARGET_SEED`], it is a function of the whole card
    /// corpus, and a completeness flip in any card-def batch re-deals it.
    const UI3_SPLIT_COMBAT_SEED: u64 = 21;

    /// **UI-3 AC 6006**: after attackers are declared, the seat payload says
    /// **which attacker is attacking which defending player**, and after blockers
    /// are declared it says which blocker is assigned to which attacker.
    ///
    /// CR 508.1a (each attacker is declared as attacking one defending player or
    /// planeswalker) / CR 509.1a (each blocker is declared as blocking one or
    /// more attacking creatures).
    ///
    /// # Why this test exists at all, given `CombatView` shipped in M9.5
    ///
    /// The playtest finding is "not clear which card are attacking which player
    /// after attackers declared", and the cause is **not** missing data —
    /// `StateViewModel::combat` has carried `attackers[].target` and
    /// `attackers[].blockers[]` since M9.5, seat-redacted by
    /// `redact::redact_combat`. The play client simply never rendered it: it
    /// composed `$viewer/StateView.svelte`, which does not include
    /// `CombatView.svelte` (the replay viewer wires those two together in its own
    /// `App.svelte`). UI-3 renders it, and this test pins the payload half so the
    /// component has something guaranteed to render.
    ///
    /// # What makes it discriminating rather than a shape check
    ///
    /// The declaration deliberately **splits the attackers across two different
    /// defending players**, which is why it runs on [`UI3_SPLIT_COMBAT_SEED`]
    /// rather than on [`COMBAT_SEED`]: with a single eligible attacker — which
    /// is what every other swept seed offers — "attacker → defender" collapses
    /// to "there is a defender", and a bug that paired two attackers with each
    /// other's defenders would pass. The split is asserted to have happened, so
    /// this cannot silently weaken back into a shape check if the deal moves.
    ///
    /// A mapping bug that collapsed every attacker onto one defender also passes
    /// any "combat is present and non-empty" assertion and fails this one.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui3_combat_view_maps_attackers_to_defenders_and_blockers() {
        let state = shared_state();
        let view = drive_until(&state, UI3_SPLIT_COMBAT_SEED, true, |v| {
            option_with_combat(v, "attack").is_some()
        })
        .await;
        let option = option_with_combat(&view, "attack").expect("the driver stopped on one");
        let attack = &option["attack"];
        let eligible = attack["eligible"].as_array().expect("eligible is an array");
        let targets = attack["targets"].as_array().expect("targets is an array");

        // Player targets only: a planeswalker defender is a different rendering
        // path (`"planeswalker:<name>"`), and this fixture has no planeswalker.
        let player_targets: Vec<&Value> =
            targets.iter().filter(|t| t["kind"] == "player").collect();
        assert!(
            !eligible.is_empty() && !player_targets.is_empty(),
            "fixture drift: expected at least one attacker and one player defender: {attack}"
        );

        // Pair attacker i with defender (i mod defenders): with two or more
        // eligible attackers and two or more defenders this genuinely splits, and
        // it degrades to "everyone at the same defender" rather than failing when
        // the deal offers only one of either.
        let mut declared: Vec<(u64, String)> = Vec::new();
        let mut pairs: Vec<Value> = Vec::new();
        for (i, creature) in eligible.iter().enumerate() {
            let defender = player_targets[i % player_targets.len()];
            let id = creature["id"].as_u64().expect("id is a number");
            let label = defender["label"].as_str().expect("a label").to_string();
            pairs.push(json!([id, defender["value"]]));
            declared.push((id, label));
        }
        let distinct_defenders: std::collections::BTreeSet<&String> =
            declared.iter().map(|(_, d)| d).collect();

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": { "attackers": pairs },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the declaration was refused: {after}"
        );

        let combat = &after["state"]["combat"];
        assert!(
            !combat.is_null(),
            "CR 506.1: combat state must be on the payload while combat is in \
             progress — without it the client has nothing to render. Payload: {}",
            after["state"]["turn"]
        );

        let human = after["summary"]["human_name"]
            .as_str()
            .expect("human_name is on the summary");
        assert_eq!(
            combat["attacking_player"], human,
            "the human declared the attack, so they are the attacking player"
        );

        // Every declared pair appears, with the right defender.
        let rendered = combat["attackers"]
            .as_array()
            .expect("combat.attackers is an array");
        assert_eq!(
            rendered.len(),
            declared.len(),
            "every declared attacker must appear exactly once: declared {declared:?}, \
             rendered {rendered:?}"
        );
        for (id, defender) in &declared {
            let entry = rendered
                .iter()
                .find(|a| a["object_id"].as_u64() == Some(*id))
                .unwrap_or_else(|| {
                    panic!("attacker {id} was declared but is missing from combat: {rendered:?}")
                });
            assert_eq!(
                entry["target"].as_str(),
                Some(format!("player:{defender}").as_str()),
                "attacker {id} was declared attacking {defender} but the payload says \
                 {:?} — this is the exact 'which card is attacking which player' \
                 mapping the playtest could not see",
                entry["target"]
            );
            // The name is the seat-redacted one, never an id or a blank.
            assert!(
                entry["name"].as_str().is_some_and(|n| !n.is_empty()),
                "an attacker has no rendered name: {entry}"
            );
        }
        // The split is the point — see the doc block. Asserted rather than
        // reported, so a re-deal that reduces this seed to one attacker fails
        // loudly and gets a fresh sweep, instead of leaving a test that still
        // passes while checking a strictly weaker property.
        assert!(
            distinct_defenders.len() >= 2,
            "fixture drift: seed {UI3_SPLIT_COMBAT_SEED} no longer splits the attack across \
             two defenders (declared {declared:?}). Re-observe it — see \
             `UI3_SPLIT_COMBAT_SEED`'s doc for the probe. Without a split this test checks \
             only that *a* defender is named, not that the RIGHT one is."
        );

        // ── Second half: blockers ────────────────────────────────────────────
        //
        // The bots are the defenders, so their blocker declarations arrive
        // without this seat acting. Drive forward until either a blocker is
        // assigned or combat ends; both are legitimate outcomes of one deal, so
        // the assertion is on the SHAPE when blockers exist, and the absence of
        // blockers is reported rather than treated as a pass.
        let mut latest = after;
        let mut saw_blocker = false;
        for _ in 0..40 {
            let combat = &latest["state"]["combat"];
            if let Some(list) = combat["attackers"].as_array() {
                for entry in list {
                    let blockers = entry["blockers"].as_array().expect("blockers is an array");
                    for b in blockers {
                        saw_blocker = true;
                        assert!(
                            b["name"].as_str().is_some_and(|n| !n.is_empty()),
                            "CR 509.1a: a blocker assigned to attacker {} has no rendered \
                             name, so the client cannot show the assignment: {b}",
                            entry["object_id"]
                        );
                        assert!(
                            b["object_id"].as_u64().is_some(),
                            "a blocker carries no object id: {b}"
                        );
                    }
                }
            }
            if saw_blocker || latest["decision"].is_null() {
                break;
            }
            let Some(pass) = decision(&latest)["actions"]
                .as_array()
                .and_then(|a| a.iter().find(|o| o["kind"] == "PassPriority"))
                .cloned()
            else {
                break;
            };
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({ "seq": seq(&latest), "action_index": pass["index"] }),
            )
            .await;
            if status != StatusCode::OK {
                break;
            }
            latest = next;
        }
        // Asserted, not merely reported. Whether the defending bots block is
        // their choice, but the bots are deterministic in the seed, and this
        // seed does block — so leaving it unasserted would mean the blocker half
        // of AC 6006 is "covered" by a loop that is allowed to find nothing. If
        // a re-deal stops producing a block, that is a fixture to re-observe,
        // not a property to quietly drop.
        assert!(
            saw_blocker,
            "fixture drift: seed {UI3_SPLIT_COMBAT_SEED} no longer reaches a declared blocker \
             within 40 passes, so the CR 509.1a half of this test checked nothing. Re-observe \
             a seed that reaches both halves — see `UI3_SPLIT_COMBAT_SEED`'s doc."
        );
        eprintln!(
            "UI-3 combat fixture: seed {UI3_SPLIT_COMBAT_SEED}, {} attacker(s) across {} \
             defender(s), blockers exercised = {saw_blocker}",
            declared.len(),
            distinct_defenders.len()
        );
    }

    /// **UI-3 AC 6010**: every target candidate carries the seat it belongs to,
    /// so the picker can segment by player.
    ///
    /// `TargetOptionView::owner` is derived inside [`view::NameIndex`] from the
    /// **already-redacted** `StateViewModel` — the same walk that produces
    /// `label`. This test pins that it is populated for real candidates and that
    /// it agrees with the view model, which is what makes the segmentation a
    /// restatement of the payload rather than a client-side guess.
    ///
    /// The check that gives it teeth is the last one: the owner of a battlefield
    /// candidate must equal that permanent's **controller** in the redacted view
    /// (CR 109.4), not merely be *some* seat name. A grouping keyed on owner
    /// rather than controller would put a stolen creature in the wrong segment,
    /// and only this comparison catches that.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui3_target_options_carry_the_owning_seat() {
        let state = shared_state();
        let view = drive_until(&state, TARGET_SEED, false, |v| {
            option_with_targets(v, 1).is_some()
        })
        .await;
        let option = option_with_targets(&view, 1).expect("the driver stopped on one");

        // Controller of every battlefield permanent, straight off the payload's
        // own seat-redacted state — the second, independent source.
        let mut controller_of: HashMap<u64, String> = HashMap::new();
        let battlefield = view["state"]["zones"]["battlefield"]
            .as_object()
            .expect("battlefield is an object");
        for (seat, permanents) in battlefield {
            for p in permanents.as_array().expect("a permanent list") {
                let id = p["object_id"].as_u64().expect("object_id is a number");
                let controller = p["controller"]
                    .as_str()
                    .filter(|c| !c.is_empty())
                    .unwrap_or(seat.as_str());
                controller_of.insert(id, controller.to_string());
            }
        }
        let seats: std::collections::BTreeSet<&String> = view["state"]["players"]
            .as_object()
            .expect("players is an object")
            .keys()
            .collect();

        let mut checked = 0usize;
        for slot in option["target_slots"].as_array().expect("slots") {
            for candidate in slot["candidates"].as_array().expect("candidates") {
                let owner = candidate["owner"].as_str();
                if candidate["kind"] == "player" {
                    // A player target sorts into their own segment.
                    assert_eq!(
                        owner,
                        candidate["label"].as_str(),
                        "a player candidate's owner is that player: {candidate}"
                    );
                    checked += 1;
                    continue;
                }
                let id = candidate["id"].as_u64().expect("id is a number");
                if let Some(expected) = controller_of.get(&id) {
                    assert_eq!(
                        owner,
                        Some(expected.as_str()),
                        "CR 109.4: candidate {id} is controlled by {expected} in the \
                         redacted view, but the payload segments it under {owner:?}"
                    );
                    checked += 1;
                } else if let Some(o) = owner {
                    // Graveyard / command-zone / stack candidates: not on the
                    // battlefield, so the cross-check above cannot reach them —
                    // but the value must still be a seat at this table and not
                    // some other string.
                    assert!(
                        seats.contains(&o.to_string()),
                        "candidate {id} is segmented under {o:?}, which is not a seat: \
                         seats are {seats:?}"
                    );
                    checked += 1;
                }
            }
        }
        // Non-vacuity: without this the loop above passes for a payload whose
        // candidates all carry `owner: null`, which is precisely the regression
        // it exists to catch.
        assert!(
            checked > 0,
            "no candidate carried an owner — the segmentation has nothing to group on: {option}"
        );
    }

    // ── 13 ────────────────────────────────────────────────────────────────────

    /// **Architecture Invariant 7 at the target picker.**
    ///
    /// A target label is the fifth rendering site the S4 handoff warned about,
    /// and the S6 handoff's HIGH is the reason it gets its own test rather than
    /// riding on test 7's whole-body sweep: redaction follows the *rendering
    /// site*, not the zone, and a new site is a new place for it to be missed.
    ///
    /// # What each assertion is worth, stated because two of them are worth less
    /// # than they look
    ///
    /// 1. **Non-vacuity, first.** At least one target label is a real card name,
    ///    and no label is empty. Without this everything below passes for a
    ///    payload with no labels at all, which is how a redaction test rots into
    ///    a no-op.
    /// 2. **The substantive check**: every object label equals the name the
    ///    **seat-redacted** `StateViewModel` carries for that object id — the
    ///    *same* view `NameIndex` is built from, re-derived here from the session
    ///    rather than read off the payload. A label sourced from anywhere else —
    ///    `state.objects()`, the omniscient view, a raw `characteristics.name` —
    ///    fails this the moment the two disagree.
    /// 3. **A forward guard, not a live proof**: no name any *other* seat holds
    ///    in hand appears in any label. **This cannot currently fire, and saying
    ///    so is the point.** `legal_targets_per_slot` enumerates candidates from
    ///    Battlefield, Stack and Graveyard only (its own doc), and the combat
    ///    lists are battlefield creatures — every one a public zone. On top of
    ///    that, `redact::redact_hands` rewrites a hidden hand card's `object_id`
    ///    to 0, so no id collected here can key into a hand entry at all. The
    ///    assertion is kept because it is the check that *would* catch a future
    ///    widening of `legal_targets_per_slot` into a hidden zone, which is
    ///    exactly the change that would make this site leak. It is not evidence
    ///    that redaction works today.
    ///
    /// # The reachable case this fixture does not contain
    ///
    /// The one way a *target* label can differ between the omniscient and
    /// redacted views today is a **face-down battlefield permanent** (CR 708.2a:
    /// no name, so `non_empty` yields [`view::HIDDEN_LABEL`]). No seeded game in
    /// the S7 fixture sweep put one on the board, so assertion 2's inequality
    /// branch is unexercised — every id in this fixture happens to agree between
    /// the two views. Recorded rather than papered over: reaching it needs a
    /// fixture with a morph or a manifest, which this driver cannot steer to.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_target_option_labels_are_seat_redacted() {
        let state = shared_state();
        let view = drive_until(&state, TARGET_SEED, false, |v| {
            option_with_targets(v, 1).is_some()
        })
        .await;

        // Every label this decision renders for a target or a combatant, paired
        // with the object id it claims to name (`None` for a player label, whose
        // name is public by CR 108.1 and lives in a different index).
        let mut labels: Vec<(Option<u64>, String)> = Vec::new();
        for option in options(&view) {
            let mut push_slots = |slots: &Value| {
                for slot in slots.as_array().into_iter().flatten() {
                    for t in slot["candidates"].as_array().into_iter().flatten() {
                        let id = (t["kind"] == "object")
                            .then(|| t["id"].as_u64().expect("a candidate carries a numeric id"));
                        labels.push((id, t["label"].as_str().unwrap_or_default().to_string()));
                    }
                }
            };
            push_slots(&option["target_slots"]);
            for mode in option["modes"].as_array().into_iter().flatten() {
                push_slots(&mode["target_slots"]);
            }
            // `attack.eligible` / `block.eligible` / `block.attackers` are all
            // creatures; `attack.targets` is a player-or-planeswalker list, so its
            // entries carry their own `kind`.
            for field in ["attack", "block"] {
                for list in ["eligible", "targets", "attackers"] {
                    for c in option[field][list].as_array().into_iter().flatten() {
                        let is_object = c["kind"].is_null() || c["kind"] == "planeswalker";
                        let id = is_object
                            .then(|| c["id"].as_u64().expect("a combatant carries a numeric id"));
                        labels.push((id, c["label"].as_str().unwrap_or_default().to_string()));
                    }
                }
            }
        }

        let placeholders = [view::HIDDEN_LABEL, view::UNKNOWN_LABEL];
        assert!(
            labels
                .iter()
                .any(|(_, l)| !l.is_empty() && !placeholders.contains(&l.as_str())),
            "vacuous: no target label named anything. Labels: {labels:?}"
        );
        assert!(
            labels.iter().all(|(_, l)| !l.is_empty()),
            "a label was empty — a client would show a bare id: {labels:?}"
        );

        let guard = state.session.lock().expect("session lock");
        let play = guard.as_ref().expect("a session exists");
        let human = play.human;

        // Assertion 2, the substantive one: every object label equals the name
        // the SEAT-REDACTED view carries for that id. Re-derived from the session
        // rather than read off the payload, so this compares two independently
        // built things.
        let redacted = StateViewModel::from_game_state_for(
            play.game.state(),
            &play.names,
            Viewer::Seat(human),
        );
        let mut redacted_names: HashMap<u64, String> = HashMap::new();
        for permanents in redacted.zones.battlefield.values() {
            for p in permanents {
                redacted_names.insert(p.object_id, p.name.clone());
            }
        }
        for zone in [&redacted.zones.graveyard, &redacted.zones.hand] {
            for cards in zone.values() {
                for c in cards {
                    if !c.hidden {
                        redacted_names.insert(c.object_id, c.name.clone());
                    }
                }
            }
        }
        for item in &redacted.zones.stack {
            if let Some(source) = item.source_object_id {
                redacted_names.insert(source, item.source_name.clone());
            }
        }

        let mut checked = 0usize;
        for (id, label) in &labels {
            let Some(id) = id else { continue };
            let Some(expected) = redacted_names.get(id) else {
                continue;
            };
            let expected = if expected.is_empty() {
                view::HIDDEN_LABEL
            } else {
                expected.as_str()
            };
            assert_eq!(
                label, expected,
                "label for object {id} is not the seat-redacted view's name for it"
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "vacuous: no object label could be cross-checked against the redacted view"
        );

        // Assertion 3, the forward guard. See the doc comment: this cannot fire
        // today, and is kept for the widening that would make it able to.
        //
        // A name is only a secret if the seat has NO legitimate way to see it. CARDS-2
        // (2026-08-02) re-pinned TARGET_SEED and the new deal put a Forest in a bot's hand
        // while Forests stood on the battlefield, so the substring search reported the
        // battlefield label "Forest" as a hand leak. That is a false positive by
        // construction — a name is not private just because someone also holds a copy of it
        // — and it is the same excusal the sibling
        // `test_seat_view_over_http_contains_no_other_hand_card_names` already makes with
        // its `allowed` set. `redacted_names` above is exactly "every name this seat may
        // see", so subtract it.
        let omniscient = StateViewModel::from_game_state(play.game.state(), &play.names);
        let visible: BTreeSet<&str> = redacted_names.values().map(String::as_str).collect();
        let mut secrets: Vec<String> = Vec::new();
        for (owner, cards) in omniscient.zones.hand.iter() {
            if play.names.get(&human).is_some_and(|n| n == owner) {
                continue;
            }
            for card in cards {
                if !card.name.is_empty() && !visible.contains(card.name.as_str()) {
                    secrets.push(card.name.clone());
                }
            }
        }
        assert!(
            !secrets.is_empty(),
            "vacuous: the oracle found no hidden card to look for"
        );

        for (_, label) in &labels {
            for secret in &secrets {
                assert!(
                    !label.contains(secret.as_str()),
                    "target label {label:?} names {secret:?}, a card in another seat's hand"
                );
            }
        }
    }

    // ── 14 ────────────────────────────────────────────────────────────────────

    /// **Plan item 6**: an announced `x_value` reaches `CastSpellData.x_value`.
    ///
    /// Proven at the far end rather than at the boundary: the assertion reads the
    /// `Command` the engine actually applied out of `LocalGame::journal()`, so it
    /// covers the whole chain — `ActionParamsDto` -> `ActionParams` ->
    /// `params.rs`'s `CastSpell` arm -> `Command::CastSpell(CastSpellData)`.
    ///
    /// # Why the test taps mana by hand first
    ///
    /// **Historical as of S8 — `OOS-M11-8` is CLOSED and this is no longer a
    /// limitation.** It was one when the test was written:
    /// `LocalGame::auto_tap_commands_for` read `obj.characteristics.mana_cost` —
    /// the **printed** cost — and knew nothing about `cast.x_value`, so it tapped
    /// for `{1}{B}` and the engine then refused the cast for want of the extra
    /// `{3}` (observed, not inferred: the same submission without the manual taps
    /// answered **422 "player does not have enough mana to pay the cost"**).
    /// `auto_tap_commands_for` now adds `x_value * mana_cost.x_count` before it
    /// solves, and the crate README's limitation 6 records the closure.
    ///
    /// The manual taps are kept anyway, and deliberately: they make this test
    /// exercise the **pool** path — S3 made auto-tap conditional on the pool
    /// (`OOS-M11-2`'s pool half), so with the cost already covered
    /// `auto_tap_commands_for` returns `None` and the surplus stays available for
    /// X. The auto-tap path for `{X}` has its own probe,
    /// `local_game_human_actions.rs::test_s8_x_value_is_included_in_the_auto_tap_plan`;
    /// between them both halves are covered rather than one being retargeted onto
    /// the other.
    ///
    /// CR 107.3 / CR 601.2b.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_x_value_is_forwarded_to_cast_spell_data() {
        const X: u64 = 3;
        const SOURCES: usize = 5;

        let state = shared_state();
        // Wait for a board where the spell is castable AND enough untapped
        // sources exist to cover its cost plus X.
        // `option_with_targets` matches ANY action carrying candidates, and that is how this
        // test broke silently in CARDS-2: the re-dealt seed 9 offered no targeted cast, so
        // the predicate stopped on Deserted Temple's "untap target land" activated ability
        // instead, and the failure surfaced three assertions later as "the cast is still
        // offered after tapping" (it was never a cast). The predicate now says what the test
        // has always meant — a CastSpell — so a fixture that drifts off a spell fails in the
        // driver, naming the decision, rather than deep inside the body.
        let is_targeted_cast = |a: &Value| {
            a["kind"] == "CastSpell"
                && a["target_slots"].as_array().is_some_and(|slots| {
                    slots
                        .iter()
                        .any(|s| !s["candidates"].as_array().unwrap().is_empty())
                })
        };
        let mut view = drive_until(&state, TARGET_SEED, false, |v| {
            options(v).iter().any(is_targeted_cast)
                && options(v)
                    .iter()
                    .filter(|a| a["kind"] == "TapForMana")
                    .count()
                    >= SOURCES
        })
        .await;

        let label = options(&view)
            .into_iter()
            .find(is_targeted_cast)
            .expect("the driver stopped on a targeted cast")["label"]
            .as_str()
            .expect("a label")
            .to_string();

        // Mana abilities do not use the stack and do not pass priority, so the
        // human keeps the decision across each one — but each answer mints a new
        // `seq`, hence the re-read of the action list every iteration.
        for i in 0..SOURCES {
            let tap = options(&view)
                .into_iter()
                .find(|a| a["kind"] == "TapForMana")
                .unwrap_or_else(|| panic!("ran out of mana sources after {i}"));
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({ "seq": seq(&view), "action_index": tap["index"] }),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "tapping failed: {next}");
            view = next;
        }

        let cast = options(&view)
            .into_iter()
            .find(|a| a["label"] == label.as_str())
            .expect("the cast is still offered after tapping");
        let target = cast["target_slots"][0]["candidates"][0]["value"].clone();
        let target_id = cast["target_slots"][0]["candidates"][0]["id"]
            .as_u64()
            .expect("id is a number");

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": cast["index"],
                "params": { "targets": [target], "x_value": X },
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "the cast was refused: {after}");

        let guard = state.session.lock().expect("session lock");
        let play = guard.as_ref().expect("a session exists");
        let cast_command = play
            .game
            .journal()
            .iter()
            .rev()
            .find_map(|record| match &record.command {
                mtg_engine::Command::CastSpell(data) => Some(data.clone()),
                _ => None,
            })
            .expect("the applied command list contains the cast");

        assert_eq!(
            cast_command.x_value, X as u32,
            "the announced X did not reach CastSpellData"
        );
        assert_eq!(
            cast_command.targets,
            vec![mtg_engine::Target::Object(mtg_engine::ObjectId(target_id))],
            "the announced target did not reach CastSpellData"
        );
        assert_eq!(
            cast_command.player, play.human,
            "the command must name the human seat and no other"
        );
    }

    // ── Review MR-M11-01 (HIGH): the seat payload carries no reconstruction key ──

    /// **Architecture Invariant 7, at the reconstruction-key level rather than the
    /// card-name level** (review MR-M11-01).
    ///
    /// `GameSummary` used to ship `seed`. `setup::build_initial_state` is deterministic
    /// in its `LocalGameConfig` alone and `session::config_for` fixes every other input,
    /// so `(seed, players, mulligan_count)` rebuild **every other seat's opening hand
    /// and library order** — the exact pair the invariant names. It was on the default
    /// payload, on every response, and the frontend rendered it.
    ///
    /// Neither existing gate could see it: the HTTP leak scan looks for another seat's
    /// card *names*, and the source gate looks for omniscient *view-model entry points*.
    /// A seed is neither. This test is the gate for the third channel.
    ///
    /// It asserts over the **raw response text**, not over a parsed field, so it also
    /// catches the seed reappearing under a different key or nested somewhere else in
    /// the payload.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_mr_m11_01_seat_payload_carries_no_reconstruction_key() {
        // A seed that cannot collide with an ordinary small integer in the payload
        // (turn numbers, object ids, life totals, counts) — otherwise a substring hit
        // would be noise rather than a leak.
        const DISTINCTIVE_SEED: u64 = 987_654_321_987;
        let state = AppState::new(NewGameDefaults {
            players: PLAYERS,
            bot: BotKind::Heuristic,
            seed: DISTINCTIVE_SEED,
        });

        let (status, _) = post_json(&state, "/api/game", json!({})).await;
        assert_eq!(status, StatusCode::OK);

        for uri in ["/api/game"] {
            let (status, text) = get_raw(&state, uri).await;
            assert_eq!(status, StatusCode::OK);
            assert!(
                !text.contains(&DISTINCTIVE_SEED.to_string()),
                "{uri} leaks the seed, which reconstructs every other seat's hidden \
                 zones (Architecture Invariant 7, review MR-M11-01)"
            );
            let view: Value = serde_json::from_str(&text).expect("a seat view");
            assert!(view["summary"]["seed"].is_null());
        }

        // Non-vacuity, and the containment claim in one: the seed IS on the opt-in
        // report route, which is the documented exception. If this half ever fails the
        // test above has stopped meaning anything.
        let (status, report) = get_json(&state, "/api/game/report").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            report["seed"],
            json!(DISTINCTIVE_SEED),
            "the seed must still be on the deliberate exception, or this test is vacuous"
        );
    }

    // ── S8: the bug-report export (plan item 5) ───────────────────────────────

    /// `GET /api/game/report` carries the whole reproduction key and the fingerprints
    /// that make it checkable (`docs/mtg-engine-runtime-integrity.md` Layer 3).
    #[tokio::test(flavor = "multi_thread")]
    async fn test_s8_report_carries_the_reproduction_key() {
        let state = shared_state();
        let view = new_game(&state).await;
        // Pass once first, deliberately. A game parked on the human's very first
        // decision has applied ZERO commands (`advance()` stops before acting), so
        // the journal assertions below would pass vacuously against a fresh game —
        // and did, on the first run of this test.
        let pass = action_indices(&view, "PassPriority")[0];
        let (status, _) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": seq(&view), "action_index": pass }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, body) = get_json(&state, "/api/game/report").await;
        assert_eq!(status, StatusCode::OK);

        assert_eq!(body["seed"], json!(SEED));
        assert_eq!(body["config"]["players"], json!(PLAYERS));
        assert_eq!(body["config"]["human_seat"], json!(1));
        assert_eq!(body["config"]["bot"], json!("Heuristic"));
        assert_eq!(body["config"]["mulligan_count"], json!(0));

        // Fingerprints, read off the engine rather than hard-coded here: this test
        // pins that the report *reports* them, not what they currently are (the
        // `core` group's protocol/hash schema suites own that).
        assert_eq!(
            body["protocol_version"],
            json!(mtg_engine::PROTOCOL_VERSION)
        );
        assert_eq!(
            body["hash_schema_version"],
            json!(mtg_engine::HASH_SCHEMA_VERSION)
        );
        assert_eq!(
            body["protocol_fingerprint"],
            json!(mtg_engine::PROTOCOL_SCHEMA_FINGERPRINT)
        );

        let hash = body["state_hash"].as_str().expect("state_hash is a string");
        assert_eq!(hash.len(), 64, "32 bytes of BLAKE3 as lowercase hex");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Non-vacuity: a game that has already advanced to the human's first
        // decision has applied commands, and every one is in the journal with its
        // events.
        let journal = body["journal"].as_array().expect("journal is an array");
        assert!(
            !journal.is_empty(),
            "the journal must not be empty — `session::config_for` sets record_journal"
        );
        assert_eq!(
            journal.len(),
            body["command_count"].as_u64().unwrap() as usize,
            "one journal entry per applied command"
        );
        for entry in journal {
            assert!(entry["command"].is_object() || entry["command"].is_string());
            assert!(entry["events"].is_array());
            assert!(entry["turn"].is_number());
        }
    }

    /// The report is a **pure read**: it neither advances the game nor consumes the
    /// event lines `GET /api/game` has not shipped yet.
    ///
    /// The second half is the one worth a test. `seat_view` drains the journal
    /// through `take_new_records`, so an export that used the same accessor would
    /// silently swallow a client's next batch of history — a bug that shows up as
    /// "the event feed skipped a turn" long after the export.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_s8_report_is_a_pure_read() {
        let state = shared_state();
        let (status, first) = post_json(&state, "/api/game", json!({})).await;
        assert_eq!(status, StatusCode::OK);
        let seq_before = first["decision"]["seq"].clone();
        let commands_before = first["summary"]["command_count"].clone();

        let (status, _) = get_json(&state, "/api/game/report").await;
        assert_eq!(status, StatusCode::OK);

        let (status, after) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            after["decision"]["seq"], seq_before,
            "the outstanding decision must survive an export unchanged"
        );
        assert_eq!(
            after["summary"]["command_count"], commands_before,
            "the export must not advance the game"
        );
    }

    /// No session, no report — 404, the same answer `GET /api/game` gives, and for
    /// the same reason (the resource the route names does not exist).
    #[tokio::test(flavor = "multi_thread")]
    async fn test_s8_report_without_a_session_is_404() {
        let state = shared_state();
        let (status, body) = get_json(&state, "/api/game/report").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["kind"], json!("no_session"));
    }

    /// A mulligan changes the effective seed, and the report has to say so — the
    /// base `seed` alone no longer rebuilds the table (`setup::redeal` derives from
    /// `redeal_seed(seed, human_seat, mulligan_count)`), so `mulligan_count` is part
    /// of the reproduction key rather than a statistic.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_s8_report_tracks_the_mulligan_count() {
        let state = shared_state();
        let (status, _) = post_json(&state, "/api/game", json!({})).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = post_json(&state, "/api/game/mulligan", json!({"take": true})).await;
        assert_eq!(status, StatusCode::OK);

        let (status, body) = get_json(&state, "/api/game/report").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["seed"], json!(SEED), "the BASE seed is unchanged");
        assert_eq!(
            body["config"]["mulligan_count"],
            json!(1),
            "the redeal count is the other half of the effective seed"
        );
    }

    // ── Review MR-M11-05: omitting `params` and sending `{}` must agree ───────

    /// **Two spellings a client would call identical used to produce different game
    /// behaviour** (review MR-M11-05).
    ///
    /// `ActionRequest::params` is `#[serde(default)]`, so an **omitted** `params` key
    /// was built by `ActionParamsDto`'s *derived* `Default` — `auto_tap: false` —
    /// while a present-but-empty `"params": {}` ran serde's own field defaults and got
    /// `auto_tap: true` (`default_auto_tap`). With `auto_tap: false` a `CastSpell`
    /// whose cost is not already floating is refused **422**, and there is no
    /// mana-tapping UI to float it with; the README's route table says `{seq,
    /// action_index, params?}` with no note, and the acceptance criterion advertises
    /// playing "through `curl` alone".
    ///
    /// Tested at the deserialization boundary rather than over HTTP because that is
    /// where the divergence lived: both spellings are parsed and lowered through the
    /// real `From<ActionParamsDto> for ActionParams`, and the results compared. The
    /// comparison is over `Debug` because `ActionParams` derives no `PartialEq` (it is
    /// a simulator-internal assembly type) — that is a structural comparison of every
    /// field, which is what this test wants, not a proxy for one.
    #[test]
    fn test_mr_m11_05_omitted_params_and_empty_params_agree() {
        fn lower(body: &str) -> mtg_simulator::ActionParams {
            let req: view::ActionRequest =
                serde_json::from_str(body).expect("the body is a valid ActionRequest");
            req.params.into()
        }

        let omitted = lower(r#"{"seq": 1, "action_index": 0}"#);
        let empty = lower(r#"{"seq": 1, "action_index": 0, "params": {}}"#);

        assert_eq!(
            format!("{omitted:?}"),
            format!("{empty:?}"),
            "an omitted `params` and an empty `params` object must lower to the same \
             announcement (review MR-M11-05)"
        );

        // Non-vacuity, and the actual subject: the agreed value is `true` — the one
        // the *derived* `Default` got wrong. If `impl Default for ActionParamsDto` is
        // ever replaced by `#[derive(Default)]` again, the assertion above still holds
        // (both spellings would agree on `false`) and only this one goes red.
        assert!(
            omitted.auto_tap,
            "CR 601.2g: with no mana-tapping UI, an omitted `params` must still \
             auto-tap, or every cast over `curl` is a 422"
        );
        assert!(empty.auto_tap);
    }

    // ── Review MR-M11-08: the game-over payload carries no raw `Debug` ────────

    /// **Architecture Invariant 7 at the halt/violation channel** (review MR-M11-08).
    ///
    /// `game_over_view` and `halted_view` used to inject `format!("{v:?}")` and
    /// `format!("{reason:?}")` straight into the seat payload, and neither string
    /// passes through *either* of this crate's Invariant-7 chokepoints —
    /// `from_game_state_for(.., Viewer::Seat(..))` or [`view::NameIndex`]. Two live
    /// carriers: `invariants::check_no_orphaned_tokens` interpolates
    /// `obj.characteristics.name` into its description, and
    /// `HaltReason::EngineError(String)` carries a `GameStateError` `Debug` produced
    /// while advancing a **bot** seat.
    ///
    /// Driven through the two rendering functions directly with a card name planted in
    /// each carrier, because the leak is in the *reduction*, not in reaching it: a
    /// healthy game has an empty `violations` and a `None` `reason`, so a fixture that
    /// merely plays to `game_over` asserts nothing at all. Planting the name is what
    /// makes this discriminating — with the pre-fix `format!("{:?}")` restored, both
    /// halves go red.
    #[test]
    fn test_mr_m11_08_game_over_payload_carries_no_engine_debug() {
        use mtg_simulator::{GameDriverError, GameResult, HaltReason, InvariantViolation};

        /// A name no `check` string, turn number or fixed prose could contain.
        const PLANTED: &str = "Sheoldred, Whispering One";

        // Half 1: a violation whose *description* names a card.
        let result = GameResult {
            seed: 7,
            winner: None,
            turn_count: 19,
            total_commands: 4_200,
            violations: vec![InvariantViolation {
                check: "no_orphaned_tokens".to_string(),
                description: format!("token {PLANTED} (id 412) is in Graveyard(2)"),
                turn_number: 19,
            }],
            error: Some(GameDriverError::EngineError(format!(
                "InvalidTarget {{ object: {PLANTED} }}"
            ))),
        };
        let over = view::game_over_view(&result, &HashMap::new());

        for line in &over.violations {
            assert!(
                !line.contains(PLANTED),
                "the seat payload leaks a card name through a violation description: \
                 {line:?} (Architecture Invariant 7, review MR-M11-08)"
            );
        }
        // Not merely absent — the useful half survives, which is what makes the
        // reduction a redaction rather than a deletion.
        assert_eq!(over.violations.len(), 1);
        assert!(
            over.violations[0].contains("no_orphaned_tokens") && over.violations[0].contains("19"),
            "the check name and turn must survive so a play-tester knows to export a \
             report; got {:?}",
            over.violations[0]
        );
        let reason = over
            .reason
            .as_deref()
            .expect("a driver error renders a reason");
        assert!(
            !reason.contains(PLANTED),
            "the seat payload leaks a card name through the driver error: {reason:?}"
        );
        assert!(
            reason.contains("/api/game/report"),
            "the reason must point at the export that does carry the detail; got \
             {reason:?}"
        );

        // Half 2: the halted arm, whose `HaltReason::EngineError` is the same carrier.
        let halted = view::halted_view(
            &HaltReason::EngineError(format!("InvalidTarget {{ object: {PLANTED} }}")),
            19,
            4_200,
        );
        let reason = halted
            .reason
            .as_deref()
            .expect("a halt always has a reason");
        assert!(
            !reason.contains(PLANTED),
            "the halted payload leaks a card name: {reason:?}"
        );

        // Every other `HaltReason` variant holds only ids and integers, so those keep
        // their numbers — pinned so a future reduction cannot quietly blank them too.
        let capped = view::halted_view(
            &HaltReason::MaxTurns {
                max_turns: 40,
                turn: 40,
            },
            40,
            9,
        );
        assert!(
            capped
                .reason
                .as_deref()
                .expect("a halt always has a reason")
                .contains("40"),
            "a numeric halt reason must stay legible: {:?}",
            capped.reason
        );
    }

    // ── Review MR-M11-10: a kept hand cannot be re-dealt ──────────────────────

    /// **CR 103.5 — the keep is terminal, and it is now the server that says so**
    /// (review MR-M11-10).
    ///
    /// *"Once a player chooses not to take a mulligan, the remaining cards become that
    /// player's opening hand."* Before this, `POST /api/game/mulligan {"take": false}`
    /// recorded nothing — `is_pregame()` was `command_count() == 0`, which stays true
    /// right up to the first applied command — so a refresh, a second tab or a plain
    /// `curl` could redeal the hand the human had just accepted. The keep lived only
    /// in `PlayApp.svelte`'s client-side `keptHand` rune, which is a UI convention and
    /// not a rule.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_mr_m11_10_a_kept_hand_cannot_be_redealt() {
        let state = shared_state();
        let view = new_game(&state).await;
        assert_eq!(
            view["summary"]["pregame"], true,
            "a new game is mulliganable"
        );

        // A redeal before the keep is still fine — otherwise the assertion below would
        // pass on a route that had simply broken.
        let (status, after) =
            post_json(&state, "/api/game/mulligan", json!({ "take": true })).await;
        assert_eq!(status, StatusCode::OK, "{after}");
        assert_eq!(after["summary"]["mulligan_count"], 1);
        assert_eq!(after["summary"]["pregame"], true, "a redeal is not a keep");

        // The subject: keep.
        let (status, kept) =
            post_json(&state, "/api/game/mulligan", json!({ "take": false })).await;
        assert_eq!(status, StatusCode::OK, "{kept}");
        assert_eq!(
            kept["summary"]["pregame"], false,
            "CR 103.5: after a keep the game is no longer mulliganable, and \
             `summary.pregame` is what says so"
        );

        // …and the choice is terminal in both spellings, since either would replace
        // the accepted hand.
        for take in [true, false] {
            let (status, err) =
                post_json(&state, "/api/game/mulligan", json!({ "take": take })).await;
            assert_eq!(
                status,
                StatusCode::CONFLICT,
                "a second mulligan ({{take: {take}}}) after a keep must be refused: {err}"
            );
            assert_eq!(err["kind"], "not_pregame");
        }

        // Non-vacuity: the session is still a live, playable game rather than having
        // been wedged by the guard.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            still["summary"]["mulligan_count"], 1,
            "the kept hand is the redealt one"
        );
        assert!(
            !still["decision"].is_null(),
            "the kept table must still be holding an answerable decision"
        );
    }

    // ── UI-1: blocking-decision pickers (task scutemob-174) ───────────────────
    //
    // `memory/playtest-triage-2026-08-02.md` F8. `StubProvider` bakes the
    // engine-accepted default into each blocking-decision `LegalAction` — cleanup
    // discard = the `count` highest `ObjectId`s, scry/surveil = the identity
    // partition, search = `candidates.first()` (the lowest `ObjectId`) — and until
    // UI-1 the view layer stripped the candidate data, so the browser rendered one
    // bare button that submitted exactly that default.
    //
    // Each probe below therefore does two things in order: **reproduce** the
    // symptom by asserting what the default IS, then **drive a different answer
    // through the real router** and assert the game actually did something else.
    // The second half is what discriminates; the first is what makes the test a
    // record of the defect rather than only of the fix.

    /// The pin for the two fixed-deck fixtures. **Read off a real run**, not
    /// reasoned to: at this seed the human's opening hand holds *both* probe spells
    /// (`["Diabolic Tutor", "Swamp", "Swamp", "Swamp", "Swamp", "Swamp", "Read the
    /// Bones"]`), which is what makes a bounded drive reach a scry and a search at
    /// all. Changing it invalidates every count asserted below.
    const UI1_SEED: u64 = 184;

    /// A 7-mana mono-black legendary creature. Deliberately the most expensive one
    /// in the corpus's mono-black set: it fixes the deck's colour identity for
    /// `validate_deck` (CR 903.5c) while being unreachable inside the probe's
    /// window, so neither seat's commander can enter the battlefield and perturb
    /// the drive.
    const UI1_COMMANDER: &str = "razaketh-the-foulblooded";

    /// CR 903.5c: 97 Swamps + the two probe spells + a mono-black commander.
    ///
    /// Almost-all-basics on purpose. Swamps are `Complete`, exempt from the
    /// singleton rule, and produce exactly one mana each — so the auto-tap solver's
    /// known source-counting defect (playtest triage F4) cannot influence what this
    /// probe observes, and the only two castable spells in the deck are the two the
    /// probes are about.
    /// The two probe spells occupy `main_deck[0]` and `main_deck[1]`. That is what
    /// makes one seed serve two different fixtures: the shuffle is a permutation of
    /// *positions* determined by `cfg.seed` alone, so whichever pair of cards sits
    /// in those two slots lands in the opening hand at [`UI1_SEED`]. Verified by
    /// sweep for both pairs, not assumed.
    fn ui1_deck(spells: [&str; 2]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = spells.iter().map(|s| CardId(s.to_string())).collect();
        while main_deck.len() < 99 {
            main_deck.push(CardId("swamp".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(UI1_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// CR 608.2d: the scry and search fixtures.
    const UI1_EFFECT_CHOICE_SPELLS: [&str; 2] = ["read-the-bones", "diabolic-tutor"];

    /// CR 603.3d: the trigger-target fixture (OOS-DP8-2).
    ///
    /// Shadow Alley Denizen ({B}, `Complete`) triggers when **another** black
    /// creature you control enters and targets an unfiltered `TargetCreature`;
    /// Nezumi Prowler ({1}{B}, `Complete`, a black creature) has an ETB targeting a
    /// creature *you* control. Casting the Denizen on turn 1 and the Prowler on the
    /// human's next turn therefore raises the CR 603.3d announcement, because by
    /// flush time each slot has **two** candidates — and a slot with two candidates
    /// is exactly the condition `abilities::forced_trigger_target_answer` fails to
    /// force, which is what makes the engine ask instead of auto-picking.
    ///
    /// Every other route was checked and rejected: every other mono-black `Complete`
    /// triggered target at this mana value was retargeted to `TargetOpponent` by
    /// PB-EF6, and `TargetOpponent` has exactly one candidate in a two-player game,
    /// so it is always forced and never asks.
    const UI1_TRIGGER_SPELLS: [&str; 2] = ["shadow-alley-denizen", "nezumi-prowler"];

    /// Install a two-player fixed-deck session, then drive every request through
    /// the real router exactly as any other test here does.
    ///
    /// `POST /api/game` cannot express this: `session::config_for` hard-codes
    /// `DeckSource::RandomPerSeat` and `NewGameDefaults` carries only
    /// `players`/`bot`/`seed`. So the fixture is installed through
    /// `session::new_game`, which is the same constructor the handler uses and runs
    /// the same two Invariant-9 gates (`validate_deck` inside
    /// `build_initial_state`, then `check_all_defs_complete` inside
    /// `LocalGame::start`). Nothing about the HTTP path is stubbed — only the deck
    /// the game starts from.
    fn ui1_install(state: &SharedState, spells: [&str; 2]) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI1_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), ui1_deck(spells)),
                (mtg_engine::PlayerId(2), ui1_deck(spells)),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the UI-1 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// The out-of-band oracle: read the *engine's* state directly, to check what
    /// the answer actually did. Only ever used to verify an effect, never to build
    /// a payload — the same role `from_game_state` plays for the redaction tests.
    fn ui1_zone(state: &SharedState, zone: mtg_engine::ZoneId) -> Vec<u64> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .zones()
            .get(&zone)
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .iter()
            .map(|id| id.0)
            .collect()
    }

    /// Ordered zone, **bottom-first** (`Zone::Ordered` keeps the top at the last
    /// index — `Zone::top()` is `v.last()`), so index 0 is the bottom of the
    /// library. That is the whole point of the scry assertion.
    fn ui1_library(state: &SharedState) -> Vec<u64> {
        ui1_zone(state, mtg_engine::ZoneId::Library(mtg_engine::PlayerId(1)))
    }

    fn ui1_hand(state: &SharedState) -> Vec<u64> {
        ui1_zone(state, mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1)))
    }

    /// The index of the offered action carrying a blocking decision whose
    /// `question` tag is `want`, if any.
    fn ui1_question_index(view: &Value, want: &str) -> Option<u64> {
        view["decision"]["actions"]
            .as_array()?
            .iter()
            .find(|a| a["decision"]["question"] == want)
            .and_then(|a| a["index"].as_u64())
    }

    /// Drive the human seat until an offered action carries the `want` question.
    ///
    /// Policy, in order: play a land; else cast anything castable (the fixture
    /// deck's only castables are the two probe spells — the commander is 7 mana and
    /// out of reach); else pass; else take the first action that is not `Concede`.
    /// A blocking decision met **on the way** is therefore answered with `{}`, i.e.
    /// the engine's own default, which is exactly the pre-UI-1 behaviour and is
    /// what lets the search probe walk straight past Read the Bones' scry.
    async fn ui1_drive_to_question(state: &SharedState, want: &str, max_steps: usize) -> Value {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if ui1_question_index(&view, want).is_some() {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} without ever asking a {want} question; \
                 UI1_SEED may need re-pinning: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| actions.iter().find(|a| a["kind"] == "CastSpell"))
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "step {step} submitting {}: {next}",
                pick["label"]
            );
            view = next;
        }
        panic!("no {want} question within {max_steps} steps");
    }

    /// **CR 701.22a — Read the Bones' scry, driven to a non-default partition over
    /// HTTP.** Closes the browser-client half of OOS-DP9-1.
    ///
    /// The playtest symptom was "it never asks me to scry": the action's baked-in
    /// answer is the *identity* partition (keep everything on top), so the one
    /// button the client rendered resolved every scry as a no-op. Both halves are
    /// asserted — the identity default is pinned as the reproduction, then a real
    /// partition is submitted and the library is checked.
    ///
    /// The discriminating assertion is the last pair. Read the Bones draws two
    /// cards immediately after its scry, so with the default the human draws
    /// `looked_at[0]`; with the answer this test sends, `looked_at[0]` is at the
    /// **bottom of the library** and was not drawn. A regression to submitting the
    /// default fails on both lines, not on a count.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui1_scry_partition_is_answered_over_http() {
        let state = shared_state();
        ui1_install(&state, UI1_EFFECT_CHOICE_SPELLS);
        let view = ui1_drive_to_question(&state, "Scry", 400).await;
        let index = ui1_question_index(&view, "Scry").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let decision = &option["decision"];

        assert_eq!(decision["answer_field"], "effect_choice_answer");
        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "Partition");
        assert_eq!(answer["kept_key"], "top");
        assert_eq!(answer["moved_key"], "bottom");
        assert_eq!(answer["moved_label"], "bottom of library");

        let looked: Vec<u64> = answer["looked_at"]
            .as_array()
            .expect("looked_at is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert_eq!(looked.len(), 2, "Read the Bones scries 2: {answer}");

        // **The new label channel, exercised.** These ids name LIBRARY cards, and
        // `StateViewModel` does not model library contents at all — so before
        // `view::question_card_label` every one of them rendered as the unknown
        // placeholder and the picker would have been three identical buttons.
        for card in answer["looked_at"].as_array().unwrap() {
            let label = card["label"].as_str().expect("label is a string");
            assert_eq!(
                label, "Swamp",
                "a scried library card must render its real name (CR 701.22a lets \
                 this seat look at it); got {label:?}"
            );
        }

        // Reproduction: the engine's own default answer is the IDENTITY partition.
        assert_eq!(
            answer["template"],
            json!({"Scry": {"bottom": [], "top": looked}}),
            "the pre-UI-1 client submitted exactly this, which is why a scry was \
             always a no-op"
        );

        // `EffectChoiceQuestion::Scry`'s `looked_at` is TOP-FIRST, and the library
        // zone is bottom-first, so these two indices are the same two cards.
        let before = ui1_library(&state);
        assert_eq!(
            before[before.len() - 1],
            looked[0],
            "looked_at is top-first"
        );
        assert_eq!(before[before.len() - 2], looked[1]);

        // Answer it: bottom the current top card, keep the other.
        let wire_seq = seq(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {
                    "effect_choice_answer": {
                        "Scry": { "bottom": [looked[0]], "top": [looked[1]] }
                    }
                }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        // **CR 400.7 is why this is asserted over the LIBRARY and not the hand.**
        // A card that changes zones becomes a NEW object with a new `ObjectId`, so
        // the two ids above cannot be followed into the hand — a first draft of
        // this test asserted `hand.contains(looked[1])` and failed against a hand
        // of freshly-minted ids. The library keeps them, which is enough: the two
        // answers are distinguished by *which* card is still in it.
        let library = ui1_library(&state);
        assert_eq!(
            library[0], looked[0],
            "the bottomed card must end at the library's BOTTOM (index 0). Under the \
             DEFAULT (identity) answer it was the TOP card and Read the Bones would \
             have drawn it, so it would not be in the library at all — this single \
             line is what discriminates the two answers."
        );
        assert!(
            !library.contains(&looked[1]),
            "the kept card became the new top and Read the Bones drew it (CR 400.7: \
             it is a different object in hand now, so it is checked by its absence)"
        );
        assert_eq!(
            library.len(),
            before.len() - 2,
            "Read the Bones draws exactly 2 after its scry; a partition moves cards \
             within the library and never removes them"
        );
    }

    /// **CR 701.23a — Diabolic Tutor's search, driven to a non-default pick over
    /// HTTP.** Closes the browser-client half of OOS-DP9-7.
    ///
    /// Three things at once: the reproduction (the baked-in default is
    /// `candidates.first()`, i.e. the lowest `ObjectId` — "it always fetches the
    /// same card"), the CR 701.23d refusal (this search states no quality, so
    /// failing to find is illegal and the server says so as a **400** rather than
    /// letting the engine answer 422), and a real non-default pick.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui1_search_pick_is_answered_over_http() {
        let state = shared_state();
        ui1_install(&state, UI1_EFFECT_CHOICE_SPELLS);
        let view = ui1_drive_to_question(&state, "SearchLibrary", 600).await;
        let index = ui1_question_index(&view, "SearchLibrary").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let answer = &option["decision"]["answer"];

        assert_eq!(option["decision"]["answer_field"], "effect_choice_answer");
        assert_eq!(answer["shape"], "PickOne");
        assert_eq!(
            answer["may_decline"], false,
            "CR 701.23d: Diabolic Tutor's filter states no quality, so finding is \
             MANDATORY and the client must not offer a fail-to-find button"
        );

        let candidates: Vec<u64> = answer["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert!(
            candidates.len() > 2,
            "a searched library should offer many cards: {}",
            candidates.len()
        );

        // Reproduction: the default is `candidates.first()`, and `candidates` is in
        // ascending `ObjectId` order — so the pre-UI-1 client always fetched the
        // lowest-id match, exactly as the playtest reported.
        assert_eq!(
            candidates[0],
            *candidates.iter().min().expect("non-empty"),
            "candidates are in ascending ObjectId order"
        );
        assert_eq!(
            answer["template"],
            json!({"SearchLibrary": {"found": candidates[0]}})
        );

        let wire_seq = seq(&view);

        // CR 701.23d, refused at the response boundary rather than by the engine.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": { "effect_choice_answer": { "SearchLibrary": { "found": null } } }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // A card the search never offered is refused the same way.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": { "effect_choice_answer": { "SearchLibrary": { "found": 999_999 } } }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // Now a real, non-default pick.
        let chosen = *candidates.last().expect("non-empty");
        assert_ne!(
            chosen, candidates[0],
            "the pick must differ from the default"
        );
        let library_before = ui1_library(&state);
        assert!(library_before.contains(&chosen) && library_before.contains(&candidates[0]));
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": { "effect_choice_answer": { "SearchLibrary": { "found": chosen } } }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        // Asserted over the LIBRARY for the CR 400.7 reason spelled out in the scry
        // probe: the found card is a new object once it reaches the hand, so it is
        // checked by what left the library rather than by what arrived.
        let library = ui1_library(&state);
        assert!(
            !library.contains(&chosen),
            "the chosen card must have left the library (CR 701.23a)"
        );
        assert!(
            library.contains(&candidates[0]),
            "the DEFAULT's pick must still be in the library — it is what a client \
             submitting `{{}}` would have fetched, and that it is untouched is what \
             discriminates this test"
        );
        assert_eq!(
            library.len(),
            library_before.len() - 1,
            "exactly one card is found (CR 701.23a)"
        );
    }

    /// **CR 514.1 — the cleanup discard, driven to a non-default subset over
    /// HTTP.** Closes the browser-client half of OOS-DP7-6.
    ///
    /// The playtest symptom was "it discards for me, always the cards on the
    /// right": `default_cleanup_discard` is the `count` **highest** `ObjectId`s,
    /// which is display order's right-hand end. This drives the opposite choice.
    ///
    /// No fixed deck needed — the default 4-player table reaches a cleanup discard
    /// on turn 1 all by itself, because a seat that never plays a land draws past
    /// the CR 514.1 maximum. Passing is also the only policy that keeps the hand
    /// composition an accident of the seed rather than of the driver.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui1_cleanup_discard_subset_is_answered_over_http() {
        let state = shared_state();
        let mut view = new_game(&state).await;

        // Pass, and only pass.
        let mut found = None;
        for step in 0..300 {
            if let Some(index) = ui1_question_index(&view, "CleanupDiscard") {
                found = Some(index);
                break;
            }
            assert!(!view["decision"].is_null(), "game ended at step {step}");
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"].as_array().unwrap().clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede at step {step}: {view}"));
            let index = pick["index"].as_u64().unwrap();
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "step {step}: {next}");
            view = next;
        }
        let index = found.expect("a pass-only seat must reach a CR 514.1 cleanup discard");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let decision = &option["decision"];
        assert_eq!(decision["answer_field"], "discard_cards");
        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "Subset");

        let count = answer["count"].as_u64().expect("count is a number");
        assert!(
            count >= 1,
            "a cleanup discard is only raised when count >= 1"
        );
        let candidates: Vec<u64> = answer["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().unwrap())
            .collect();
        assert_eq!(
            candidates.len() as u64,
            count + 7,
            "the candidate set is the WHOLE hand, not just the cards to be discarded"
        );
        // Hand cards are labelled through `NameIndex` like everything else — this
        // seat's own hand is in its redacted view.
        for card in answer["candidates"].as_array().unwrap() {
            let label = card["label"].as_str().unwrap();
            assert!(
                label != view::HIDDEN_LABEL && label != view::UNKNOWN_LABEL,
                "a seat's own hand card must be named to itself (CR 402.1): {label:?}"
            );
        }

        // Reproduction: the default is the `count` HIGHEST ObjectIds.
        let default: Vec<u64> = answer["default"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_u64().unwrap())
            .collect();
        let mut highest = candidates.clone();
        highest.sort_unstable();
        assert_eq!(
            default,
            highest[highest.len() - count as usize..].to_vec(),
            "`default_cleanup_discard` is the count highest ObjectIds — the \
             right-hand cards the playtest reported losing"
        );

        let wire_seq = seq(&view);

        // The wrong number of cards is a 400 against the response, not a 422.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": index,
                   "params": {"discard_cards": candidates}}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // So is a card that is not in this hand.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": index,
                   "params": {"discard_cards": [999_999]}}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // Now discard the LOWEST ids — the opposite end from the default.
        let chosen: Vec<u64> = highest[..count as usize].to_vec();
        assert!(
            chosen.iter().all(|id| !default.contains(id)),
            "the chosen subset must be disjoint from the default, or this proves \
             nothing: chosen={chosen:?} default={default:?}"
        );
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": index,
                   "params": {"discard_cards": chosen}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        let hand = ui1_hand(&state);
        for id in &chosen {
            assert!(!hand.contains(id), "object {id} was chosen for discard");
        }
        for id in &default {
            assert!(
                hand.contains(id),
                "object {id} is what the DEFAULT would have discarded; it must \
                 still be in hand"
            );
        }
    }

    /// **CR 603.3d — a trigger's target announcement, driven to a non-default
    /// choice over HTTP. This is OOS-DP8-2's extension proof, executed.**
    ///
    /// The claim UI-1 makes is that a blocking-decision payload keyed on the
    /// answer's *shape* rather than on the question extends to a new question with
    /// no rework. `ChooseTriggerTargets` was filed as the identical gap to the
    /// discard and scry ones, and its shape — [`view::AnswerShapeView::Slots`] — is
    /// the same `Vec<TargetSlotView>` the CR 601.2c target picker already draws. So
    /// the browser reuses `TargetPicker` and the server reuses `target_options`.
    ///
    /// A claim of that kind is worth exactly as much as its test. This one drives a
    /// real pair of `Complete` card definitions to a real announcement and picks
    /// something other than the engine's default, then reads the *rendered
    /// keywords* back out of the seat view — so the whole loop, payload to picker
    /// answer to game state to next payload, is checked over HTTP.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui1_trigger_targets_are_answered_over_http() {
        let state = shared_state();
        ui1_install(&state, UI1_TRIGGER_SPELLS);
        let view = ui1_drive_to_question(&state, "TriggerTargets", 400).await;
        let index = ui1_question_index(&view, "TriggerTargets").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let decision = &option["decision"];

        assert_eq!(decision["answer_field"], "trigger_targets");
        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "Slots");

        let slots = answer["slots"].as_array().expect("slots is an array");
        assert_eq!(slots.len(), 1, "both fixture triggers have one target slot");
        let slot = &slots[0];
        assert_eq!(slot["min"], 1, "a required slot takes exactly one target");
        assert_eq!(slot["max"], 1);

        // **The condition that makes the engine ask at all.** With one candidate
        // `forced_trigger_target_answer` determines the announcement and no decision
        // is raised, so this assertion is the fixture's own non-vacuity check.
        let candidates = slot["candidates"].as_array().expect("candidates");
        assert!(
            candidates.len() >= 2,
            "a slot with fewer than two candidates is FORCED and would never have \
             produced this decision: {slot}"
        );
        for candidate in candidates {
            assert_eq!(candidate["kind"], "object");
            let label = candidate["label"].as_str().expect("label");
            assert!(
                label != view::HIDDEN_LABEL && label != view::UNKNOWN_LABEL,
                "a battlefield creature is public (CR 400.1) and must be named \
                 through NameIndex: {label:?}"
            );
        }

        // Reproduction: the engine's own default announcement, one entry per slot.
        let default = answer["default"].as_array().expect("default is an array");
        assert_eq!(default.len(), 1, "one entry per slot");
        let default_target = &default[0].as_array().expect("slot 0's targets")[0];

        // Pick a candidate the default did NOT.
        let chosen = candidates
            .iter()
            .map(|c| &c["value"])
            .find(|value| *value != default_target)
            .unwrap_or_else(|| panic!("no candidate differs from the default: {answer}"));
        let chosen_id = candidates
            .iter()
            .find(|c| &c["value"] == chosen)
            .and_then(|c| c["id"].as_u64())
            .expect("the chosen candidate's id");
        let default_id = candidates
            .iter()
            .find(|c| &c["value"] == default_target)
            .and_then(|c| c["id"].as_u64())
            .expect("the default is always one of the candidates");

        // **Baselines, and the reason they are needed.** Nezumi Prowler is printed
        // with Ninjutsu, so "the chosen creature has a keyword" is true before any
        // trigger resolves. A first draft asserted exactly that and passed against
        // the un-fixed code — vacuously, on a printed keyword. What discriminates is
        // what each creature *gains*.
        let chosen_before = ui1_battlefield_keywords(&view, chosen_id);
        let default_before = ui1_battlefield_keywords(&view, default_id);

        // Answer every announcement in this CR 603.3b batch with the SAME
        // non-default creature. Both fixture triggers offer the same two candidates,
        // so leaving the second to its own default would land a grant on
        // `default_id` and blunt the assertion below.
        //
        // The `Target` is echoed back VERBATIM out of the candidate's `value`, never
        // rebuilt from `kind`/`id` — the rule `TargetPicker` already follows.
        let mut view = view;
        let mut answered = 0usize;
        for _ in 0..8 {
            let Some(index) = ui1_question_index(&view, "TriggerTargets") else {
                break;
            };
            let option = view["decision"]["actions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["index"] == index)
                .expect("the option with that index")
                .clone();
            let slots = option["decision"]["answer"]["slots"].as_array().unwrap();
            let pick = slots[0]["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == chosen_id)
                .unwrap_or_else(|| panic!("the chosen creature must be offered: {option}"))
                ["value"]
                .clone();
            let wire_seq = seq(&view);
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({
                    "seq": wire_seq,
                    "action_index": index,
                    "params": { "trigger_targets": [[pick]] }
                }),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{next}");
            view = next;
            answered += 1;
        }
        assert!(
            answered >= 1,
            "at least one announcement must have been answered"
        );

        // The grant happens at RESOLUTION, not at announcement, so the stack has to
        // drain before there is anything to read. Pass until it does.
        for _ in 0..40 {
            let stack_empty = view["state"]["zones"]["stack"]
                .as_array()
                .map(|s| s.is_empty())
                .unwrap_or(true);
            if stack_empty {
                break;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended mid-stack: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"].as_array().unwrap().clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .expect("something other than Concede");
            let index = pick["index"].as_u64().unwrap();
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{next}");
            view = next;
        }

        // `PermanentView.keywords` renders the layer-resolved set, so the effect is
        // read back out of the seat view rather than out of the engine.
        let chosen_gained = ui1_gained(&view, chosen_id, &chosen_before);
        let default_gained = ui1_gained(&view, default_id, &default_before);
        assert!(
            !chosen_gained.is_empty(),
            "the chosen creature must have GAINED the trigger's keyword: gained \
             {chosen_gained:?} (was {chosen_before:?})"
        );
        assert!(
            default_gained.is_empty(),
            "the DEFAULT's target must have gained NOTHING — a client submitting \
             `{{}}` would have produced exactly the opposite pair, and that \
             asymmetry is the whole discriminator: default gained {default_gained:?}"
        );
    }

    /// Keywords `object_id` has in `view` that it did not have in `before`.
    fn ui1_gained(view: &Value, object_id: u64, before: &[String]) -> Vec<String> {
        ui1_battlefield_keywords(view, object_id)
            .into_iter()
            .filter(|k| !before.contains(k))
            .collect()
    }

    /// Rendered keywords of a battlefield permanent, out of the seat payload.
    fn ui1_battlefield_keywords(view: &Value, object_id: u64) -> Vec<String> {
        view["state"]["zones"]["battlefield"]
            .as_object()
            .expect("battlefield is an object keyed by player name")
            .values()
            .filter_map(|permanents| permanents.as_array())
            .flatten()
            .find(|p| p["object_id"] == object_id)
            .and_then(|p| p["keywords"].as_array())
            .map(|ks| {
                ks.iter()
                    .filter_map(|k| k.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// **Architecture Invariant 7: a decision addressed to another seat carries
    /// none of its look entitlement into this seat's payload** (UI-1 review, HIGH 2).
    ///
    /// `view::question_card_label` renders the REAL name of a library card, and its
    /// safety argument's second premise is that the `EffectChoiceQuestion` it reads
    /// belongs to the seat being rendered. Nothing enforced that: it held only
    /// because `session::config_for` hard-codes one human seat, so `pending.player`
    /// happened to always equal `session.human`. A second human seat — the obvious
    /// M10a direction — would have put seat A's scried library cards, **named**,
    /// into seat B's payload.
    ///
    /// This drives a real scry, confirms the entitlement is being exercised (the
    /// candidates render as `Swamp`, not as a placeholder), then moves
    /// `PlaySession::human` to the other seat and re-reads — so the payload is being
    /// built for a seat that is *not* the one the outstanding question belongs to,
    /// which is precisely the M10a shape.
    ///
    /// **Retargeting the viewer rather than the decision, and the reason is worth
    /// recording**: a first version mutated `PlaySession::pending.player` instead,
    /// and it did nothing — every route calls `advance()`, which refreshes `pending`
    /// straight back off `LocalGame`. `human` is the play server's own field and
    /// survives.
    ///
    /// **Two-sided on BOTH halves**, each verified by deleting the line and running
    /// this test: without `seat_view`'s filter the decision comes back with its
    /// scried cards named, and without `post_action`'s guard the final POST returns
    /// 200 and applies the other seat's scry.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload() {
        let state = shared_state();
        ui1_install(&state, UI1_EFFECT_CHOICE_SPELLS);
        let view = ui1_drive_to_question(&state, "Scry", 400).await;
        let index = ui1_question_index(&view, "Scry").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");

        // Non-vacuity: the entitlement really is in use, so its absence below is a
        // consequence of the filter and not of the payload being empty anyway.
        let looked = option["decision"]["answer"]["looked_at"]
            .as_array()
            .expect("looked_at");
        assert!(!looked.is_empty());
        for card in looked {
            assert_eq!(
                card["label"], "Swamp",
                "the look entitlement must be rendering real names before this test \
                 can say anything about withholding them"
            );
        }

        // Captured BEFORE the move, while this harness is still seat 1. A real seat-2
        // client could NOT obtain it: the write guard sits above the `seq` check, so
        // a foreign decision answers 409 `no_pending_decision` with no `expected`
        // field rather than the usual `stale_decision` body that carries the current
        // `seq`. Reading it here is therefore the STRONGEST case for the guard, not
        // a representative one — it grants the attacker something the code does not.
        let wire_seq_before = seq(&view);

        // Render for the OTHER seat while seat 1's question is outstanding.
        {
            let mut guard = state.session.lock().expect("lock");
            let session = guard.as_mut().expect("a session is installed");
            assert_eq!(
                session.pending.as_ref().map(|p| p.player),
                Some(session.human),
                "precondition: the question belongs to the seat being rendered"
            );
            session.human = mtg_engine::PlayerId(2);
        }

        let (status, body) = get_raw(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let refetched: Value = serde_json::from_str(&body).expect("body is JSON");
        assert_eq!(refetched["summary"]["human"], 2, "the viewer really moved");
        assert!(
            refetched["decision"].is_null(),
            "seat 1's question must not appear in seat 2's payload: {}",
            refetched["decision"]
        );

        // Asserted over the RAW body, not over parsed fields — the MR-M11-01 idiom.
        // A future field that carried the question's cards under another name would
        // be caught by this and not by a field-by-field check.
        //
        // The needle is the `looked_at` **key**, not a card name, and deliberately:
        // seat 2 legitimately holds Swamps of its own, so "no card name appears"
        // is not assertable here and claiming it would be the overstatement this
        // whole review cycle is about.
        assert!(
            !body.contains("\"looked_at\""),
            "the foreign seat's look entitlement leaked into the body: {body}"
        );

        // **The write half.** Hiding the decision does not stop this seat answering
        // it — `LocalGame::submit` builds the command for `pending.player` and has
        // no notion of a viewer, so without `post_action`'s guard this exact post
        // returns 200 and applies the other seat's scry (verified by deleting the
        // guard). The `seq` used here is the pre-move one; see the note above for
        // why that is a deliberately generous gift to the attacker.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq_before, "action_index": index, "params": {}}),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "answering another seat's decision must be refused, not applied: {refused}"
        );
        assert_eq!(refused["kind"], "no_pending_decision");

        // **And the guard suppresses the `seq` disclosure**, which is why it sits
        // ABOVE the staleness check. A wrong `seq` against a foreign decision must
        // not fall through to `stale_decision`, whose body carries `expected: <the
        // real seq>` — that would hand a seat-2 client the one thing it needs to
        // answer a decision it is not allowed to see.
        //
        // This is asserted rather than described. An earlier draft of the comment
        // above stated the disclosure as a live fact *after* this guard had closed
        // it, which is the same "prose out of step with the code" fault this whole
        // review chain kept finding — in its harmless direction, but the fix is the
        // same: make a test hold the claim.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": 999_999, "action_index": 0, "params": {}}),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{refused}");
        assert_eq!(
            refused["kind"], "no_pending_decision",
            "a foreign decision must not answer `stale_decision`, which would \
             disclose its `seq`: {refused}"
        );
        assert!(
            refused["error"]
                .as_str()
                .map(|e| !e.contains("expected"))
                .unwrap_or(true),
            "the refusal body must not carry the foreign decision's seq: {refused}"
        );
    }

    /// **Architecture Invariant 7, at the look-entitlement channel.**
    ///
    /// `view::question_card_label` reads a card's name off `GameState` rather than
    /// off the seat-redacted view, because the redacted view does not model library
    /// contents at all. That is deliberate and argued in place — CR 701.22a /
    /// 701.23a / 701.25a each tell *this seat* to look at *these cards*, and the
    /// engine encodes the entitlement structurally
    /// (`GameEvent::EffectChoiceRequired::private_to()`).
    ///
    /// But it is a **new channel**, and MR-M11-01's lesson is that a redaction gate
    /// checks the channel it was written for. Neither existing gate can see this
    /// one: the source gate scans for omniscient *view-model* entry points and this
    /// uses none, and the HTTP body scan looks for another seat's *hand* card names
    /// and these are library cards.
    ///
    /// So the channel is pinned by count. `.objects()` may appear in `view.rs`'s
    /// production code exactly twice — in `question_card_label`, and in
    /// `action_modes`' card-registry lookup, which reads an id the seat already
    /// holds and no name at all. A third is not forbidden; it is required to be
    /// *deliberate*, which is the most a gate can enforce and is exactly what was
    /// missing when `GameSummary.seed` shipped for three sessions.
    #[test]
    fn test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source =
            std::fs::read_to_string(root.join("src").join("view.rs")).expect("view.rs is readable");
        // `test_region` returns the suffix STARTING at the `#[cfg(test)]` cut, so
        // the production region is its complement. (`view.rs` has no test module
        // at all, in which case that suffix is empty and the whole file is
        // production — which is the case this gate is written for.)
        let cut = source.len() - test_region(&source).len();
        let production = code_only(&source[..cut]);
        // `concat!` for the same reason the sibling gates use it: this line is in a
        // file the walk reads, and a plainly-written needle would be found by the
        // gate rather than by the code it is meant to describe.
        let needle = concat!(".obj", "ects()");
        let found = production.matches(needle).count();
        assert_eq!(
            found, 2,
            "view.rs's production code reads the raw GameState object table {found} \
             time(s), not the 2 that are accounted for (question_card_label's \
             CR 701.22a/23a/25a look entitlement, and action_modes' card-registry \
             lookup). A third read is a NEW hidden-information channel and neither \
             Invariant-7 gate can see it — document it and update this count \
             deliberately, or route it through NameIndex."
        );
    }

    // ── SIM-1: commander castable from the command zone (task scutemob-175) ──
    //
    // Playtest triage F7 / `memory/primitives/sim-1-plan.md` §5. Criterion 5984:
    // "human casts their commander from the browser end-to-end (probe test over
    // HTTP)". Criterion 5985's browser half: "probe covers 0-tax and 2-tax
    // casts" — CR 903.8 charges an additional {2} for each PREVIOUS command-zone
    // cast this game, so the SECOND cast pays the "2-tax" (tax count 1) and,
    // per the dispatch brief's explicit ask, this probe goes one step further
    // and drives a THIRD cast paying the {4} "tax 2" (tax count 2) as well.
    //
    // Modeled on the UI-1 fixed-deck harness just above (`ui1_install` /
    // `ui1_drive_to_question`), not on the seed-swept `COMBAT_SEED`/`TARGET_SEED`
    // fixtures: `session::new_game` with `DeckSource::Fixed` is the same
    // constructor the real handler uses, running the same two Invariant-9 gates,
    // so nothing about the HTTP path is stubbed.

    /// Same numeric value as [`UI1_SEED`], and for the same reason it is safe to
    /// reuse: `setup::build_initial_state` shuffles each seat's `main_deck` with
    /// `SliceRandom::shuffle` on a single `StdRng` seeded from `cfg.seed` alone
    /// (`setup.rs:206`, `:280`) — the permutation depends only on the RNG stream
    /// and the deck's LENGTH, never on what `CardId`s sit at which index. UI-1's
    /// fixture is also a 2-player, `DeckSource::Fixed` game with a 99-card
    /// `main_deck` for both seats, so at this seed player 1's shuffle draws the
    /// exact same permutation of POSITIONS regardless of what this deck puts at
    /// each one — including landing `main_deck[0]`/`main_deck[1]` in the opening
    /// hand, which is UI-1's own pinned observation. **Verified empirically for
    /// this deck too** (not merely inferred): the drive below reaches both
    /// sacrifice spells well inside [`SIM1_MAX_STEPS`].
    const SIM1_SEED: u64 = UI1_SEED;

    /// `{1}{B}`, Legendary Creature — Human Wizard 1/1, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/jadar_ghoulcaller_of_nephalia.rs`). Mono-black,
    /// and its only ability is an end-step trigger (no ETB), so nothing about
    /// entering the battlefield perturbs this drive. MV 2 — castable on the
    /// human's second land drop at tax 0.
    const SIM1_COMMANDER: &str = "jadar-ghoulcaller-of-nephalia";

    /// `{B}`, Instant, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/village_rites.rs`). "As an additional cost to
    /// cast this spell, sacrifice a creature. Draw two cards." No target, so with
    /// Jadar the only creature the human controls, the sacrifice is unambiguous —
    /// this probe never has to answer a "which creature" picker that does not
    /// exist yet.
    const SIM1_SAC_SPELL_1: &str = "village-rites";

    /// `{B}`, Instant, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/culling_the_weak.rs`). Same shape as
    /// [`SIM1_SAC_SPELL_1`] (mandatory, untargeted `SacrificeCreature`) under a
    /// different name, so the SECOND kill is a different singleton card rather
    /// than a second copy of the first.
    const SIM1_SAC_SPELL_2: &str = "culling-the-weak";

    /// Generous bound: reaching the third cast needs roughly ten of the human's
    /// own turns (two land drops to afford the first cast, a further turn or two
    /// to draw and afford each sacrifice spell, four lands for the tax-1 recast,
    /// six for the tax-2 recast), each with several priority windows. Panics
    /// print the last payload, matching `drive_until`'s failure ergonomics
    /// (`main.rs:1619-1622`).
    const SIM1_MAX_STEPS: usize = 500;

    /// CR 903.5c: 97 Swamps, the two sacrifice-outlet spells, and the Jadar
    /// commander. Almost-all-basics for the same reason as [`ui1_deck`]: Swamps
    /// are `Complete`, exempt from the singleton rule, and produce exactly one
    /// mana each, so the auto-tap solver's known source-counting defect
    /// (playtest triage F4) cannot influence what this probe observes.
    ///
    /// The two sacrifice spells occupy `main_deck[0]` and `main_deck[1]` —
    /// see [`SIM1_SEED`] for why that is what puts them in the opening hand.
    fn sim1_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![
            CardId(SIM1_SAC_SPELL_1.to_string()),
            CardId(SIM1_SAC_SPELL_2.to_string()),
        ];
        while main_deck.len() < 99 {
            main_deck.push(CardId("swamp".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(SIM1_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install a two-player fixed-deck session through the same constructor the
    /// real handler uses — see [`ui1_install`]'s doc for why `POST /api/game`
    /// itself cannot express this fixture.
    fn sim1_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: SIM1_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), sim1_deck()),
                (mtg_engine::PlayerId(2), sim1_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the SIM-1 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Out-of-band oracle, exactly [`ui1_zone`]'s role: read the engine's own
    /// `commander_tax` directly, never used to build a payload — only to verify
    /// what an HTTP-driven cast actually did (CR 903.8).
    fn sim1_commander_tax(state: &SharedState) -> u32 {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let cid = mtg_engine::CardId(SIM1_COMMANDER.to_string());
        session
            .game
            .state()
            .player(mtg_engine::PlayerId(1))
            .expect("player 1 exists")
            .commander_tax
            .get(&cid)
            .copied()
            .unwrap_or(0)
    }

    /// Out-of-band oracle: the human's Jadar object currently on the
    /// battlefield, if any. Used only to name the sacrifice target for
    /// [`SIM1_SAC_SPELL_1`]/[`SIM1_SAC_SPELL_2`]'s `additional_costs` — the
    /// engine requires the caster to supply this `ObjectId` explicitly
    /// (`casting.rs:3312`'s `sacrifice_from_additional_costs.ok_or_else`), and
    /// CR 400.7 means a fresh id is minted every time Jadar re-enters the
    /// battlefield, so this must be re-read after each cast rather than reused.
    fn sim1_jadar_on_battlefield_opt(state: &SharedState) -> Option<u64> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let gs = session.game.state();
        let cid = mtg_engine::CardId(SIM1_COMMANDER.to_string());
        gs.zones()
            .get(&mtg_engine::ZoneId::Battlefield)
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .into_iter()
            .find_map(|id| {
                let obj = gs.objects().get(&id)?;
                if obj.controller == mtg_engine::PlayerId(1) && obj.card_id.as_ref() == Some(&cid) {
                    Some(id.0)
                } else {
                    None
                }
            })
    }

    /// A `CastSpell` command does not resolve synchronously — CR 117.3c hands
    /// priority back to the ACTOR (not straight to resolution), so the very next
    /// decision after casting Jadar is still a priority window with Jadar on the
    /// STACK. Drive priority passes (playing a land first if one is still owed)
    /// until [`sim1_jadar_on_battlefield_opt`] finds it, so callers never read the
    /// sacrifice target's id one priority window too early.
    async fn sim1_wait_for_jadar_on_battlefield(state: &SharedState, max_steps: usize) -> u64 {
        if let Some(id) = sim1_jadar_on_battlefield_opt(state) {
            return id;
        }
        for step in 0..max_steps {
            let (status, view) = get_json(state, "/api/game").await;
            assert_eq!(status, StatusCode::OK, "{view}");
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before Jadar resolved onto the battlefield: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "step {step} submitting {}: {next}",
                pick["label"]
            );
            if let Some(id) = sim1_jadar_on_battlefield_opt(state) {
                return id;
            }
        }
        panic!("Jadar never resolved onto the battlefield within {max_steps} steps");
    }

    /// Drive the human seat — playing lands and otherwise passing priority,
    /// exactly [`ui1_drive_to_question`]'s policy — until an offered action
    /// satisfies `stop`. Returns the view it was found in and the matching
    /// action, cloned.
    ///
    /// **This is the discriminating half of the probe.** Before SIM-1's Step 5
    /// (the command-zone enumeration in `legal_actions.rs`), no offered action
    /// ever names the command-zone object, so a stop condition looking for the
    /// commander's `CastSpell` drives every land it can and then exhausts
    /// `max_steps` passing priority forever.
    ///
    /// **Recorded from a real run** with that enumeration temporarily disabled
    /// (`if false && !cast_restricted && ...` in `legal_actions.rs`, then
    /// reverted with `git checkout`): the probe panics at exactly
    /// `sim1_drive_until`'s `panic!` site below, having burned all 500 steps —
    /// the drive reaches turn 59 (`"turn":59` in the payload), the human holds
    /// both sacrifice spells and four Swamps in hand with nothing left to do,
    /// and the payload's own `zones.command_zone` still shows **both** players'
    /// Jadar sitting untouched (`"Human-1":[{"name":"Jadar, Ghoulcaller of
    /// Nephalia","object_id":1}]`) while `decision.actions` contains only
    /// `PassPriority`, the two sacrifice-spell `CastSpell`s and a page of
    /// `TapForMana` — never a `CastSpell` naming `object_id` 1. Abbreviated:
    ///
    /// ```text
    /// thread '...' panicked at .../main.rs:4137:9:
    /// awaited action not offered within 500 steps: {"decision":{"actions":[
    ///   {"kind":"PassPriority", ...},
    ///   {"kind":"CastSpell","label":"Cast Culling the Weak","object_id":2,...},
    ///   {"kind":"CastSpell","label":"Cast Village Rites","object_id":8,...},
    ///   {"kind":"TapForMana", ...}, ... (29 more TapForMana) ...
    /// ], "kind":"Priority","player":1,"seq":501}, ...,
    /// "state":{... "turn":{"active_player":"Human-1","number":59,...},
    /// "zones":{... "command_zone":{
    ///   "Bot-2":[{"name":"Jadar, Ghoulcaller of Nephalia","object_id":101}],
    ///   "Human-1":[{"name":"Jadar, Ghoulcaller of Nephalia","object_id":1}]
    /// }, ...}}, "summary":{... "turn":59}}
    /// ```
    ///
    /// i.e. the action list never contains a `CastSpell` naming the command-zone
    /// object — only `PassPriority`/`PlayLand`/`TapForMana`, the pre-SIM-1
    /// hand-only offer — and both commanders are still exactly where CR 903.6
    /// put them at the start of the game, 59 turns later.
    async fn sim1_drive_until(
        state: &SharedState,
        stop: impl Fn(&Value) -> bool,
        max_steps: usize,
    ) -> (Value, Value) {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if let Some(action) = view["decision"]["actions"]
                .as_array()
                .and_then(|actions| actions.iter().find(|a| stop(a)))
                .cloned()
            {
                return (view, action);
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before the awaited action was offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "step {step} submitting {}: {next}",
                pick["label"]
            );
            view = next;
        }
        panic!("awaited action not offered within {max_steps} steps: {view}");
    }

    /// Submit `action` (found by [`sim1_drive_until`] against `view`) with the
    /// given `params`.
    async fn sim1_submit(
        state: &SharedState,
        view: &Value,
        action: &Value,
        params: Value,
    ) -> Value {
        let wire_seq = seq(view);
        let index = action["index"].as_u64().expect("index is a number");
        let (status, next) = post_json(
            state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": index, "params": params}),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "submitting {}: {next}",
            action["label"]
        );
        next
    }

    /// **CR 903.6/903.8/903.9a — the human casts their commander from the
    /// command zone, end to end, over HTTP.** Closes criterion 5984 and the
    /// browser half of criterion 5985.
    ///
    /// Three casts, each discriminating a different half of SIM-1:
    ///   * **Cast 1 (tax 0)** — `legal_actions.rs`'s command-zone enumeration
    ///     (Step 5) offers the commander at all, at the printed cost.
    ///   * **Cast 2 ("2-tax")** — after Jadar dies (sacrificed to
    ///     [`SIM1_SAC_SPELL_1`]) and CR 903.9a's state-based choice returns it to
    ///     the command zone, the SECOND cast is offered/paid at `{1}{B}` PLUS the
    ///     {2} CR 903.8 tax — `effective_cast_cost` (`legal_actions.rs`) and
    ///     `auto_tap_commands_for` (`local_game.rs`) agreeing is what makes this
    ///     succeed instead of 422ing.
    ///   * **Cast 3 ("tax 2")** — one more kill/return cycle (this time via
    ///     [`SIM1_SAC_SPELL_2`], a different singleton card) and a THIRD cast,
    ///     now taxed {4} (tax count 2). Goes beyond the "0-tax and 2-tax" wire
    ///     wording on purpose, per the dispatch brief's explicit ask.
    ///
    /// The commander's `ObjectId` changes on every zone transition (CR 400.7),
    /// so the command-zone object is re-read out of band before each cast and
    /// never reused across one.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim1_human_casts_their_commander_from_the_command_zone_over_http() {
        let state = shared_state();
        sim1_install(&state);

        let read_command_zone_object = |state: &SharedState| -> u64 {
            let ids = ui1_zone(state, mtg_engine::ZoneId::Command(mtg_engine::PlayerId(1)));
            assert_eq!(
                ids.len(),
                1,
                "exactly the commander should be in the command zone: {ids:?}"
            );
            ids[0]
        };

        // ── Cast 1: tax 0 ──────────────────────────────────────────────────
        let commander_obj_1 = read_command_zone_object(&state);
        assert_eq!(sim1_commander_tax(&state), 0, "no cast has happened yet");

        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "CastSpell" && a["object_id"].as_u64() == Some(commander_obj_1),
            SIM1_MAX_STEPS,
        )
        .await;
        assert_eq!(
            action["label"], "Cast Jadar, Ghoulcaller of Nephalia",
            "the label resolves via NameIndex::from_view, which already indexes \
             view.zones.command_zone (CR 903.6: the command zone is face up) — no \
             play-server production change is needed for this"
        );
        sim1_submit(&state, &view, &action, json!({})).await;

        let after_cast_1 = ui1_zone(&state, mtg_engine::ZoneId::Command(mtg_engine::PlayerId(1)));
        assert!(
            !after_cast_1.contains(&commander_obj_1),
            "the commander must have left the command zone: {after_cast_1:?}"
        );
        assert_eq!(
            sim1_commander_tax(&state),
            1,
            "CR 903.8: the tax counter increments as soon as the cast happens"
        );

        // ── Sacrifice Jadar (Village Rites) so CR 903.9a can offer it back ──
        let jadar_bf_1 = sim1_wait_for_jadar_on_battlefield(&state, SIM1_MAX_STEPS).await;
        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "CastSpell" && a["label"] == "Cast Village Rites",
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(
            &state,
            &view,
            &action,
            json!({
                "additional_costs": [{"Sacrifice": {"ids": [jadar_bf_1], "lki": []}}]
            }),
        )
        .await;

        // ── CR 903.9a: accept the state-based choice back to the command zone ──
        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "ReturnCommanderToCommandZone",
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(&state, &view, &action, json!({})).await;

        // ── Cast 2 ("2-tax"): {1}{B} + the {2} CR 903.8 tax ──────────────────
        let commander_obj_2 = read_command_zone_object(&state);
        assert_eq!(
            mtg_simulator::effective_cast_cost(
                state
                    .session
                    .lock()
                    .expect("lock")
                    .as_ref()
                    .expect("session")
                    .game
                    .state(),
                mtg_engine::PlayerId(1),
                mtg_engine::ObjectId(commander_obj_2),
            )
            .expect("Jadar has a mana cost")
            .mana_value(),
            4,
            "CR 903.8/601.2f: {{1}}{{B}} (MV 2) plus one previous cast's {{2}} tax is MV 4 — \
             the same arithmetic `can_afford` and `auto_tap_commands_for` both consume"
        );

        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "CastSpell" && a["object_id"].as_u64() == Some(commander_obj_2),
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(&state, &view, &action, json!({})).await;
        assert_eq!(
            sim1_commander_tax(&state),
            2,
            "the second cast increments the tax counter again"
        );

        // ── Sacrifice again (Culling the Weak, a different singleton card) ──
        let jadar_bf_2 = sim1_wait_for_jadar_on_battlefield(&state, SIM1_MAX_STEPS).await;
        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "CastSpell" && a["label"] == "Cast Culling the Weak",
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(
            &state,
            &view,
            &action,
            json!({
                "additional_costs": [{"Sacrifice": {"ids": [jadar_bf_2], "lki": []}}]
            }),
        )
        .await;

        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "ReturnCommanderToCommandZone",
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(&state, &view, &action, json!({})).await;

        // ── Cast 3 ("tax 2"): {1}{B} + the {4} CR 903.8 tax ──────────────────
        let commander_obj_3 = read_command_zone_object(&state);
        assert_eq!(
            mtg_simulator::effective_cast_cost(
                state
                    .session
                    .lock()
                    .expect("lock")
                    .as_ref()
                    .expect("session")
                    .game
                    .state(),
                mtg_engine::PlayerId(1),
                mtg_engine::ObjectId(commander_obj_3),
            )
            .expect("Jadar has a mana cost")
            .mana_value(),
            6,
            "CR 903.8/601.2f: {{1}}{{B}} (MV 2) plus two previous casts' {{4}} tax is MV 6"
        );

        let (view, action) = sim1_drive_until(
            &state,
            |a| a["kind"] == "CastSpell" && a["object_id"].as_u64() == Some(commander_obj_3),
            SIM1_MAX_STEPS,
        )
        .await;
        sim1_submit(&state, &view, &action, json!({})).await;

        let after_cast_3 = ui1_zone(&state, mtg_engine::ZoneId::Command(mtg_engine::PlayerId(1)));
        assert!(
            !after_cast_3.contains(&commander_obj_3),
            "the third, doubly-taxed cast really did leave the command zone: \
             {after_cast_3:?}"
        );
        assert_eq!(
            sim1_commander_tax(&state),
            3,
            "three casts, three increments — the tax counter does not stop at 2"
        );
    }

    // ── UI-2 (CR 118.8 / CR 702.157): additional-cost surfacing (task scutemob-178) ──
    //
    // Stage 3: the `view.rs`/`api.rs` rendering and validation, unit-tested against a
    // hand-built `GameState` / a synthetic `LegalAction`, immediately below. Stage 5's
    // HTTP probes (Life's Legacy over `POST /api/game/action`, Squad 0/1/N, SR-38
    // suppression) drive the real router end to end and are the section starting at
    // "UI-2 stage 5" further down this file.

    /// Build a minimal `GameState` for UI-2: `p1` holds Life's Legacy in hand
    /// with `{1}{G}` available and controls one eligible creature; `p2` exists
    /// so the state builds.
    ///
    /// Mirrors `crates/simulator/src/legal_actions.rs`'s own UI-2 test fixtures
    /// (`make_lifes_legacy`/`lifes_legacy_pool`, same card, same shape) --
    /// duplicated rather than shared, for the reason
    /// `setup_skullclamp_view_scenario` above already gives: no shared
    /// test-support crate between `crates/simulator` and `tools/play-server`.
    fn setup_lifes_legacy_view_scenario() -> (
        mtg_engine::GameState,
        mtg_engine::ObjectId,
        mtg_engine::ObjectId,
        mtg_engine::PlayerId,
    ) {
        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let defs: HashMap<String, mtg_engine::CardDefinition> = mtg_engine::all_cards()
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect();

        let lifes_legacy = mtg_engine::enrich_spec_from_def(
            mtg_engine::ObjectSpec::card(p1, "Life's Legacy")
                .with_card_id(mtg_engine::CardId("lifes-legacy".to_string()))
                .in_zone(mtg_engine::ZoneId::Hand(p1)),
            &defs,
        );
        let bear = mtg_engine::ObjectSpec::creature(p1, "P1 Bear", 2, 2)
            .in_zone(mtg_engine::ZoneId::Battlefield);

        let mut state = mtg_engine::GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(mtg_engine::CardRegistry::new(mtg_engine::all_cards()))
            .object(lifes_legacy)
            .object(bear)
            .active_player(p1)
            .at_step(mtg_engine::Step::PreCombatMain)
            .build()
            .unwrap();

        {
            let pool = &mut state.players_mut().get_mut(&p1).unwrap().mana_pool;
            pool.add(mtg_engine::ManaColor::Green, 1);
            pool.add(mtg_engine::ManaColor::White, 1);
        }
        state.turn_mut().priority_holder = Some(p1);

        let find = |name: &str, controller: mtg_engine::PlayerId| -> mtg_engine::ObjectId {
            state
                .objects()
                .iter()
                .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
                .map(|(id, _)| *id)
                .unwrap_or_else(|| panic!("object '{name}' controlled by {controller:?} not found"))
        };
        let card_id = find("Life's Legacy", p1);
        let bear_id = find("P1 Bear", p1);

        (state, card_id, bear_id, p1)
    }

    /// Render the wire `costs` value for Life's Legacy's `CastSpell` option in
    /// [`setup_lifes_legacy_view_scenario`]'s state, plus the eligible bear's id.
    /// Shared by the two tests below so each stays focused on its own assertion.
    fn lifes_legacy_costs_wire() -> (Value, mtg_engine::ObjectId) {
        use mtg_simulator::LegalActionProvider as _;

        let (state, card_id, bear_id, p1) = setup_lifes_legacy_view_scenario();
        let p2 = mtg_engine::PlayerId(2);
        let player_names: HashMap<mtg_engine::PlayerId, String> =
            [(p1, "Human-1".to_string()), (p2, "Bot-2".to_string())]
                .into_iter()
                .collect();

        let actions = mtg_simulator::StubProvider.legal_actions(&state, p1);
        let (index, _) = actions
            .iter()
            .enumerate()
            .find(|(_, a)| {
                matches!(a, mtg_simulator::LegalAction::CastSpell { card, .. } if *card == card_id)
            })
            .expect(
                "Life's Legacy must be offered: p1 has {1}{G} in the pool and an eligible \
                 creature to sacrifice",
            );

        let pending = mtg_simulator::PendingDecision {
            seq: 0,
            player: p1,
            kind: mtg_simulator::DecisionKind::Priority,
            actions,
        };
        let state_view =
            StateViewModel::from_game_state_for(&state, &player_names, Viewer::Seat(p1));
        let names = view::NameIndex::from_view(&state_view);
        let decision = view::decision_view(&pending, 0, &state, &names, &player_names);
        let wire = serde_json::to_value(&decision).expect("DecisionView serializes");
        (wire["actions"][index]["costs"].clone(), bear_id)
    }

    /// T: `ActionOptionView.costs` is `None` (wire `null`) for a plain spell with
    /// no additional cost, and `Some` with a populated `sacrifice` descriptor for
    /// Life's Legacy — the exact playtest-triage F9 gap (`CastSpell` offering a
    /// mandatory-sacrifice spell with no channel for the client to announce it).
    #[test]
    fn test_ui2_costs_field_is_none_for_plain_spell_and_populated_for_lifes_legacy() {
        use mtg_simulator::LegalActionProvider as _;

        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let defs: HashMap<String, mtg_engine::CardDefinition> = mtg_engine::all_cards()
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect();
        let bolt = mtg_engine::enrich_spec_from_def(
            mtg_engine::ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(mtg_engine::CardId("lightning-bolt".to_string()))
                .in_zone(mtg_engine::ZoneId::Hand(p1)),
            &defs,
        );
        let mut state = mtg_engine::GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(mtg_engine::CardRegistry::new(mtg_engine::all_cards()))
            .object(bolt)
            .active_player(p1)
            .at_step(mtg_engine::Step::PreCombatMain)
            .build()
            .unwrap();
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(mtg_engine::ManaColor::Red, 1);
        state.turn_mut().priority_holder = Some(p1);

        let player_names: HashMap<mtg_engine::PlayerId, String> =
            [(p1, "Human-1".to_string()), (p2, "Bot-2".to_string())]
                .into_iter()
                .collect();

        let actions = mtg_simulator::StubProvider.legal_actions(&state, p1);
        let bolt_index = actions
            .iter()
            .position(|a| matches!(a, mtg_simulator::LegalAction::CastSpell { .. }))
            .expect("Lightning Bolt must be offered");
        let pending = mtg_simulator::PendingDecision {
            seq: 0,
            player: p1,
            kind: mtg_simulator::DecisionKind::Priority,
            actions,
        };
        let state_view =
            StateViewModel::from_game_state_for(&state, &player_names, Viewer::Seat(p1));
        let names = view::NameIndex::from_view(&state_view);
        let decision = view::decision_view(&pending, 0, &state, &names, &player_names);
        let wire = serde_json::to_value(&decision).expect("DecisionView serializes");
        assert!(
            wire["actions"][bolt_index]["costs"].is_null(),
            "a plain spell's costs must be null: {:?}",
            wire["actions"][bolt_index]["costs"]
        );

        let (costs, bear_id) = lifes_legacy_costs_wire();
        assert!(
            !costs.is_null(),
            "Life's Legacy must carry a costs descriptor"
        );
        assert_eq!(costs["answer_field"], "additional_costs");
        assert!(
            costs["squad"].is_null(),
            "Life's Legacy has no Squad ability"
        );
        let sacrifice = &costs["sacrifice"];
        assert!(
            !sacrifice.is_null(),
            "Life's Legacy must offer a sacrifice picker"
        );
        assert_eq!(sacrifice["default"].as_u64(), Some(bear_id.0));
        let candidates = sacrifice["candidates"]
            .as_array()
            .expect("candidates is an array");
        assert!(
            candidates
                .iter()
                .any(|c| c["id"].as_u64() == Some(bear_id.0)),
            "the eligible bear must be among the sacrifice candidates: {candidates:?}"
        );
    }

    /// T: the sacrifice template round-trips as `{"Sacrifice":{"ids":[<default>],
    /// "lki":[]}}` — `lki` stays EMPTY on the wire because `casting.rs`'s
    /// sacrifice site (CR 118.8) patches it from LKI captured before the zone move
    /// (CR 608.2b/608.2h/608.2i); a client-supplied `lki` would be a second
    /// opinion about LKI the engine already owns.
    #[test]
    fn test_ui2_sacrifice_template_round_trips_with_lki_empty() {
        let (costs, bear_id) = lifes_legacy_costs_wire();
        let sacrifice = &costs["sacrifice"];
        assert_eq!(sacrifice["ids_key"], "ids");
        assert_eq!(
            sacrifice["template"],
            json!({"Sacrifice": {"ids": [bear_id.0], "lki": []}})
        );
    }

    /// A `LegalAction::CastSpell` carrying a fully-populated `AdditionalCostPlan`
    /// (one eligible sacrifice candidate, a Squad option with `max_count`), for
    /// unit-testing `api::validate_additional_cost_params` without going through
    /// HTTP (that probe is a later stage — see this section's header comment).
    fn ui2_cast_spell_action_with_costs(
        eligible: Vec<mtg_engine::ObjectId>,
        default: mtg_engine::ObjectId,
        squad_max_count: u32,
    ) -> mtg_simulator::LegalAction {
        mtg_simulator::LegalAction::CastSpell {
            card: mtg_engine::ObjectId(1),
            from_zone: mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1)),
            additional_costs: mtg_simulator::legal_actions::AdditionalCostPlan {
                sacrifice: Some(mtg_simulator::legal_actions::SacrificeCostOption {
                    requirement: mtg_engine::SpellAdditionalCost::SacrificeCreature,
                    eligible,
                    default,
                }),
                squad: Some(mtg_simulator::legal_actions::SquadCostOption {
                    cost: mtg_engine::ManaCost {
                        generic: 1,
                        ..Default::default()
                    },
                    max_count: squad_max_count,
                }),
            },
        }
    }

    /// T: a submitted `Sacrifice` naming an id outside the offered `eligible` set
    /// is refused 400 `bad_params` (CR 118.8) rather than reaching the engine.
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_out_of_set_sacrifice_id() {
        let eligible_id = mtg_engine::ObjectId(10);
        let action = ui2_cast_spell_action_with_costs(vec![eligible_id], eligible_id, 2);
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![mtg_engine::ObjectId(999)],
                lki: vec![],
            }],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("an id outside `eligible` must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// T: a submitted `Sacrifice` naming TWO ids is refused 400 — CR 118.8 /
    /// `casting.rs`'s own "exactly one mandatory sacrifice" support.
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_two_sacrifice_ids() {
        let eligible_a = mtg_engine::ObjectId(10);
        let eligible_b = mtg_engine::ObjectId(11);
        let action = ui2_cast_spell_action_with_costs(vec![eligible_a, eligible_b], eligible_a, 2);
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![eligible_a, eligible_b],
                lki: vec![],
            }],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("two sacrifice ids must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// T: a submitted `Squad { count }` above the offered `max_count` is refused
    /// 400 (CR 702.157a).
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_squad_over_max_count() {
        let eligible_id = mtg_engine::ObjectId(10);
        let action = ui2_cast_spell_action_with_costs(vec![eligible_id], eligible_id, 2);
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![mtg_engine::AdditionalCost::Squad { count: 3 }],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("a count above max_count must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// T: a submitted `Squad` on an action whose plan offers no Squad option is
    /// refused 400, rather than silently accepted.
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_squad_when_none_offered() {
        let eligible_id = mtg_engine::ObjectId(10);
        let action = mtg_simulator::LegalAction::CastSpell {
            card: mtg_engine::ObjectId(1),
            from_zone: mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1)),
            additional_costs: mtg_simulator::legal_actions::AdditionalCostPlan {
                sacrifice: Some(mtg_simulator::legal_actions::SacrificeCostOption {
                    requirement: mtg_engine::SpellAdditionalCost::SacrificeCreature,
                    eligible: vec![eligible_id],
                    default: eligible_id,
                }),
                squad: None,
            },
        };
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![mtg_engine::AdditionalCost::Squad { count: 1 }],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("Squad on an action offering none must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// **Fix cycle (review Issue 2): a DUPLICATE `Squad` entry is refused 400.**
    ///
    /// Two entries are not additive, and the engine resolves them silently:
    /// `casting.rs`'s destructuring loop is `squad_count = *count`, so the LAST wins
    /// and the first is dropped with no error and no diagnostic. A client that sent
    /// two would have one of them applied and never be told which — and, before the
    /// matching `effective_cast_cost_with_additional` fix, the auto-tap would have
    /// SUMMED them, reached for more mana than the engine charges, found no plan,
    /// and let the engine refuse the cast for want of mana. A 422 after a clean
    /// offer is exactly the SR-38 shape this batch exists to delete.
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_a_duplicate_squad_entry() {
        let eligible_id = mtg_engine::ObjectId(10);
        let action = ui2_cast_spell_action_with_costs(vec![eligible_id], eligible_id, 2);
        let params = crate::view::ActionParamsDto {
            // BOTH within `max_count`, so the per-entry bound check cannot be what
            // rejects this — only the duplicate check can.
            additional_costs: vec![
                mtg_engine::AdditionalCost::Squad { count: 2 },
                mtg_engine::AdditionalCost::Squad { count: 1 },
            ],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("two Squad announcements must be refused, not silently resolved");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// **Fix cycle (review Issue 2): a DUPLICATE `Sacrifice` entry is refused 400.**
    ///
    /// The other half, and it resolves the OTHER way: `casting.rs:186` extracts the
    /// sacrifice with a `find_map` over `ids.first()`, so the FIRST entry wins and
    /// the rest are dropped. A human who somehow announced two would watch one of
    /// their creatures die for no stated reason.
    #[test]
    fn test_ui2_validate_additional_cost_params_rejects_a_duplicate_sacrifice_entry() {
        let a = mtg_engine::ObjectId(10);
        let b = mtg_engine::ObjectId(11);
        let action = ui2_cast_spell_action_with_costs(vec![a, b], a, 0);
        let params = crate::view::ActionParamsDto {
            // BOTH ids are eligible and each entry is well-formed on its own, so
            // only the duplicate check can reject this.
            additional_costs: vec![
                mtg_engine::AdditionalCost::Sacrifice {
                    ids: vec![a],
                    lki: vec![],
                },
                mtg_engine::AdditionalCost::Sacrifice {
                    ids: vec![b],
                    lki: vec![],
                },
            ],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(&action, &params)
            .expect_err("two Sacrifice announcements must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// T: the happy path — a valid single eligible sacrifice id plus a Squad
    /// count within `max_count` — does NOT fire any of the four checks above.
    #[test]
    fn test_ui2_validate_additional_cost_params_accepts_the_happy_path() {
        let eligible_id = mtg_engine::ObjectId(10);
        let action = ui2_cast_spell_action_with_costs(vec![eligible_id], eligible_id, 2);
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![
                mtg_engine::AdditionalCost::Sacrifice {
                    ids: vec![eligible_id],
                    lki: vec![],
                },
                mtg_engine::AdditionalCost::Squad { count: 2 },
            ],
            ..Default::default()
        };
        api::validate_additional_cost_params(&action, &params)
            .expect("a legal sacrifice id and an in-bound squad count must be accepted");
    }

    // ── UI-2 stage 5 (CR 118.8 / CR 702.157): additional-cost surfacing, end to
    // end over HTTP (task scutemob-178) ─────────────────────────────────────
    //
    // The stage-3 tests just above check `view.rs`/`api.rs` against a hand-built
    // `GameState` and a synthetic `LegalAction`. These three probes drive the REAL
    // router — `session::new_game`, the same two Invariant-9 gates, the same
    // `LocalGame::submit` auto-tap — exactly the UI-1/SIM-1 pattern
    // (`ui1_install`/`sim1_install`), so nothing about the HTTP path is stubbed.
    //
    // P1: Life's Legacy's mandatory sacrifice, driven to a non-default pick.
    // P2: Galadhrim Brigade's Squad, declined (0) and paid twice (N=2).
    // P3: SR-38 — the offer is absent with no eligible creature, present with one.

    /// Reused for every UI-2 stage-5 fixture, for the identical reason
    /// [`SIM1_SEED`] reuses [`UI1_SEED`]: `setup::build_initial_state` shuffles each
    /// seat's `main_deck` with `SliceRandom::shuffle` on a single `StdRng` seeded
    /// from `cfg.seed` ALONE (`setup.rs:206`), and for a `DeckSource::Fixed` game
    /// player 1's shuffle is the FIRST rng draw at all (`setup.rs:227-244`'s
    /// `RandomPerSeat` branch, the only one that draws earlier, never runs). A
    /// `SliceRandom::shuffle` is a permutation of INDICES that depends only on the
    /// rng stream and the slice's LENGTH — never on what sits at each index — so
    /// for any 99-card `Fixed` main deck at this seed, the **same set of original
    /// positions** lands in player 1's opening 7-card hand.
    ///
    /// That set was computed directly (not assumed): replaying
    /// `(0..99).collect::<Vec<usize>>().shuffle(&mut StdRng::seed_from_u64(184))`
    /// gives original indices `[1, 53, 70, 19, 39, 50, 0]` as the top 7 — i.e. the
    /// hand is exactly positions `{0, 1, 19, 39, 50, 53, 70}`. UI-1's own doc only
    /// ever needed two of those (`main_deck[0]`/`main_deck[1]`); this batch needs a
    /// third (`main_deck[19]`) for the two-eligible-creature fixture (P1), and the
    /// pin above is what justifies using it.
    const UI2_SEED: u64 = UI1_SEED;

    /// `{10}{G}{G}`, Legendary Creature, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/ghalta_primal_hunger.rs`). Mono-green (fixes CR
    /// 903.5c colour identity to plain green) and the single most expensive
    /// mono-green creature in the corpus — `self_cost_reduction` only reduces it by
    /// the total power of creatures controlled, so even with both fixture
    /// creatures out (power 1 and 2) it is still far outside every probe's mana
    /// window (a handful of Forests). Reused as the commander for every UI-2
    /// stage-5 fixture, including the opponent seat's.
    const UI2_COMMANDER: &str = "ghalta-primal-hunger";

    /// `{1}{G}`, Sorcery, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/lifes_legacy.rs`). `SpellAdditionalCost::SacrificeCreature`
    /// — the F9 defect card this whole batch is about.
    const UI2_SAC_SPELL: &str = "lifes-legacy";

    /// `{G}`, Creature -- Phyrexian Elf Warrior 1/1, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/glistener_elf.rs`). Its only ability is the
    /// keyword Infect, which never fires here (no combat damage is ever dealt) --
    /// deliberately NOT a mana dork: an earlier draft used Llanowar Elves /
    /// Elvish Mystic here and both broke the fixture, because `LocalGame`'s
    /// auto-tap solver (playtest triage F4's known source-counting defect)
    /// would greedily reach for a JUST-CAST creature's own `{T}: Add {G}` ability
    /// to help pay for the SECOND creature and fail on "has summoning sickness
    /// and cannot tap for mana (no haste)" -- reproduced, not guessed at, by
    /// running this fixture with that pair first. No creature with a mana
    /// ability appears anywhere in this batch's fixtures for that reason.
    const UI2_ELF_A: &str = "glistener-elf";
    /// [`UI2_ELF_A`]'s rendered `CardDefinition.name` -- distinct from the `CardId`
    /// above, and load-bearing: both the battlefield-name lookups
    /// ([`ui2_battlefield_ids_by_name`]) and the driven action's label (`"Cast
    /// {name}"`) are keyed on this, never on the kebab-case `CardId`.
    const UI2_ELF_A_NAME: &str = "Glistener Elf";

    /// `{1}{G}`, Creature -- Ouphe 2/2, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/collector_ouphe.rs`). Its only ability is a
    /// static restriction ("Activated abilities of artifacts can't be
    /// activated") that never matters here -- no artifact is ever in play -- and,
    /// as important as [`UI2_ELF_A`]'s doc, it has NO mana ability either. Gives
    /// P1's two-eligible-creature fixture two DISTINCT sacrifice candidates
    /// rather than two copies of one; verified against an enumeration over
    /// `all_cards()`, not assumed, that this corpus has no mono-green creature
    /// simpler than this pair.
    const UI2_ELF_B: &str = "collector-ouphe";
    /// See [`UI2_ELF_A_NAME`]'s doc -- the same distinction, for [`UI2_ELF_B`].
    const UI2_ELF_B_NAME: &str = "Collector Ouphe";

    /// `{2}{G}`, Creature -- Elf Soldier 2/2, Squad `{1}{G}`, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/galadhrim_brigade.rs`) -- the exact card the
    /// first human playtest hit (F9: "spell has squad keyword but no squad cost
    /// defined").
    const UI2_SQUAD_SPELL: &str = "galadhrim-brigade";

    /// Generous bound for driving through 2-3 of the human's own turns (one land
    /// drop and one creature cast each), matching the order of magnitude of
    /// [`SIM1_MAX_STEPS`] for a similarly-shallow drive. Each step here is one
    /// HUMAN decision -- the bot's whole turn collapses into zero extra steps,
    /// since every route drives straight to the next human decision.
    const UI2_MAX_STEPS: usize = 500;

    /// Generous bound for driving through up to 7 of the human's own land drops
    /// (P2's `count = 2` fixture). Same "one step per human decision" accounting
    /// as [`UI2_MAX_STEPS`], scaled up for the extra turns.
    const UI2_LAND_DRIVE_MAX_STEPS: usize = 1500;

    /// CR 903.5c: `commander` plus 96-98 Forests plus 1-2 non-land probe cards,
    /// placed at whichever of `{0, 1, 19, 39, 50, 53, 70}` [`UI2_SEED`]'s doc names
    /// are needed -- the rest of the 99 slots are Forests. Shared builder for every
    /// UI-2 stage-5 deck; the probe-specific constructors below just choose which
    /// slots to overwrite.
    fn ui2_deck_with(commander: &str, overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("forest".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(commander.to_string()),
            main_deck,
        }
    }

    /// All-Forest deck (plus commander) -- the harmless opponent-seat fixture for
    /// every UI-2 stage-5 probe. No spell in this deck at all, so the bot can only
    /// ever play lands and pass; it cannot perturb anything this batch checks on
    /// player 1's side of the board.
    fn ui2_forest_only_deck() -> mtg_simulator::DeckConfig {
        ui2_deck_with(UI2_COMMANDER, &[])
    }

    /// P1's two-eligible-creature fixture: Life's Legacy at position 0,
    /// [`UI2_ELF_A`] at position 1, [`UI2_ELF_B`] at position 19 -- all three of
    /// [`UI2_SEED`]'s pinned hand positions that are not left as Forest.
    fn ui2_lifes_legacy_two_elves_deck() -> mtg_simulator::DeckConfig {
        ui2_deck_with(
            UI2_COMMANDER,
            &[(0, UI2_SAC_SPELL), (1, UI2_ELF_A), (19, UI2_ELF_B)],
        )
    }

    /// P3 half B's one-eligible-creature fixture: Life's Legacy at position 0,
    /// [`UI2_ELF_A`] at position 1, Forest everywhere else (including position 19,
    /// unlike the two-elf deck above) -- the minimal fixture that has an eligible
    /// sacrifice target at all.
    fn ui2_lifes_legacy_one_elf_deck() -> mtg_simulator::DeckConfig {
        ui2_deck_with(UI2_COMMANDER, &[(0, UI2_SAC_SPELL), (1, UI2_ELF_A)])
    }

    /// P3 half A's no-creature fixture: Life's Legacy at position 0, Forest
    /// everywhere else. No creature anywhere in this 99-card deck, so "0 eligible
    /// creatures" is guaranteed by construction rather than by a drive that merely
    /// never got around to casting one.
    fn ui2_lifes_legacy_no_creature_deck() -> mtg_simulator::DeckConfig {
        ui2_deck_with(UI2_COMMANDER, &[(0, UI2_SAC_SPELL)])
    }

    /// P2's fixture: Galadhrim Brigade at position 0, Forest everywhere else.
    fn ui2_squad_deck() -> mtg_simulator::DeckConfig {
        ui2_deck_with(UI2_COMMANDER, &[(0, UI2_SQUAD_SPELL)])
    }

    /// Install a two-player fixed-deck session through the same constructor the
    /// real handler uses -- see [`ui1_install`]'s doc for why `POST /api/game`
    /// itself cannot express this fixture.
    fn ui2_install(
        state: &SharedState,
        p1_deck: mtg_simulator::DeckConfig,
        p2_deck: mtg_simulator::DeckConfig,
    ) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI2_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), p1_deck),
                (mtg_engine::PlayerId(2), p2_deck),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the UI-2 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Out-of-band oracle, [`ui1_zone`]'s role: every battlefield permanent
    /// `controller` controls whose rendered name is `name`, by `ObjectId`. Reads
    /// the engine's own object table directly -- never used to build a payload,
    /// only to verify what an HTTP-driven cast actually did.
    fn ui2_battlefield_ids_by_name(
        state: &SharedState,
        controller: mtg_engine::PlayerId,
        name: &str,
    ) -> Vec<u64> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let gs = session.game.state();
        gs.zones()
            .get(&mtg_engine::ZoneId::Battlefield)
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| {
                let obj = gs.objects().get(&id)?;
                if obj.controller == controller && obj.characteristics.name == name {
                    Some(id.0)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Count form of [`ui2_battlefield_ids_by_name`], for P2's "how many copies"
    /// assertions -- CR 400.7 mints a fresh `ObjectId` for a real cast and every
    /// token copy alike, so counting BY NAME is the only stable check.
    fn ui2_battlefield_count_by_name(
        state: &SharedState,
        controller: mtg_engine::PlayerId,
        name: &str,
    ) -> usize {
        ui2_battlefield_ids_by_name(state, controller, name).len()
    }

    /// Rendered names of every object currently in `zone`, out of band. Used for
    /// P1's graveyard check: CR 400.7 means the sacrificed creature's BATTLEFIELD
    /// `ObjectId` never appears in the graveyard at all (`move_object_to_zone`
    /// mints a fresh one and clones `characteristics` across the move), so "did
    /// the sacrifice happen" has to be read back by NAME, not by the id that was
    /// submitted.
    fn ui2_zone_names(state: &SharedState, zone: mtg_engine::ZoneId) -> Vec<String> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let gs = session.game.state();
        gs.zones()
            .get(&zone)
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| {
                gs.objects()
                    .get(&id)
                    .map(|o| o.characteristics.name.clone())
            })
            .collect()
    }

    /// Out-of-band oracle: `player`'s current mana pool total, for P2's "the
    /// mana really was charged" assertion.
    fn ui2_mana_pool_total(state: &SharedState, player: mtg_engine::PlayerId) -> u32 {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .player(player)
            .expect("player exists")
            .mana_pool
            .total()
    }

    /// Drive the human seat -- playing a land, else casting whichever of
    /// `elf_names` is not yet on this seat's battlefield, else passing priority --
    /// until an offered action is `"Cast Life's Legacy"` AND every name in
    /// `elf_names` is already controlled on the battlefield. `elf_names` is
    /// checked in order, so the same helper serves P1's two-creature fixture and
    /// P3 half B's one-creature fixture.
    ///
    /// The two conditions are checked TOGETHER (not "stop as soon as every elf is
    /// out") because Life's Legacy is a sorcery (CR 117.1a): with a creature still
    /// resolving on the stack, `can_cast_at_this_time`'s stack-empty gate means the
    /// spell is not offered yet even once every elf is technically controlled --
    /// the loop must keep passing priority until the stack actually drains.
    async fn ui2_drive_to_lifes_legacy_offer(
        state: &SharedState,
        elf_names: &[&str],
        max_steps: usize,
    ) -> (Value, Value) {
        let p1 = mtg_engine::PlayerId(1);
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            let all_present = elf_names
                .iter()
                .all(|name| !ui2_battlefield_ids_by_name(state, p1, name).is_empty());
            if all_present {
                if let Some(action) = view["decision"]["actions"]
                    .as_array()
                    .and_then(|actions| {
                        actions.iter().find(|a| {
                            a["kind"] == "CastSpell" && a["label"] == "Cast Life's Legacy"
                        })
                    })
                    .cloned()
                {
                    return (view, action);
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before Life's Legacy (with {elf_names:?} \
                 in play) was offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let next_elf_label = elf_names
                .iter()
                .find(|name| ui2_battlefield_ids_by_name(state, p1, name).is_empty())
                .map(|name| format!("Cast {name}"));
            let pick = next_elf_label
                .as_deref()
                .and_then(|label| {
                    actions
                        .iter()
                        .find(|a| a["kind"] == "CastSpell" && a["label"] == label)
                })
                .or_else(|| actions.iter().find(|a| a["kind"] == "PlayLand"))
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "step {step} submitting {}: {next}",
                pick["label"]
            );
            view = next;
        }
        panic!(
            "Life's Legacy (with {elf_names:?} in play) was never offered within \
             {max_steps} steps: {view}"
        );
    }

    /// Drive the human seat -- playing a land every turn it can, else passing
    /// priority -- until `land_target` lands have been played, then return the
    /// view at that point. Never casts anything (P2's fixture has exactly one
    /// non-land card, and the whole point is to accumulate MORE mana than its base
    /// cost needs before ever casting it).
    async fn ui2_drive_playing_lands(
        state: &SharedState,
        land_target: usize,
        max_steps: usize,
    ) -> Value {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        let mut lands_played = 0usize;
        for step in 0..max_steps {
            if lands_played >= land_target {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before {land_target} lands were played: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let is_land = pick["kind"] == "PlayLand";
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "step {step} submitting {}: {next}",
                pick["label"]
            );
            view = next;
            if is_land {
                lands_played += 1;
            }
        }
        assert!(
            lands_played >= land_target,
            "only {lands_played}/{land_target} lands played within {max_steps} steps"
        );
        view
    }

    /// Pass priority until the stack is empty (a cast spell -- and any trigger it
    /// queues -- has fully resolved), the same drain loop
    /// [`test_ui1_trigger_targets_are_answered_over_http`] uses for the identical
    /// reason: a `CastSpell` command does not resolve synchronously (CR 117.3c
    /// hands priority back to the actor, not straight to resolution).
    async fn ui2_drain_stack(state: &SharedState, view: Value, max_steps: usize) -> Value {
        let mut view = view;
        for _ in 0..max_steps {
            let stack_empty = view["state"]["zones"]["stack"]
                .as_array()
                .map(|s| s.is_empty())
                .unwrap_or(true);
            if stack_empty {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended mid-stack: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"].as_array().unwrap().clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .expect("something other than Concede");
            let index = pick["index"].as_u64().unwrap();
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{next}");
            view = next;
        }
        panic!("the stack never drained within {max_steps} steps: {view}");
    }

    /// **CR 118.8 -- Life's Legacy's mandatory sacrifice, driven to a non-default
    /// pick over HTTP.** Closes playtest-triage F9's Life's Legacy half and
    /// criterion 5997.
    ///
    /// Three things, in the order the task brief asks for: (1) the descriptor
    /// itself -- present, well-formed, naming a real eligible creature; (2) the
    /// reproduction -- an id this decision never offered as eligible is refused at
    /// the 400 boundary, never reaching the engine's own 422 (see this function's
    /// doc for why the true pre-fix 422 is no longer reachable through the HTTP
    /// surface at all, and how that was checked rather than assumed); (3) the real
    /// cast, with the NON-default candidate, verified out of band against the
    /// engine's own graveyard/library/battlefield state.
    ///
    /// # Why the pre-fix 422 could not be reproduced by submitting an empty answer
    ///
    /// The obvious reproduction -- submit `"additional_costs": []` and watch the
    /// engine refuse it -- no longer exists: `params.rs`'s
    /// `merge_required_additional_costs` APPENDS the plan's default sacrifice the
    /// moment no `Sacrifice` is announced, precisely so a bot's default-params cast
    /// stays engine-legal (SR-38). So an empty announcement now succeeds instead of
    /// 422ing, and that success IS the fix, not a hole in this test.
    ///
    /// # Why an out-of-set id 400s instead of 422ing, and whether the 422 is
    /// reachable at all through this surface -- checked, not assumed
    ///
    /// `api::validate_additional_cost_params` checks a submitted `Sacrifice` id
    /// against `plan.sacrifice.eligible` BEFORE any command reaches the engine, and
    /// `legal_actions::build_additional_cost_plan`'s eligibility mirrors
    /// `casting.rs`'s own gate (filter, controller, zone, `CantBeSacrificed`) by
    /// construction (UI-2 plan §1.2) -- so any id this 400 boundary accepts is,
    /// by that mirror, an id the engine's own check would ALSO accept. There is
    /// therefore no id a well-formed HTTP submission can carry that passes this
    /// 400 gate and still fails the engine's 422 -- unlike UI-1's search/discard
    /// probes, where the 400 boundary and the engine's own check look at different
    /// things. Here they look at the same set, by design.
    ///
    /// This was verified empirically, not reasoned to only in prose: with
    /// `api.rs`'s `validate_additional_cost_params(action, &req.params)?` call
    /// temporarily commented out of `post_action` (`api.rs:1140`), submitting this
    /// exact test's out-of-set land id reached the engine directly and came back
    /// **422** with body (verbatim, `kind: "rejected"` -- `LocalGameError::Rejected`,
    /// NOT `"engine_error"`, which this crate reserves for the separately-unreachable
    /// `LocalGameError::Engine` variant per this file's `impl From<LocalGameError>`):
    /// `{"error":"invalid command: spell additional cost: sacrificed permanent \
    /// does not match required filter SacrificeCreature (CR 118.8)","kind":"rejected"}`
    /// -- i.e. `casting.rs:3360-3364`'s own filter-mismatch message, reached and
    /// observed, then the call site was restored and the full suite re-run green.
    /// That is the F9 defect's ORIGINAL shape (a client, offered nothing, submits
    /// something the engine refuses); UI-2's fix moves the refusal from that 422
    /// to this test's 400 and leaves it unreachable beyond that boundary.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui2_lifes_legacy_sacrifice_is_answered_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        ui2_install(
            &state,
            ui2_lifes_legacy_two_elves_deck(),
            ui2_forest_only_deck(),
        );

        let (view, action) = ui2_drive_to_lifes_legacy_offer(
            &state,
            &[UI2_ELF_A_NAME, UI2_ELF_B_NAME],
            UI2_MAX_STEPS,
        )
        .await;
        let index = action["index"].as_u64().expect("index is a number");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let costs = &option["costs"];
        assert!(
            !costs.is_null(),
            "Life's Legacy must carry a costs descriptor"
        );
        assert_eq!(costs["answer_field"], "additional_costs");
        assert!(
            costs["squad"].is_null(),
            "Life's Legacy has no Squad ability"
        );
        let sacrifice = &costs["sacrifice"];
        assert!(
            !sacrifice.is_null(),
            "Life's Legacy must offer a sacrifice picker"
        );

        let candidates: Vec<(u64, String)> = sacrifice["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| {
                (
                    c["id"].as_u64().expect("id is a number"),
                    c["label"].as_str().expect("label is a string").to_string(),
                )
            })
            .collect();
        assert_eq!(
            candidates.len(),
            2,
            "both fixture creatures must be eligible candidates: {candidates:?}"
        );
        let names: Vec<&str> = candidates.iter().map(|(_, n)| n.as_str()).collect();
        assert!(names.contains(&UI2_ELF_A_NAME), "{names:?}");
        assert!(names.contains(&UI2_ELF_B_NAME), "{names:?}");

        let default = sacrifice["default"].as_u64().expect("default is a number");
        let min_id = candidates
            .iter()
            .map(|(id, _)| *id)
            .min()
            .expect("non-empty");
        assert_eq!(
            default, min_id,
            "the default is eligible[0] -- the lowest ObjectId"
        );
        assert_eq!(sacrifice["ids_key"], "ids");
        assert_eq!(
            sacrifice["template"],
            json!({"Sacrifice": {"ids": [default], "lki": []}})
        );

        let wire_seq = seq(&view);

        // Reproduction: an id this decision never offered as eligible (a Forest,
        // not a creature) is refused at the 400 boundary -- see this test's doc
        // for why the true engine 422 is no longer reachable beyond it.
        let land_id = ui2_battlefield_ids_by_name(&state, p1, "Forest")
            .into_iter()
            .next()
            .expect("at least one Forest must be in play by now");
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {
                    "additional_costs": [{"Sacrifice": {"ids": [land_id], "lki": []}}]
                }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // The real cast: the NON-default candidate, so the answer discriminates
        // from a client that submitted nothing (which would have sacrificed the
        // default instead).
        let (chosen, chosen_name) = candidates
            .iter()
            .find(|(id, _)| *id != default)
            .cloned()
            .expect("two distinct candidates, so a non-default one exists");
        let (survivor, survivor_name) = candidates
            .iter()
            .find(|(id, _)| *id != chosen)
            .cloned()
            .expect("the other candidate");
        assert_eq!(
            survivor, default,
            "sanity: the two candidates are default+chosen"
        );

        let library_before = ui1_library(&state).len();
        let graveyard_before = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard_before.is_empty(),
            "sanity: nothing has died yet: {graveyard_before:?}"
        );

        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {
                    "additional_costs": [{"Sacrifice": {"ids": [chosen], "lki": []}}]
                }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        let view = ui2_drain_stack(&state, after_cast, 40).await;

        // The chosen creature is gone (CR 400.7: a fresh graveyard object was
        // minted, so this checks by NAME, never by the submitted id).
        assert!(
            ui2_battlefield_ids_by_name(&state, p1, &chosen_name).is_empty(),
            "the sacrificed creature ({chosen_name}) must have left the battlefield"
        );
        assert!(
            !ui2_battlefield_ids_by_name(&state, p1, &survivor_name).is_empty(),
            "the OTHER (default) creature ({survivor_name}) must still be controlled -- \
             a client submitting the default would have killed this one instead"
        );

        let graveyard_after = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert_eq!(
            graveyard_after.len(),
            2,
            "the sacrificed creature AND the resolved sorcery both end in the \
             graveyard: {graveyard_after:?}"
        );
        assert!(
            graveyard_after.contains(&chosen_name),
            "the sacrificed creature must be in the graveyard by name: {graveyard_after:?}"
        );
        assert!(
            graveyard_after.contains(&"Life's Legacy".to_string()),
            "the resolved sorcery itself must be in the graveyard: {graveyard_after:?}"
        );

        // CR 608.2b: Life's Legacy draws cards equal to the sacrificed creature's
        // power -- [`UI2_ELF_A`] and [`UI2_ELF_B`] print DIFFERENT power (1 and 2),
        // deliberately, so this reads the expected count off whichever name was
        // actually chosen rather than a single hard-coded number.
        let expected_draw = if chosen_name == UI2_ELF_A_NAME {
            1
        } else if chosen_name == UI2_ELF_B_NAME {
            2
        } else {
            panic!("unexpected sacrificed creature name: {chosen_name}");
        };
        let library_after = ui1_library(&state).len();
        assert_eq!(
            library_after,
            library_before - expected_draw,
            "Life's Legacy must have drawn exactly {expected_draw} card(s) (the \
             sacrificed {chosen_name}'s power)"
        );
        let _ = view;
    }

    /// **CR 702.157a -- Squad, declined (count 0).** Closes half of criterion
    /// 5998.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui2_squad_declined_casts_plain_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        ui2_install(&state, ui2_squad_deck(), ui2_forest_only_deck());

        // 3 lands = Galadhrim Brigade's own base cost ({2}{G}), nothing spare.
        let view = ui2_drive_playing_lands(&state, 3, UI2_LAND_DRIVE_MAX_STEPS).await;
        let action = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Galadhrim Brigade")
            .cloned()
            .unwrap_or_else(|| {
                panic!("Galadhrim Brigade must be offered once its base cost is affordable: {view}")
            });
        let index = action["index"].as_u64().expect("index is a number");
        let costs = &action["costs"];
        assert!(
            !costs.is_null(),
            "Galadhrim Brigade must carry a costs descriptor"
        );
        assert_eq!(costs["answer_field"], "additional_costs");
        assert!(
            costs["sacrifice"].is_null(),
            "Galadhrim Brigade has no additional sacrifice cost"
        );
        let squad = &costs["squad"];
        assert!(
            !squad.is_null(),
            "Galadhrim Brigade must offer a Squad picker"
        );
        // The label must match the PRINTING: Galadhrim Brigade prints "Squad {1}{G}",
        // so `format_mana_cost_compact` emits the generic component first. (The TUI's
        // own copy of that formatter emits colours first and would render `{G}{1}`;
        // see that function's doc for why this one deliberately diverges.)
        assert_eq!(squad["cost_label"], "{1}{G}");
        assert_eq!(squad["count_key"], "count");
        assert_eq!(squad["template"], json!({"Squad": {"count": 0}}));
        assert_eq!(
            squad["max_count"].as_u64(),
            Some(0),
            "with exactly 3 mana available and a base cost of 3, nothing is left \
             over for Squad"
        );

        let wire_seq = seq(&view);
        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": index, "params": {}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        ui2_drain_stack(&state, after_cast, 20).await;
        assert_eq!(
            ui2_battlefield_count_by_name(&state, p1, "Galadhrim Brigade"),
            1,
            "declining Squad must cast the spell plainly -- no token copies"
        );
    }

    /// **CR 702.157a -- Squad, paid twice (count 2).** Closes the other half of
    /// criterion 5998, going one step past "N >= 1" per the task brief's
    /// preference, since 7 lands are reachable within this fixture's budget and a
    /// count of 2 discriminates "count is read" from "count is a boolean" in a way
    /// count 1 cannot.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui2_squad_paying_twice_produces_two_token_copies_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        ui2_install(&state, ui2_squad_deck(), ui2_forest_only_deck());

        // 7 lands: base cost 3 + 2 x Squad's {1}{G} (MV 2) = 7, exactly.
        let view = ui2_drive_playing_lands(&state, 7, UI2_LAND_DRIVE_MAX_STEPS).await;
        let action = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Galadhrim Brigade")
            .cloned()
            .unwrap_or_else(|| panic!("Galadhrim Brigade must still be offered: {view}"));
        let index = action["index"].as_u64().expect("index is a number");
        let squad = &action["costs"]["squad"];
        assert!(!squad.is_null());
        let max_count = squad["max_count"].as_u64().expect("max_count is a number");
        assert_eq!(
            max_count, 2,
            "7 mana available, base cost 3, Squad {{1}}{{G}} (MV 2) per payment -> \
             exactly 2 affordable"
        );

        let wire_seq = seq(&view);

        // The over-count 400, using the offer's OWN max_count rather than a
        // hard-coded number.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": [{"Squad": {"count": max_count + 1}}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params");

        // The real cast, at the full max_count.
        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": [{"Squad": {"count": max_count}}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        // The creature spell resolves, THEN its CR 702.157a ETB trigger goes on the
        // stack and must ALSO resolve before the token copies exist.
        ui2_drain_stack(&state, after_cast, 40).await;

        assert_eq!(
            ui2_battlefield_count_by_name(&state, p1, "Galadhrim Brigade"),
            (max_count + 1) as usize,
            "1 real permanent plus {max_count} token copies (CR 702.157a)"
        );
        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "all 7 available mana must have been spent -- base {{2}}{{G}} plus 2x \
             Squad {{1}}{{G}}"
        );
    }

    /// **SR-38 (criterion 5999) -- the offer is absent with no eligible creature,
    /// present with one.** Two-sided in the same test, exactly the shape
    /// [`test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`]'s
    /// Invariant-7 check uses for the same reason: a one-sided absence assertion
    /// alone cannot distinguish "correctly suppressed" from "broken and never
    /// offers anything".
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui2_lifes_legacy_offer_suppressed_without_an_eligible_creature_over_http() {
        let p1 = mtg_engine::PlayerId(1);

        // Half A: no creature anywhere in this 99-card deck (by construction, not
        // by a drive that merely never got around to casting one).
        let state_a = shared_state();
        ui2_install(
            &state_a,
            ui2_lifes_legacy_no_creature_deck(),
            ui2_forest_only_deck(),
        );
        // 2 lands = Life's Legacy's own cost ({1}{G}), so the offer's absence
        // below is due to the missing creature, not missing mana.
        let view_a = ui2_drive_playing_lands(&state_a, 2, UI2_LAND_DRIVE_MAX_STEPS).await;
        assert!(
            ui2_battlefield_ids_by_name(&state_a, p1, UI2_ELF_A_NAME).is_empty(),
            "sanity: this deck has no creature at all"
        );
        let actions_a = view_a["decision"]["actions"]
            .as_array()
            .expect("actions is an array");
        assert!(
            !actions_a
                .iter()
                .any(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Life's Legacy"),
            "SR-38: with no eligible creature to sacrifice, and the engine's own \
             gate refusing the cast outright, Life's Legacy must not be offered at \
             all -- offering it would be the F9 defect: {actions_a:?}"
        );

        // Half B: the SAME mana window, but with one eligible creature.
        let state_b = shared_state();
        ui2_install(
            &state_b,
            ui2_lifes_legacy_one_elf_deck(),
            ui2_forest_only_deck(),
        );
        let (_, action_b) =
            ui2_drive_to_lifes_legacy_offer(&state_b, &[UI2_ELF_A_NAME], UI2_MAX_STEPS).await;
        assert_eq!(action_b["label"], "Cast Life's Legacy");
        assert!(
            !action_b["costs"]["sacrifice"].is_null(),
            "with an eligible creature in play, the sacrifice descriptor must be \
             present: {action_b}"
        );
    }

    // ── 15 ────────────────────────────────────────────────────────────────────

    /// **Architecture Invariant 7's chokepoint, machine-enforced instead of
    /// asserted in prose.**
    ///
    /// The README says "Neither omniscient entry point (`from_game_state`,
    /// `Viewer::Omniscient`) is reachable from the production paths of this
    /// crate", and `view.rs`'s module doc says every label comes from a
    /// [`view::NameIndex`] derived from the seat-redacted view. Until this test
    /// both were held by review alone.
    ///
    /// # Why a source gate and not a behavioural test — the answer was measured
    ///
    /// The obvious behavioural test is "swap the view `NameIndex` is built from
    /// and watch something go red". **Nothing goes red.** That was run, not
    /// assumed: `api.rs::seat_view` was edited to build its `NameIndex` from
    /// `StateViewModel::from_game_state` (the omniscient path) and the whole
    /// crate stayed green — all 23 tests, including
    /// `test_target_option_labels_are_seat_redacted` **and** S5's whole-body
    /// sweep `test_seat_view_over_http_contains_no_other_hand_card_names`.
    ///
    /// The reason is structural rather than a gap in those tests. `NameIndex` is
    /// only ever *queried* for ids that appear in an action, a target candidate
    /// or a combat list — and every one of those comes from a public zone
    /// (`legal_targets_per_slot` enumerates Battlefield / Stack / Graveyard
    /// only; combatants are battlefield creatures; a `CastSpell` names the
    /// seat's **own** hand card, which it is entitled to). On every id that ever
    /// gets labelled, the omniscient and redacted views **agree**. The one
    /// construct that would separate them is a face-down battlefield permanent
    /// (CR 708.2a), and no seeded game the fixture sweep found puts one on the
    /// board.
    ///
    /// So the invariant is real, currently unfalsifiable by any payload this
    /// crate can produce, and therefore exactly the kind of claim that rots. A
    /// source gate catches the edit; a fixture would catch the consequence, and
    /// there is no reachable fixture. Both facts are recorded rather than one of
    /// them being implied.
    ///
    /// # What is checked
    ///
    /// Over the **production region** of every `.rs` file under `src/` — the
    /// part above the `#[cfg(test)]` cut, comment- and string-blanked by
    /// [`code_only`], so a doc comment naming the symbol cannot satisfy or trip
    /// it — neither omniscient entry point may appear. The test module below the
    /// cut is deliberately exempt: it reaches `from_game_state` on purpose, as
    /// the out-of-band oracle the redaction tests check the payload against.
    ///
    /// Needles are assembled with `concat!` for the same reason the no-socket
    /// gate does it: this function sits below the cut in a file the gate reads,
    /// and a plainly-written needle would be found by the sibling gate.
    ///
    /// # What this gate cannot see (review MR-M11-04)
    ///
    /// It scans for *view-model* entry points, so it is blind to a route that
    /// serializes engine types directly and never touches the view model at all —
    /// and that is not hypothetical: it is exactly how the crate's one real
    /// Invariant-7 exception shipped. `GET /api/game/report` returns
    /// [`view::BugReportView`], which serializes `mtg_engine::Command` and
    /// `mtg_engine::GameEvent` verbatim; it was added, shipped and reviewed with this
    /// gate green, because there was nothing here for the gate to match on.
    ///
    /// The failure message above was therefore narrowed to the property actually
    /// checked ("every label rendered **through the view model**"), and the wider
    /// claim it used to assert now lives where it belongs: with
    /// [`test_mr_m11_01_seat_payload_carries_no_reconstruction_key`], which asserts
    /// over the raw response body, and with the README's exception section. The
    /// general lesson is the one MR-M11-01 turns on — **a redaction gate checks the
    /// channel it was written for, and a new channel is invisible to it**.
    #[test]
    fn test_production_code_never_builds_an_omniscient_view() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_rs_files(&root.join("src"), &mut sources);
        assert!(!sources.is_empty(), "the walk found no source files");

        let needles = [
            concat!("from_game_", "state("),
            concat!("Viewer::", "Omniscient"),
        ];

        let mut checked = 0usize;
        for (name, source) in &sources {
            // Everything above the `#[cfg(test)]` cut is production code. A file
            // with no cut at all is production code in full.
            let region = test_region(source);
            let production_len = source.len() - region.len();
            let production = code_only(&source[..production_len]);
            for needle in needles {
                assert!(
                    !production.contains(needle),
                    "{name} reaches an omniscient view from production code (matched \
                     {needle:?}). Every label this crate renders **through the view \
                     model** must come from \
                     `StateViewModel::from_game_state_for(.., Viewer::Seat(..))` — \
                     Architecture Invariant 7. If this is deliberate, the README's \
                     'Hidden information' section has to change with it."
                );
            }
            checked += 1;
        }
        assert!(checked > 0, "vacuous: no production region was scanned");

        // Non-vacuity — and the two needles are NOT in the same position, which
        // this check established by going red the first time it ran, on a draft
        // that assumed they were.
        //
        // `from_game_state(` is used for real, in this file's test region, as the
        // out-of-band oracle the redaction tests compare the payload against. So
        // finding it there proves the scan above matches real code rather than
        // nothing at all.
        let (_, this_file) = sources
            .iter()
            .find(|(name, _)| name.ends_with("main.rs"))
            .expect("the walk found this file");
        let tests_here = code_only(test_region(this_file));
        assert!(
            tests_here.contains(needles[0]),
            "vacuous: needle {:?} matches nothing even in the test region, so the \
             production scan above proved nothing",
            needles[0]
        );

        // The second needle is a **forward guard**, and that is a weaker claim
        // stated rather than glossed: no code in this crate names it today, in
        // either region — the omniscient path is reached through the
        // `from_game_state` shim — so there is nothing real to find it in and the
        // check above cannot be repeated for it. What is pinned instead is that
        // the *mechanism* would catch a future call site: `code_only` leaves the
        // symbol intact in code and blanks it inside a comment, so production
        // code naming it trips the scan while a doc comment naming it does not.
        let sample = code_only("let v = Viewer::Omnisc\u{69}ent; // Viewer::Omnisc\u{69}ent\n");
        assert!(
            sample.contains(needles[1]),
            "the gate cannot see {:?} in code at all",
            needles[1]
        );
        assert_eq!(
            sample.matches(needles[1]).count(),
            1,
            "the gate must see {:?} in code but not in a comment",
            needles[1]
        );
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

    /// Walk `dir` recursively, collecting every frontend source file as
    /// `(path, text)`. Companion to [`collect_rs_files`] for the Svelte client.
    ///
    /// `node_modules/` and `dist/` are skipped: neither is authored here, both
    /// are gitignored, and a bundled dependency that happens to deep-copy is not
    /// this crate's rule to enforce. The extensions are the three this client
    /// actually contains — a new one (`.ts`, `.svelte.ts`) is NOT silently
    /// covered, which is why the caller asserts a file-count floor.
    fn collect_frontend_files(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "node_modules" || name == "dist" {
                continue;
            }
            if path.is_dir() {
                collect_frontend_files(&path, out);
            } else if path
                .extension()
                .is_some_and(|ext| ext == "svelte" || ext == "js" || ext == "css")
            {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{} is unreadable: {e}", path.display()));
                out.push((path.display().to_string(), text));
            }
        }
    }

    /// **UI-4 (`scutemob-185`), G1 of `memory/playtest-triage-2026-08-02b.md` —
    /// no file in the Svelte client may hand a value to a platform primitive that
    /// rejects `Proxy` objects.**
    ///
    /// # The defect this exists to prevent recurring
    ///
    /// `ActionBar.svelte` holds the option being answered in `$state`. Svelte 5
    /// wraps that in a `Proxy` and deep-proxies on read, so every DTO threaded
    /// down to a picker as a prop is a proxy by the time it arrives. The
    /// structured-clone algorithm rejects proxies outright — `DataCloneError:
    /// #<Object> could not be cloned.` — and the throw escapes an ordinary DOM
    /// handler without touching the DOM. Three pickers took a deep copy of their
    /// answer template that way, and **five CR flows were dead in the browser for
    /// the entire life of the feature**: library search (CR 701.23), scry
    /// (CR 701.22a), surveil (CR 701.25a), sacrifice additional costs (CR 118.8)
    /// and Squad (CR 702.157a).
    ///
    /// The sanctioned replacement is `frontend/src/lib/plainClone.svelte.js`,
    /// which wraps Svelte's own `$state.snapshot`.
    ///
    /// # Why a source gate and not a test of the components
    ///
    /// Because there is still no frontend test harness (plan §8 R7) — that is the
    /// standing debt this defect collected on, and it is deliberately not paid
    /// here. A source gate cannot prove a picker works; it can prove the one
    /// three-line pattern that broke all three of them is absent, and it costs
    /// nothing to run. When the harness lands, this stays: it is a *class* rule,
    /// covering a Worker or a persistence layer that does not exist yet.
    ///
    /// # Vacuity
    ///
    /// A ban with zero permitted uses is exactly the shape that rots into a gate
    /// over an empty file set (the pinned-empty-roster problem, PB-DX6 R2/R4). So
    /// four things are asserted rather than assumed: the walk saw a floor of
    /// files **and** every file the rule is about by name; the three pickers each
    /// call the sanctioned helper; the helper is really implemented in terms of
    /// `$state.snapshot`; and the matcher is fired at a synthetic offending line
    /// to prove it discriminates. Delete any one of those and this test goes red
    /// rather than green-on-nothing.
    ///
    /// No `concat!` splitting is needed (unlike
    /// `test_no_socket_symbol_appears_in_the_test_region`): this file is Rust and
    /// the walk reads `frontend/src/` only, so the gate cannot match itself.
    #[test]
    fn test_frontend_never_structured_clones_reactive_state() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);

        // Call forms, not bare identifiers: the prose in `plainClone.svelte.js`
        // has to be able to name what it replaced. `indexedDB` is spelled with a
        // leading lowercase because that is the global's actual name — the docs
        // that discuss it write "IndexedDB", which is a different string.
        let forbidden = ["structuredClone(", ".postMessage(", "indexedDB"];

        for (path, text) in &sources {
            for needle in forbidden {
                assert!(
                    !text.contains(needle),
                    "{path} calls {needle:?}. Every DTO in this client reaches a component \
                     as a Svelte 5 reactive proxy, and that primitive rejects proxies with \
                     a DataCloneError thrown out of the click handler — no request, no \
                     error strip, nothing on screen. Use `plainClone` from \
                     `lib/plainClone.svelte.js` instead. This is UI-4 (`scutemob-185`, G1); \
                     it cost five CR flows the last time."
                );
            }
        }

        // ── non-vacuity, four directions ──
        // (a) The walk saw the files this rule is about, by name, plus a floor —
        //     a moved directory or a new extension must not silently empty it.
        let seen: BTreeSet<&str> = sources
            .iter()
            .filter_map(|(p, _)| p.rsplit('/').next())
            .collect();
        for expected in [
            "ActionBar.svelte",
            "SearchPicker.svelte",
            "PartitionPicker.svelte",
            "CostPicker.svelte",
            "plainClone.svelte.js",
            "stores.js",
            "main.js",
        ] {
            assert!(
                seen.contains(expected),
                "the frontend walk missed {expected}; it saw {seen:?}"
            );
        }
        assert!(
            sources.len() >= 14,
            "the frontend walk found only {} files under {} — this client has more than \
             that, so the walk is reading the wrong place and the ban above checked nothing",
            sources.len(),
            frontend_src.display()
        );

        // (b) The three pickers really route through the sanctioned helper. A
        //     picker that stopped taking a copy at all would satisfy the ban
        //     above while quietly mutating its parent's reactive state.
        for picker in [
            "SearchPicker.svelte",
            "PartitionPicker.svelte",
            "CostPicker.svelte",
        ] {
            let (_, text) = sources
                .iter()
                .find(|(p, _)| p.ends_with(picker))
                .unwrap_or_else(|| panic!("{picker} is in the walk"));
            assert!(
                text.contains("plainClone(") && text.contains("plainClone.svelte.js"),
                "{picker} neither imports nor calls `plainClone`. It builds its answer by \
                 copying a template prop, and that copy must be proxy-safe."
            );
        }

        // (c) The helper is implemented in terms of Svelte's own unwrapper. If it
        //     ever became a hand-rolled copy the whole rule would be a rename.
        let (_, helper) = sources
            .iter()
            .find(|(p, _)| p.ends_with("plainClone.svelte.js"))
            .expect("the helper is in the walk");
        assert!(
            helper.contains("$state.snapshot("),
            "`plainClone` must be `$state.snapshot` — that is the only API that unwraps a \
             Svelte 5 reactive proxy without re-serializing the value"
        );

        // (d) The matcher discriminates. Proven by execution against a synthetic
        //     offending line rather than argued from the needle strings, because
        //     a typo in a needle is invisible in a green run.
        let synthetic = "const answer = structuredClone(template);";
        assert!(
            forbidden.iter().any(|n| synthetic.contains(n)),
            "the ban above would not have caught the exact line UI-4 removed"
        );
    }

    /// **UI-4 (`scutemob-185`) — a picker may not fail in silence.**
    ///
    /// The three-line clone bug was survivable; what made it a conceded game was
    /// that it produced *no* observable effect. The player clicked Confirm, the
    /// picker stayed open, no request went out, and no message appeared. G8 in
    /// the same triage records the consequence: with the answer button dead and
    /// `legal_actions.rs` offering nothing but the answer and `Concede` while a
    /// blocking decision stands, Concede was the only live control on screen.
    ///
    /// Two independent mechanisms are pinned here, because either alone is a
    /// half-measure:
    ///
    /// 1. Each template-copying picker wraps its emit path in `try` and reports
    ///    through an `onError` prop, which `ActionBar` routes to the error strip.
    ///    That buys a message naming which picker failed.
    /// 2. `stores.js` installs `window` handlers for `error` and
    ///    `unhandledrejection`, and `main.js` calls that installer. That buys the
    ///    *guarantee*, for the five pickers with no `try` and for every handler
    ///    written later. Svelte 5's `<svelte:boundary>` is not a substitute — it
    ///    catches render and effect errors, not DOM handler ones.
    ///
    /// Source-level, for the same reason as the gate above: there is no frontend
    /// harness (plan §8 R7). This proves the wiring exists, not that it renders.
    #[test]
    fn test_frontend_picker_failures_reach_the_error_strip() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);
        let text_of = |name: &str| -> &str {
            sources
                .iter()
                .find(|(p, _)| p.ends_with(name))
                .map(|(_, t)| t.as_str())
                .unwrap_or_else(|| panic!("{name} is in the frontend walk"))
        };

        // 1. Per-picker try/catch, reported upward.
        for picker in [
            "SearchPicker.svelte",
            "PartitionPicker.svelte",
            "CostPicker.svelte",
        ] {
            let text = text_of(picker);
            assert!(
                text.contains("onError"),
                "{picker} has no `onError` prop — a failure while building its answer would \
                 be invisible to the player"
            );
            assert!(
                text.contains("try {") && text.contains("catch (err)"),
                "{picker} does not guard its emit path; a throw there escapes the click \
                 handler and leaves the DOM untouched"
            );
        }

        // `ActionBar` must actually pass the prop down and route it out, or the
        // pickers report into nothing.
        let action_bar = text_of("ActionBar.svelte");
        assert_eq!(
            action_bar.matches("onError={onPickerError}").count(),
            3,
            "all three template-copying pickers must be given `onPickerError`"
        );
        assert!(
            action_bar.contains("onClientError?.("),
            "`ActionBar` must forward picker failures to its caller"
        );
        let play_app = text_of("PlayApp.svelte");
        assert!(
            play_app.contains("onClientError={reportClientError}"),
            "`PlayApp` must route `ActionBar`'s picker failures into the shared error store"
        );

        // 2. The global net, and the call that arms it. An installer nobody calls
        //    is the same as no installer, and that is not visible in the module.
        let stores = text_of("stores.js");
        assert!(
            stores.contains("export function reportClientError(")
                && stores.contains("export function installGlobalErrorReporting("),
            "`stores.js` must expose both the client-error reporter and the global installer"
        );
        for event in ["'error'", "'unhandledrejection'"] {
            assert!(
                stores.contains(&format!("addEventListener({event}")),
                "the global net must listen for {event}; a DOM handler's throw surfaces \
                 nowhere else"
            );
        }
        assert!(
            text_of("main.js").contains("installGlobalErrorReporting()"),
            "`main.js` must arm the global net — an installer that is never called is not a net"
        );

        // Non-vacuity: the strip can actually say what a client-side failure is.
        // Without this arm the message renders under "Request failed", which is a
        // lie about which side of the wire broke.
        assert!(
            action_bar.contains("case 'client_error':"),
            "the error strip must have prose for the client-side kind `stores.js` sets"
        );
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

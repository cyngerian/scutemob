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
    ///
    /// Since `scutemob-187` the *procedure* for using those four fields has one
    /// more step: build at this base seed, take `setup::dealt_decks` of the
    /// result, then rebuild at the derived seed with `DeckSource::Fixed(dealt)`.
    /// The decklists are pinned at the base-seed build (CR 103.5 — a mulligan may
    /// not change them), so rebuilding at the derived seed with the random recipe
    /// no longer reproduces the table. See the README's "Reproducing a table from
    /// a bug report".
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
    ///
    /// PB-DX27 (2026-08-13, `scutemob-209`): **Dispatch -> Cyclonic Rift**, and [`TARGET_SEED`]
    /// 13 -> 16 with it. The batch flipped 6 completeness markers (net +4, the `Complete`-def
    /// count 1,133 -> 1,137), which moves `deck.rs::random_deck`'s commander pool and re-deals
    /// every seat — the completeness channel this doc's sibling pins already describe. At seed
    /// 13 the re-dealt board reaches only `Goblin War Strike`, whose candidates are all
    /// **players**, which broke `test_target_option_labels_are_seat_redacted` (nothing to
    /// cross-check) and `test_x_value_is_forwarded_to_cast_spell_data` (a `Target::Player`
    /// where the assertion wants a `Target::Object`). Cyclonic Rift is `{1}{U}` "Return target
    /// nonland permanent to its owner's hand" (CR 601.2c) — a player is not a permanent, so the
    /// caller's `422 invalid target` still fires for the reason it names. Measured at seed 16,
    /// not reasoned to: 4 object candidates, all cross-checkable against the redacted view, and
    /// the cast is ACCEPTED (200) with an object target rather than refused for mana.
    const TARGETED_SPELL: &str = "Cast Cyclonic Rift";

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
        // PB-DX27 (2026-08-13, `scutemob-209`): re-derived a third time, through the
        // completeness-marker channel this time. The batch flipped markers (net **+3**; the
        // `Complete`-def count moved 1,133 -> 1,136), `deck.rs::random_deck` draws its
        // commander from the `Complete` pool and fills by colour identity, and a different
        // index into `rng.random_range(0..commanders.len())` re-picked every seat. Read off
        // a real run at SEED with the corpus at that count; not reasoned to.
        //
        // And then re-observed a FOURTH time in the same batch, which is the point worth
        // carrying: PB-DX27's own `/review` demoted `green_suns_zenith` back to `partial`,
        // moving the count 1,137 -> 1,136 and re-dealing every seat again. One marker flip
        // anywhere in 1,803 defs invalidates this pin. Do not hand-edit it to match a diff;
        // re-run and read the hand off the run.
        assert_eq!(
            own_names,
            vec![
                "Hedron Archive",
                "Sol Ring",
                "Simic Initiate",
                "Farseek",
                "Marwyn, the Nurturer",
                "Cankerbloom",
                "Molimo, Maro-Sorcerer",
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

    /// PB-DX18 (`OOS-M11-5`), CR 601.2c — a spell that requires NO targets, given one
    /// through the real HTTP channel, is refused.
    ///
    /// This is the seed's own observation turned into a probe. `OOS-M11-5` was found here
    /// (M11-local S5, `scutemob-167`, while writing
    /// `test_post_action_illegal_target_returns_422` above): casting **Accorder's Shield**
    /// — `{0}`, `Completeness::Complete`, deck-legal, whose SPELL declares no
    /// `TargetRequirement` — with `params.targets = [Target::Player(2)]` returned **HTTP
    /// 200**, and the bogus player target was recorded on the resulting `StackObject`,
    /// from which `push_target_announcement` then emitted `GameEvent::PermanentTargeted`
    /// and dispatched Ward.
    ///
    /// **The subject is the CLASS, not the card.** The driver stops at the first offered
    /// cast whose `target_slots` is empty rather than hunting Accorder's Shield through a
    /// seeded deck — a card-specific driver would go silently vacuous the day the seeded
    /// pool moves, which this queue has watched happen to `UI3_SPLIT_COMBAT_SEED` three
    /// times. The card that actually satisfies it is PRINTED, so the reader knows what
    /// was exercised.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx18_targetless_spell_given_a_target_is_refused() {
        let state = shared_state();
        let view = drive_until(&state, TARGET_SEED, false, |v| {
            decision(v)["actions"]
                .as_array()
                .map(|acts| {
                    acts.iter().any(|a| {
                        a["label"]
                            .as_str()
                            .map(|l| l.starts_with("Cast "))
                            .unwrap_or(false)
                            && a["target_slots"].as_array().map(|s| s.is_empty()) == Some(true)
                    })
                })
                .unwrap_or(false)
        })
        .await;
        let at_seq = seq(&view);
        let before = command_count(&view);
        let acts = decision(&view)["actions"]
            .as_array()
            .expect("actions is an array")
            .clone();
        let (idx, subject) = acts
            .iter()
            .enumerate()
            .find(|(_, a)| {
                a["label"]
                    .as_str()
                    .map(|l| l.starts_with("Cast "))
                    .unwrap_or(false)
                    && a["target_slots"].as_array().map(|s| s.is_empty()) == Some(true)
            })
            .map(|(i, a)| (i, a["label"].as_str().unwrap_or("?").to_string()))
            .expect("the driver stopped on one");
        eprintln!("DX18 targetless-cast subject: {subject:?} (action_index {idx})");

        // NON-VACUITY: the offer really does ask for nothing, so the target below is
        // spurious by the OFFER's own account and not merely by the engine's.
        assert!(
            acts[idx]["target_slots"]
                .as_array()
                .expect("target_slots is an array")
                .is_empty(),
            "the subject must be a cast the offer layer asks no targets for"
        );

        let (status, err) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": at_seq,
                "action_index": idx,
                "params": { "targets": [{ "Player": 2 }] },
            }),
        )
        .await;
        assert!(
            status.is_client_error(),
            "CR 601.2c: a spell that requires no targets cannot be given one; got \
             {status} with {err}"
        );
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "the command IS built (targets have a channel on a cast) and the ENGINE \
             refuses it, so this is 422/rejected rather than 400/bad_params: {err}"
        );
        assert_eq!(err["kind"], "rejected");

        // The refusal touched nothing, and the decision is still answerable — so the
        // 422 was about the target, not about the game having become unplayable.
        let (status, still) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(seq(&still), at_seq, "the decision is still outstanding");
        assert_eq!(command_count(&still), before, "no command was applied");
        let (status, ok) = post_json(
            &state,
            "/api/game/action",
            json!({ "seq": at_seq, "action_index": idx }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the SAME cast with no targets is accepted: {ok}"
        );
    }

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
    /// [`TARGETED_SPELL`] is Cyclonic Rift, "return target nonland permanent to its owner's
    /// hand" (CR 601.2c); a player is not a permanent, so `handle_cast_spell`'s target
    /// validation refuses it with `GameStateError::InvalidTarget`. (This paragraph named
    /// Dispel for two batches after the constant had moved on, and then Dispatch after
    /// PB-DX27 moved it again — it is derived from the constant now, not restated.)
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
        // run; see the exact-hand pin for the mechanism. PB-DX27, 2026-08-13: 18 -> 20,
        // re-read off a real run — that batch flipped 6 completeness markers, net +4, moving
        // the `Complete`-def count 1,133 -> 1,137 and so the commander pool `random_deck`
        // indexes into, which re-deals every seat.)
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
    //
    // PB-DX27 (2026-08-13, `scutemob-209`): 13 -> 16, the third re-derivation, and by the
    // completeness channel this time — the batch flipped 6 markers (net +4; the `Complete`-def
    // count 1,133 -> 1,137), so `random_deck`'s commander pool changed length AND membership and
    // every seeded seat re-dealt. Swept `seed` in 0..47 with a throwaway probe (since deleted)
    // reporting FOUR properties per seed, because this constant serves SIX tests: (P1) which
    // targeted `CastSpell` labels are offered and whether their candidates are objects or
    // players, for [`TARGETED_SPELL`]; (P2) a target slot with >= 2 candidates, for
    // `test_action_option_target_slots_match_engine_query`; (P4) at the `option_with_targets(v, 1)`
    // stop, how many object labels are cross-checkable against the seat-redacted view, for
    // `test_target_option_labels_are_seat_redacted`; (P5) after tapping five sources, whether the
    // cast is still offered, whether candidate 0 is an OBJECT, and whether the engine ACCEPTS it,
    // for `test_x_value_is_forwarded_to_cast_spell_data`.
    //
    // Seed 13 failed P4 and P5 together, and for one cause: its only targeted cast is
    // `Goblin War Strike`, whose four candidates are all PLAYERS. P4 collected 0 object ids and
    // hit its own "vacuous" guard; P5 got a `Target::Player(1)` where it asserts
    // `Target::Object`. Both are the guards doing their job, not drift to be tuned around.
    //
    // Every seed below 16 fails at least one property, so 16 is the smallest that serves all
    // six — measured, not assumed: 0/1/2/4/5/9/11/12 never reach a targeted cast at all inside
    // `S7_MAX_STEPS`; 3/6/7/8 reach one but never five untapped sources beside it; 10/14 are
    // OFFERED the cast and then refused "player does not have enough mana to pay the cost" once
    // five sources are tapped (OOS-CARDS2-9/F4 again, on two more seeds); 13 is the pair above;
    // and 15 reaches `Red Elemental Blast`, which wants 2 targets where the fixture announces 1.
    // Confirmed by RUNNING all six tests at 16 — the SIM-2 precedent, which is stricter than a
    // property sweep because it asserts the fixtures rather than their preconditions.
    const TARGET_SEED: u64 = 16;

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
            // Candidates in POLICY order (only the first is ever tried -- see below):
            // prefer PlayLand, then a zero-target develop cast, then PassPriority, then
            // whatever is first in the offered list. This is priority selection among
            // simultaneously-legal actions, not a fallback-on-refusal mechanism; the
            // latter used to exist here (to fall through past a documented false
            // offer) and was deleted along with the excusal register it existed for
            // -- see the comment on the single `if let` below.
            //
            // This list used to also carry a documented false offer: an **Aura**
            // carries its target requirement in `KeywordAbility::Enchant(...)`, which
            // `casting.rs` special-cases (CR 303.4a, "Aura spells require exactly one
            // target"); the *provider* did not read that keyword, so the offer
            // reported `target_min: 0` — "announces nothing" — and the engine
            // rejected the cast with a 422. PB-DX20 closed that: `spell_target_
            // requirements` now synthesizes the Enchant-derived requirement for both
            // the offer and the cast path from the SAME function, so an Aura's
            // `target_min` is correctly 1 and the develop policy's `target_min == 0`
            // filter now excludes Auras at the source rather than selecting them and
            // eating the refusal. `OOS-CARDS2-4` is CLOSED.
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

            // Only the FIRST candidate is ever actually tried: every documented false
            // offer this driver used to excuse is now closed (SIM-2 deleted the
            // mana-affordability pair; PB-DX20 closes the Aura entry above), so the
            // excusal register is EMPTY — and the whole excusal mechanism is deleted
            // along with it (`crates/simulator/tests/local_game_playthrough.rs:495-506`'s
            // own precedent: "an excusal list is a debt register with a maturity
            // date... the whole excusal mechanism is deleted along with it"). This is
            // the staleness assertion the register never had: ANY refusal is now
            // unconditionally fatal, naming the refused label and reason, so a future
            // provider/engine disagreement of this SR-38 class ("never offer what the
            // engine rejects") fails loudly instead of being silently re-excused —
            // which is also why trying a SECOND candidate after a refusal would never
            // be reachable: there is no longer a tolerated failure to fall through past.
            let mut advanced = false;
            if let Some(pick) = candidates.first() {
                let (status, next) = post_json(
                    state,
                    "/api/game/action",
                    json!({ "seq": seq(&view), "action_index": pick["index"] }),
                )
                .await;
                if status == StatusCode::OK {
                    view = next;
                    advanced = true;
                } else {
                    let reason = next["error"].as_str().unwrap_or_default();
                    panic!(
                        "driving seed {seed}: the engine refused {} with reason {reason:?}. The \
                         excusal register is empty — every action this driver's policy offers \
                         must be one the engine actually accepts; a refusal here is a NEW SR-38 \
                         provider/engine disagreement and must be filed as a finding, not driven \
                         past.",
                        pick["label"]
                    );
                }
            }
            assert!(
                advanced,
                "driving seed {seed}: no candidate action (PlayLand / develop CastSpell / \
                 PassPriority / anything) was found among the actions offered at this \
                 decision — {}",
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
                mtg_engine::spell_target_requirements(game_state, *card, &[], None, false),
                *card,
            ),
            mtg_simulator::LegalAction::ActivateAbility {
                source,
                ability_index,
                ..
            } => (
                mtg_engine::ability_target_requirements(game_state, *source, *ability_index, &[]),
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
                    // PB-DX52: a stack entry's id, which the wire carries under
                    // `kind: "stack_object"`. `slot_ids` reads the wire's `id` field
                    // regardless of kind, so the comparison stays honest.
                    mtg_engine::Target::StackObject(id) => id.0,
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

    // ── CARDS-1 (OOS-M11-10E) ────────────────────────────────────────────────────

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

    /// **CARDS-1 (OOS-M11-10E), browser-path half.** Engine coverage already
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

        // Regression floor for OOS-M11-10E itself: this is the assertion that
        // would have caught the original defect at the layer the playtest
        // actually observed it. With the pre-fix `targets: vec![]`,
        // `target_slots` here is empty and the picker never asks.
        let target_slots = action["target_slots"]
            .as_array()
            .expect("target_slots is an array");
        assert_eq!(
            target_slots.len(),
            1,
            "OOS-M11-10E: Skullclamp's ActivateAbility option must carry exactly one target \
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
            "OOS-M11-10E: the slot's candidate list must be non-empty -- Skullclamp's own \
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
    /// second constant. Observed by a throwaway probe driving each seed to its
    /// first attack offer and recording the offer's shape; the probe is then
    /// deleted. **Every** seed offers 3 player targets (the three opponents,
    /// which is just CR 506.2), and only a few offer more than one eligible
    /// attacker — most offer exactly 1, because at the turn the first attack
    /// becomes available the boards hold a single creature.
    ///
    /// With one attacker, "attacker → defender" degenerates to "there is a
    /// defender", and a mapping bug that *swapped two attackers' defenders*
    /// would pass. Re-observe rather than guess if this stops splitting: like
    /// [`COMBAT_SEED`] and [`TARGET_SEED`], it is a function of the whole card
    /// corpus, and a completeness flip in any card-def batch re-deals it.
    ///
    /// UI-3 (`scutemob-180`) observed seed **21** over `seed` ∈ 0..24.
    /// **PB-DX26 (`scutemob-206`, 2026-08-11) re-observed it twice and settled on
    /// 26** — and the two re-observations are worth recording together, because the
    /// second one contradicts the intuition the first would leave you with.
    ///
    /// *First re-observation*, after one completeness flip UP
    /// (`sword_of_body_and_mind` `partial` -> `Complete`) grew the deck pool: seed
    /// 21 dropped to one eligible attacker; the sweep over 0..40 gave **9, 26, 28,
    /// 29, 30**, of which only **28 and 29** also reached a declared blocker.
    ///
    /// *Second re-observation*, after the batch's `/review` demoted
    /// `the_reaver_cleaver` back down: the Complete COUNT returned to exactly what
    /// it was before the batch — and **the deal moved anyway**, because the pool
    /// holds a *different card*, not a different number of them. Seed 28 lost its
    /// split. The fresh sweep is **26, 29, 30, 36, 38**, of which **26, 29 and 38**
    /// also reach a declared blocker (30 and 36 split and then no bot blocks).
    /// 26 is the lowest.
    ///
    /// The durable lesson for whoever re-observes this next: **a stable
    /// `CORPUS_COMPLETE` is not evidence that the deal is stable.** Two markers
    /// moving in opposite directions cancel in the count and not in the set, and
    /// `pb_dx32_fuzz_output`'s pinned constant — the thing that normally shouts
    /// when the pool changes — stays green through it. Run the sweep; do not infer
    /// it from the count. And the seed must satisfy BOTH halves of the test (the
    /// split AND a declared blocker), which is a second filter this doc did not
    /// mention before PB-DX26 hit it.
    ///
    /// *Third re-observation* — **PB-DX27 (`scutemob-209`, 2026-08-13): 26 -> 28.** This
    /// time the count did move: the batch flipped 6 completeness markers, net +4, taking
    /// the `Complete`-def count 1,133 -> 1,137, so `random_deck`'s commander pool changed
    /// both length and membership and every seeded seat re-dealt. Seed 26 lost its split
    /// outright — it now declares a single attacker, `[(442, "Bot-2")]`, which is exactly
    /// the silent downgrade the assertion below exists to refuse.
    ///
    /// Fresh sweep over `seed` ∈ 0..56 (throwaway probe: drive each seed to its first
    /// attack offer, declare `attacker[i] -> defender[i % defenders]`, then pass up to 40
    /// times looking for an assigned blocker; probe deleted). **Seven seeds split** — 13,
    /// 21, 28, 32, 35, 37, 48 — and of those only **28, 32 and 48** also reach a declared
    /// blocker; every other seed in the range offers exactly one eligible attacker. 28 is
    /// the lowest. Measured at 28: 2 attackers across 2 distinct defenders, blockers
    /// exercised.
    ///
    /// *Fourth re-observation* — **PB-DX43 (`scutemob-213`, 2026-08-14): 28 -> 32.**
    /// The CR 305.6 intrinsic-mana derivation (`rules::layers::derive_intrinsic_land_
    /// mana_abilities`) makes a nonbasic land that has been turned into a basic land
    /// type (Urborg, the Dryad, Yavimaya, awaken_the_woods' Forest token, ...) able
    /// to tap for mana where it previously produced nothing — this is the exact
    /// "fuzz/seeded fixtures that depended on a land producing nothing may move"
    /// hazard the batch's own plan names. RandomBot's action-index selection over a
    /// board with one more legal `TapForMana` diverges from that point on, so a seed
    /// number no longer replays the same game. Fresh sweep over `seed` ∈ 0..80 (same
    /// throwaway-probe recipe as above): seed 28 no longer reaches a declared blocker
    /// (split, blocker=false). Hits satisfying BOTH halves: **32, 47, 48, 79**. 32 is
    /// the lowest. Measured at 32: 2 attackers across 2 distinct defenders, blockers
    /// exercised.
    ///
    /// *Fifth re-observation* — **PB-DX45 (`scutemob-217`, 2026-09-02): 32 -> 13.**
    /// One completeness marker moved (`vampire_gourmand` `partial` -> `Complete`, the
    /// CR 118.12 policy re-adjudication), taking `CORPUS_COMPLETE` 1,136 -> 1,137, so
    /// `random_deck`'s pool changed and every seeded seat re-dealt. Seed 32 lost its
    /// split outright and declares a single attacker, `[(449, "Bot-2")]` — the silent
    /// downgrade the assertion below refuses. Fresh sweep over `seed` ∈ 0..46 (same
    /// throwaway-probe recipe; probe deleted): **five seeds split** — 0, 13, 26, 28, 36
    /// — of which **13, 26 and 36** also reach a declared blocker. 13 is the lowest.
    /// Measured at 13: 2 attackers across 2 distinct defenders, blockers exercised.
    ///
    /// **The sweep was bounded at 46 and the reason is a finding, not a shortcut.**
    /// The probe reaches seed 46 and `drive_until`'s develop policy dies there on an
    /// SR-38 provider/engine disagreement — the engine refuses `Cast Impact Tremors`
    /// with *"player does not have enough mana to pay the cost"* against an empty
    /// excusal register. Earlier sweeps drove 0..80 clean, so this is the re-deal
    /// exposing a pre-existing disagreement on a seed nobody had driven with THIS
    /// corpus. Filed as `OOS-DX45-8`; 13 is well below the bound, so nothing about
    /// this pin depends on it.
    ///
    /// **And the standing lesson lands a third time**: PB-DX26 recorded that a stable
    /// COUNT is not a stable DEAL, and PB-DX45 adds the converse — a count that moves
    /// by ONE moved exactly one seeded pin in the whole workspace, while
    /// `pb_dx32_fuzz_output.rs`'s own `MOVED_MSG` predicts its five named sibling
    /// gates "will redden alongside this one". They did not. Run the sweep; do not
    /// infer the blast radius from the size of the count change in either direction.
    /// *Sixth re-observation* — **PB-DX36 (`scutemob-228`, 2026-09-04): 13 -> 26.**
    /// One completeness marker moved (`exalted_angel` `partial` -> `Complete`), taking
    /// `CORPUS_COMPLETE` 1,138 -> 1,139, so `random_deck`'s pool changed and every
    /// seeded seat re-dealt. Seed 13 lost its split outright and declares a single
    /// attacker, `[(435, "Bot-2")]` — the silent downgrade the assertion below refuses.
    ///
    /// **The cause is attributed by an EXECUTED ablation rather than assumed.** In an
    /// isolated worktree at PB-DX36's own commit, with the entire engine change in the
    /// tree and ONLY that marker forced back to `partial`, this test is GREEN. So none
    /// of this is the new damage-trigger dispatch; it is entirely `OOS-CARDS2-3`'s
    /// re-deal.
    ///
    /// Fresh sweep over `seed` ∈ 0..46 (same throwaway-probe recipe; probe deleted).
    /// **Four seeds split** — 26, 28, 39, 42 — against 41 that offer a single eligible
    /// attacker and one (5) that the develop policy refuses, the `OOS-DX45-8`
    /// disagreement noted above, now reached at 5 instead of 46. **26 is the lowest,
    /// and it satisfies the blocker half as well**, verified by running this test
    /// itself rather than by trusting the probe — the probe's blocker detector reported
    /// `false` for every seed including the one that passes, so it is a floor on the
    /// split and says nothing about blockers. Recorded because the next re-observer
    /// will reuse this recipe: **drive the real test against each split seed in order;
    /// do not filter on the probe's blocker column.**
    const UI3_SPLIT_COMBAT_SEED: u64 = 26;

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

    /// **SIM-5 fix (3), G5.** The report carries the engine's own refusals of
    /// *bot* commands, not just the commands that were applied.
    ///
    /// This is the half of the G5 triage that could not be settled from the artefact:
    /// `journal` records applied commands only, so "why did that bot tap six sources
    /// at upkeep and then pass?" had to be inferred from the surrounding commands
    /// because `LocalGame::advance()` bound the engine's error and dropped it. The
    /// rejections are now recorded (`mtg_simulator::local_game::RejectedCommand`) and
    /// exported here, so the next triage classifies instead of inferring.
    ///
    /// Driven by passing, not by `drive_until`: the rejections this asserts on are
    /// **bot**-seat ones, which accumulate on their own while the human does nothing.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim5_report_exposes_bot_command_rejections() {
        let state = shared_state();
        let mut view = new_game(&state).await;
        let mut report = Value::Null;

        for _ in 0..S7_MAX_STEPS {
            let (status, body) = get_json(&state, "/api/game/report").await;
            assert_eq!(status, StatusCode::OK);
            // The two fields are asserted present on EVERY iteration, so this test
            // fails on a dropped field even if the game never produces a rejection.
            assert!(
                body["rejections"].is_array(),
                "the report must carry a rejections array: {body}"
            );
            assert!(
                body["rejection_count"].is_u64(),
                "the report must carry a rejection_count: {body}"
            );
            if body["rejection_count"].as_u64().unwrap_or(0) > 0 {
                report = body;
                break;
            }
            if view["decision"].is_null() {
                break;
            }
            // Pass when passing is offered; otherwise answer whatever is first with
            // its server-side default -- the same dull policy `drive_until` uses, and
            // the only way past an out-of-band blocking decision (a cleanup discard
            // offers no `PassPriority` at all, CR 514.3).
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
            assert_eq!(
                status,
                StatusCode::OK,
                "the human's answer must be accepted: {next}"
            );
            view = next;
        }

        // Non-vacuity: the loop above must actually have found a refusal, or every
        // assertion below is about an empty list. Bots are offered actions the engine
        // refuses on every seed measured for SIM-5 (30/44/92 refusals in 25 turns on
        // seeds 0/7/42) -- if this ever goes red, `StubProvider` got MORE accurate,
        // which is a good problem and this fixture is where to notice it.
        assert!(
            !report.is_null(),
            "no bot command was refused within {S7_MAX_STEPS} steps at SEED"
        );
        let rejections = report["rejections"].as_array().expect("array");
        assert!(!rejections.is_empty());
        assert!(
            report["rejection_count"].as_u64().unwrap() >= rejections.len() as u64,
            "the count is never truncated, the retained list is: {report}"
        );
        for r in rejections {
            assert!(r["turn"].is_u64(), "{r}");
            assert!(
                r["player"].as_u64().is_some_and(|p| p != 1),
                "only BOT seats reach the rejection recorder -- a human submission \
                 returns its error to the client instead: {r}"
            );
            assert!(
                r["error"].as_str().is_some_and(|e| !e.is_empty()),
                "the engine's reason must be kept: {r}"
            );
            assert!(
                r["command"].is_object(),
                "the refused command is serialized verbatim, like a journal entry: {r}"
            );
        }
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
            ..Default::default()
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

    // ── UI-6: the whole-library search view (task scutemob-194) ───────────────
    //
    // G9 of `memory/playtest-triage-2026-08-02b.md`: *"only showed legal basic
    // lands — should be able to view whole library when searching"*.
    //
    // The filter was never the defect — `candidates` IS the answer space and
    // `handle_answer_effect_choice` refuses anything outside it (SR-38). What was
    // missing is CR 701.23a's **look**: *"To search for a card in a zone, look at
    // all cards in that zone (even if it's a hidden zone)."* `AnswerShapeView::
    // PickOne` therefore carries a second, look-only list, and the two probes
    // below assert the two halves that matter: the look really is the whole
    // library, and it really is look-only.
    //
    // **The fixture cannot be the UI-1 one, and the reason is the point of the
    // feature.** `ui1_install`'s search is Diabolic Tutor — an *unrestricted*
    // search, so its candidate set is the entire library and `all_cards` would be
    // set-equal to it. A fixture like that can never exhibit a look-only card, so
    // it could never falsify the claim under test. UI-6 needs a search with a
    // *stated quality*, which is exactly the shape the playtest complained about.

    /// Same seed as [`UI1_SEED`], reused for the same reason [`SIM1_SEED`] does:
    /// `setup::build_initial_state` shuffles a 99-card `main_deck` on an RNG
    /// seeded from `cfg.seed` alone, so the permutation of *positions* is
    /// identical for any 2-player `DeckSource::Fixed` game with 99-card decks —
    /// whichever cards sit at `main_deck[0]`/`[1]` land in the opening hand.
    /// **Verified empirically for this deck too**, not merely inherited: the
    /// drive below reaches Solemn Simulacrum's ETB search well inside its budget.
    const UI6_SEED: u64 = UI1_SEED;

    /// `{4}`, Artifact Creature — Golem 2/2, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/solemn_simulacrum.rs`). Its ETB trigger is
    /// `Effect::SearchLibrary` with `basic_land_filter()`, which is the whole
    /// reason it is the UI-6 fixture rather than a tutor:
    ///
    /// * the search states a **quality** (CR 701.23b), so `candidates` is a
    ///   strict subset of the library and a look-only card exists to be probed;
    /// * it is **colourless**, so it is legal beside [`UI1_COMMANDER`]'s
    ///   mono-black colour identity (CR 903.5c);
    /// * `may_fail_to_find` is therefore `true`, the opposite of the UI-1 search
    ///   probe's `false` — so between them the two CR 701.23b/d branches are both
    ///   exercised over HTTP.
    const UI6_SEARCHER: &str = "solemn-simulacrum";

    /// Six mono-black `Complete` cards, **all MV ≥ 6**, seeded into the deck for
    /// exactly one purpose: to be cards in the library that the basic-land search
    /// **cannot find**, i.e. the look-only population this batch exists to show.
    ///
    /// Chosen by three constraints, each of which rejected earlier drafts:
    ///
    /// 1. **Mono-black or colourless** — CR 903.5c colour identity against
    ///    [`UI1_COMMANDER`].
    /// 2. **No search / scry / surveil of their own.** [`ui1_drive_to_question`]
    ///    stops at the *first* action carrying the wanted question tag, so a
    ///    filler that searched would be answered instead of Solemn's ETB.
    /// 3. **Unreachably expensive.** At MV ≥ 6 none is castable inside the drive's
    ///    window, so a filler that reaches the hand sits there rather than
    ///    resolving something that perturbs the board.
    ///
    /// Singleton (CR 903.5b) is why these are six *distinct* cards and not six
    /// copies of one — only basic lands are exempt.
    const UI6_LOOK_ONLY_FILLER: [&str; 6] = [
        "butcher-of-malakir",
        "kokusho-the-evening-star",
        "grave-titan",
        "in-garruks-wake",
        "kindred-dominance",
        "dreadhound",
    ];

    /// CR 903.5b/c: Solemn Simulacrum at `main_deck[0]`, a Swamp at `[1]` (the two
    /// opening-hand slots — see [`UI6_SEED`]), the six look-only fillers, then
    /// Swamps to 99.
    fn ui6_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![CardId(UI6_SEARCHER.to_string())];
        main_deck.push(CardId("swamp".to_string()));
        main_deck.extend(UI6_LOOK_ONLY_FILLER.iter().map(|c| CardId(c.to_string())));
        while main_deck.len() < 99 {
            main_deck.push(CardId("swamp".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(UI1_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install the UI-6 fixture through `session::new_game` — the same
    /// constructor the real handler uses, running the same two Invariant-9 gates.
    /// See [`ui1_install`]'s doc for why `POST /api/game` cannot express this.
    fn ui6_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI6_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), ui6_deck()),
                (mtg_engine::PlayerId(2), ui6_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the UI-6 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// The `PickOne` answer shape of the first offered `SearchLibrary` question,
    /// as `(action_index, candidate ids, all_cards as (id, label))`.
    fn ui6_search_shape(view: &Value) -> (u64, Vec<u64>, Vec<(u64, String)>) {
        let index = ui6_question_index_or_panic(view);
        let option = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let answer = &option["decision"]["answer"];
        assert_eq!(answer["shape"], "PickOne");
        let candidates = answer["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        let all_cards = answer["all_cards"]
            .as_array()
            .expect("all_cards is an array — UI-6's CR 701.23a look entitlement")
            .iter()
            .map(|c| {
                (
                    c["id"].as_u64().expect("id is a number"),
                    c["label"].as_str().expect("label is a string").to_string(),
                )
            })
            .collect();
        (index, candidates, all_cards)
    }

    fn ui6_question_index_or_panic(view: &Value) -> u64 {
        ui1_question_index(view, "SearchLibrary").expect("a SearchLibrary question is offered")
    }

    /// **Read off a real run, not reasoned to** — the [`UI1_SEED`] convention.
    /// A sweep of seeds 0..300 over the [`ui6_restricted_install`] fixture found
    /// this to be the first at which the bot has Aven Mindcensor on the
    /// battlefield *before* the human's Solemn Simulacrum resolves. Most seeds
    /// reach no search at all (the bot's Plains deck plays slowly and the drive
    /// budget runs out); of those that do, this is the first restricted one.
    const UI6_RESTRICTED_SEED: u64 = 29;

    /// `{2}{W}`, Creature — Bird Wizard 2/1, **Flash**, and its third line is the
    /// one this fixture is for: *"If an opponent would search a library, that
    /// player searches the top four cards of that library instead."*
    /// (`crates/card-defs/src/defs/aven_mindcensor.rs`, `Completeness::Complete`
    /// by derive — so `validate_deck` accepts it and a human can meet it in a
    /// real game).
    const UI6_RESTRICTOR: &str = "aven-mindcensor";

    /// CR 121.1's restriction is `RestrictSearchTopN(4)` here.
    const UI6_RESTRICTED_TO: usize = 4;

    /// Seat 2 gets a mono-white deck holding the restrictor; seat 1 keeps
    /// [`ui6_deck`]. The commander is Elesh Norn ({5}{W}{W}) purely to fix seat
    /// 2's colour identity (CR 903.5c) at a mana value the drive never reaches.
    fn ui6_restricted_install(state: &SharedState) {
        use mtg_engine::CardId;
        let mut bot_main: Vec<CardId> = vec![CardId(UI6_RESTRICTOR.to_string())];
        while bot_main.len() < 99 {
            bot_main.push(CardId("plains".to_string()));
        }
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI6_RESTRICTED_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), ui6_deck()),
                (
                    mtg_engine::PlayerId(2),
                    mtg_simulator::DeckConfig {
                        commander: CardId("elesh-norn-grand-cenobite".to_string()),
                        main_deck: bot_main,
                    },
                ),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session =
            session::new_game(cfg, 0).expect("the UI-6 restricted fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// **CR 121.1 / CR 614.1: "all cards in that zone" is not always the whole
    /// library, and the look narrows with the search** (UI-6 `/review`, finding 1).
    ///
    /// `library_look_cards` originally enumerated the library unconditionally.
    /// Under an opponent's Aven Mindcensor the searcher *"searches the top four
    /// cards of that library instead"*, so the CR 701.23a entitlement is four
    /// cards — and showing 89 with 85 marked "look only" is a real
    /// over-disclosure of this seat's own library, not a cosmetic one. The fix
    /// calls the same `apply_search_library_replacement` the engine's search path
    /// calls and narrows through the same `Zone::top_n`.
    ///
    /// **This is the discriminating test for that fix**: before it, `all_cards`
    /// here was 89. It is asserted against the engine's own zone read rather than
    /// against the literal `4`, so a fixture whose library happened to be four
    /// cards long could not satisfy it by accident.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui6_the_look_narrows_with_a_search_restriction() {
        let state = shared_state();
        ui6_restricted_install(&state);
        let view = ui1_drive_to_question(&state, "SearchLibrary", 800).await;
        let (_, candidates, all_cards) = ui6_search_shape(&view);

        // `ui1_library` is BOTTOM-first (`Zone::Ordered` keeps the top last), so
        // the searched set is the TAIL. This is the engine's own zone, read out
        // of band — the same oracle role `from_game_state` plays for the
        // redaction tests.
        let library = ui1_library(&state);
        assert!(
            library.len() > UI6_RESTRICTED_TO * 2,
            "non-vacuity: the library must be much longer than the restriction, or \
             'narrowed' means nothing here. len={}",
            library.len()
        );
        let top_n: std::collections::HashSet<u64> = library[library.len() - UI6_RESTRICTED_TO..]
            .iter()
            .copied()
            .collect();

        let all_ids: std::collections::HashSet<u64> = all_cards.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            all_ids,
            top_n,
            "CR 121.1: the look must be exactly the top {UI6_RESTRICTED_TO} cards the \
             restriction leaves searchable — not the whole library, and not some \
             other {UI6_RESTRICTED_TO}. If this fixture stopped putting \
             {UI6_RESTRICTOR} on the battlefield in time, UI6_RESTRICTED_SEED needs \
             re-pinning; library len={}",
            library.len()
        );

        // The engine narrowed its candidates by the same rule, so the two agree.
        // That containment is the property the client's `pickable` flag rests on.
        for id in &candidates {
            assert!(
                all_ids.contains(id),
                "a candidate the restriction allows must be visible in the look list"
            );
        }
    }

    /// **CR 701.23a — the search view shows the WHOLE library, look-only** (G9).
    ///
    /// Four claims, in the order they build on each other:
    ///
    /// 1. `all_cards` is the searcher's entire library, card for card, against the
    ///    engine's own zone read as the out-of-band oracle.
    /// 2. It is a **strict** superset of `candidates` — so the look-only set this
    ///    feature exists to expose is non-empty, and the test is not vacuously
    ///    true against a fixture whose filter happens to match everything (which
    ///    is precisely what the UI-1 Diabolic Tutor fixture is).
    /// 3. It is ordered by **name**, not by library position. CR 701.23a grants a
    ///    look at the cards; it does not grant a look at the shuffle, and
    ///    Architecture Invariant 7 names library order explicitly.
    /// 4. **A look-only card cannot be submitted** — the refusal path, at the
    ///    server boundary rather than by the engine. This is the SR-38 half: the
    ///    look widened, the answer space did not.
    ///
    /// Then a real, non-default legal pick resolves, so the widened look has not
    /// broken the thing it wraps.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui6_search_view_shows_the_whole_library_look_only() {
        let state = shared_state();
        ui6_install(&state);
        let view = ui1_drive_to_question(&state, "SearchLibrary", 600).await;
        let (index, candidates, all_cards) = ui6_search_shape(&view);

        // (1) The whole library, against the engine's own zone.
        let library = ui1_library(&state);
        let all_ids: Vec<u64> = all_cards.iter().map(|(id, _)| *id).collect();
        let mut sorted_all = all_ids.clone();
        sorted_all.sort_unstable();
        let mut sorted_library = library.clone();
        sorted_library.sort_unstable();
        assert_eq!(
            sorted_all,
            sorted_library,
            "CR 701.23a: `all_cards` must be exactly the searcher's library — \
             {} entries against a library of {}",
            all_ids.len(),
            library.len()
        );

        // Non-vacuity on the LABELS, not just the ids: the look entitlement has to
        // be rendering real names before anything below can be said about it.
        assert!(
            all_cards.iter().any(|(_, label)| label == "Swamp"),
            "the look entitlement must render real card names: {all_cards:?}"
        );

        // (2) A strict superset — the look-only population exists.
        let candidate_set: std::collections::HashSet<u64> = candidates.iter().copied().collect();
        let all_set: std::collections::HashSet<u64> = all_ids.iter().copied().collect();
        assert!(
            candidate_set.is_subset(&all_set),
            "this fixture searches the library only, so every candidate must also be \
             in the look list"
        );
        let look_only: Vec<&(u64, String)> = all_cards
            .iter()
            .filter(|(id, _)| !candidate_set.contains(id))
            .collect();
        assert!(
            !look_only.is_empty(),
            "the whole point of UI-6 is a card you may LOOK at and may not FIND. \
             Solemn Simulacrum searches for a basic land, so the six \
             UI6_LOOK_ONLY_FILLER cards should supply them — if none is left in \
             the library at this point, UI6_SEED needs re-pinning. candidates={} \
             all_cards={}",
            candidates.len(),
            all_cards.len()
        );

        // (3) Ordered by name, never by library position.
        let mut by_name = all_cards.clone();
        by_name.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        assert_eq!(
            all_cards, by_name,
            "`all_cards` must arrive sorted by (label, id) — see `library_look_cards`"
        );
        // And that ordering really is different from the library's own, so the
        // claim above is a fact about this payload and not a coincidence of a
        // library that happened to be in name order.
        assert_ne!(
            all_ids, library,
            "if the look list were in library order it would disclose the shuffle \
             (Architecture Invariant 7); this assertion is what proves it is not"
        );

        let wire_seq = seq(&view);

        // (4) The refusal path. A look-only id is a REAL object in a zone this
        // seat is entitled to look at — which is exactly why it is the
        // interesting refusal, and why `999_999` (the UI-1 probe's needle) does
        // not cover this case: that one could be refused by any sanity check,
        // this one can only be refused by checking membership in `candidates`.
        let look_only_id = look_only[0].0;
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": { "effect_choice_answer": { "SearchLibrary": { "found": look_only_id } } }
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a card the search may look at but not find must be refused (SR-38: the \
             look widened, the answer space did not): {refused}"
        );
        assert_eq!(refused["kind"], "bad_params");

        // And the refusal changed nothing — the question is still outstanding.
        let library_before = ui1_library(&state);
        assert!(
            library_before.contains(&look_only_id),
            "a refused answer must not move a card"
        );

        // A real, non-default legal pick still resolves.
        let chosen = *candidates.last().expect("non-empty");
        assert_ne!(
            chosen, candidates[0],
            "the pick must differ from the default"
        );
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
        let library_after = ui1_library(&state);
        assert!(
            !library_after.contains(&chosen),
            "the chosen basic land must have left the library (CR 701.23a)"
        );
        assert!(
            library_after.contains(&candidates[0]),
            "the DEFAULT's pick must still be in the library — that is what \
             discriminates a real choice from `candidates.first()`"
        );
    }

    /// **Architecture Invariant 7: the CR 701.23a whole-library look never reaches
    /// a foreign seat** (UI-6).
    ///
    /// # Why this is a NEW gate and not an assertion added to the scry one
    ///
    /// MR-M11-01's lesson, applied rather than restated: *a redaction gate checks
    /// the channel it was written for.* `test_ui1_a_foreign_seats_effect_choice_
    /// never_reaches_this_payload` needles the raw body for the `looked_at` key —
    /// the scry/surveil channel. A search payload has no `looked_at` at all, so
    /// that gate would have stayed green with every card of seat 1's library,
    /// **named**, in seat 2's body. `GameSummary.seed` shipped for three sessions
    /// past two green gates for exactly this reason. A new channel gets a new gate.
    ///
    /// Same construction as its sibling and for the same recorded reason: it moves
    /// the **viewer** (`PlaySession::human`), not the decision — `advance()`
    /// refreshes `pending` straight back off `LocalGame`, so mutating that does
    /// nothing.
    ///
    /// Two-sided, and **proven by executing the revert**, not by arguing it:
    /// deleting `seat_view`'s `pending.player == human` filter (`api.rs`) and
    /// running this test returns seat 1's entire library — every card named,
    /// `Butcher of Malakir` through `Swamp` — inside seat 2's payload, and the
    /// assertion below quotes it.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ui6_a_foreign_seat_never_receives_the_whole_library_look() {
        let state = shared_state();
        ui6_install(&state);
        let view = ui1_drive_to_question(&state, "SearchLibrary", 600).await;
        let (_, _, all_cards) = ui6_search_shape(&view);

        // Non-vacuity: the entitlement is really in use and really carries names,
        // so its absence below is a consequence of the filter and not of an empty
        // payload. Asserted BEFORE the move, while this harness is still seat 1.
        assert!(
            all_cards.len() > 50,
            "a Commander library should be tens of cards long: {}",
            all_cards.len()
        );
        assert!(
            all_cards.iter().any(|(_, label)| label == "Swamp"),
            "real names, not placeholders: {all_cards:?}"
        );

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

        // Asserted over the RAW body — the MR-M11-01 idiom. The needle is the
        // `all_cards` **key**, not a card name, and for the same reason its
        // sibling gate needles `looked_at`: seat 2 legitimately holds Swamps of
        // its own, so "no card name appears" is not assertable here and claiming
        // it would overstate what this proves.
        assert!(
            !body.contains("\"all_cards\""),
            "the foreign seat's CR 701.23a whole-library look leaked into the body: {body}"
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
    /// So the channel is pinned by count, at **three** raw reads, each one
    /// accounted for below.
    ///
    /// # The re-pin from two to three, and the entitlement that bought it (UI-6)
    ///
    /// This gate read `2` from UI-1 until `scutemob-194`, and it going red was the
    /// gate **working**: UI-6 (G9 of `memory/playtest-triage-2026-08-02b.md`)
    /// opened a third read on purpose, and the count is moved deliberately rather
    /// than routed around.
    ///
    /// The entitlement is **CR 701.23a**: *"To search for a card in a zone, look at
    /// **all** cards in that zone (even if it's a hidden zone)."* Before UI-6 the
    /// browser showed a searcher only the cards the effect could *find* — correct
    /// as an answer space (`handle_answer_effect_choice` refuses anything outside
    /// it, and offering more would violate SR-38) but a rules-level *under*-showing
    /// of the look. `view::library_look_cards` closes that: the searcher's own
    /// library, look-only, in a field the answer space never reads.
    ///
    /// Three things bound it, and each is the reason a wider read was not taken:
    ///
    /// 1. **The searcher's own library only.** `player` is
    ///    `PendingDecision::player`, which `api.rs::seat_view` has already filtered
    ///    to the viewing seat; the engine's search effect builds its candidates
    ///    from `ZoneId::Library(p)` for that same `p`. No other seat's library is
    ///    reachable from this call.
    /// 2. **Sorted by name, never in library order.** Architecture Invariant 7
    ///    names library *order* explicitly. CR 701.23a grants a look at the cards,
    ///    not at their sequence, and CR 701.23e's shuffle exists to keep the
    ///    sequence unknown — so sending `Zone::object_ids()` verbatim would leak
    ///    draw order to the seat that just failed to find.
    /// 3. **Look-only, and separately named.** `all_cards` is a different field
    ///    from `candidates`; nothing on the write path reads it.
    ///
    /// # Why this counts a needle SET and not one needle
    ///
    /// The new read spells `.zone(`, not `.objects()`. Against the single-needle
    /// gate this file shipped with, UI-6's whole-library channel would have been
    /// **invisible** — the count would have stayed at 2 and the gate would have
    /// stayed green while a new hidden-information channel opened underneath it.
    /// That is MR-M11-01's lesson arriving a second time, in the same file, three
    /// sessions later. **Measured, not argued**: with UI-6's channel in the tree,
    /// `.objects()` in `view.rs`'s production region is still exactly 2. The
    /// needles are therefore enumerated and asserted individually, so the failure
    /// message says *which* read moved.
    ///
    /// # The zero pins, and the revert that put them there
    ///
    /// Five needles are pinned at **0**. They are not decoration: the first
    /// revert run against this gate replaced `state.zone(..)` with
    /// `state.zones().get(..)` — the same channel, one accessor over — and the
    /// two-needle draft of this gate went **green**. A gate whose own revert
    /// proof can be defeated by a synonym is not holding the property it claims
    /// to. So the accessors that reach the same data are pinned closed, and a
    /// bypass now fails on the pin it uses rather than on none at all. Two of the
    /// five (`.object(`, `.players()`) came from the `/review` cycle, which
    /// pointed out that the first draft closed the *plural* of one needle and the
    /// *singular* of another while leaving each one's opposite number open —
    /// exactly the class this paragraph is about.
    ///
    /// A **delegating** call into the engine is not a raw read and is not pinned:
    /// `library_look_cards` calls `rules::replacement::apply_search_library_replacement`
    /// (CR 121.1), which takes `&GameState` and returns an `Option<u32>` and some
    /// events. It reads no object table and can name no card.
    ///
    /// This is still an enumerated set and not a proof about *every* raw read —
    /// see [`view::question_card_label`]'s doc, which says so in the same terms.
    ///
    /// A fourth read is not forbidden; it is required to be *deliberate*, which is
    /// the most a gate can enforce and is exactly what was missing when
    /// `GameSummary.seed` shipped for three sessions.
    #[test]
    fn test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source =
            std::fs::read_to_string(root.join("src").join("view.rs")).expect("view.rs is readable");
        // `test_region` returns the suffix STARTING at the `#[cfg(test)]` cut, so
        // the production region is its complement. (`view.rs` has no test module
        // at all, in which case that suffix is empty and the whole file is
        // production — which is the case this gate is written for.)
        let cut = source.len() - test_region(&source).len();
        let production = code_only(&source[..cut]);
        // `concat!` for the same reason the sibling gates use it: a plainly-written
        // needle would be found by the gate rather than by the code it is meant to
        // describe, and both needles are named in `view.rs`'s own prose (which
        // `code_only` blanks — belt and braces).
        //
        // Each entry is (needle, expected count, what that read is for). The two
        // non-zero pins are the three accounted-for reads; the zero pins are
        // BYPASSES held closed — see below for why they are here at all.
        let pins: [(&str, usize, &str); 7] = [
            (
                concat!(".obj", "ects()"),
                2,
                "question_card_label's CR 701.22a/23a/25a look entitlement, and \
                 action_modes' card-registry lookup (an id the seat already holds, \
                 no name at all)",
            ),
            (
                concat!(".zo", "ne("),
                1,
                "library_look_cards' CR 701.23a whole-library look (UI-6) — the \
                 SEARCHER'S OWN library, sorted by name so no library order is \
                 disclosed",
            ),
            (
                concat!(".zo", "nes()"),
                0,
                "the raw zone table — the exact bypass that would re-open the \
                 whole-library channel while leaving the `.zone(` pin at 1",
            ),
            (
                concat!(".objects_in_", "zone("),
                0,
                "the same bypass by another accessor: it returns whole objects, \
                 names and all, for any zone of any seat",
            ),
            (
                concat!(".pla", "yer("),
                0,
                "raw `PlayerState`, which the seat-redacted view already models",
            ),
            (
                concat!(".obj", "ect("),
                0,
                "the SINGULAR of the needle pinned at 2 above — it returns one \
                 whole GameObject, name included, for any id in any zone, and is \
                 the most natural way a fourth look channel would be written",
            ),
            (
                concat!(".pla", "yers()"),
                0,
                "the plural/singular pair of `.player(` — the same shape of \
                 synonym the revert below defeated a two-needle draft with",
            ),
        ];
        let mut total = 0usize;
        for (needle, want, purpose) in pins {
            let found = production.matches(needle).count();
            total += found;
            assert_eq!(
                found, want,
                "view.rs's production code makes {found} raw `{needle}` GameState \
                 read(s), not the {want} that are accounted for ({purpose}). A read \
                 this gate does not know about is a NEW hidden-information channel \
                 and no other Invariant-7 gate can see it — document it and update \
                 this pin deliberately, or route it through NameIndex."
            );
        }
        assert_eq!(
            total, 3,
            "the per-needle pins above must sum to the three accounted-for raw reads"
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
    /// PB-DX29 `/review` H1: a minimal two-player `GameState` for the
    /// `validate_additional_cost_params` unit tests.
    ///
    /// These tests exercise the **400 boundary's own rules** — "this decision never
    /// offered that", "that count exceeds the offer's own bound", "that card is not in
    /// the eligible set" — none of which needs game state. The state argument exists for
    /// the whole-answer affordability check, which deliberately **fails open** when it
    /// cannot compute a cost: the synthetic `ObjectId(1)` these fixtures name is not in
    /// this state, so that check abstains and each test still measures exactly the rule
    /// it was written for. The affordability check has its own probes, on real boards.
    fn dx29_empty_state() -> mtg_engine::GameState {
        mtg_engine::GameStateBuilder::new()
            .add_player(mtg_engine::PlayerId(1))
            .add_player(mtg_engine::PlayerId(2))
            .active_player(mtg_engine::PlayerId(1))
            .build()
            .expect("a two-player state with no objects must build")
    }

    /// PB-DX29 `/review`: a corpus-backed `GameState` for the three fix-cycle probes.
    ///
    /// `objects` is `(name, zone)`; each is enriched from its real def so keywords,
    /// costs and subtypes are the corpus's rather than a fixture's invention.
    fn dx29_corpus_state(
        objects: &[(&str, mtg_engine::ZoneId)],
        pool: mtg_engine::ManaPool,
    ) -> mtg_engine::GameState {
        let defs: std::collections::HashMap<String, mtg_engine::CardDefinition> =
            mtg_engine::all_cards()
                .into_iter()
                .map(|d| (d.name.clone(), d))
                .collect();
        let mut builder = mtg_engine::GameStateBuilder::new()
            .add_player(mtg_engine::PlayerId(1))
            .add_player(mtg_engine::PlayerId(2))
            .active_player(mtg_engine::PlayerId(1))
            .at_step(mtg_engine::Step::PreCombatMain)
            .with_registry(mtg_simulator::build_registry())
            .player_mana(mtg_engine::PlayerId(1), pool);
        for (name, zone) in objects {
            let card_id = defs
                .get(*name)
                .unwrap_or_else(|| panic!("corpus def {name:?} not found"))
                .card_id
                .clone();
            builder = builder.object(mtg_engine::enrich_spec_from_def(
                mtg_engine::ObjectSpec::card(mtg_engine::PlayerId(1), name)
                    .with_card_id(card_id)
                    .in_zone(*zone),
                &defs,
            ));
        }
        builder.build().expect("PB-DX29 fixture must build")
    }

    fn dx29_object_named(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("object {name:?} not found"))
    }

    /// **PB-DX29 `/review` M1** — CR 702.132a and five siblings. Each of the six
    /// `AdditionalCost` kinds this batch does NOT surface is refused at the 400 boundary
    /// rather than forwarded to the engine.
    ///
    /// The review proved by execution that the previous `_ => {}` let an `Assist` through
    /// and the engine ACCEPTED it, draining another seat's mana pool 5 -> 3 without that
    /// seat being asked. The batch's own doc argued the kinds were safe because they are
    /// "deliberately not surfaced" — which closes the picker and not the wire.
    ///
    /// All six are asserted, not just the one that was executed: the fix is one arm and
    /// a partial test would leave five of them resting on the same refuted argument.
    #[test]
    fn test_dx29_every_unsurfaced_cost_kind_is_refused_at_the_400_boundary() {
        let action = ui2_cast_spell_action_with_costs(
            vec![mtg_engine::ObjectId(7)],
            mtg_engine::ObjectId(7),
            1,
        );
        let unsurfaced: Vec<(&str, mtg_engine::AdditionalCost)> = vec![
            (
                "Assist",
                mtg_engine::AdditionalCost::Assist {
                    player: mtg_engine::PlayerId(2),
                    amount: 2,
                },
            ),
            (
                "Mutate",
                mtg_engine::AdditionalCost::Mutate {
                    target: mtg_engine::ObjectId(7),
                },
            ),
            (
                "Discard",
                mtg_engine::AdditionalCost::Discard(vec![mtg_engine::ObjectId(7)]),
            ),
            (
                "EscapeExile",
                mtg_engine::AdditionalCost::EscapeExile {
                    cards: vec![mtg_engine::ObjectId(7)],
                },
            ),
            (
                "CollectEvidenceExile",
                mtg_engine::AdditionalCost::CollectEvidenceExile {
                    cards: vec![mtg_engine::ObjectId(7)],
                },
            ),
            // `ExileFromHand` is NOT in this list -- PB-DX44 surfaced it (CR
            // 118.9, the pitch alt cost), so it is no longer refused by the
            // catch-all this test is for. It is refused a DIFFERENT way now
            // (this action's plan carries no `pitch` offer), which
            // `test_dx44_pitch_answer_without_a_pitch_offer_is_refused` below
            // pins.
        ];
        for (label, cost) in unsurfaced {
            let err = api::validate_additional_cost_params(
                &action,
                &crate::view::ActionParamsDto {
                    additional_costs: vec![cost],
                    ..Default::default()
                },
                &dx29_empty_state(),
                mtg_engine::PlayerId(1),
            )
            .expect_err(&format!(
                "{label} is not surfaced by any picker, so an answer naming it is wrong \
                 against the payload the client is holding -- it must be a 400 here, not a \
                 forward to the engine. This is the arm that let an Assist drain another \
                 seat's mana pool."
            ));
            assert_eq!(err.status, StatusCode::BAD_REQUEST, "{label}");
            assert!(
                err.body.error.contains(label),
                "{label}: the refusal must NAME the kind, so a client can tell which entry \
                 was wrong: {}",
                err.body.error
            );
        }
    }

    /// **PB-DX44** — `ExileFromHand` (CR 118.9's pitch alt cost) is now a
    /// SURFACED kind, so a submission naming it on an action whose plan
    /// carries no `pitch` offer is refused by its OWN arm
    /// (`AdditionalCost::ExileFromHand { card } => { let Some(pitch) =
    /// plan.pitch.as_ref() else { ... } }`), not by the default-deny
    /// catch-all the test above pins. Both are 400s; this test is what
    /// proves the NEW arm is reached at all.
    #[test]
    fn test_dx44_pitch_answer_without_a_pitch_offer_is_refused() {
        let action = ui2_cast_spell_action_with_costs(
            vec![mtg_engine::ObjectId(7)],
            mtg_engine::ObjectId(7),
            1,
        );
        let err = api::validate_additional_cost_params(
            &action,
            &crate::view::ActionParamsDto {
                additional_costs: vec![mtg_engine::AdditionalCost::ExileFromHand {
                    card: mtg_engine::ObjectId(7),
                }],
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("this action's plan carries no pitch offer, so this must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
        assert!(
            err.body.error.contains("pitch"),
            "the refusal must name the missing offer: {}",
            err.body.error
        );
    }

    /// **PB-DX29 `/review` H1** — CR 601.2f-h / CR 702.47a. An unaffordable SPLICE is
    /// refused at the 400 boundary, and an affordable one is accepted.
    ///
    /// The review proved by execution that the shipped code offered the splice, accepted
    /// the answer here, and let the engine return `422 InsufficientMana` — a clean offer
    /// followed by a server rejection, one family over from the marker affordability the
    /// batch had already fixed. `SpliceCostOption`'s doc gives a real reason not to
    /// publish a bound in the OFFER (bounding it is a subset-sum over `eligible`); it is
    /// not a reason to skip the check HERE, where the chosen list is known.
    ///
    /// Both directions on ONE board, so the refusal is provably about the mana and not
    /// about the fixture.
    #[test]
    fn test_dx29_an_unaffordable_splice_is_refused_at_the_400_boundary() {
        use mtg_engine::{ManaPool, ZoneId};
        let p1 = mtg_engine::PlayerId(1);
        let probe = |pool: ManaPool| {
            let state = dx29_corpus_state(
                &[
                    ("Reach Through Mists", ZoneId::Hand(p1)),
                    ("Glacial Ray", ZoneId::Hand(p1)),
                ],
                pool,
            );
            let spell = dx29_object_named(&state, "Reach Through Mists");
            let splice_card = dx29_object_named(&state, "Glacial Ray");
            let action = mtg_simulator::LegalAction::CastSpell {
                card: spell,
                from_zone: ZoneId::Hand(p1),
                additional_costs: mtg_simulator::legal_actions::AdditionalCostPlan {
                    splice: Some(mtg_simulator::legal_actions::SpliceCostOption {
                        eligible: vec![splice_card],
                    }),
                    ..Default::default()
                },
                alt_cost: None,
            };
            api::validate_additional_cost_params(
                &action,
                &crate::view::ActionParamsDto {
                    additional_costs: vec![mtg_engine::AdditionalCost::Splice {
                        cards: vec![splice_card],
                    }],
                    ..Default::default()
                },
                &state,
                p1,
            )
        };

        // {U} alone pays Reach Through Mists and not Glacial Ray's {1}{R} splice.
        let err = probe(ManaPool {
            blue: 1,
            ..Default::default()
        })
        .expect_err(
            "CR 601.2f-h: one blue mana cannot pay {U} plus a {1}{R} splice, so this must be \
             a 400 naming the offer -- not a 422 from the engine after the server made the \
             offer itself",
        );
        assert_eq!(err.status, StatusCode::BAD_REQUEST);

        // Discriminating control: the same board with the mana really available accepts.
        // Without this the assertion above could pass on a blanket refusal.
        probe(ManaPool {
            blue: 1,
            red: 1,
            colorless: 1,
            ..Default::default()
        })
        .expect("with {U} plus {1}{R} available the same splice must be ACCEPTED");
    }

    /// **PB-DX29 `/review` M2** — CR 606.6. An announced `{X}` above the planeswalker's
    /// loyalty counters is refused at the 400 boundary, and one within them is accepted.
    ///
    /// `x_value` was hard-coded `None` before this batch, so it could not be
    /// over-announced; PB-DX29 opened the channel and bounded nothing. The review
    /// measured X = 9 on `chandra_flamecaller` (`Complete`, deck-legal, 4 loyalty)
    /// reaching the engine and coming back as a 422.
    #[test]
    fn test_dx29_an_over_loyalty_x_value_is_refused_at_the_400_boundary() {
        use mtg_engine::{CounterType, ManaPool, ZoneId};
        let mut state = dx29_corpus_state(
            &[("Chandra, Flamecaller", ZoneId::Battlefield)],
            ManaPool::default(),
        );
        let chandra = dx29_object_named(&state, "Chandra, Flamecaller");
        // CR 606.5b: a planeswalker enters with loyalty counters; the fixture sets them
        // directly because no ETB replacement runs on a hand-built state.
        state
            .objects_mut()
            .get_mut(&chandra)
            .expect("just built")
            .counters
            .insert(CounterType::Loyalty, 4);

        // Non-vacuity: index 2 really is the `-X` ability, from the engine's own query.
        assert!(
            mtg_engine::loyalty_ability_needs_x(&state, chandra, 2),
            "precondition (CR 606.4/107.3m): Chandra's loyalty index 2 is the `-X` ability"
        );

        let probe = |x: u32| {
            api::validate_loyalty_x_value(
                &mtg_simulator::LegalAction::ActivateLoyaltyAbility {
                    source: chandra,
                    ability_index: 2,
                },
                &crate::view::ActionParamsDto {
                    x_value: x,
                    ..Default::default()
                },
                &state,
            )
        };

        let err = probe(9).expect_err(
            "CR 606.6: 4 loyalty counters cannot pay a -9. The engine refuses this too, but \
             as a 422 -- and the client was handed an unbounded number input, so the offer \
             invited the answer it then rejected",
        );
        assert_eq!(err.status, StatusCode::BAD_REQUEST);

        // Discriminating controls, both sides of the boundary.
        probe(4).expect("X == the available counters is exactly payable (CR 606.6)");
        probe(0).expect("X = 0 is always legal (CR 107.3m)");
    }

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
                ..Default::default()
            },
            alt_cost: None,
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
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
                ..Default::default()
            },
            alt_cost: None,
        };
        let params = crate::view::ActionParamsDto {
            additional_costs: vec![mtg_engine::AdditionalCost::Squad { count: 1 }],
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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
        api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
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

        // `$viewer` — the replay viewer's component library, imported IN PLACE
        // rather than copied (`vite.config.js`'s alias, plan §8 R8). Those files
        // are compiled into *this* bundle by `npm run build`, so a call added
        // there would ship into the play client and the rule would have a hole
        // exactly the size of the shared library. Added after a `/review`
        // finding; currently zero hits, so this arm is coverage, not a repair.
        let viewer_lib =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../replay-viewer/frontend/src/lib");
        let mut shared: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&viewer_lib, &mut shared);

        // Call forms, not bare identifiers: the prose in `plainClone.svelte.js`
        // has to be able to name what it replaced. `indexedDB` is spelled with a
        // leading lowercase because that is the global's actual name — the docs
        // that discuss it write "IndexedDB", which is a different string.
        let forbidden = ["structuredClone(", ".postMessage(", "indexedDB"];

        for (path, text) in sources.iter().chain(shared.iter()) {
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
        // The shared library needs its own floor and its own named file: it is
        // reached by a `..` path, which is the arrangement most likely to resolve
        // to nothing after a move and leave the arm silently checking zero bytes.
        let shared_seen: BTreeSet<&str> = shared
            .iter()
            .filter_map(|(p, _)| p.rsplit('/').next())
            .collect();
        assert!(
            shared_seen.contains("cardTooltip.js") && shared.len() >= 8,
            "the `$viewer` walk under {} found {} files ({shared_seen:?}) — `vite.config.js` \
             aliases that directory into this bundle, so an empty walk is a hole in the ban",
            viewer_lib.display(),
            shared.len()
        );

        // (b) The four pickers really route through the sanctioned helper. A
        //     picker that stopped taking a copy at all would satisfy the ban
        //     above while quietly mutating its parent's reactive state.
        //     `DiscardPicker.svelte` joined this list at ENG-1: its `PickN`
        //     branch (CR 701.9b) builds an answer by cloning a `template` prop,
        //     the exact shape the other three pickers were already guarded for.
        for picker in [
            "SearchPicker.svelte",
            "PartitionPicker.svelte",
            "CostPicker.svelte",
            "DiscardPicker.svelte",
            // PB-DX45 (CR 118.12): the fifth template-copying picker.
            "ConfirmPicker.svelte",
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
    ///    *guarantee*, for the four pickers with no `try` (ENG-1 moved
    ///    `DiscardPicker` into the guarded group — its `PickN` branch now clones a
    ///    template too) and for every handler written later. Svelte 5's
    ///    `<svelte:boundary>` is not a substitute — it catches render and effect
    ///    errors, not DOM handler ones.
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
            "DiscardPicker.svelte",
            // PB-DX45 (CR 118.12): the fifth.
            "ConfirmPicker.svelte",
            // PB-DX50 (CR 702.140c): the sixth. Structurally the same shape as
            // `ConfirmPicker` and deliberately a different component -- see its own
            // doc for why "Pay {cost}" / "Decline" is the wrong label for an
            // over/under question.
            "BinaryChoicePicker.svelte",
        ] {
            let text = text_of(picker);
            // `onError?.(` and not the bare identifier: the identifier matches the
            // prop's own doc-comment line, so a picker that documented the prop
            // and never called it would pass. Anchoring on the CALL is the
            // difference between "the prop exists" and "a failure is reported"
            // (`/review` finding).
            assert!(
                text.contains("onError?.("),
                "{picker} never CALLS `onError` — a failure while building its answer would \
                 be invisible to the player"
            );
            assert!(
                text.contains("try {") && text.contains("} catch (err) {"),
                "{picker} does not guard its emit path; a throw there escapes the click \
                 handler and leaves the DOM untouched"
            );
        }

        // `ActionBar` must actually pass the prop down and route it out, or the
        // pickers report into nothing.
        let action_bar = text_of("ActionBar.svelte");
        assert_eq!(
            action_bar.matches("onError={onPickerError}").count(),
            6,
            "all six template-copying pickers must be given `onPickerError` \
             (PB-DX45 added `ConfirmPicker`, CR 118.12 -- and this ratchet is what \
             caught it: a new picker wired into `ActionBar` without the error prop \
             would be a silent dead button, which is exactly the UI-4 symptom this \
             gate exists for)"
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

    /// PB-DX29 — the balanced-brace body of the JS function whose header text is
    /// `header`, taken from raw Svelte/JS source.
    ///
    /// **Raw text, deliberately not [`code_only`].** That helper is a Rust lexer:
    /// on a `.svelte` file it blanks every single-quoted JS string as though it
    /// were a char literal, and every HTML attribute value with it — which is the
    /// same reason [`test_frontend_search_picker_looks_wider_than_it_picks`] reads
    /// raw text and picks needles that cannot occur in prose. Both callers here do
    /// the same, and each needle below was checked against the file's comments
    /// before being used.
    ///
    /// **Residual, stated rather than glossed**: this is a brace counter, not a JS
    /// parser. A `{` or `}` inside a string literal or a comment within the body
    /// would desynchronise it. So the callers assert the SHAPE of the slice it
    /// returns — that it closes on a brace and contains the function's own emit
    /// call — instead of trusting the walk.
    fn js_function_body<'a>(source: &'a str, header: &str) -> &'a str {
        let start = source
            .find(header)
            .unwrap_or_else(|| panic!("`{header}` does not appear in this file at all"));
        let open = start
            + source[start..]
                .find('{')
                .unwrap_or_else(|| panic!("`{header}` has no opening brace"));
        let bytes = source.as_bytes();
        let mut depth = 0i32;
        for (i, byte) in bytes.iter().enumerate().skip(open) {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return &source[open..=i];
                    }
                }
                _ => {}
            }
        }
        panic!("`{header}` has unbalanced braces");
    }

    /// PB-DX29 — the first argument of every CALL to `CostPicker`'s template
    /// filler, in source order. The declaration is not a call and is skipped.
    fn template_filler_arguments(source: &str) -> Vec<String> {
        const NEEDLE: &str = "fillTemplate(";
        let mut args = Vec::new();
        for (idx, _) in source.match_indices(NEEDLE) {
            if source[..idx].ends_with("function ") {
                continue;
            }
            let rest = &source[idx + NEEDLE.len()..];
            let end = rest
                .find(',')
                .expect("every template-filler call takes three arguments");
            args.push(rest[..end].trim().to_string());
        }
        args
    }

    /// **PB-DX29 — the cost picker must answer every family the offer can carry.**
    ///
    /// `AdditionalCostsView` grew from two cast-side families (CR 118.8 sacrifice,
    /// CR 702.157a Squad) to six: `counts` (Replicate CR 702.56a / Escalate
    /// CR 702.120a), `markers` (Entwine CR 702.42a / Fuse CR 702.102a / Offspring
    /// CR 702.175a), `gift` (CR 702.174a) and `splice` (CR 702.47a).
    ///
    /// # The defect this exists to prevent
    ///
    /// A family dropped from the answer builder is **silent in both directions**.
    /// The picker still opens (the stage gate is `option.costs`, not the family),
    /// the widget may even still render, Confirm still works, and the server
    /// happily accepts an `additional_costs` array with that entry missing —
    /// because every one of these riders is optional and an absent entry IS the
    /// legal decline. The human pays no mana and gets no replicate copy, no gift,
    /// no spliced text, with nothing anywhere saying so. That is `OOS-UI2-4`'s
    /// symptom exactly, and it is the reason this is a per-family gate rather than
    /// one "the picker handles costs" assertion.
    ///
    /// Two layers are pinned, because either alone leaves the hole open: the
    /// component's own `confirm()` must reference each family, and `ActionBar` must
    /// actually pass each one down. A prop that is never threaded is a family that
    /// is always `null`, and every check inside the component would be vacuously
    /// green.
    ///
    /// Source-level for the standing reason — there is no frontend test harness
    /// (plan §8 R7).
    #[test]
    fn test_frontend_cost_picker_answers_every_cost_family() {
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

        let picker = text_of("CostPicker.svelte");
        let body = js_function_body(picker, "function confirm()");

        // The slice really is the emit path and not some earlier brace — see
        // `js_function_body`'s residual note.
        assert!(
            body.ends_with('}') && body.contains("onConfirm?.("),
            "the extracted `confirm()` body does not end at a closing brace or does not \
             contain the emit call, so the walk read the wrong region and every assertion \
             below would be about the wrong text"
        );
        assert!(
            body.len() > 800,
            "the extracted `confirm()` body is only {} bytes — that is too short to be the \
             six-family answer builder, so this gate is checking a stub",
            body.len()
        );

        // Each family, by the two identifiers it cannot be answered without: the
        // template it contributes and the key (or, for the unit-variant markers,
        // the template itself) that carries the human's answer.
        for (family, cr, needles) in [
            (
                "sacrifice",
                "CR 118.8",
                ["sacrifice.template", "sacrifice.ids_key"],
            ),
            (
                "squad",
                "CR 702.157a",
                ["squad.template", "squad.count_key"],
            ),
            (
                "counts",
                "CR 702.56a / CR 702.120a",
                ["countList", "count.count_key"],
            ),
            (
                "markers",
                "CR 702.42a / CR 702.102a / CR 702.175a",
                ["markerList", "marker.template"],
            ),
            ("gift", "CR 702.174a", ["gift.template", "gift.player_key"]),
            (
                "splice",
                "CR 702.47a",
                ["splice.template", "splice.ids_key"],
            ),
        ] {
            for needle in needles {
                assert!(
                    body.contains(needle),
                    "`CostPicker.confirm()` never mentions {needle:?}, so the {family} family \
                     ({cr}) contributes nothing to the answer. The server ACCEPTS that — every \
                     one of these riders is optional and an absent entry is the legal decline — \
                     so the human simply loses the cost with no error anywhere."
                );
            }
        }

        // The decline semantics, which are the other half of "answered". An entry
        // contributed at zero/empty would be a payment of nothing rather than a
        // decline, and would stop a fully-declined answer being byte-identical to a
        // plain cast.
        for (rule, needle) in [
            ("a count of 0 declines the rider", "if (n <= 0) continue;"),
            ("an unchecked marker is not paid", "markerPaid[i] !== true"),
            ("no seat picked means no gift", "giftPicked !== null"),
            (
                "an empty splice list is a decline",
                "splicePicked.length > 0",
            ),
        ] {
            assert!(
                body.contains(needle),
                "`CostPicker.confirm()` lost the rule that {rule} (expected {needle:?}). \
                 Declining every optional rider must produce the same bytes as a plain cast."
            );
        }

        // `giftPicked !== null` and never a truth test: `PlayerId(0)` is a real seat
        // and is falsy in JS. Same class as `ObjectId::SENTINEL` serialising as `0`,
        // which UI-4's review found leaving Confirm live over an empty candidate set.
        assert!(
            !body.contains("if (gift && giftPicked)"),
            "the gift contribution is gated on a truth test; seat 0 is a real player and is \
             falsy, so the first seat at the table could never be promised a gift"
        );

        // The props exist, default to the empty answer, and survive a `null`.
        //
        // **This comment used to say the two list families are
        // `skip_serializing_if = Vec::is_empty` server-side and "arrive ABSENT on
        // almost every cast". That attribute does not exist** — `view.rs` carries no
        // `skip_serializing_if` on `counts` or `markers`, and its own doc records that
        // an earlier draft's was removed, because two presence conventions in one
        // struct is a trap for the next client. A gate resting on a false premise is
        // still a gate resting on a false premise even when what it asserts is true
        // (PB-DX29 `/review` L4). The defaults below are asserted because they are
        // correct defensive practice, not because the field is ever absent.
        for decl in [
            "counts = []",
            "markers = []",
            "gift = null",
            "splice = null",
        ] {
            assert!(
                picker.contains(decl),
                "`CostPicker` does not declare the prop {decl:?} with its declining default"
            );
        }
        for derived in ["$derived(counts ?? [])", "$derived(markers ?? [])"] {
            assert!(
                picker.contains(derived),
                "`CostPicker` must derive its list families from the props with {derived:?} — \
                 `counts` and `markers` are omitted from the payload when empty"
            );
        }

        // And `ActionBar` really passes each one down. A family checked inside a
        // component it is never given is a vacuously green check.
        let action_bar = text_of("ActionBar.svelte");
        for prop in [
            "counts={activeOption.costs.counts}",
            "markers={activeOption.costs.markers}",
            "gift={activeOption.costs.gift}",
            "splice={activeOption.costs.splice}",
        ] {
            assert!(
                action_bar.contains(prop),
                "`ActionBar` never threads {prop:?} into `CostPicker`, so that family is always \
                 `null` in the component and every check above is vacuous for it"
            );
        }

        // Non-vacuity of the matcher itself, by execution against a synthetic
        // one-family builder rather than by argument.
        let synthetic = "{ entries.push(fillTemplate(sacrifice.template, sacrifice.ids_key, \
                         [chosenId])); onConfirm?.(); }";
        assert!(
            !synthetic.contains("splice.template") && !synthetic.contains("markerList"),
            "the per-family needles above would not have caught a builder that answers only \
             the sacrifice"
        );
    }

    /// **PB-DX29 — a marker cost is a bare JSON string and must never be filled in
    /// like an object.**
    ///
    /// # The wire fact
    ///
    /// `AdditionalCost::Entwine`, `::Fuse` and `::Offspring` are Rust **unit**
    /// variants. Serde's externally-tagged encoding serialises a unit variant as a
    /// bare string — `"Entwine"` — not as `{"Entwine": {…}}`. Every other cost
    /// family in this client is answered by the clone-and-write-one-field idiom
    /// (`fillTemplate`), and that idiom is *structurally wrong* on a string: it
    /// reads `Object.keys(template)[0]`, which on a string yields the character
    /// index `"0"`, and then assigns into a primitive. The result is either a throw
    /// out of the click handler — UI-4's dead-Confirm symptom, from a new cause —
    /// or a corrupted entry the server 400s.
    ///
    /// It is the same shape-of-JSON trap PB-DP10 measured on `Effect::Proliferate`,
    /// where a serde walk that matched object keys only was structurally blind to a
    /// unit variant.
    ///
    /// # What is pinned
    ///
    /// 1. The client checks that a marker template really is a string, and reports
    ///    through `onError` when it is not — a server-side encoding change is a
    ///    thing to report, not to coerce.
    /// 2. The SET of first arguments handed to the template filler is exactly the
    ///    five families that have a fillable object template. That is an exhaustive
    ///    pin rather than a "no marker" ban on purpose: a seventh family added to
    ///    the offer must be classified here — object or unit — instead of joining
    ///    whichever branch happened to compile.
    /// 3. The markers really are contributed, verbatim. A gate that only forbade
    ///    the wrong call would stay green on a picker that dropped the family
    ///    entirely, which is the sibling gate's subject.
    #[test]
    fn test_frontend_cost_picker_never_fills_a_unit_variant_marker_template() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);
        let picker = sources
            .iter()
            .find(|(p, _)| p.ends_with("CostPicker.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("CostPicker.svelte is in the frontend walk");

        // (1) The guard. Counted, not merely found: this is raw text, so a doc
        //     comment quoting the check would otherwise make the gate vacuous.
        const GUARD: &str = "typeof marker.template !== 'string'";
        assert_eq!(
            picker.matches(GUARD).count(),
            1,
            "`CostPicker` must contain exactly one {GUARD:?} — the executable guard, and no \
             prose copy of it. A marker cost arrives as a bare JSON string (serde's unit-variant \
             encoding); anything else is a server change to report through `onError`, not to \
             paper over."
        );

        // (2) Every template-filler call site, classified by exhaustion.
        let args = template_filler_arguments(picker);
        let seen: BTreeSet<&str> = args.iter().map(String::as_str).collect();
        let expected: BTreeSet<&str> = [
            "sacrifice.template",
            "squad.template",
            "count.template",
            "gift.template",
            "splice.template",
            // PB-DX44: `AdditionalCost::ExileFromHand { card }` carries a
            // SCALAR field, exactly like `gift.template`'s `opponent` --
            // object-shaped, not a unit variant, so it belongs in this list.
            "pitch.template",
        ]
        .into_iter()
        .collect();
        assert_eq!(
            seen, expected,
            "the set of templates handed to `fillTemplate` changed. That function clones an \
             OBJECT and writes one named key; a unit-variant cost (Entwine / Fuse / Offspring) \
             is a bare string and has no key to write, so passing one here reads a character \
             index as the variant name and assigns into a primitive. Add the new family to this \
             list only after deciding which encoding it has."
        );
        assert_eq!(
            args.len(),
            6,
            "expected exactly six template-filler call sites, one per object-shaped family; \
             found {args:?}"
        );
        for arg in &args {
            assert!(
                !arg.starts_with("marker"),
                "{arg:?} is a marker template being passed to `fillTemplate` — see this test's \
                 doc comment; that is the exact unit-variant trap it exists to prevent"
            );
        }

        // (3) The markers are still contributed, verbatim and proxy-safely.
        let body = js_function_body(picker, "function confirm()");
        assert!(
            body.contains("entries.push(plainClone(marker.template));"),
            "a paid marker must be pushed verbatim (through `plainClone`, per UI-4). Without \
             this, banning the wrong call would be satisfied by dropping the family."
        );

        // Non-vacuity: the parser saw real call sites and skipped the declaration.
        assert!(
            picker.contains("function fillTemplate("),
            "`CostPicker` no longer declares `fillTemplate`; this gate's parser is keyed on that \
             name and would silently see zero call sites"
        );
    }

    /// **UI-6: the search picker LOOKS at the whole library and PICKS only from
    /// `candidates`** (G9, CR 701.23a / SR-38).
    ///
    /// The server sends two lists on purpose. This gate is what stops the client
    /// collapsing them back into one — in either direction, both of which are real
    /// regressions and only one of which is obvious:
    ///
    /// * rendering `candidates` alone re-creates the playtest complaint verbatim
    ///   (*"only showed legal basic lands"*) while every test on the server side
    ///   stays green, because the payload is fine and the client throws it away —
    ///   which is **exactly** how UI-1's own defect shipped;
    /// * making a look-only card selectable would post an id
    ///   `handle_answer_effect_choice` refuses, i.e. offer an illegal answer
    ///   (SR-38). The server 400s it, so the failure is safe — but it reaches the
    ///   player as "request failed", not as a rule.
    ///
    /// Source-level for the standing reason: there is still no frontend test
    /// harness (plan §8 R7). This proves the wiring exists, not that it renders —
    /// the rendering was verified in a browser instead, and the observations are
    /// in the task record.
    ///
    /// Needles are chosen to be **code-only** — `card.pickable`, `candidateIds.has(`,
    /// the CSS class — rather than blanking comments first, because
    /// [`code_only`] would also blank the HTML attribute strings this gate reads.
    /// Each needle was checked against the file's prose before being used here.
    #[test]
    fn test_frontend_search_picker_looks_wider_than_it_picks() {
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

        let picker = text_of("SearchPicker.svelte");

        // 1. The look list is consumed at all. Without this the server's whole
        //    CR 701.23a channel is dead weight and nobody would notice.
        assert!(
            picker.contains("allCards = []"),
            "SearchPicker must accept the `allCards` prop — CR 701.23a's whole-library \
             look is otherwise sent and discarded, which is precisely how UI-1's \
             defect shipped"
        );
        assert!(
            picker.contains("for (const card of allCards)"),
            "SearchPicker must actually iterate `allCards` into its rows"
        );

        // 2. Pickability is decided by membership in the ENGINE'S answer space,
        //    at all three places a card can become an answer.
        assert!(
            picker.contains("candidateIds.has(card.id)"),
            "a row's `pickable` flag must come from the candidate set, not from \
             whether the server happened to send the card"
        );
        assert!(
            picker.contains("if (!candidateIds.has(id)) return;"),
            "`select` must refuse a look-only id (SR-38: never build an illegal answer)"
        );
        assert!(
            picker.contains("!candidateIds.has(found)"),
            "`emit` must re-check membership before posting — a render-only guard is \
             one refactor away from being no guard, and the server's own refusal is \
             a 400 the player reads as `request failed` rather than as a rule"
        );

        // 3. The distinction is VISIBLE, and a look-only row is not a control.
        assert!(
            picker.contains("{#if card.pickable}"),
            "the template must branch on pickability — an invisible distinction is \
             not a distinction"
        );
        assert!(
            picker.contains("class=\"candidate look-only\""),
            "look-only rows need their own class so they read as unavailable"
        );
        // The full opening tag, not the bare class name: `look-tag` also occurs in
        // the stylesheet, so the bare needle stayed green with the `<span>`
        // deleted — the assertion did not prove what its message claimed
        // (`/review` finding, the same overstatement class this batch's other
        // gates are written against).
        assert!(
            picker.contains("<span class=\"look-tag\">look only</span>"),
            "a look-only row must RENDER its visible `look only` tag, not merely \
             have a style rule for one"
        );
        // Not a `<button>`: a disabled control reads as "not right now", and this
        // is a permanent rules fact. Checked by position — the look-only element's
        // opening tag must be a `div`.
        let look_only_at = picker
            .find("class=\"candidate look-only\"")
            .expect("just asserted");
        let tag_start = picker[..look_only_at]
            .rfind('<')
            .expect("the class sits inside an element");
        assert!(
            picker[tag_start..].starts_with("<div"),
            "the look-only row must not be a button (disabled or otherwise): it is \
             not a control that is unavailable right now, it is a card this search \
             can never find. Found: {:?}",
            &picker[tag_start..tag_start + 20.min(picker.len() - tag_start)]
        );

        // 4. And the server's field actually reaches the picker.
        let action_bar = text_of("ActionBar.svelte");
        assert!(
            action_bar.contains("allCards={currentShape.all_cards"),
            "`ActionBar` must pass `AnswerShapeView::PickOne::all_cards` down; the \
             picker cannot show a look it is never given"
        );
    }

    /// Replace every `<!-- … -->` body with spaces, keeping newlines and byte
    /// offsets aligned with the source.
    ///
    /// Blanking rather than deleting so a failure message can still quote the
    /// real line. Used by the two gates that must not read a comment as code:
    /// template comments in this client explain the very elements and props the
    /// gates assert on, so they quote them verbatim.
    fn blank_html_comments(text: &str) -> String {
        let mut out: Vec<u8> = text.as_bytes().to_vec();
        let mut i = 0usize;
        while i + 4 <= out.len() {
            if &out[i..i + 4] == b"<!--" {
                let end = out[i..]
                    .windows(3)
                    .position(|w| w == b"-->")
                    .map(|p| i + p + 3)
                    .unwrap_or(out.len());
                for b in &mut out[i..end] {
                    if *b != b'\n' {
                        *b = b' ';
                    }
                }
                i = end;
            } else {
                i += 1;
            }
        }
        String::from_utf8(out).expect("blanking preserves UTF-8 boundaries")
    }

    /// Every opening tag in `text` that carries `use:cardTooltip`, returned whole.
    ///
    /// Companion to [`test_frontend_card_elements_carry_no_native_title`]. A
    /// substring search would be wrong in both directions here: `title=` occurs
    /// legitimately on buttons and panels that are not tooltip anchors, and a
    /// tag's attributes are spread over a dozen lines, so "the same line" is not
    /// the unit either. The unit is the **element**, so this walks one.
    ///
    /// Both the `{…}` expression depth and the quote state are tracked, because
    /// a Svelte attribute value can legally contain `>` (`class:pt-damaged={p
    /// .damage_marked > 0}`), and stopping at the first `>` would truncate the
    /// tag and read as "no title here".
    ///
    /// # Only the template, and not its comments
    ///
    /// Two exclusions, both found by this gate failing on its first run rather
    /// than reasoned out in advance — which is the point of firing it at a
    /// synthetic element as well:
    ///
    ///  - **Everything up to the last `</script>` is dropped.** A component's
    ///    module doc names `use:cardTooltip` in prose, and walking back from
    ///    there finds the nearest `<` — `<script` itself, or worse, a `<`
    ///    comparison operator in code — and reports a tag that does not exist.
    ///    A file with no `</script>` has no template and yields nothing, which
    ///    is also how `cardTooltip.js` is excluded without naming it.
    ///  - **HTML comments are blanked.** Template comments explain these very
    ///    elements, so they quote them.
    fn card_tooltip_anchor_tags(text: &str) -> Vec<String> {
        let Some(script_end) = text.rfind("</script>") else {
            return Vec::new();
        };
        let template = blank_html_comments(&text[script_end..]);
        let text: &str = &template;

        let bytes = text.as_bytes();
        let mut tags = Vec::new();
        for (idx, _) in text.match_indices("use:cardTooltip") {
            let Some(start) = text[..idx].rfind('<') else {
                continue;
            };
            let mut depth = 0usize;
            let mut quote: Option<u8> = None;
            let mut end = bytes.len();
            for (i, &c) in bytes.iter().enumerate().skip(start + 1) {
                match quote {
                    Some(q) => {
                        if c == q {
                            quote = None;
                        }
                    }
                    None => match c {
                        b'"' | b'\'' => quote = Some(c),
                        b'{' => depth += 1,
                        b'}' => depth = depth.saturating_sub(1),
                        b'>' if depth == 0 => {
                            end = i;
                            break;
                        }
                        _ => {}
                    },
                }
            }
            tags.push(text[start..end.min(bytes.len())].to_string());
        }
        tags
    }

    /// **UI-5 (`scutemob-190`), G11 of `memory/playtest-triage-2026-08-02b.md` —
    /// no card element may carry a native `title` attribute.**
    ///
    /// # The defect
    ///
    /// Playtest note: *"hover card name interferes with the card image"*. The
    /// image is `cardTooltip`'s floating `position:fixed` div at `z-index:9999`.
    /// The interfering text was a browser-native `title=` on the **same
    /// element**, and a native tooltip is chrome: the browser/OS draws it at the
    /// cursor, above every z-index this document can reach, exactly where the
    /// image is anchored. **No CSS can fix it** — there is no selector for it,
    /// no stacking context that contains it. The `title` has to go.
    ///
    /// It went into `cardTooltip`'s new caption, which renders inside the
    /// floating div. Nine sites were named by the triage (`ZoneBattlefield` ×5,
    /// `ZoneHand`, `ZoneGraveyard`, `ZoneExile`, `SeatCard`); the badges nested
    /// *inside* those anchors carried the same attribute and produce the
    /// identical collision over a smaller hit area, so they went too.
    ///
    /// # Why the rule is per-element and not per-file
    ///
    /// `title` is fine, and useful, on a control that is not a tooltip anchor —
    /// `PlayApp`'s Export-report button, `SeatCard`'s drawer toggle,
    /// `StepControls`' whole row. Banning the attribute outright would have
    /// deleted working affordances to fix an unrelated bug. So the unit is the
    /// element: [`card_tooltip_anchor_tags`] extracts each opening tag carrying
    /// `use:cardTooltip`, and the ban applies to those tags alone. The
    /// descendant half is covered by a second, narrower arm: the five `$viewer`
    /// zone components render nothing *but* card chips and their badges, so
    /// inside those files the attribute is banned outright.
    ///
    /// # Vacuity
    ///
    /// Four arms, for the reason the sibling clone gate gives: named files plus
    /// a floor on both walks; a positive assertion that the anchors really do
    /// pass a caption (an anchor that dropped the `title` *and* the caption
    /// would satisfy the ban while losing the information); the caption builders
    /// exist and are called; and the extractor is fired at a synthetic offending
    /// element, so a bug in the tag walk cannot make this green on nothing.
    #[test]
    fn test_frontend_card_elements_carry_no_native_title() {
        let play_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let viewer_lib =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../replay-viewer/frontend/src/lib");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&play_src, &mut sources);
        let mut shared: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&viewer_lib, &mut shared);

        // The two attribute spellings Svelte accepts. `title=` bare is NOT the
        // needle: this file's own prose, and the components', name the attribute
        // in backticks, and a gate that cannot survive being described is a gate
        // nobody will keep.
        let attr_forms = ["title=\"", "title={"];

        let mut anchors_seen = 0usize;
        for (path, text) in sources.iter().chain(shared.iter()) {
            for tag in card_tooltip_anchor_tags(text) {
                anchors_seen += 1;
                for form in attr_forms {
                    assert!(
                        !tag.contains(form),
                        "{path} has a `use:cardTooltip` element that also carries a native \
                         `title` attribute:\n{tag}\n\nThe browser draws a native tooltip at the \
                         cursor above every z-index in this document — over the card image this \
                         action exists to show, which no CSS can prevent. Pass the text as \
                         `use:cardTooltip={{{{ name, caption }}}}` instead. UI-5 \
                         (`scutemob-190`, G11)."
                    );
                }
                // The information must not merely have been deleted.
                assert!(
                    tag.contains("caption") || tag.contains("tooltipArg("),
                    "{path}'s tooltip anchor passes no caption:\n{tag}\nG11 moved the `title` \
                     text into the floating div; an anchor with neither is a silent loss."
                );
            }
        }

        // ── non-vacuity ──
        // (a) The walks saw the files this rule is about, and a floor on each.
        let seen: BTreeSet<&str> = sources
            .iter()
            .chain(shared.iter())
            .filter_map(|(p, _)| p.rsplit('/').next())
            .collect();
        for expected in [
            "ZoneBattlefield.svelte",
            "ZoneHand.svelte",
            "ZoneGraveyard.svelte",
            "ZoneExile.svelte",
            "ZoneStack.svelte",
            "SeatCard.svelte",
            "cardTooltip.js",
        ] {
            assert!(
                seen.contains(expected),
                "the frontend walks missed {expected}; they saw {seen:?}"
            );
        }
        // Six components anchor the tooltip today (five `$viewer` zones plus
        // `SeatCard`) across ten elements. The floor is stated below the current
        // count so an ordinary addition does not fail it, but a walk that
        // resolved to nothing does.
        assert!(
            anchors_seen >= 8,
            "only {anchors_seen} `use:cardTooltip` elements were found — the walk is reading \
             the wrong directory and the ban above checked nothing"
        );

        // (b) The five zone components render card chips and nothing else, so
        //     the attribute is banned outright there — this is the descendant
        //     half of the rule, which the per-element arm cannot see.
        for zone in [
            "ZoneBattlefield.svelte",
            "ZoneHand.svelte",
            "ZoneGraveyard.svelte",
            "ZoneExile.svelte",
            "ZoneStack.svelte",
        ] {
            let (path, text) = shared
                .iter()
                .find(|(p, _)| p.ends_with(zone))
                .unwrap_or_else(|| panic!("{zone} is in the `$viewer` walk"));
            for form in attr_forms {
                assert!(
                    !text.contains(form),
                    "{path} carries a native `title` attribute. Every element in this file is a \
                     card chip or a badge inside one, so it sits under a `use:cardTooltip` \
                     anchor and collides with the image. Put the text in the caption."
                );
            }
        }

        // (b2) The two card-ish elements that still carry a `title` and are
        //      knowingly OUT of scope, named rather than left to be discovered:
        //      `StateView.svelte`'s command-zone chip (the exact mirror of the
        //      `SeatCard` site that WAS fixed) and `CombatView.svelte`'s
        //      attacker/blocker boxes. Neither anchors `cardTooltip` today, so
        //      neither collides with anything, and giving them a caption would
        //      mean giving them a tooltip — a feature, not this batch's repair.
        //
        //      What is asserted is the premise that makes them safe: they are
        //      NOT anchors. The moment one grows a `use:cardTooltip` this arm
        //      goes red and the per-element ban above starts applying to it,
        //      which is the only honest way to write an exemption down.
        //      (`/review` finding.)
        for exempt in ["StateView.svelte", "CombatView.svelte"] {
            let (path, text) = shared
                .iter()
                .find(|(p, _)| p.ends_with(exempt))
                .unwrap_or_else(|| panic!("{exempt} is in the `$viewer` walk"));
            assert!(
                !text.contains("use:cardTooltip"),
                "{path} now anchors `cardTooltip`, and it still carries native `title` \
                 attributes on its card elements. Those two cannot coexist — move the text \
                 into the caption (see this test's doc), then delete this file from the \
                 exemption list."
            );
        }

        // (c) The caption really is rendered, and the shared builder is used.
        let tooltip = shared
            .iter()
            .find(|(p, _)| p.ends_with("cardTooltip.js"))
            .map(|(_, t)| t.as_str())
            .expect("cardTooltip.js is in the walk");
        assert!(
            tooltip.contains("captionEl.textContent = ")
                && tooltip.contains("export function zoneCaption("),
            "`cardTooltip` must render the caption into its own element and export the shared \
             `zoneCaption` builder the four zone/seat sites call"
        );
        for caller in [
            "ZoneHand.svelte",
            "ZoneGraveyard.svelte",
            "ZoneExile.svelte",
        ] {
            let (_, text) = shared
                .iter()
                .find(|(p, _)| p.ends_with(caller))
                .unwrap_or_else(|| panic!("{caller} is in the walk"));
            assert!(
                text.contains("zoneCaption("),
                "{caller} must build its caption with the shared helper, not a local template — \
                 four sites wrote the same string before G11 and that is how they drifted"
            );
        }

        // (d) The extractor discriminates. Proven by execution against the exact
        //     element shape G11 removed — including a `>` inside an attribute
        //     expression, which is what a naive scan-to-first-`>` gets wrong,
        //     and with the two shapes that made this gate's first run report a
        //     tag that does not exist: a `use:cardTooltip` named in the module
        //     doc, and one named in a template comment.
        let synthetic = r#"<script>
              /** Anchored with use:cardTooltip; the walk must not see this. */
              const n = a < b;
            </script>

            <!-- This comment mentions use:cardTooltip too. -->
            <div
              class="permanent-card"
              class:pt-damaged={p.damage_marked > 0}
              title={typeLineStr(p)}
              use:cardTooltip={p.name}
            >
        "#;
        let tags = card_tooltip_anchor_tags(synthetic);
        assert_eq!(
            tags.len(),
            1,
            "the tag walk found {} tags in the synthetic component: {tags:?}",
            tags.len()
        );
        assert!(
            attr_forms.iter().any(|f| tags[0].contains(f)),
            "the tag walk would not have caught the exact element G11 removed: {}",
            tags[0]
        );
        assert!(
            tags[0].starts_with("<div"),
            "the tag walk anchored on the wrong element: {}",
            tags[0]
        );
    }

    /// **UI-5 (`scutemob-190`), G8 — `Concede` is not in the priority row, and
    /// a picker's escape hatch does not read as its peer.**
    ///
    /// Playtest note, written by someone who conceded a game by accident:
    /// *"had to cancel and concede, which ended the game? — i thought the
    /// concede button would concede the choice — this option should be next to
    /// new game, not in the priority changing area"*.
    ///
    /// Two halves, and the gate pins both because either alone leaves the trap
    /// half-set: `Concede` is filtered out of the action bar entirely and
    /// rendered in the header behind a confirmation, and the pickers' abort
    /// button says **Back** rather than Cancel.
    ///
    /// The submission is deliberately NOT changed and that is asserted too — the
    /// header button still routes an `option.index` through `ActionBar`'s single
    /// entry point, so this is placement, not a second code path to the most
    /// destructive action on the surface.
    #[test]
    fn test_concede_lives_in_the_header_behind_a_confirmation() {
        let play_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&play_src, &mut sources);
        let text_of = |name: &str| -> &str {
            sources
                .iter()
                .find(|(p, _)| p.ends_with(name))
                .map(|(_, t)| t.as_str())
                .unwrap_or_else(|| panic!("{name} is in the frontend walk"))
        };

        // 1. Out of the action bar — out of BOTH groups. A kind dropped from
        //    `controlKinds` alone reappears in the middle of the play list.
        let action_bar = text_of("ActionBar.svelte");
        // Read the two array LITERALS rather than matching whole source lines.
        // A `/review` finding: an exact-line assertion goes red on a reflow or a
        // reordered array, neither of which changes what the code does, and a
        // gate that cries wolf is a gate someone deletes.
        let array_after = |decl: &str| -> String {
            let rest = action_bar
                .split_once(decl)
                .unwrap_or_else(|| panic!("`ActionBar` no longer declares {decl}"))
                .1;
            rest[..rest.find(';').unwrap_or(rest.len())].to_string()
        };
        let control_kinds = array_after("const controlKinds =");
        assert!(
            control_kinds.contains("'PassPriority'") && !control_kinds.contains("'Concede'"),
            "`Concede` must not be a control kind — that is the row it was next to Pass in.              Found: {control_kinds}"
        );
        let relocated = array_after("const relocatedKinds =");
        assert!(
            relocated.contains("'Concede'")
                && relocated.contains("'TapForMana'")
                && action_bar.contains("!relocatedKinds.includes(a.kind)"),
            "`Concede` and `TapForMana` must both be filtered out of `plays` as well, or they              simply move left. Found: {relocated}"
        );

        // 2. In the header, behind two clicks, disabled-with-a-reason rather
        //    than absent.
        let play_app = text_of("PlayApp.svelte");
        for needle in [
            "const concedeAction = $derived(",
            "let concedeArmed = $state(false);",
            "function confirmConcede()",
            "const concedeDisabledReason = $derived.by(()",
            "class=\"concede-reason\"",
        ] {
            assert!(
                play_app.contains(needle),
                "`PlayApp` is missing {needle:?} — G8 wants a header concede that confirms \
                 before it fires and explains itself when it cannot"
            );
        }
        assert!(
            play_app.contains("actionBar?.beginExternal(concedeAction)"),
            "the header button must submit the SAME option through the SAME entry point; a \
             second path to conceding is the last thing this surface needs"
        );

        // 3. The pickers say Back. All nine (PB-DX45 added `ConfirmPicker`), plus
        //    the unknown-shape fallback, which aborts the same chain.
        for picker in [
            "ConfirmPicker.svelte",
            "DiscardPicker.svelte",
            "SearchPicker.svelte",
            "PartitionPicker.svelte",
            "CostPicker.svelte",
            "TargetPicker.svelte",
            "AttackerPicker.svelte",
            "BlockerPicker.svelte",
            "ValuePrompt.svelte",
        ] {
            let text = text_of(picker);
            assert!(
                text.contains(">Back</button>"),
                "{picker}'s abort button must say Back — it steps out of the picker and leaves \
                 the decision standing, which is not what Cancel reads as next to Concede"
            );
            assert!(
                !text.contains(">Cancel</button>"),
                "{picker} still has a button labelled Cancel"
            );
        }
        assert!(
            action_bar.contains("onclick={cancelChain}>Back</button>"),
            "the unknown-shape fallback aborts the same chain and must be labelled the same way"
        );
    }

    /// **UI-5 (`scutemob-190`), G10 — `TapForMana` is grouped, never hidden.**
    ///
    /// The playtest note asks for it to be *"removed from the list of legal
    /// actions"*, and doing that literally would remove capabilities the client
    /// has no other way to reach. The evidence, all of it checkable in this
    /// repo:
    ///
    ///   - `local_game.rs::auto_tap_commands_for` opens with
    ///     `let Command::CastSpell(cast) = command else { return None; };` —
    ///     auto-tap covers casts and nothing else.
    ///   - An activated ability's mana cost is paid out of the existing pool, so
    ///     a human has no other way to fill it (that residual gap is
    ///     `OOS-SIM6-3`, on the SIM track, deliberately untouched here).
    ///   - `PayEcho` / `PayCumulativeUpkeep` / `PayRecover` are only offered when
    ///     the pool already covers them, and `legal_actions.rs`' own comment says
    ///     *"CR 608.2g lets the player activate mana abilities first — so
    ///     TapForMana must stay available alongside these"*.
    ///
    /// So this gate is two-sided on purpose: the kind must be out of the plays
    /// list **and** still reachable. A future tidy-up that deletes the group
    /// satisfies half of the note and breaks every activation cost on the
    /// surface.
    #[test]
    fn test_tap_for_mana_is_grouped_and_still_reachable() {
        let play_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&play_src, &mut sources);
        let action_bar = sources
            .iter()
            .find(|(p, _)| p.ends_with("ActionBar.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ActionBar.svelte is in the frontend walk");

        assert!(
            action_bar.contains("a.kind === 'TapForMana'"),
            "`ActionBar` must partition `TapForMana` into its own group"
        );
        assert!(
            action_bar.contains("let manaOpen = $state(false);"),
            "the group must be COLLAPSED by default — that is the entire request"
        );
        assert!(
            action_bar.contains("mana sources ({manaSources.length})"),
            "the disclosure must say how many sources are folded behind it"
        );
        // Still reachable: a row that submits. `beginChain(row.options[0])` is
        // the assertion that matters — a group rendered as inert text would
        // satisfy every check above.
        assert!(
            action_bar.contains("beginChain(row.options[0])"),
            "a mana-source row must still submit its option. Hiding the kind removes the only \
             channel a human has for activation costs, echo, cumulative upkeep and recover \
             (CR 608.2g) — see this test's doc comment."
        );
        // Folded by name with a count, which is what makes eight Forests one row.
        assert!(
            action_bar.contains("const manaSourceRows = $derived.by(()"),
            "sources must fold by label with a count; eight identical buttons IS the clutter"
        );
    }

    /// **UI-5 (`scutemob-190`), G13 — a stacked land chip merges only genuinely
    /// fungible permanents, and tapped never merges with untapped.**
    ///
    /// Tap state is the information the playtest note is *about* ("same-name
    /// lands should stack **when tapped**"), so a key that omitted it would
    /// answer the request by destroying its subject. The rest of the key is
    /// every other field a `PermanentView` carries that distinguishes two
    /// permanents; merging is a claim of interchangeability and a land with a
    /// counter or an aura is not interchangeable with one without.
    ///
    /// Source-level, like its siblings, because there is still no frontend test
    /// harness (plan §8 R7). It cannot prove the chip renders; it can prove the
    /// key has not quietly narrowed to `name`, which is the one change that
    /// makes this feature lie.
    #[test]
    fn test_land_stacking_key_is_not_just_the_name() {
        let viewer_lib =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../replay-viewer/frontend/src/lib");
        let mut shared: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&viewer_lib, &mut shared);
        let zone = shared
            .iter()
            .find(|(p, _)| p.ends_with("ZoneBattlefield.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ZoneBattlefield.svelte is in the `$viewer` walk");

        assert!(
            zone.contains("function landStackKey(p)"),
            "the land stack must key on a named function, not an inline template"
        );
        let key_body = zone
            .split_once("function landStackKey(p)")
            .expect("checked above")
            .1;
        let key_body = &key_body[..key_body.find("\n  }").unwrap_or(key_body.len())];
        for field in [
            "p.name",
            "p.tapped",
            "counters",
            "p.attached_to",
            "p.is_commander",
            "p.is_token",
            "p.summoning_sick",
            "p.damage_marked",
        ] {
            assert!(
                key_body.contains(field),
                "the land fungibility key does not read {field}. Two permanents that differ in \
                 it are not interchangeable, and merging them into one chip is a lie about the \
                 board — `p.tapped` most of all, since tap state is what the request is about."
            );
        }

        // Opt-in, not default: the replay viewer is a step debugger and needs
        // per-object identity. See the component's module doc.
        assert!(
            zone.contains("stackLands = false,"),
            "`stackLands` must default OFF so the replay viewer keeps one chip per object"
        );
        let mut play_sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("frontend")
                .join("src"),
            &mut play_sources,
        );
        let play_board = play_sources
            .iter()
            .find(|(p, _)| p.ends_with("PlayBoard.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("PlayBoard.svelte is in the frontend walk");
        // A floor, not an equality: a third `ZoneBattlefield` instance on this
        // surface is an ordinary change, and an equality here would fail it for
        // no semantic reason (`/review`). What must hold is that EVERY instance
        // opts in — checked by counting the instances too, so an added one that
        // forgot the prop is caught.
        // Comments blanked first: this file's own prose names `stackLands`, and
        // counting that as an opt-in would let an added instance that forgot
        // the prop pass on the strength of the comment explaining it.
        let play_board_code = blank_html_comments(play_board);
        let instances = play_board_code.matches("<ZoneBattlefield").count();
        let opt_ins = play_board_code.matches("stackLands").count();
        assert!(
            instances >= 2 && opt_ins >= instances,
            "every `ZoneBattlefield` on the play surface must opt into `stackLands` — found \
             {instances} instances and {opt_ins} opt-ins"
        );

        // The click path is decided rather than implicit: the chip nominates a
        // representative and the caller can fall through to a sibling that
        // carries an offered action.
        assert!(
            zone.contains("onCardClick?.(stack.members[0], stack.members)"),
            "a stacked chip must hand its whole group to the caller — the caller is the only \
             party that knows which actions the server offered"
        );
        let play_app = play_sources
            .iter()
            .find(|(p, _)| p.ends_with("PlayApp.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("PlayApp.svelte is in the frontend walk");
        assert!(
            play_app.contains("function representativeFor(card, group)"),
            "`PlayApp` must resolve the stack's representative; without it, clicking a \
             5-Forest stack is undefined, which is the thing G13 asked to be decided"
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

    // ── SIM-4 (G2, CR 103.5): a mulligan permutes a FIXED deck ───────────────
    //
    // The first human playtest reported *"mulligans seem to change decks instead
    // of drawing a new hand"*, and it was literal: the session held
    // `DeckSource::RandomPerSeat`, a recipe in which every card of every seat —
    // commander included — is a function of `cfg.seed`, and `PlaySession::mulligan`
    // rebuilds from a **perturbed** seed. All four decklists and all four
    // commanders were re-rolled per mulligan, and CR 903.6 puts the commander in
    // the public command zone, so the playtester watched three opponents'
    // commanders change (`memory/playtest-triage-2026-08-02b.md` G2).
    //
    // Two probes, because the property has two halves and only one of them is
    // visible over HTTP:
    //
    // * P1 drives the REAL router — `POST /api/game` then two `POST
    //   /api/game/mulligan` — and reads the commanders back out of the seat view's
    //   own public payload. That is exactly the channel the defect was observed
    //   through.
    // * P2 goes through `session::new_game` directly, because the thing that must
    //   also hold — every seat's 100-card multiset — lives in hidden zones that no
    //   seat view will ever render (Architecture Invariant 7). It also pins the
    //   mechanism: the session must be *holding* resolved decklists.
    //
    // `crates/simulator/tests/setup.rs` pins the CR 103.5 property of `redeal`
    // itself. It cannot catch this defect and never could: `DeckSource::Fixed` was
    // always immune, so a simulator-level test passes whatever the play server
    // stores. The defect is in what the session *keeps*, so the gate has to be here.

    /// Every seat's command-zone card names, keyed by seat name, read from the
    /// seat view's own payload.
    ///
    /// By name, not by object id: `LocalGame::start` mints fresh `ObjectId`s on
    /// every rebuild (CR 400.7), so ids differ across a mulligan even when nothing
    /// is wrong. The card's identity is the assertion.
    fn sim4_commanders_by_seat(view: &Value) -> std::collections::BTreeMap<String, Vec<String>> {
        view["state"]["zones"]["command_zone"]
            .as_object()
            .expect("command_zone is an object")
            .iter()
            .map(|(seat, cards)| {
                let names = cards
                    .as_array()
                    .expect("a command zone is an array")
                    .iter()
                    .map(|c| {
                        c["name"]
                            .as_str()
                            .expect("a command-zone card has a name")
                            .to_string()
                    })
                    .collect();
                (seat.clone(), names)
            })
            .collect()
    }

    /// P1 — CR 103.5 / CR 903.6 over HTTP: two mulligans, and every seat's
    /// commander is the card it was before.
    ///
    /// **Proven to discriminate by executing the revert**: with
    /// `session::new_game` restored to passing `cfg` through unresolved, this test
    /// fails on the first mulligan with all four commanders replaced.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim4_mulligan_preserves_every_seats_commander() {
        let state = shared_state();
        let before = new_game(&state).await;

        let commanders_before = sim4_commanders_by_seat(&before);
        // Non-vacuity floor: an empty map compares equal to itself. CR 903.6 puts
        // exactly one commander in each of the four seats' command zones.
        assert_eq!(
            commanders_before.len(),
            PLAYERS as usize,
            "every seat's command zone must be visible — it is public (CR 903.6): {before}"
        );
        for (seat, names) in &commanders_before {
            assert_eq!(
                names.len(),
                1,
                "seat {seat} must show exactly one commander"
            );
        }
        let hand_before = before["state"]["zones"]["hand"][HUMAN].clone();
        assert!(
            hand_before.as_array().is_some_and(|h| h.len() == 7),
            "the human opens with 7 cards (CR 103.5): {hand_before}"
        );

        let mut latest = before;
        for n in 1..=2u64 {
            let (status, after) =
                post_json(&state, "/api/game/mulligan", json!({ "take": true })).await;
            assert_eq!(status, StatusCode::OK, "{after}");
            assert_eq!(after["summary"]["mulligan_count"], n);
            assert_eq!(
                sim4_commanders_by_seat(&after),
                commanders_before,
                "CR 103.5: mulligan {n} must permute a fixed deck, not re-roll the table — \
                 and CR 903.6 makes every one of these commanders public, so a change here \
                 is a change the other three players can see"
            );
            latest = after;
        }

        // Still a mulligan: same cards, new order, new hand.
        assert_ne!(
            latest["state"]["zones"]["hand"][HUMAN], hand_before,
            "the human's hand must actually redraw"
        );
        assert!(
            latest["state"]["zones"]["hand"][HUMAN]
                .as_array()
                .is_some_and(|h| h.len() == 7),
            "and it is still 7 cards"
        );
    }

    /// Every card seat `pid` owns, by name, sorted — the CR 103.5 multiset (hand ∪
    /// library ∪ command zone), read out of band from the engine's own state
    /// because two of those three zones are hidden from every seat view.
    fn sim4_deck_multiset(play: &session::PlaySession, pid: mtg_engine::PlayerId) -> Vec<String> {
        let gs = play.game.state();
        let mut names: Vec<String> = [
            mtg_engine::ZoneId::Hand(pid),
            mtg_engine::ZoneId::Library(pid),
            mtg_engine::ZoneId::Command(pid),
        ]
        .iter()
        .flat_map(|zone| {
            gs.objects_in_zone(zone)
                .iter()
                .map(|obj| obj.characteristics.name.clone())
                .collect::<Vec<_>>()
        })
        .collect();
        names.sort();
        names
    }

    /// P2 — the session resolves its decks once and keeps them, and a mulligan
    /// leaves every seat's 100-card multiset untouched (CR 103.5, CR 903.5a).
    ///
    /// Also pins the seam: `config_for` still hands `new_game` the *recipe*. The
    /// resolve happens in `new_game`, which is the one constructor both
    /// `POST /api/game` and every fixture goes through.
    #[test]
    fn test_sim4_session_resolves_decks_once_and_a_mulligan_preserves_them() {
        let defaults = NewGameDefaults {
            players: PLAYERS,
            bot: BotKind::Heuristic,
            seed: SEED,
        };
        let cfg = session::config_for(defaults).expect("the default table must be legal");
        assert!(
            matches!(cfg.decks, mtg_simulator::DeckSource::RandomPerSeat),
            "config_for describes a random table; resolving it is new_game's job"
        );

        let mut play = session::new_game(cfg, 0).expect("the default table must build");
        match &play.cfg.decks {
            mtg_simulator::DeckSource::Fixed(pairs) => assert_eq!(
                pairs.len(),
                PLAYERS as usize,
                "one resolved decklist per seat"
            ),
            other => panic!(
                "the session must hold resolved decklists so a redeal cannot re-roll them \
                 (CR 103.5), got {other:?}"
            ),
        }

        let seats: Vec<mtg_engine::PlayerId> =
            (1..=u64::from(PLAYERS)).map(mtg_engine::PlayerId).collect();
        let decks_before: Vec<Vec<String>> = seats
            .iter()
            .map(|&pid| sim4_deck_multiset(&play, pid))
            .collect();
        let commanders_before: Vec<Vec<mtg_engine::CardId>> = seats
            .iter()
            .map(|&pid| {
                play.game
                    .state()
                    .players()
                    .get(&pid)
                    .expect("seat exists")
                    .commander_ids
                    .iter()
                    .cloned()
                    .collect()
            })
            .collect();
        let hand_before: Vec<String> = {
            let gs = play.game.state();
            let mut names: Vec<String> = gs
                .objects_in_zone(&mtg_engine::ZoneId::Hand(session::HUMAN_SEAT))
                .iter()
                .map(|o| o.characteristics.name.clone())
                .collect();
            names.sort();
            names
        };

        play.mulligan()
            .expect("a pregame mulligan must be accepted");

        for (i, &pid) in seats.iter().enumerate() {
            // Non-vacuity floor: CR 903.5a — 99 main-deck cards plus the commander.
            assert_eq!(
                decks_before[i].len(),
                100,
                "seat {pid:?} must own exactly 100 cards, or this equality asserts nothing"
            );
            assert_eq!(
                decks_before[i],
                sim4_deck_multiset(&play, pid),
                "CR 103.5: seat {pid:?}'s card multiset must survive another seat's mulligan"
            );
            let after: Vec<mtg_engine::CardId> = play
                .game
                .state()
                .players()
                .get(&pid)
                .expect("seat exists")
                .commander_ids
                .iter()
                .cloned()
                .collect();
            assert_eq!(commanders_before[i].len(), 1);
            assert_eq!(
                commanders_before[i], after,
                "CR 903.6: seat {pid:?}'s registered commander must not change"
            );
        }

        let hand_after: Vec<String> = {
            let gs = play.game.state();
            let mut names: Vec<String> = gs
                .objects_in_zone(&mtg_engine::ZoneId::Hand(session::HUMAN_SEAT))
                .iter()
                .map(|o| o.characteristics.name.clone())
                .collect();
            names.sort();
            names
        };
        assert_eq!(hand_before.len(), 7, "CR 103.5 — seven before");
        assert_eq!(hand_after.len(), 7, "and seven after");
        assert_ne!(
            hand_before, hand_after,
            "the mulligan must still redraw the hand"
        );
    }

    // ── SIM-6 (CR 602.2, triage G4): the activation-cost payment channel ────────
    //
    // The `ActivateAbility` sibling of the UI-2 section above. Before this batch
    // `additional_costs_view` early-returned for anything that was not a
    // `CastSpell`, so `ActionOptionView.costs` was `null` for every
    // sacrifice-cost or discard-cost ability, the browser never entered its cost
    // stage, and `params.rs` submitted `sacrifice_target: None` -> the engine's
    // `InvalidCommand` -> 422.

    /// `{5}{B}{B}{B}`, Legendary Creature, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/razaketh_the_foulblooded.rs`). Mono-black, which
    /// fixes CR 903.5c colour identity to plain black so a Swamp deck is legal, and
    /// at 8 mana it is far outside this probe's window (a handful of Swamps) — the
    /// same two properties [`UI2_COMMANDER`] was chosen for on the green side.
    const SIM6_COMMANDER: &str = "razaketh-the-foulblooded";

    /// `{2}{B}`, Legendary Creature — Aetherborn Vampire 2/2, `Completeness::Complete`.
    /// "Sacrifice **another** creature: Yahenni gains indestructible until end of
    /// turn" — the exact card the first human playtest could not activate (G4), and
    /// the CR 109.1 half of this probe: its own id must NOT appear among the
    /// candidates the wire offers.
    const SIM6_YAHENNI: &str = "yahenni-undying-partisan";
    const SIM6_YAHENNI_NAME: &str = "Yahenni, Undying Partisan";

    /// `{B}`, Creature — Zombie 1/1, `Completeness::Complete`. "Sacrifice **a**
    /// creature: Put a +1/+1 counter on this creature" — printed WITHOUT "another",
    /// so this card is the control for Yahenni's exclusion: on the same board, its
    /// own offer must include itself. No mana ability (see [`UI2_ELF_A`]'s doc for
    /// why that matters to the auto-tap solver).
    const SIM6_FODDER: &str = "carrion-feeder";
    const SIM6_FODDER_NAME: &str = "Carrion Feeder";

    /// Generous bound for driving through ~4 of the human's own land drops and two
    /// casts. Same "one step per human decision" accounting as [`UI2_MAX_STEPS`].
    const SIM6_MAX_STEPS: usize = 800;

    /// CR 903.5c: `SIM6_COMMANDER` plus 99 Swamps, with `overrides` written into
    /// the given deck positions. Mirrors [`ui2_deck_with`] on the black side; the
    /// pinned opening-hand positions are the same because the shuffle is a function
    /// of the seed and the deck SIZE, not of which cards are in it.
    fn sim6_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("swamp".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(SIM6_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install a two-player fixed-deck session, exactly as [`ui2_install`] does.
    fn sim6_install(state: &SharedState, p1_deck: mtg_simulator::DeckConfig) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI2_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), p1_deck),
                // All-Swamp opponent: no spell in the deck at all, so the bot can
                // only play lands and pass and cannot perturb P1's board.
                (mtg_engine::PlayerId(2), sim6_deck_with(&[])),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the SIM-6 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive the human seat — playing a land, else casting whichever of the two
    /// fixture creatures is not yet out, else passing — until BOTH are on the
    /// battlefield and an `ActivateAbility` option for `SIM6_YAHENNI_NAME` is
    /// offered. Returns the view at that point.
    async fn sim6_drive_to_yahenni_activation(state: &SharedState, max_steps: usize) -> Value {
        let p1 = mtg_engine::PlayerId(1);
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            let both_out = [SIM6_FODDER_NAME, SIM6_YAHENNI_NAME]
                .iter()
                .all(|name| !ui2_battlefield_ids_by_name(state, p1, name).is_empty());
            if both_out {
                let yahenni = ui2_battlefield_ids_by_name(state, p1, SIM6_YAHENNI_NAME)[0];
                let offered = view["decision"]["actions"]
                    .as_array()
                    .is_some_and(|actions| {
                        actions
                            .iter()
                            .any(|a| a["kind"] == "ActivateAbility" && a["object_id"] == yahenni)
                    });
                if offered {
                    return view;
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before Yahenni's activation was offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let next_cast = [SIM6_FODDER_NAME, SIM6_YAHENNI_NAME]
                .iter()
                .find(|name| ui2_battlefield_ids_by_name(state, p1, name).is_empty())
                .map(|name| format!("Cast {name}"));
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    next_cast.as_deref().and_then(|label| {
                        actions
                            .iter()
                            .find(|a| a["kind"] == "CastSpell" && a["label"] == label)
                    })
                })
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
        panic!("Yahenni's activation was never offered within {max_steps} steps");
    }

    /// The `ActivateAbility` option whose `object_id` is `source`.
    fn sim6_activate_option(view: &Value, source: u64) -> Value {
        view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "ActivateAbility" && a["object_id"] == source)
            .unwrap_or_else(|| panic!("no ActivateAbility option for object {source}: {view}"))
            .clone()
    }

    /// SIM6-P1 (criterion 6066): the wire payload for Yahenni's activation carries a
    /// populated `costs.activation_sacrifice` whose candidate list is the OTHER
    /// creature and **not Yahenni itself** (CR 109.1) — while the SAME board's
    /// Carrion Feeder offer, printed "Sacrifice **a** creature", DOES include itself.
    ///
    /// Two cards on one board is what makes this discriminating: an implementation
    /// that always excluded the source, or never did, fails one half or the other.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim6_yahenni_offer_excludes_itself_while_carrion_feeder_includes_itself() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        sim6_install(
            &state,
            sim6_deck_with(&[(0, SIM6_YAHENNI), (1, SIM6_FODDER)]),
        );

        let view = sim6_drive_to_yahenni_activation(&state, SIM6_MAX_STEPS).await;
        let yahenni = ui2_battlefield_ids_by_name(&state, p1, SIM6_YAHENNI_NAME)[0];
        let fodder = ui2_battlefield_ids_by_name(&state, p1, SIM6_FODDER_NAME)[0];

        let option = sim6_activate_option(&view, yahenni);
        let costs = &option["costs"];
        assert!(
            !costs.is_null(),
            "G4: an activation with a sacrifice cost must carry a cost descriptor: {option}"
        );
        let sac = &costs["activation_sacrifice"];
        assert!(
            !sac.is_null(),
            "the sacrifice block must be present: {costs}"
        );
        assert!(
            costs["activation_discard"].is_null() && costs["sacrifice"].is_null(),
            "this ability has no discard cost and this is not a CastSpell: {costs}"
        );
        assert_eq!(sac["answer_field"], "cost_sacrifice_target");
        assert_eq!(sac["default"], fodder);
        let ids: Vec<u64> = sac["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert!(
            ids.contains(&fodder),
            "the other creature must be offered: {sac}"
        );
        assert!(
            !ids.contains(&yahenni),
            "CR 109.1: 'Sacrifice ANOTHER creature' must not offer Yahenni itself: {sac}"
        );
        assert!(
            sac["prompt"]
                .as_str()
                .expect("prompt is a string")
                .contains("another"),
            "the prompt must say 'another' so a human can tell an exclusion from a \
             missing card: {sac}"
        );

        // The control, on the SAME board: Carrion Feeder prints "Sacrifice a
        // creature" and may pay with itself.
        let feeder_option = sim6_activate_option(&view, fodder);
        let feeder_ids: Vec<u64> = feeder_option["costs"]["activation_sacrifice"]["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert!(
            feeder_ids.contains(&fodder) && feeder_ids.contains(&yahenni),
            "'Sacrifice a creature' offers every creature INCLUDING the source: {feeder_ids:?}"
        );
    }

    /// SIM6-P2 (criterion 6066): the human's chosen sacrifice is submitted over
    /// HTTP, ACCEPTED (200, not the 422 this batch exists to remove), and the
    /// sacrifice really happened — read back out of band by NAME, since CR 400.7
    /// mints a fresh `ObjectId` on the zone change.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim6_activation_sacrifice_is_answered_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        sim6_install(
            &state,
            sim6_deck_with(&[(0, SIM6_YAHENNI), (1, SIM6_FODDER)]),
        );

        let view = sim6_drive_to_yahenni_activation(&state, SIM6_MAX_STEPS).await;
        let yahenni = ui2_battlefield_ids_by_name(&state, p1, SIM6_YAHENNI_NAME)[0];
        let fodder = ui2_battlefield_ids_by_name(&state, p1, SIM6_FODDER_NAME)[0];
        let option = sim6_activate_option(&view, yahenni);

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": {"cost_sacrifice_target": fodder},
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the activation with its cost paid must be accepted: {after}"
        );

        assert!(
            ui2_battlefield_ids_by_name(&state, p1, SIM6_FODDER_NAME).is_empty(),
            "the sacrificed creature must have left the battlefield"
        );
        assert!(
            ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1))
                .contains(&SIM6_FODDER_NAME.to_string()),
            "the sacrificed creature must be in its owner's graveyard (CR 602.2)"
        );
        assert!(
            !ui2_battlefield_ids_by_name(&state, p1, SIM6_YAHENNI_NAME).is_empty(),
            "Yahenni itself must still be on the battlefield"
        );
    }

    /// SIM6-P3: submitting Yahenni's OWN id as the sacrifice is refused **400
    /// `bad_params`** at the response boundary — it is not among the ids this
    /// decision offered — rather than reaching the engine and coming back 422.
    ///
    /// The same boundary argument `validate_additional_cost_params` makes for the
    /// cast side, at the id a human is most likely to try.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim6_sacrificing_the_source_of_an_another_cost_is_refused_400() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        sim6_install(
            &state,
            sim6_deck_with(&[(0, SIM6_YAHENNI), (1, SIM6_FODDER)]),
        );

        let view = sim6_drive_to_yahenni_activation(&state, SIM6_MAX_STEPS).await;
        let yahenni = ui2_battlefield_ids_by_name(&state, p1, SIM6_YAHENNI_NAME)[0];
        let option = sim6_activate_option(&view, yahenni);

        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": {"cost_sacrifice_target": yahenni},
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(body["kind"], "bad_params", "{body}");
        assert!(
            !ui2_battlefield_ids_by_name(&state, p1, SIM6_YAHENNI_NAME).is_empty(),
            "the refused submission must not have changed the board"
        );
    }

    /// A `LegalAction::ActivateAbility` carrying a fully-populated
    /// `ActivationCostPlan`, for unit-testing `api::validate_additional_cost_params`
    /// without an HTTP drive — the [`ui2_cast_spell_action_with_costs`] of this
    /// section.
    fn sim6_activate_action_with_costs(
        sacrifice: Option<Vec<mtg_engine::ObjectId>>,
        discard: Option<Vec<mtg_engine::ObjectId>>,
    ) -> mtg_simulator::LegalAction {
        use mtg_simulator::legal_actions::{
            ActivationCostPlan, ActivationDiscardOption, ActivationSacrificeOption,
        };
        mtg_simulator::LegalAction::ActivateAbility {
            source: mtg_engine::ObjectId(1),
            ability_index: 0,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            activation_costs: ActivationCostPlan {
                sacrifice: sacrifice.map(|eligible| ActivationSacrificeOption {
                    filter: mtg_engine::state::game_object::SacrificeFilter::Creature,
                    exclude_self: true,
                    default: eligible[0],
                    eligible,
                }),
                discard: discard.map(|eligible| ActivationDiscardOption {
                    default: eligible[0],
                    eligible,
                }),
            },
        }
    }

    /// SIM6-P4: an out-of-set activation sacrifice id is refused 400 (CR 602.2).
    #[test]
    fn test_sim6_validate_rejects_an_out_of_set_activation_sacrifice_id() {
        let eligible = mtg_engine::ObjectId(10);
        let action = sim6_activate_action_with_costs(Some(vec![eligible]), None);
        let params = crate::view::ActionParamsDto {
            cost_sacrifice_target: Some(mtg_engine::ObjectId(999)),
            ..Default::default()
        };
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("an id outside `eligible` must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");
    }

    /// SIM6-P5: an out-of-set activation DISCARD id is refused 400 (CR 111.10g),
    /// and an in-set one is accepted — so this discriminates the check rather than
    /// observing a blanket refusal.
    #[test]
    fn test_sim6_validate_checks_the_activation_discard_set_both_ways() {
        let in_hand = mtg_engine::ObjectId(20);
        let action = sim6_activate_action_with_costs(None, Some(vec![in_hand]));
        let err = api::validate_additional_cost_params(
            &action,
            &crate::view::ActionParamsDto {
                cost_discard_card: Some(mtg_engine::ObjectId(999)),
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("a card outside the offered hand must be refused");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");

        api::validate_additional_cost_params(
            &action,
            &crate::view::ActionParamsDto {
                cost_discard_card: Some(in_hand),
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect("the offered card must be accepted");
    }

    /// SIM6-P6: a cost answer on an action that offers no such cost is refused 400
    /// — both the "this ability has no discard cost" case and the "this decision is
    /// not an ActivateAbility at all" case, which `params.rs`'s own per-variant
    /// guard cannot catch for `CastSpell` (a consuming arm).
    #[test]
    fn test_sim6_validate_rejects_cost_answers_the_decision_never_offered() {
        let eligible = mtg_engine::ObjectId(10);
        let sacrifice_only = sim6_activate_action_with_costs(Some(vec![eligible]), None);
        let err = api::validate_additional_cost_params(
            &sacrifice_only,
            &crate::view::ActionParamsDto {
                cost_discard_card: Some(eligible),
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("this ability has no discard cost");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");

        let a_cast = ui2_cast_spell_action_with_costs(vec![eligible], eligible, 0);
        let err = api::validate_additional_cost_params(
            &a_cast,
            &crate::view::ActionParamsDto {
                cost_sacrifice_target: Some(eligible),
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("an activation answer on a CastSpell decision is a 400");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");

        // The mirror image (`/review` finding 2): a CR 118.8 `additional_costs`
        // array on an ACTIVATION decision. `params.rs`'s `ActivateAbility` arm never
        // reads that field and `ActivateAbility` is inside its consuming allowlist,
        // so without this guard the array is dropped in silence.
        let err = api::validate_additional_cost_params(
            &sacrifice_only,
            &crate::view::ActionParamsDto {
                additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                    ids: vec![eligible],
                    lki: vec![],
                }],
                ..Default::default()
            },
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err("a spell's additional-cost array on an activation decision is a 400");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.body.kind, "bad_params");

        // And the control: an activation decision with NO cost answers at all is
        // accepted, so the guard above cannot pass by refusing everything.
        api::validate_additional_cost_params(
            &sacrifice_only,
            &crate::view::ActionParamsDto::default(),
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect("an activation with no announced cost answer is accepted");
    }

    /// `{4}{R}{R}`, Legendary Creature, `Completeness::Complete`
    /// (`crates/card-defs/src/defs/lathliss_dragon_queen.rs`). Mono-red, which fixes
    /// CR 903.5c colour identity to plain red so a Mountain deck is legal — the same
    /// role [`SIM6_COMMANDER`] plays on the black side. Never cast by
    /// [`sim6_drive_to_rummaging_goblin_activation`], which only ever plays a land,
    /// casts the one fixture creature, or passes.
    const SIM6_RED_COMMANDER: &str = "lathliss-dragon-queen";

    /// `{2}{R}`, Creature — Goblin Rogue 1/1, `Completeness::Complete`.
    /// "{T}, Discard a card: Draw a card" — the DISCARD half of this batch's channel,
    /// and deliberately an ability with **no mana component**: an activation that also
    /// costs mana is refused `InsufficientMana` on this surface for an unrelated
    /// reason (`OOS-SIM6-3` — auto-tap covers `CastSpell` and nothing else), which
    /// would make a probe of the discard channel fail for the wrong cause.
    ///
    /// This is the card whose live browser activation surfaced the missing CR 302.6
    /// gate (see `legal_actions::activated_ability_is_activatable`).
    const SIM6_DISCARDER: &str = "rummaging-goblin";
    const SIM6_DISCARDER_NAME: &str = "Rummaging Goblin";

    /// [`sim6_deck_with`] on the red side: `SIM6_RED_COMMANDER` plus 99 Mountains.
    fn sim6_red_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("mountain".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(SIM6_RED_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install the red fixture, both seats. Mirrors [`sim6_install`].
    fn sim6_install_red(state: &SharedState, p1_deck: mtg_simulator::DeckConfig) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: UI2_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), p1_deck),
                (mtg_engine::PlayerId(2), sim6_red_deck_with(&[])),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the SIM-6 red fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive the human seat — playing a land, else casting the one fixture creature,
    /// else passing — until an `ActivateAbility` option for that creature is offered.
    ///
    /// The commander is never cast: this loop has no branch that would.
    ///
    /// **The offer does not appear on the turn the creature lands**, and that is the
    /// point: `activated_ability_is_activatable` withholds a `{T}` ability from a
    /// summoning-sick creature (CR 302.6), so this loop keeps passing until the
    /// following turn. Before that gate the offer appeared immediately and the
    /// activation came back 422.
    async fn sim6_drive_to_rummaging_goblin_activation(
        state: &SharedState,
        max_steps: usize,
    ) -> Value {
        let p1 = mtg_engine::PlayerId(1);
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if !ui2_battlefield_ids_by_name(state, p1, SIM6_DISCARDER_NAME).is_empty() {
                let goblin = ui2_battlefield_ids_by_name(state, p1, SIM6_DISCARDER_NAME)[0];
                let offered = view["decision"]["actions"]
                    .as_array()
                    .is_some_and(|actions| {
                        actions
                            .iter()
                            .any(|a| a["kind"] == "ActivateAbility" && a["object_id"] == goblin)
                    });
                if offered {
                    return view;
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before the activation was offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let cast_label = format!("Cast {SIM6_DISCARDER_NAME}");
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label)
                })
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
        panic!("the Rummaging Goblin activation was never offered within {max_steps} steps");
    }

    /// SIM6-P7 (criterion 6068, `/review` finding 4): the DISCARD half of the channel
    /// over HTTP, which the unit tests and the `params.rs` engine round-trip together
    /// still left unexercised on the wire — `activation_costs_view`'s discard block
    /// (its prompt and its `answer_field`) had no automated coverage at all.
    ///
    /// A NON-DEFAULT card is chosen, so the resulting game state distinguishes the
    /// human's answer from the offer's own default.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_sim6_activation_discard_is_answered_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        sim6_install_red(&state, sim6_red_deck_with(&[(0, SIM6_DISCARDER)]));

        let view = sim6_drive_to_rummaging_goblin_activation(&state, SIM6_MAX_STEPS).await;
        let goblin = ui2_battlefield_ids_by_name(&state, p1, SIM6_DISCARDER_NAME)[0];
        let option = sim6_activate_option(&view, goblin);

        let discard = &option["costs"]["activation_discard"];
        assert!(
            !discard.is_null(),
            "the discard block must be present on the wire: {option}"
        );
        assert_eq!(discard["answer_field"], "cost_discard_card");
        assert!(
            discard["prompt"]
                .as_str()
                .expect("prompt is a string")
                .contains("Discard"),
            "{discard}"
        );
        assert!(
            option["costs"]["activation_sacrifice"].is_null(),
            "this ability has no sacrifice cost: {option}"
        );
        let candidates: Vec<u64> = discard["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert!(
            candidates.len() >= 2,
            "the fixture needs at least two hand cards for a non-default choice: {discard}"
        );
        let default = discard["default"].as_u64().expect("default is a number");
        assert_eq!(default, candidates[0], "the default is the first candidate");
        let chosen = candidates[1];

        // Read the chosen card's NAME before the submission: CR 400.7 mints a fresh
        // `ObjectId` on the zone change, so the graveyard check below has to be by
        // name, exactly as `ui2_zone_names`' own doc says.
        let chosen_name = {
            let guard = state.session.lock().expect("lock");
            let session = guard.as_ref().expect("a session is installed");
            session
                .game
                .state()
                .objects()
                .get(&mtg_engine::ObjectId(chosen))
                .expect("the chosen card exists")
                .characteristics
                .name
                .clone()
        };

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": {"cost_discard_card": chosen},
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the activation with its discard paid must be accepted: {after}"
        );
        assert!(
            ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1)).contains(&chosen_name),
            "the chosen card must be in its owner's graveyard (CR 602.2 / CR 111.10g)"
        );

        // The negative half, on the same live offer: a card this decision never
        // offered is refused 400 at the boundary rather than reaching the engine.
        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&after),
                "action_index": 0,
                "params": {"cost_discard_card": 999_999},
            }),
        )
        .await;
        assert_ne!(
            status,
            StatusCode::OK,
            "an unofferable discard id must not be accepted: {body}"
        );
    }

    // ── ENG-1: an effect-driven discard is a real player choice (task scutemob-191) ──
    //
    // `memory/primitives/pb-plan-ENG1.md` §8 rows (i)/(j). `Effect::DiscardCards` used
    // to call `discard_cards` straight through -- the lowest `ObjectId`(s) in the
    // affected player's hand, never asking (CR 701.9b). The engine half now suspends
    // into `EffectChoiceQuestion::Discard`; these two probes close the browser-client
    // half: the wire shape really is `PickN` with real hand-card names, and a foreign
    // seat's discard question is invisible on that seat's own payload.
    //
    // **Fixture choice, and a finding it surfaced.** The obvious fixture --
    // Faithless Looting ("draw two, then discard two") -- reddens this probe for a
    // REAL reason, not a test bug: `resolve_top_of_stack`'s CR 608.2d suspend wraps
    // the WHOLE resolution in a roll-back (`*state = restart_point`), so the two
    // freshly-drawn cards the recorded `EffectChoiceQuestion::Discard.hand` names do
    // not exist in the rolled-back state the redacted view is built from, and
    // `NameIndex` correctly has no entry for them -- `(unknown card)`, not a bug in
    // the label lookup. That is a genuine, corpus-wide gap (every "draw N, discard
    // N" card in the roster: Frantic Search, Pull from Tomorrow, Chart a Course, and
    // Faithless Looting itself) filed as `OOS-ENG1-9`, out of scope for this probe to
    // fix. Fell Specter's ETB ("target opponent discards a card") has no preceding
    // same-resolution draw, so its candidate hand is entirely PRE-EXISTING objects
    // and the premise holds -- used here instead.

    /// CR 701.9b (ENG-1): Fell Specter's ETB targets an opponent for a single
    /// discard. In a 2-player game the only legal opponent is the human, so
    /// whichever seat casts it forces the question onto the OTHER seat with no
    /// target choice to engineer around -- and this fixture's passive human never
    /// casts anything, so it is always the BOT's copy that resolves. `{3}{B}`
    /// Creature — Specter 1/3, `Complete`.
    const ENG1_FELL_SPECTER: &str = "fell-specter";

    /// **Read off a real run, not reasoned to** (the [`UI1_SEED`] convention):
    /// [`ENG1_SEED`] is the smallest of several thousand candidates, found by a
    /// throwaway brute-force sweep, for which [`ENG1_FELL_SPECTER`] at
    /// `main_deck[0]` of the BOT's own deck lands in the BOT's OPENING hand.
    /// Deliberately NOT [`UI1_SEED`]: a first draft reused it with the identical
    /// two-seat deck [`ui1_deck`] builds, on the theory that "the shuffle is a
    /// permutation of positions" (that helper's own doc) would put the override at
    /// the same post-shuffle slot for both seats. It does not -- each seat's
    /// shuffle draws from a SHARED RNG stream in seat order (the same mechanism
    /// SIM-4's `OOS-SIM4-*` notes for mulligan re-dealing), so the two seats'
    /// permutations differ even from an identical pre-shuffle deck. At
    /// [`UI1_SEED`] the bot's opening hand was seven Swamps and Fell Specter
    /// stayed unseen for 28 turns, long enough for the bot's OWN 7-mana commander
    /// (unreachable "inside the probe's window" only because that window used to
    /// be short) to kill the passive human by commander damage first.
    const ENG1_SEED: u64 = 7;

    /// [`ENG1_FELL_SPECTER`] at `main_deck[0]`, 98 Swamps, `UI1_COMMANDER` (mono-black,
    /// unreachable inside the drive window). The human's deck is plain Swamps --
    /// this fixture's human never casts anything, so nothing else it could hold
    /// matters.
    fn eng1_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("swamp".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(UI1_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install the ENG-1 fixture: seat 1 (human) a plain-Swamp deck with nothing
    /// to do but pass; seat 2 (bot) the same shape plus [`ENG1_FELL_SPECTER`] at
    /// `main_deck[0]`, at [`ENG1_SEED`] so it is in the bot's OPENING hand.
    fn eng1_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: ENG1_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), eng1_deck_with(&[])),
                (
                    mtg_engine::PlayerId(2),
                    eng1_deck_with(&[(0, ENG1_FELL_SPECTER)]),
                ),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the ENG-1 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive a PASSIVE human seat -- pass every time, nothing else -- until an
    /// offered action carries the `want` question. The human never acts in this
    /// fixture (in particular, never casts anything), so any `Discard` question
    /// raised is always the BOT's copy of [`ENG1_FELL_SPECTER`] targeting the
    /// human.
    async fn eng1_drive_pass_only(state: &SharedState, want: &str, max_steps: usize) -> Value {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if ui1_question_index(&view, want).is_some() {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} without ever asking a {want} question: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"].as_array().unwrap().clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .or_else(|| actions.iter().find(|a| a["kind"] != "Concede"))
                .unwrap_or_else(|| panic!("only Concede was offered at step {step}: {view}"));
            let index = pick["index"].as_u64().unwrap();
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": index, "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "step {step}: {next}");
            view = next;
        }
        panic!("no {want} question within {max_steps} steps");
    }

    /// **CR 701.9b (ENG-1) — an effect-driven discard, driven to a non-default
    /// pick over HTTP.**
    ///
    /// Three things at once, matching the other UI-1-shaped probes: the reproduction
    /// (the baked-in default is the count LOWEST `ObjectId`s -- `default_discard_answer`
    /// mirrors the pre-ENG-1 `min_by_key` auto-pick byte-for-byte), the §4 hidden-info
    /// premise check (a seat's own hand candidates must carry real names, never the
    /// unknown-card placeholder -- if this regresses, "the decision belongs to the
    /// viewer, so its hand ids are already in the viewer's redacted view" is false and
    /// the `PickN` arm's premise does not hold), and a real, non-default pick.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_eng1_the_browser_renders_a_pickn_discard() {
        let state = shared_state();
        eng1_install(&state);

        let view = eng1_drive_pass_only(&state, "Discard", 1200).await;
        let index = ui1_question_index(&view, "Discard").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let decision = &option["decision"];

        assert_eq!(decision["answer_field"], "effect_choice_answer");
        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "PickN");
        assert_eq!(answer["chosen_key"], "chosen");

        let count = answer["count"].as_u64().expect("count is a number") as usize;
        assert_eq!(count, 1, "Fell Specter discards exactly 1: {answer}");

        let candidates: Vec<u64> = answer["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert!(
            candidates.len() > count,
            "the candidate set is the WHOLE hand, not just the cards to discard: {answer}"
        );

        // **The §4 hidden-info premise, checked.** These are the answerer's OWN
        // hand cards, and Fell Specter's discard reaches no cards drawn earlier in
        // the same resolution, so EVERY candidate must render its real name here --
        // never `UNKNOWN_LABEL`/`HIDDEN_LABEL`, and never the `OOS-ENG1-9`
        // same-resolution-draw placeholder either. Asserting against that exact
        // prefix (not just the two pre-existing constants) is deliberate: a
        // placeholder-shaped label would silently satisfy `!= UNKNOWN_LABEL`, so a
        // genuine label regression that started emitting it here would ship green
        // (review Finding 2). If the labels come back as ANY placeholder, the guard
        // this arm rests on is not what the plan believes it is.
        for card in answer["candidates"].as_array().unwrap() {
            let label = card["label"].as_str().expect("label is a string");
            assert!(
                label != view::UNKNOWN_LABEL
                    && label != view::HIDDEN_LABEL
                    && !label.starts_with("(card drawn this resolution #"),
                "a seat's own hand card must render its real name (CR 402.1), not a \
                 placeholder: {label:?}"
            );
        }

        // Reproduction: the default is the count LOWEST ObjectIds (§6:
        // `default_discard_answer` reproduces the pre-ENG-1 `min_by_key` pick).
        let mut ascending = candidates.clone();
        ascending.sort_unstable();
        let default = ascending[..count].to_vec();
        assert_eq!(
            answer["template"],
            json!({"Discard": {"chosen": default}}),
            "default_discard_answer takes the count LOWEST ObjectIds"
        );

        let wire_seq = seq(&view);

        // Now a real, non-default pick: the count HIGHEST ids instead -- the
        // opposite end of the hand from the default.
        let chosen: Vec<u64> = ascending[ascending.len() - count..].to_vec();
        assert!(
            chosen.iter().all(|id| !default.contains(id)),
            "the chosen subset must be disjoint from the default, or this proves \
             nothing: chosen={chosen:?} default={default:?}"
        );
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"effect_choice_answer": {"Discard": {"chosen": chosen}}},
            }),
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

    /// **The new-channel gate (Architecture Invariant 7).** The hand-zone analogue
    /// of `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`.
    ///
    /// The shipped `GameSummary.seed` HIGH is precisely what happens when a
    /// redaction gate checks the channel it was written for and a new channel is
    /// invisible to it: one gate scans for omniscient view-model entry points, the
    /// other scans the HTTP body for another seat's hand card names, and a hand's
    /// own `ObjectId`s carried inside a `Discard` question are neither. So this is
    /// pinned directly: drive a discard block for seat 1, move the viewer to seat
    /// 2 while it is still outstanding, and assert seat 2's payload carries neither
    /// the decision nor the `candidates` key at all.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_eng1_a_foreign_seats_discard_question_never_reaches_this_payload() {
        let state = shared_state();
        eng1_install(&state);

        let view = eng1_drive_pass_only(&state, "Discard", 1200).await;
        let index = ui1_question_index(&view, "Discard").expect("just found");

        // Non-vacuity: the entitlement really is in use before it is withheld.
        let candidates = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index")["decision"]["answer"]["candidates"]
            .as_array()
            .expect("candidates")
            .clone();
        assert!(!candidates.is_empty());

        // Captured BEFORE the move, for the same reason UI-1's sibling test
        // captures it early: a real seat-2 client could never obtain it (the write
        // guard sits above the `seq` check), so reading it here is the STRONGEST
        // case for the guard, not a representative one.
        let wire_seq_before = seq(&view);

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
            "seat 1's discard question must not appear in seat 2's payload: {}",
            refetched["decision"]
        );

        // Asserted over the RAW body, not over parsed fields (the MR-M11-01 idiom):
        // a future field that carried the hand under another name would be caught
        // by this and not by a field-by-field check. `"candidates"` is the
        // new-channel needle -- it is the hand-question analogue of the scry
        // probe's `"looked_at"` check, and it is safe here for the same reason:
        // with `decision` entirely absent, nothing else on this payload renders a
        // `candidates` array (every one of them lives inside a decision view).
        assert!(
            !body.contains("\"candidates\""),
            "the foreign seat's hand entitlement leaked into the body: {body}"
        );

        // review Finding 7 (LOW): gate (j) needled only the `"candidates"` KEY --
        // strengthen it so a rename of that key (while the payload still carried
        // this seat's hand content under another name) would still be caught.
        // Anchored on ONE SPECIFIC candidate's (id, label) pair rather than a bare
        // card name: this fixture's seat-1 hand is uniformly "Swamp"
        // (`eng1_deck_with`), and seat 2 legitimately holds Swamps of its own, so
        // "no card named Swamp appears" would be exactly the overstatement the
        // sibling `"looked_at"` gate above already refused to make. The `id` is a
        // globally unique `ObjectId`, so the PAIR cannot legitimately appear
        // anywhere in seat 2's payload for any reason other than this leak, and
        // the check is independent of whatever key wraps it.
        let leaked_candidate = &candidates[0];
        let leaked_id = leaked_candidate["id"]
            .as_u64()
            .expect("candidate id is a number");
        let leaked_label = leaked_candidate["label"]
            .as_str()
            .expect("candidate label is a string");
        let leak_needle = format!("\"id\":{leaked_id},\"label\":\"{leaked_label}\"");
        assert!(
            !body.contains(&leak_needle),
            "one specific candidate from seat 1's hand-discard question leaked \
             into seat 2's payload verbatim: needle={leak_needle:?} body={body}"
        );

        // The write half: hiding the decision must not let this seat answer it.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq_before, "action_index": index, "params": {}}),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "answering another seat's discard question must be refused, not applied: {refused}"
        );
        assert_eq!(refused["kind"], "no_pending_decision");
    }

    // ── ENG-2 ────────────────────────────────────────────────────────────────

    /// **ENG-2 §7(h): the UI-4/SIM-6 lesson.** A wire proven below the browser is
    /// not proven at the browser -- this batch's deliverable is a line a human
    /// reads, so this test drives a real targeted cast through the HTTP API and
    /// asserts a `TargetsAnnounced` line reaches the seat payload's `events`
    /// array, i.e. that the browser really receives
    /// `{"kind":"TargetsAnnounced","tier":"stack","text":"... targets ..."}`.
    ///
    /// Test-only: `tools/play-server/src/` carries zero SOURCE changes for this
    /// batch (confirmed by `Grep "GameEvent::"` over `tools/play-server/src`
    /// returning only doc comments) -- this probe does not contradict that; it
    /// exercises the existing `event_view_for` -> `EventView` -> JSON pipeline
    /// with the new variant flowing through it unmodified.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_eng2_targets_announced_reaches_the_browser_over_http() {
        let state = shared_state();
        let view = drive_until(&state, TARGET_SEED, false, |v| {
            option_with_targets(v, 1).is_some()
        })
        .await;
        let option = option_with_targets(&view, 1).expect("the driver stopped on one");
        let target = option["target_slots"][0]["candidates"][0]["value"].clone();

        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": option["index"],
                "params": { "targets": [target] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the targeted cast was refused: {after}"
        );

        let events = after["events"].as_array().expect("events is an array");
        let announced = events
            .iter()
            .find(|e| e["kind"] == "TargetsAnnounced")
            .unwrap_or_else(|| {
                panic!("no TargetsAnnounced event reached the client: events were {events:?}")
            });
        assert_eq!(announced["tier"], "stack");
        let text = announced["text"].as_str().expect("text is a string");
        assert!(
            text.contains("targets"),
            "the rendered line must say who targets what: {text:?}"
        );
        assert!(
            !text.is_empty() && text != "TargetsAnnounced",
            "the line must be the rendered prose arm, not the kind-only redaction \
             floor: {text:?}"
        );
    }

    // ── PB-DX20 T6 ───────────────────────────────────────────────────────────

    /// Mono-green, `{2}{G}{G}`, `Complete` -- fixes color identity (CR 903.5c) for
    /// a deck holding Rancor + Llanowar Elves. Cheap enough that the drive loop
    /// below never needs to worry about it: neither seat casts its own
    /// commander in this fixture (the driver only ever offers-and-picks
    /// PlayLand / "Cast Llanowar Elves" / "Cast Rancor" / PassPriority, in that
    /// order, so a `CastSpell` for the commander is simply never selected).
    const DX20_T6_COMMANDER: &str = "dwynen-gilt-leaf-daen";

    /// 97 Forests + Llanowar Elves + Rancor for the human seat; 99 Forests for
    /// the bot seat. Swept (throwaway scratch test, run then deleted, never
    /// committed -- mirrors the `[UI6_RESTRICTED_SEED]`/`[ENG1_SEED]`
    /// precedent) directly against `setup::build_initial_state`'s dealt hand
    /// and library, not against a played-out game: at this seed Llanowar
    /// Elves is in the OPENING hand and Rancor is drawn within the next two
    /// draws, so the drive below reaches its target well before the bot's own
    /// commander (also mono-green, also castable) can attack the human down.
    const DX20_T6_SEED: u64 = 411;

    fn dx20_t6_human_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![
            CardId("llanowar-elves".to_string()),
            CardId("rancor".to_string()),
        ];
        while main_deck.len() < 99 {
            main_deck.push(CardId("forest".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX20_T6_COMMANDER.to_string()),
            main_deck,
        }
    }

    fn dx20_t6_bot_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        mtg_simulator::DeckConfig {
            commander: CardId(DX20_T6_COMMANDER.to_string()),
            main_deck: (0..99).map(|_| CardId("forest".to_string())).collect(),
        }
    }

    /// Install the T6 fixture through `session::new_game` -- the same
    /// constructor the real handler uses, running the same two Invariant-9
    /// gates (`validate_deck`, `check_all_defs_complete`). See [`ui1_install`]'s
    /// doc for why `POST /api/game` cannot express a `DeckSource::Fixed` game.
    fn dx20_t6_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX20_T6_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx20_t6_human_deck()),
                (mtg_engine::PlayerId(2), dx20_t6_bot_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX20 T6 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive the human seat: play a land every chance, cast Llanowar Elves the
    /// moment it is offered (getting an own creature onto the battlefield),
    /// otherwise pass -- UNTIL "Cast Rancor" is offered AT ALL. Returns that view
    /// and the Rancor `CastSpell` option itself, submitting NOTHING for that option
    /// (the caller does the actual targeted-cast assertion).
    ///
    /// E5 (pb-review-DX20.md): the search predicate here is deliberately WEAKER than
    /// it once was -- it used to also require a non-empty `target_slots[0].candidates`
    /// before returning, which meant a reverted synthesis (where `target_min` stays 0
    /// and no candidates are ever populated) made the loop run to the game's end and
    /// panic HERE, in the drive loop, rather than at the `target_min == 1` /
    /// candidate-content assertions the committed test is actually named for and
    /// advertises. Confirmed by executing the T2-class revert against the COMMITTED
    /// test (not a scratch stand-in): it failed exactly this way, at this assertion,
    /// with the message "the game ended ... without Rancor ever being offered" --
    /// which is what motivated this split. Finding Rancor by LABEL ALONE here means
    /// the discriminating checks now live where the test advertises them.
    async fn dx20_t6_drive_to_rancor(state: &SharedState, max_steps: usize) -> (Value, Value) {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if !view["decision"].is_null() {
                if let Some(rancor) = view["decision"]["actions"].as_array().and_then(|actions| {
                    actions
                        .iter()
                        .find(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Rancor")
                }) {
                    return (view.clone(), rancor.clone());
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} without Rancor ever being offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Llanowar Elves")
                })
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
        panic!("Rancor was never offered within {max_steps} steps");
    }

    /// **CR 303.4a (Aura half, acceptance criterion 1 second half) -- Rancor
    /// castable end to end over the REAL HTTP API.** Before PB-DX20 this was a
    /// 422: `action_option_view` read `target_min: 0` for every Aura (no
    /// picker was ever rendered), and `POST /api/game/action` with a target
    /// anyway was refused by `casting.rs`'s independent CR 303.4a gate.
    ///
    /// This is a REAL HTTP round trip through `app(state.clone()).oneshot(...)`
    /// (the same router `main()` serves), not a `view::decision_view` call --
    /// stated per the dispatch brief's instruction to say which was done.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx20_t6_rancor_castable_with_a_real_target_over_http() {
        let state = shared_state();
        dx20_t6_install(&state);

        let (view, rancor) = dx20_t6_drive_to_rancor(&state, 4_000).await;

        // E5 (pb-review-DX20.md): the drive loop above now finds "Cast Rancor" by
        // LABEL ALONE (it no longer also requires a non-empty candidates array before
        // returning -- that requirement moved HERE, into the assertions the test is
        // actually named for), so a reverted synthesis reddens on THIS assertion
        // (target_min stays 0) instead of panicking inside the drive loop with a
        // misleading "game ended without Rancor ever being offered" message. Executed
        // against the committed test: verbatim record in `scratchpad/dx20-reverts.md`.
        assert_eq!(
            rancor["target_min"], 1,
            "Rancor's offered CastSpell action should carry target_min == 1 (today: 0), \
             got {:?}",
            rancor["target_min"]
        );
        // E8 (pb-review-DX20.md): with the drive-loop split above, this emptiness
        // check is NO LONGER tautological -- the drive loop no longer guarantees a
        // non-empty candidates array, so this is now a genuine assertion (kept, not
        // dropped, unlike the original E8 finding's context) that fails cleanly
        // instead of panicking on an out-of-bounds `candidates[0]` index.
        let candidates = rancor["target_slots"][0]["candidates"]
            .as_array()
            .expect("candidates is an array");
        assert!(
            !candidates.is_empty(),
            "Rancor's target_slots[0].candidates should contain the creature on the \
             battlefield, got {:?}",
            rancor["target_slots"]
        );
        let target_label = candidates[0]["label"].as_str().unwrap_or_default();
        assert!(
            target_label.contains("Llanowar Elves"),
            "the candidate should be the real (redacted) Llanowar Elves label, got {:?}",
            candidates[0]
        );
        let target = candidates[0]["value"].clone();

        let before_commands = command_count(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": rancor["index"],
                "params": { "targets": [target] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "POST /api/game/action with Rancor's declared target should return 200 \
             (today: 422): {after}"
        );
        assert!(
            command_count(&after) > before_commands,
            "the command count should have advanced past {before_commands}, got {:?}",
            command_count(&after)
        );
    }

    // ── PB-DX20b -- CR 702.5a's printed Enchant line, over the REAL HTTP API ──
    //
    // `OOS-DX20-10` (HIGH). `imprisoned_in_the_moon` prints "Enchant creature, land,
    // or planeswalker" and declared `EnchantTarget::Permanent` until PB-DX20b, which
    // also admitted artifacts and enchantments -- and PB-DX20 is what made that
    // widened offer CLICKABLE in a browser. So the wire-shaped exclusion is the whole
    // point of this probe: the assertion that matters is that Sol Ring, a real
    // `Complete` deck-legal artifact sitting on the battlefield at the moment of the
    // ask, is NOT in `target_slots[0].candidates` while the Islands are.
    //
    // The simulator-side half (the offer SET, the accept-every-offer sweep, the
    // attach-by-resolution-effect drive and the bot path) is
    // `crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs`. This is the HTTP
    // half: a real `POST /api/game/action` through `app(state.clone()).oneshot(..)`,
    // answered with a NON-DEFAULT target (the UI-4/SIM-6 standard).

    /// {3}{U}{U}{U}, mono-blue, `Complete` -- fixes CR 903.5c colour identity for a
    /// deck holding Imprisoned in the Moon, and at mana value 6 it is never
    /// affordable inside this fixture's two-turn window, so it cannot perturb the
    /// drive (the `T5_DX23_COMMANDER` / `DX20_T6_COMMANDER` rationale).
    /// `sol-ring` is colourless and so constrains identity not at all.
    const DX20B_COMMANDER: &str = "arcanis-the-omnipotent";

    /// **Read off an executed sweep, not reasoned to** (the `UI1_SEED` /
    /// `DX20_T6_SEED` precedent). A throwaway scratch test -- written, run, deleted,
    /// never committed -- swept seeds 1..=800 directly against
    /// `setup::build_initial_state`'s dealt hand and library for this exact
    /// `DeckSource::Fixed` pair, looking for a seat-1 opening where BOTH `sol-ring`
    /// and `imprisoned-in-the-moon` are reachable within the first few draws. Eleven
    /// seeds qualified; **five put both cards in the opening seven** (87, 146, 184,
    /// 752, 778) and 87 is the first of them, which is the only reason it was chosen
    /// over the other four.
    const DX20B_SEED: u64 = 87;

    /// Sol Ring + Imprisoned in the Moon + 97 Islands. Almost-all-basics on purpose,
    /// the `ui1_deck` rationale: those two are the ONLY castable non-land cards in
    /// the whole 99, so the drive's "play a land, else cast Sol Ring, else pass"
    /// policy can never be confused about what to do.
    ///
    /// **Sol Ring is not decoration and is not merely the artifact witness** -- it is
    /// also two of the three mana that pay `{2}{U}`, so the class under exclusion is
    /// load-bearing in the fixture rather than a prop that could be deleted without
    /// anyone noticing.
    fn dx20b_human_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![
            CardId("sol-ring".to_string()),
            CardId("imprisoned-in-the-moon".to_string()),
        ];
        while main_deck.len() < 99 {
            main_deck.push(CardId("island".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX20B_COMMANDER.to_string()),
            main_deck,
        }
    }

    fn dx20b_bot_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        mtg_simulator::DeckConfig {
            commander: CardId(DX20B_COMMANDER.to_string()),
            main_deck: (0..99).map(|_| CardId("island".to_string())).collect(),
        }
    }

    /// Install through `session::new_game` -- the same constructor the real handler
    /// uses, running the same two Invariant-9 gates (`validate_deck`,
    /// `check_all_defs_complete`). See [`ui1_install`]'s doc for why
    /// `POST /api/game` cannot express a `DeckSource::Fixed` game.
    fn dx20b_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX20B_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx20b_human_deck()),
                (mtg_engine::PlayerId(2), dx20b_bot_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX20b fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Every permanent on the battlefield in this seat payload, as
    /// `(object_id, name, card_types)`.
    fn dx20b_battlefield(view: &Value) -> Vec<(u64, String, Vec<String>)> {
        view["state"]["zones"]["battlefield"]
            .as_object()
            .expect("battlefield is an object keyed by player name")
            .values()
            .filter_map(|permanents| permanents.as_array())
            .flatten()
            .map(|p| {
                (
                    p["object_id"].as_u64().expect("object_id is a number"),
                    p["name"].as_str().unwrap_or_default().to_string(),
                    p["card_types"]
                        .as_array()
                        .map(|ts| {
                            ts.iter()
                                .filter_map(|t| t.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    /// Drive the human seat: play a land every chance, cast Sol Ring the moment it is
    /// offered, otherwise pass -- UNTIL "Cast Imprisoned in the Moon" is offered AT
    /// ALL. Returns that view and the option itself, submitting NOTHING for it.
    ///
    /// The search predicate is deliberately **label alone** -- it does not also
    /// require a populated `candidates` array. That is `pb-review-DX20.md`'s E5
    /// finding applied here rather than rediscovered: a drive that filters on the
    /// property the test is named for turns every failure of that property into a
    /// misleading "the game ended without the card ever being offered" panic inside
    /// the drive loop, instead of a clean failure at the assertion that advertises
    /// it.
    async fn dx20b_drive_to_imprisoned(state: &SharedState, max_steps: usize) -> (Value, Value) {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if !view["decision"].is_null() {
                if let Some(found) = view["decision"]["actions"].as_array().and_then(|actions| {
                    actions.iter().find(|a| {
                        a["kind"] == "CastSpell" && a["label"] == "Cast Imprisoned in the Moon"
                    })
                }) {
                    return (view.clone(), found.clone());
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} without Imprisoned in the Moon ever \
                 being offered: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Sol Ring")
                })
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
        panic!("Imprisoned in the Moon was never offered within {max_steps} steps");
    }

    /// **CR 702.5a / CR 303.4a -- the printed Enchant line reaches the browser, and
    /// the artifact is NOT in it (`OOS-DX20-10`, HIGH).**
    ///
    /// A REAL HTTP round trip through `app(state.clone()).oneshot(..)` (the same
    /// router `main()` serves), not a `view::decision_view` call -- stated per the
    /// dispatch brief's instruction to say which was done.
    ///
    /// Four things are asserted and each is a different failure mode:
    ///
    /// 1. **Sol Ring is on the battlefield** at the moment of the ask. Without this
    ///    non-vacuity floor, assertion 2 would pass on a board that simply has no
    ///    artifact -- the exclusion would be measuring nothing.
    /// 2. **`target_slots[0].candidates` does not contain Sol Ring.** This is the
    ///    HIGH. Before PB-DX20b this array contained it, and PB-DX20 is what made it
    ///    a clickable button.
    /// 3. **The candidates DO contain the lands**, so the fix is not the obvious
    ///    over-correction (narrowing to `Creature`, which is exactly what
    ///    `kayas_ghostform` shipped for years -- `OOS-DX20-5`). Asserting only 2
    ///    would pass on an engine that offers nothing at all.
    /// 4. **A NON-DEFAULT answer is accepted and resolves onto the chosen land**, so
    ///    game state distinguishes the human's answer from any fallback. `attached_to`
    ///    is read back out of the seat payload rather than trusting the 200: a cast
    ///    can be accepted and the Aura can still fail to attach (CR 704.5m would then
    ///    bin it), which is the silent-fizzle shape `OOS-CARDS1-2` was filed for.
    ///
    /// The non-default answer is measured, not hoped for: at `DX20B_SEED` the offer
    /// carries **two** candidates -- the human's own Island and the BOT's Island --
    /// and this probe picks the last, i.e. an OPPONENT's permanent. That is as far
    /// from "the engine's own first choice" as this board can get.
    ///
    /// # What this HTTP probe does NOT cover, stated rather than left to be assumed
    ///
    /// PB-DX45's disclosure standard. A `play-server` session installs from a DECK
    /// and plays it out, so the only permanents on the board at the moment of the ask
    /// are the ones the drive could actually put there in two turns: Islands and Sol
    /// Ring. So over HTTP this probe exercises the **Land** class (offered, chosen,
    /// resolved) and the **Artifact** class (present, excluded) and **nothing else**.
    ///
    /// The three untested-over-HTTP combinations are named individually:
    ///
    /// * (Creature   x HTTP) -- printed-legal, not on this board;
    /// * (Planeswalker x HTTP) -- printed-legal, not on this board;
    /// * (Enchantment x HTTP) -- printed-ILLEGAL, not on this board, so the
    ///   enchantment half of the exclusion is asserted only on the simulator side.
    ///
    /// All five classes ARE covered, as an exact SET in both directions, by
    /// `crates/simulator/tests/pb_dx20b_enchant_offer_channel.rs::c1`, which calls the
    /// identical pair (`action_target_requirements` + `legal_targets_per_slot`) that
    /// `view::action_option_view` calls two lines above the JSON this probe reads. The
    /// untested combination is therefore *(three card types x the HTTP transport)*
    /// alone, and the transport itself is exercised here on the two classes that
    /// matter most -- the one the HIGH is about and the one the fix must not lose.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx20b_imprisoned_offer_excludes_the_artifact_over_http() {
        let state = shared_state();
        dx20b_install(&state);

        let (view, imprisoned) = dx20b_drive_to_imprisoned(&state, 4_000).await;

        // (1) Non-vacuity floor.
        let battlefield = dx20b_battlefield(&view);
        let sol_ring = battlefield
            .iter()
            .find(|(_, name, _)| name == "Sol Ring")
            .unwrap_or_else(|| {
                panic!(
                    "precondition: Sol Ring must be ON THE BATTLEFIELD when the Aura is \
                     offered, or the exclusion below measures nothing. Board: \
                     {battlefield:?}"
                )
            })
            .clone();
        assert!(
            sol_ring.2.iter().any(|t| t == "Artifact"),
            "Sol Ring must render as an Artifact for this probe to be about card \
             TYPES at all: {sol_ring:?}"
        );
        let lands: Vec<u64> = battlefield
            .iter()
            .filter(|(_, _, types)| types.iter().any(|t| t == "Land"))
            .map(|(id, _, _)| *id)
            .collect();

        let candidates = imprisoned["target_slots"][0]["candidates"]
            .as_array()
            .unwrap_or_else(|| {
                panic!(
                    "the Aura's offer must carry one target slot with a candidate \
                     array: {:?}",
                    imprisoned["target_slots"]
                )
            })
            .clone();
        let candidate_ids: Vec<u64> = candidates
            .iter()
            .map(|c| c["value"]["Object"].as_u64().unwrap_or(u64::MAX))
            .collect();

        // (2) THE HIGH.
        assert!(
            !candidate_ids.contains(&sol_ring.0),
            "CR 702.5a -- 'Enchant creature, land, or planeswalker'. Sol Ring is an \
             ARTIFACT and the browser was offered it as a target. That is \
             OOS-DX20-10 verbatim: candidates were {candidates:?}"
        );

        // (3) The other direction.
        assert!(
            !lands.is_empty(),
            "precondition: at least one land must be on the battlefield: {battlefield:?}"
        );
        for land in &lands {
            assert!(
                candidate_ids.contains(land),
                "CR 702.5a: a Land IS a printed-legal target, so land {land} must be \
                 offered. Missing it means the declaration was narrowed too far -- the \
                 OOS-DX20-5 shape. Candidates: {candidates:?}, board: {battlefield:?}"
            );
        }

        // (4) A NON-DEFAULT answer: the LAST candidate, never `candidates[0]`.
        assert!(
            candidates.len() >= 2,
            "this probe needs at least two candidates for its answer to be \
             distinguishable from the engine's own default; got {candidates:?}"
        );
        let chosen = candidates.last().expect("non-empty").clone();
        assert_ne!(
            chosen["value"], candidates[0]["value"],
            "the answer must not be candidates[0] -- an echo of the default proves \
             nothing about the human's choice reaching the engine"
        );
        let chosen_id = chosen["value"]["Object"]
            .as_u64()
            .expect("an object target carries an object id");

        let before_commands = command_count(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": imprisoned["index"],
                "params": { "targets": [chosen["value"].clone()] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "POST /api/game/action with a printed-legal Land target must return 200: \
             {after}"
        );
        assert!(
            command_count(&after) > before_commands,
            "the command count should have advanced past {before_commands}, got {:?}",
            command_count(&after)
        );

        // The RESOLUTION EFFECT, read back off the wire. Pass priority until the Aura
        // is a battlefield permanent.
        let mut view = after;
        let mut attached_to = None;
        for step in 0..200 {
            if let Some(p) = view["state"]["zones"]["battlefield"]
                .as_object()
                .expect("battlefield is an object")
                .values()
                .filter_map(|ps| ps.as_array())
                .flatten()
                .find(|p| p["name"] == "Imprisoned in the Moon")
            {
                attached_to = p["attached_to"].as_u64();
                break;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before the Aura resolved: {view}"
            );
            let wire_seq = seq(&view);
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .unwrap_or_else(|| panic!("no PassPriority at step {step}: {view}"));
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": pick["index"], "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "step {step} passing: {next}");
            view = next;
        }
        assert_eq!(
            attached_to,
            Some(chosen_id),
            "CR 303.4a/CR 702.5a: the resolved Aura must be attached to the LAND the \
             human chose ({chosen_id}). A 200 alone would not have caught a cast that \
             is accepted and then fails to attach."
        );
    }

    // ── PB-DX23 -- the human dredge channel (CR 702.52a, 400.1; plan §5 T5.1) ──
    //
    // Q6 (the play-server shape): NO new `AnswerShapeView` variant, NO new
    // `ActionParamsDto` field, NO new picker component -- the choice lives in
    // the `LegalAction` itself, one action per choice, the `PayEcho`/
    // `PayRecover` shape verbatim. This probe drives a real game to a real
    // dredge offer over the REAL HTTP API and answers it with the
    // NON-DEFAULT choice (`Some(troll)`, not the decline) so game state
    // distinguishes the human's answer from any fallback (the UI-4/SIM-6
    // standard), and pins that the offered option carries no blocking-decision
    // payload (the Q6 divergence from the brief's literal "blocking-decision
    // UI" wording).

    /// A mono-green commander (5+ mana, never affordable inside this
    /// fixture's short window) so it fixes CR 903.5c color identity without
    /// ever perturbing the drive -- same role `UI1_COMMANDER` plays for the
    /// mono-black UI-1 fixtures.
    const T5_DX23_COMMANDER: &str = "azusa-lost-but-seeking";

    /// CR 903.5c: one Golgari Grave-Troll ({4}{G}) plus 98 Forests,
    /// mono-green. Almost-all-basics on purpose, same rationale as
    /// `ui1_deck`: the Troll is the ONLY castable non-land card in the whole
    /// 99-card deck, so the drive's "play a land, else cast anything
    /// castable" policy can never be confused about what to do, regardless
    /// of exactly which Forests land in the opening hand at this seed.
    fn t5_dx23_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![CardId("golgari-grave-troll".to_string())];
        while main_deck.len() < 99 {
            main_deck.push(CardId("forest".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(T5_DX23_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// **Read off a sweep, not reasoned to** (the `ui1_deck`/`UI1_SEED` precedent):
    /// at this seed the Troll is in p1's opening 7-card hand, which is what keeps
    /// this drive to a handful of turns instead of racing two near-identical
    /// mono-Forest decks toward a mutual deck-out (empirically ~90+ turns at an
    /// unswept seed — a first draft of this fixture hit exactly that and never
    /// reached a dredge offer at all).
    const T5_DX23_SEED: u64 = 1;

    /// Install the fixture through `session::new_game` -- the same
    /// constructor the real handler uses, running the same two Invariant-9
    /// gates. See `ui1_install`'s doc for why `POST /api/game` cannot
    /// express this (it hard-codes `DeckSource::RandomPerSeat`).
    fn t5_dx23_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: T5_DX23_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), t5_dx23_deck()),
                (mtg_engine::PlayerId(2), t5_dx23_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session =
            session::new_game(cfg, 0).expect("the PB-DX23 T5.1 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    fn t5_dx23_hand(state: &SharedState) -> Vec<u64> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .zones()
            .get(&mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1)))
            .map(|z| z.object_ids())
            .unwrap_or_default()
            .iter()
            .map(|id| id.0)
            .collect()
    }

    /// Drive the human seat -- play a land, else cast anything castable, else
    /// pass, else take the first action that is not `Concede` -- until a
    /// `ChooseDredge` option naming an OBJECT (the `Some` arm, CR 702.52a,
    /// distinguished from the always-present decline by a non-null
    /// `object_id`) is offered.
    ///
    /// The Troll reaches the graveyard by LEGAL means along the way: cast
    /// with an empty graveyard, it enters as a 0/0 and dies to CR 704.5f --
    /// already proven by `crates/engine/tests/mechanics_e_l/
    /// golgari_grave_troll.rs::test_golgari_grave_troll_empty_graveyard_dies_to_sba`
    /// -- so this drive never pokes state; every step is a real POST through
    /// the router `main()` itself serves.
    async fn t5_dx23_drive_to_a_named_dredge_offer(state: &SharedState, max_steps: usize) -> Value {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            if let Some(actions) = view["decision"]["actions"].as_array() {
                if actions
                    .iter()
                    .any(|a| a["kind"] == "ChooseDredge" && !a["object_id"].is_null())
                {
                    return view;
                }
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} without ever offering a named \
                 ChooseDredge option; T5_DX23_SEED may need re-pinning: {view}"
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
        panic!("no named ChooseDredge option within {max_steps} steps");
    }

    /// **T5.1** -- CR 702.52a, 400.1 (plan §5 T5.1, acceptance criterion 1
    /// human half). A human seat can answer a dredge offer over the REAL
    /// HTTP API with the NON-DEFAULT answer (`Some(troll)`, not the
    /// decline), so game state distinguishes the human's choice from any
    /// fallback (the UI-4/SIM-6 standard).
    ///
    /// Also pins the Q6 divergence from the brief's literal
    /// "blocking-decision UI" wording: the offered option carries NO
    /// `decision` payload -- CR 702.52a is "you MAY instead", so the engine
    /// deliberately does not block on it, and a dredge offer is an ORDINARY
    /// play, not a blocking decision.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx23_browser_can_answer_a_dredge_offer() {
        let state = shared_state();
        t5_dx23_install(&state);

        let view = t5_dx23_drive_to_a_named_dredge_offer(&state, 4_000).await;
        let actions = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array");
        let option = actions
            .iter()
            .find(|a| a["kind"] == "ChooseDredge" && !a["object_id"].is_null())
            .expect("just found by the drive loop");

        // The Q6 pin: an ordinary play, not a blocking decision.
        assert!(
            option["decision"].is_null(),
            "CR 702.52a is \"you MAY instead\" -- the engine deliberately does not \
             block on it, so a ChooseDredge option must carry no blocking-decision \
             payload. got {:?}",
            option["decision"]
        );
        let label = option["label"].as_str().unwrap_or_default();
        assert!(
            label.contains("Dredge"),
            "the label should name the dredge action, got {label:?}"
        );

        let wire_seq = seq(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({"seq": wire_seq, "action_index": option["index"], "params": {}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        // Out-of-band oracle: read the engine's own journal for the Dredged
        // event, exactly as `ui1_zone`/`ui1_library` read the engine's own
        // state -- never used to build the payload, only to verify the
        // effect. CR 400.7: the dredged card gets a NEW ObjectId in hand, so
        // this reads `card_new_id` from the event rather than re-using the
        // graveyard-zone `object_id` the offer carried.
        let (dredged_new_id, milled) = {
            let guard = state.session.lock().expect("lock");
            let session = guard.as_ref().expect("a session is installed");
            session
                .game
                .journal()
                .iter()
                .flat_map(|r| r.events.iter())
                .find_map(|e| match e {
                    mtg_engine::GameEvent::Dredged {
                        player,
                        card_new_id,
                        milled,
                    } if *player == mtg_engine::PlayerId(1) => Some((card_new_id.0, *milled)),
                    _ => None,
                })
                .expect("a GameEvent::Dredged for p1 must be in the journal")
        };
        assert_eq!(
            milled, 6,
            "CR 702.52a: Golgari Grave-Troll mills exactly its own Dredge N (6)"
        );

        let after_hand = t5_dx23_hand(&state);
        assert!(
            after_hand.contains(&dredged_new_id),
            "CR 702.52a: dredging must return the card to hand (under its new, \
             CR 400.7 zone-change id). hand: {:?}",
            after_hand
        );
    }

    // ── PB-DX29 (OOS-UI2-4): the cost-kind surface ────────────────────────────
    //
    // UI-2 surfaced 2 of `AdditionalCost`'s 15 variants to the browser. PB-DX29
    // added seven — Replicate, EscalateModes, Entwine, Fuse, Offspring, Gift and
    // Splice — across `legal_actions.rs` (the plan), `view.rs` (the DTOs),
    // `CostPicker.svelte` (the widgets) and `api.rs` (the 400 boundary).
    //
    // Group 1 below unit-tests that boundary, in the UI-2 / SIM-6 style
    // (`test_ui2_validate_additional_cost_params_rejects_*`,
    // `test_sim6_validate_*`): a hand-built `AdditionalCostPlan` plus a hand-built
    // `ActionParamsDto`, no HTTP and no game state, so each check is exercised in
    // isolation from whatever a real drive happens to reach. Group 2 drives the
    // whole chain over HTTP against a real `LocalGame`.
    //
    // The three PB-DX29 code paths are deliberately all covered: `counts`
    // (`count_option`), `markers` (`has_marker`), and the two bespoke arms
    // (`gift.eligible` / `splice.eligible` + CR 702.47b).

    /// PB-DX29: a `CastSpell` action carrying `plan` verbatim.
    ///
    /// [`ui2_cast_spell_action_with_costs`]'s shape with the plan handed in rather
    /// than built inline, because these probes need plans that DIFFER in which
    /// family they offer — "the offer never carried this kind" is half of what
    /// `validate_additional_cost_params` checks, and it cannot be exercised by a
    /// fixture that always offers everything.
    fn dx29_cast_action(
        plan: mtg_simulator::legal_actions::AdditionalCostPlan,
    ) -> mtg_simulator::LegalAction {
        mtg_simulator::LegalAction::CastSpell {
            card: mtg_engine::ObjectId(1),
            from_zone: mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1)),
            additional_costs: plan,
            alt_cost: None,
        }
    }

    /// PB-DX29: the seat a [`dx29_full_plan`] gift may name.
    const DX29_GIFT_SEAT: mtg_engine::PlayerId = mtg_engine::PlayerId(2);
    /// PB-DX29: a seat NO plan here offers — the out-of-set gift answer.
    const DX29_FOREIGN_SEAT: mtg_engine::PlayerId = mtg_engine::PlayerId(7);
    /// PB-DX29: the two cards a [`dx29_full_plan`] splice may name.
    const DX29_SPLICE_A: mtg_engine::ObjectId = mtg_engine::ObjectId(20);
    const DX29_SPLICE_B: mtg_engine::ObjectId = mtg_engine::ObjectId(21);
    /// PB-DX29: a card NO plan here offers — the out-of-set splice answer.
    const DX29_FOREIGN_CARD: mtg_engine::ObjectId = mtg_engine::ObjectId(999);
    /// PB-DX29: [`dx29_full_plan`]'s Replicate ceiling.
    const DX29_REPLICATE_MAX: u32 = 2;
    /// PB-DX29: [`dx29_full_plan`]'s Escalate ceiling. Deliberately DIFFERENT from
    /// [`DX29_REPLICATE_MAX`], so a check that read the wrong `counts` entry — the
    /// exact failure `count_option`'s `kind` lookup exists to prevent — shows up as
    /// a wrong bound rather than a coincidence.
    const DX29_ESCALATE_MAX: u32 = 1;

    /// PB-DX29: a plan offering one of EVERY family this batch surfaced at once.
    ///
    /// No real spell carries all seven (Fuse needs a split card, Splice needs a
    /// matching subtype in hand, Gift is its own keyword), and that is fine: this is
    /// a unit fixture for a function whose whole job is to compare an ANSWER against
    /// a PLAN. Offering everything makes the happy path below discriminating —
    /// every arm is reached with a legal value and must not fire — which a
    /// one-family fixture cannot do.
    fn dx29_full_plan() -> mtg_simulator::legal_actions::AdditionalCostPlan {
        use mtg_simulator::legal_actions::{
            CountCostKind, CountCostOption, GiftCostOption, MarkerCostKind, MarkerCostOption,
            SpliceCostOption,
        };
        mtg_simulator::legal_actions::AdditionalCostPlan {
            counts: vec![
                CountCostOption {
                    kind: CountCostKind::Replicate,
                    cost: mtg_engine::ManaCost {
                        generic: 1,
                        blue: 1,
                        ..Default::default()
                    },
                    max_count: DX29_REPLICATE_MAX,
                },
                CountCostOption {
                    kind: CountCostKind::Escalate,
                    cost: mtg_engine::ManaCost {
                        generic: 1,
                        red: 1,
                        ..Default::default()
                    },
                    max_count: DX29_ESCALATE_MAX,
                },
            ],
            markers: vec![
                MarkerCostOption {
                    kind: MarkerCostKind::Entwine,
                    cost: Some(mtg_engine::ManaCost {
                        generic: 2,
                        red: 1,
                        ..Default::default()
                    }),
                    affordable: true,
                },
                MarkerCostOption {
                    kind: MarkerCostKind::Fuse,
                    // CR 702.102b: no separate fuse cost — see `MarkerCostOption::cost`.
                    cost: None,
                    affordable: true,
                },
                MarkerCostOption {
                    kind: MarkerCostKind::Offspring,
                    cost: Some(mtg_engine::ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                    affordable: true,
                },
            ],
            gift: Some(GiftCostOption {
                gift_type: mtg_engine::cards::card_definition::GiftType::Card,
                eligible: vec![DX29_GIFT_SEAT],
            }),
            splice: Some(SpliceCostOption {
                eligible: vec![DX29_SPLICE_A, DX29_SPLICE_B],
            }),
            ..Default::default()
        }
    }

    /// PB-DX29: an announcement of `costs` against [`dx29_full_plan`].
    fn dx29_params(costs: Vec<mtg_engine::AdditionalCost>) -> crate::view::ActionParamsDto {
        crate::view::ActionParamsDto {
            additional_costs: costs,
            ..Default::default()
        }
    }

    /// PB-DX29: assert that `costs` is refused 400 `bad_params` against `plan`.
    fn dx29_expect_400(
        plan: mtg_simulator::legal_actions::AdditionalCostPlan,
        costs: Vec<mtg_engine::AdditionalCost>,
        why: &str,
    ) {
        let action = dx29_cast_action(plan);
        let params = dx29_params(costs);
        let err = api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect_err(&format!("must be refused: {why}"));
        assert_eq!(err.status, StatusCode::BAD_REQUEST, "{why}");
        assert_eq!(err.body.kind, "bad_params", "{why}");
    }

    /// **T1 — CR 702.56a: a Replicate count above the offered `max_count` is 400.**
    ///
    /// The Squad shape (`test_ui2_validate_additional_cost_params_rejects_squad_over_max_count`)
    /// on the first of the two `counts` kinds.
    #[test]
    fn test_dx29_validate_rejects_replicate_over_max_count() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![mtg_engine::AdditionalCost::Replicate {
                count: DX29_REPLICATE_MAX + 1,
            }],
            "CR 702.56a: a replicate count above what the offer vouched for",
        );
    }

    /// **T2 — CR 702.120a: an Escalate count above the offered `max_count` is 400.**
    ///
    /// The other `counts` kind, and it is not a duplicate of T1: `count_option`
    /// looks the bound up BY KIND, and the two fixture bounds differ
    /// ([`DX29_ESCALATE_MAX`] < [`DX29_REPLICATE_MAX`]), so an implementation that
    /// read the first `counts` entry regardless of kind would accept this value.
    #[test]
    fn test_dx29_validate_rejects_escalate_over_max_count() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![mtg_engine::AdditionalCost::EscalateModes {
                count: DX29_ESCALATE_MAX + 1,
            }],
            "CR 702.120a: more additional modes than the offer vouched for",
        );
    }

    /// **T3 — CR 702.56a: a Replicate on a plan whose `counts` is empty is 400.**
    ///
    /// The `counts` half of the "a kind the offer never carried" check.
    #[test]
    fn test_dx29_validate_rejects_replicate_when_no_count_rider_was_offered() {
        let plan = mtg_simulator::legal_actions::AdditionalCostPlan {
            counts: Vec::new(),
            ..dx29_full_plan()
        };
        dx29_expect_400(
            plan,
            vec![mtg_engine::AdditionalCost::Replicate { count: 1 }],
            "CR 702.56a: replicate announced against a plan offering no count rider",
        );
    }

    /// **T4 — CR 702.42a: an Entwine on a plan whose `markers` is empty is 400.**
    ///
    /// The `markers` half — a different code path (`has_marker`, not
    /// `count_option`), which is why it is checked separately rather than assumed
    /// to follow from T3.
    #[test]
    fn test_dx29_validate_rejects_entwine_when_no_marker_rider_was_offered() {
        let plan = mtg_simulator::legal_actions::AdditionalCostPlan {
            markers: Vec::new(),
            ..dx29_full_plan()
        };
        dx29_expect_400(
            plan,
            vec![mtg_engine::AdditionalCost::Entwine],
            "CR 702.42a: entwine announced against a plan offering no marker rider",
        );
    }

    /// **T4b — CR 702.175a: an Offspring against a plan carrying only OTHER markers
    /// is 400.**
    ///
    /// `has_marker` is a per-kind lookup, and T4 (an empty `markers`) could not tell
    /// a per-kind lookup from a bare `!markers.is_empty()`. Here Entwine and Fuse
    /// are both on offer and Offspring is not, so only a per-kind check can refuse
    /// it.
    #[test]
    fn test_dx29_validate_rejects_offspring_when_only_other_markers_were_offered() {
        use mtg_simulator::legal_actions::{MarkerCostKind, MarkerCostOption};
        let plan = mtg_simulator::legal_actions::AdditionalCostPlan {
            markers: vec![
                MarkerCostOption {
                    kind: MarkerCostKind::Entwine,
                    cost: Some(mtg_engine::ManaCost {
                        generic: 2,
                        red: 1,
                        ..Default::default()
                    }),
                    affordable: true,
                },
                MarkerCostOption {
                    kind: MarkerCostKind::Fuse,
                    cost: None,
                    affordable: true,
                },
            ],
            ..dx29_full_plan()
        };
        dx29_expect_400(
            plan,
            vec![mtg_engine::AdditionalCost::Offspring],
            "CR 702.175a: offspring is not among the markers this offer carried",
        );
    }

    /// **T5 — CR 702.174a: a Gift on a plan with no gift is 400.**
    ///
    /// The third code path. Gift is the only additional cost whose answer is a
    /// `PlayerId`, so nothing about T3/T4 covers it.
    #[test]
    fn test_dx29_validate_rejects_gift_when_none_was_offered() {
        let plan = mtg_simulator::legal_actions::AdditionalCostPlan {
            gift: None,
            ..dx29_full_plan()
        };
        dx29_expect_400(
            plan,
            vec![mtg_engine::AdditionalCost::Gift {
                opponent: DX29_GIFT_SEAT,
            }],
            "CR 702.174a: gift announced against a plan that has no gift to give",
        );
    }

    /// **T6 — CR 702.174a: a Gift naming a seat outside `eligible` is 400.**
    ///
    /// The gift analogue of UI-2's out-of-set sacrifice id. `casting.rs` accepts any
    /// OTHER player still in the game; a seat this offer never listed is an
    /// announcement the response never made.
    #[test]
    fn test_dx29_validate_rejects_gift_naming_a_seat_outside_eligible() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![mtg_engine::AdditionalCost::Gift {
                opponent: DX29_FOREIGN_SEAT,
            }],
            "CR 702.174a: a seat this gift never offered",
        );
    }

    /// **T7 — CR 702.47a: a Splice naming a card outside `eligible` is 400.**
    ///
    /// The list-valued arm's membership half. The other entry in the same list is
    /// legal, so this cannot pass or fail for want of a well-formed list.
    #[test]
    fn test_dx29_validate_rejects_splice_of_a_card_outside_eligible() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![mtg_engine::AdditionalCost::Splice {
                cards: vec![DX29_SPLICE_A, DX29_FOREIGN_CARD],
            }],
            "CR 702.47a: a card this splice offer never accepted",
        );
    }

    /// **T8 — CR 702.47b: a Splice naming the SAME card twice is 400.**
    ///
    /// "one or more OTHER cards" — each may be spliced once. Both ids here are
    /// eligible and the list is well-formed, so only the duplicate-within-the-list
    /// check can refuse it; that distinguishes this from T7.
    #[test]
    fn test_dx29_validate_rejects_splicing_the_same_card_twice() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![mtg_engine::AdditionalCost::Splice {
                cards: vec![DX29_SPLICE_A, DX29_SPLICE_A],
            }],
            "CR 702.47b: the same card spliced twice",
        );
    }

    /// **T9 — a DUPLICATE `Replicate` entry is 400 (the `DUPLICABLE_COST_KINDS`
    /// table).**
    ///
    /// UI-2's argument, one kind over: `casting.rs`'s destructuring loop is
    /// `replicate_count = *count`, a plain assignment, so the LAST entry wins and
    /// the first is dropped with no error and no diagnostic. **Both counts here are
    /// within `max_count`**, so the per-entry bound check of T1 cannot be what
    /// rejects this — only the table-driven duplicate check can.
    #[test]
    fn test_dx29_validate_rejects_a_duplicate_replicate_entry() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![
                mtg_engine::AdditionalCost::Replicate { count: 2 },
                mtg_engine::AdditionalCost::Replicate { count: 1 },
            ],
            "CR 702.56a: two replicate announcements, both individually in bounds",
        );
    }

    /// **T10 — a DUPLICATE `Entwine` entry is 400.**
    ///
    /// The unit-variant half of the same table. It matters on its own because
    /// `Entwine` carries no payload at all: a duplicate detector keyed on the
    /// announced VALUE rather than the discriminant would see two identical,
    /// individually-legal answers and wave them through — and the offer still only
    /// ever made one such announcement.
    #[test]
    fn test_dx29_validate_rejects_a_duplicate_entwine_entry() {
        dx29_expect_400(
            dx29_full_plan(),
            vec![
                mtg_engine::AdditionalCost::Entwine,
                mtg_engine::AdditionalCost::Entwine,
            ],
            "CR 702.42a: two entwine announcements",
        );
    }

    /// **T11 — a DUPLICATE `Gift` naming two DIFFERENT seats is 400.**
    ///
    /// The reason the table matches the discriminant and not the payload, stated as
    /// a test: two gifts naming different opponents is exactly the ambiguity being
    /// refused, and both seats here are eligible, so the `eligible` check of T6
    /// cannot be what fires.
    #[test]
    fn test_dx29_validate_rejects_two_gifts_naming_different_eligible_seats() {
        let other = mtg_engine::PlayerId(3);
        let plan = mtg_simulator::legal_actions::AdditionalCostPlan {
            gift: Some(mtg_simulator::legal_actions::GiftCostOption {
                gift_type: mtg_engine::cards::card_definition::GiftType::Card,
                eligible: vec![DX29_GIFT_SEAT, other],
            }),
            ..dx29_full_plan()
        };
        dx29_expect_400(
            plan,
            vec![
                mtg_engine::AdditionalCost::Gift {
                    opponent: DX29_GIFT_SEAT,
                },
                mtg_engine::AdditionalCost::Gift { opponent: other },
            ],
            "CR 702.174a: two gifts naming two different eligible seats",
        );
    }

    /// **T12 — the discriminating happy path: one legal answer of EVERY PB-DX29
    /// family at once is ACCEPTED.**
    ///
    /// Without this, T1-T11 prove only that the function refuses things; they cannot
    /// distinguish a correct boundary from one that refuses every PB-DX29 kind
    /// outright (which is what the pre-batch code did, by falling through to the
    /// engine's 422). Every arm added by this batch is reached here with a value the
    /// offer vouched for, and none of them may fire.
    ///
    /// CR 702.56a / 702.120a / 702.42a / 702.102a / 702.175a / 702.174a / 702.47a.
    #[test]
    fn test_dx29_validate_accepts_one_legal_answer_of_every_family() {
        let action = dx29_cast_action(dx29_full_plan());
        let params = dx29_params(vec![
            mtg_engine::AdditionalCost::Replicate {
                count: DX29_REPLICATE_MAX,
            },
            mtg_engine::AdditionalCost::EscalateModes {
                count: DX29_ESCALATE_MAX,
            },
            mtg_engine::AdditionalCost::Entwine,
            mtg_engine::AdditionalCost::Fuse,
            mtg_engine::AdditionalCost::Offspring,
            mtg_engine::AdditionalCost::Gift {
                opponent: DX29_GIFT_SEAT,
            },
            mtg_engine::AdditionalCost::Splice {
                cards: vec![DX29_SPLICE_A, DX29_SPLICE_B],
            },
        ]);
        api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect(
            "one in-bounds answer of every offered family must be accepted -- \
             otherwise the 400 boundary is a blanket refusal, not a check",
        );
    }

    /// **T13 — `count: 0` is a legal answer, not a decline the boundary may refuse.**
    ///
    /// `CountCostOption::max_count`'s own doc says zero is a legal value and does not
    /// suppress the offer (CR 702.56a "any number of times" includes zero). Pinned
    /// separately from T12 because a bound check written as `>= max_count` or a
    /// presence check written as "an announced rider must be paid" would both pass
    /// T12 and fail here.
    #[test]
    fn test_dx29_validate_accepts_a_zero_count_rider() {
        let action = dx29_cast_action(dx29_full_plan());
        let params = dx29_params(vec![
            mtg_engine::AdditionalCost::Replicate { count: 0 },
            mtg_engine::AdditionalCost::EscalateModes { count: 0 },
        ]);
        api::validate_additional_cost_params(
            &action,
            &params,
            &dx29_empty_state(),
            mtg_engine::PlayerId(1),
        )
        .expect("CR 702.56a: paying a rider zero times is legal");
    }

    /// **T14 — the wire shape this batch exists to document: a MARKER template is a
    /// bare JSON STRING, a COUNT template is an object with one key.**
    ///
    /// `AdditionalCost::Entwine` / `::Fuse` / `::Offspring` are Rust **unit**
    /// variants, and serde's externally-tagged encoding renders a unit variant as
    /// `"Entwine"`, never `{"Entwine": {}}`. So `MarkerCostView` carries no `*_key`
    /// and the `fillTemplate` idiom every other cost picker uses — clone the object,
    /// write the field the server named — has nothing to write into and would throw
    /// on `Object.keys(entry)[0]`. Same shape-of-JSON trap PB-DP10 measured on
    /// `Effect::Proliferate`.
    ///
    /// Asserted here rather than left in prose because it is invisible in Rust: the
    /// two families are the same enum and the same `Serialize` derive, and only the
    /// rendered value tells them apart.
    #[test]
    fn test_dx29_marker_templates_are_bare_json_strings_and_count_templates_are_not() {
        for (variant, expected) in [
            (mtg_engine::AdditionalCost::Entwine, "Entwine"),
            (mtg_engine::AdditionalCost::Fuse, "Fuse"),
            (mtg_engine::AdditionalCost::Offspring, "Offspring"),
        ] {
            let wire = serde_json::to_value(&variant).expect("AdditionalCost serializes");
            assert!(
                wire.is_string(),
                "{expected} must serialize as a bare JSON string, got {wire}"
            );
            assert_eq!(wire, json!(expected));
            assert!(
                wire.as_object().is_none(),
                "{expected} must NOT be an object -- the picker's \
                 `Object.keys(entry)[0]` fill idiom would throw on it"
            );
        }

        // The contrast, in the same test so the two conventions cannot drift apart
        // unnoticed: a count rider IS an object with exactly one named key, which is
        // why `CountCostView` carries `count_key` and `MarkerCostView` carries no key
        // at all.
        assert_eq!(
            serde_json::to_value(mtg_engine::AdditionalCost::Replicate { count: 0 })
                .expect("serializes"),
            json!({"Replicate": {"count": 0}})
        );
        assert_eq!(
            serde_json::to_value(mtg_engine::AdditionalCost::EscalateModes { count: 0 })
                .expect("serializes"),
            json!({"EscalateModes": {"count": 0}})
        );
        assert_eq!(
            serde_json::to_value(mtg_engine::AdditionalCost::Gift {
                opponent: DX29_GIFT_SEAT
            })
            .expect("serializes"),
            json!({"Gift": {"opponent": DX29_GIFT_SEAT.0}})
        );
        assert_eq!(
            serde_json::to_value(mtg_engine::AdditionalCost::Splice { cards: vec![] })
                .expect("serializes"),
            json!({"Splice": {"cards": []}})
        );
    }

    /// **T15 — the same wire fact one layer up: a rendered `MarkerCostView` carries
    /// its whole answer in `template`, as a bare string, and names NO key.**
    ///
    /// T14 checks the enum; this checks the DTO the browser actually receives, which
    /// is where the trap bites. Every other cost view tells the client which field of
    /// a cloned template to fill (`count_key` / `ids_key` / `player_key`); a
    /// `MarkerCostView` must not, because there is no object to fill — and a client
    /// that went looking for one would find `undefined` rather than an error.
    ///
    /// CR 702.42a / CR 702.102a / CR 702.175a.
    #[test]
    fn test_dx29_rendered_marker_cost_view_is_a_keyless_bare_string_template() {
        let view = crate::view::MarkerCostView {
            kind: "Entwine".to_string(),
            prompt: "Pay the entwine cost to choose all modes (CR 702.42a)".to_string(),
            // CR 702.102b's `None` case is exercised by the Fuse row below.
            cost_label: Some("{2}{R}".to_string()),
            template: mtg_engine::AdditionalCost::Entwine,
            affordable: true,
        };
        let wire = serde_json::to_value(&view).expect("MarkerCostView serializes");
        assert!(
            wire["template"].is_string(),
            "the whole answer is the template, and it is a bare string: {wire}"
        );
        assert_eq!(wire["template"], json!("Entwine"));
        for key in ["count_key", "ids_key", "player_key", "key", "field"] {
            assert!(
                wire.get(key).is_none(),
                "a marker view must name no fill key ({key} present): {wire}"
            );
        }

        // CR 702.102b: Fuse's `cost_label` is genuinely absent, not `{0}` — the fused
        // cost is the two halves summed, so there is no separate figure to print.
        let fuse = crate::view::MarkerCostView {
            kind: "Fuse".to_string(),
            prompt: "Cast both halves".to_string(),
            cost_label: None,
            template: mtg_engine::AdditionalCost::Fuse,
            affordable: true,
        };
        let fuse_wire = serde_json::to_value(&fuse).expect("serializes");
        assert!(fuse_wire["cost_label"].is_null(), "{fuse_wire}");
        assert_eq!(fuse_wire["template"], json!("Fuse"));
    }

    // ── PB-DX29 group 2: Replicate, end to end over HTTP ──────────────────────
    //
    // The UI-2 stage-5 pattern verbatim (`ui2_install` / `ui2_deck_with` / the drive
    // loop / `post_json` / reading results back BY NAME), with an Island deck instead
    // of a Forest one because the subject card is blue.

    /// `{3}{U}{U}{U}`, Legendary Creature — Wizard 3/4, `Completeness::Complete` by
    /// derive (`crates/card-defs/src/defs/arcanis_the_omnipotent.rs`). Mono-blue, so
    /// CR 903.5c colour identity admits an Island deck and a blue spell; verified to
    /// be a legendary creature (CR 903.3) by reading the def, not assumed — and
    /// `session::new_game` runs the real `validate_deck`, so an illegal commander
    /// would fail this fixture's install rather than pass silently.
    ///
    /// It is also the most expensive `Complete` mono-blue legend in the corpus (6
    /// mana), enumerated over `crates/card-defs/src/defs` rather than guessed:
    /// Nezahal is `known_wrong` and Azami/Alandra/Tetsuko are `inert`, so none of
    /// them is deck-legal at all.
    const DX29_COMMANDER: &str = "arcanis-the-omnipotent";

    /// `{1}{U}`, Sorcery, `Completeness::Complete` by derive
    /// (`crates/card-defs/src/defs/train_of_thought.rs`). Replicate `{1}{U}`, "Draw a
    /// card." — the cheapest Replicate card in the corpus and the one whose result is
    /// most directly observable: each payment copies the spell, and each copy draws,
    /// so N is readable straight off the library count.
    const DX29_REPLICATE_SPELL: &str = "train-of-thought";
    /// [`DX29_REPLICATE_SPELL`]'s rendered `CardDefinition.name` — the offer's label
    /// and every by-name lookup are keyed on this, never on the kebab-case `CardId`
    /// ([`UI2_ELF_A_NAME`]'s distinction).
    const DX29_REPLICATE_SPELL_NAME: &str = "Train of Thought";

    /// CR 903.5c: [`DX29_COMMANDER`] plus 99 Islands, with `overrides` written over
    /// the named positions. The Island twin of [`ui2_deck_with`] — a separate builder
    /// rather than a parameter on that one, because that function is UI-2's and is
    /// cited by name in four of its docs.
    fn dx29_island_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("island".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX29_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// All-Island deck (plus commander) — the harmless opponent-seat fixture, the
    /// role [`ui2_forest_only_deck`] plays for UI-2. No spell in it at all, so the bot
    /// can only ever play lands and pass.
    fn dx29_island_only_deck() -> mtg_simulator::DeckConfig {
        dx29_island_deck_with(&[])
    }

    /// The probe fixture: [`DX29_REPLICATE_SPELL`] at position 0, Island everywhere
    /// else.
    ///
    /// Position 0 is in the opening hand, and [`UI2_SEED`]'s pin is what says so —
    /// it applies unchanged here even though the deck's CONTENT is entirely
    /// different, because `SliceRandom::shuffle` permutes INDICES and depends only on
    /// the rng stream and the slice LENGTH, never on what sits at each index. This
    /// fixture is installed through [`ui2_install`], which seeds at [`UI2_SEED`], and
    /// is a 99-card `Fixed` deck like every UI-2 stage-5 fixture, so the pinned
    /// opening positions `{0, 1, 19, 39, 50, 53, 70}` still hold; only position 0 is
    /// needed.
    fn dx29_train_of_thought_deck() -> mtg_simulator::DeckConfig {
        dx29_island_deck_with(&[(0, DX29_REPLICATE_SPELL)])
    }

    /// **CR 702.56a — Replicate, offered, refused when over-paid, and PAID TWICE over
    /// HTTP.** The PB-DX29 end-to-end probe: `legal_actions.rs`'s plan →
    /// `view.rs`'s `counts` DTO → the 400 boundary → `params.rs` → `casting.rs` →
    /// three cards drawn.
    ///
    /// Structured exactly as [`test_ui2_squad_paying_twice_produces_two_token_copies_over_http`],
    /// for the same reasons: the offer is read first (so the descriptor itself is
    /// checked, not just the outcome), an ILLEGAL answer is submitted first and must
    /// 400 `bad_params`, and the real answer is NON-DEFAULT — `count = max_count = 2`
    /// — because a decline (`count = 0`, or an empty `additional_costs`) is
    /// indistinguishable from a client that sent nothing at all.
    ///
    /// `count = 2` rather than 1 for the reason that probe gives: 2 discriminates
    /// "the count is read" from "the count is a boolean".
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx29_replicate_is_offered_and_paid_twice_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        ui2_install(
            &state,
            dx29_train_of_thought_deck(),
            dx29_island_only_deck(),
        );

        // 6 Islands: base {1}{U} (MV 2) + 2 x Replicate {1}{U} (MV 2) = 6, exactly.
        let view = ui2_drive_playing_lands(&state, 6, UI2_LAND_DRIVE_MAX_STEPS).await;
        let cast_label = format!("Cast {DX29_REPLICATE_SPELL_NAME}");
        let action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!("{DX29_REPLICATE_SPELL_NAME} must be offered once 6 Islands are out: {view}")
            });
        let index = action["index"].as_u64().expect("index is a number");

        // ── the descriptor ────────────────────────────────────────────────────
        let costs = &action["costs"];
        assert!(
            !costs.is_null(),
            "a Replicate spell must carry a costs descriptor: {action}"
        );
        assert_eq!(costs["answer_field"], "additional_costs");
        assert!(
            costs["sacrifice"].is_null() && costs["squad"].is_null(),
            "Train of Thought has neither of UI-2's two kinds: {costs}"
        );
        // The "always serialized, empty when there is nothing to ask" convention
        // (`AdditionalCostsView::counts`' own doc): `markers` must be an empty ARRAY
        // here, not absent and not null, or a client has two presence conventions to
        // learn in one struct.
        assert_eq!(
            costs["markers"].as_array().map(|a| a.len()),
            Some(0),
            "markers must be present-and-empty, not absent: {costs}"
        );
        assert!(costs["gift"].is_null(), "{costs}");
        assert!(costs["splice"].is_null(), "{costs}");

        let counts = costs["counts"].as_array().expect("counts is an array");
        assert_eq!(counts.len(), 1, "exactly one count rider: {counts:?}");
        let replicate = &counts[0];
        assert_eq!(
            replicate["kind"], "Replicate",
            "the mechanic's PRINTED name, not the wire tag"
        );
        // The printing: Train of Thought prints "Replicate {1}{U}", and
        // `format_mana_cost_compact` emits the generic component first.
        assert_eq!(replicate["cost_label"], "{1}{U}");
        assert_eq!(replicate["count_key"], "count");
        assert_eq!(replicate["template"], json!({"Replicate": {"count": 0}}));
        let max_count = replicate["max_count"]
            .as_u64()
            .expect("max_count is a number");
        assert_eq!(
            max_count, 2,
            "6 mana available, base cost 2, Replicate {{1}}{{U}} (MV 2) per payment \
             -> exactly 2 affordable: {replicate}"
        );

        let wire_seq = seq(&view);

        // ── the illegal answer: over the offer's OWN max_count ────────────────
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": [{"Replicate": {"count": max_count + 1}}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params", "{refused}");

        // ── the real, NON-DEFAULT answer ──────────────────────────────────────
        let library_before = ui1_library(&state).len();
        let graveyard_before = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard_before.is_empty(),
            "sanity: nothing has resolved yet: {graveyard_before:?}"
        );

        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": [{"Replicate": {"count": max_count}}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        // The replicate trigger (CR 702.56b, "When you cast this spell, copy it for
        // each time you paid its replicate cost") goes on the stack ABOVE the spell,
        // so the copies exist and resolve before the original does. The drain loop
        // runs until the whole stack is empty.
        ui2_drain_stack(&state, after_cast, 60).await;

        // ── the result, read out of band ──────────────────────────────────────
        //
        // CR 702.56b: 2 payments -> 2 copies, each of which draws (CR 707.10: a copy
        // of a spell is put onto the stack and resolves like the spell), plus the
        // original's own draw = 3.
        let library_after = ui1_library(&state).len();
        assert_eq!(
            library_after,
            library_before - 3,
            "CR 702.56a/702.56b: 2 replicate payments produce 2 copies, and each copy \
             plus the original draws one card. A client whose count was dropped would \
             have drawn 1; a count read as a boolean would have drawn 2."
        );

        // BY NAME (CR 400.7): the resolved sorcery itself is in the graveyard exactly
        // once -- the copies cease to exist on resolution (CR 707.10a) rather than
        // being put anywhere, so a graveyard holding three would mean the engine had
        // moved real cards.
        let graveyard_after = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert_eq!(
            graveyard_after
                .iter()
                .filter(|n| n.as_str() == DX29_REPLICATE_SPELL_NAME)
                .count(),
            1,
            "the real card resolves to the graveyard once; its copies cease to exist: \
             {graveyard_after:?}"
        );

        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "all 6 available mana must have been spent -- base {{1}}{{U}} plus 2x \
             Replicate {{1}}{{U}}; a leftover pool would mean the rider was announced \
             and never charged"
        );
    }

    // ── PB-DX29 group 2b: Entwine, end to end over HTTP ───────────────────────
    //
    // The Replicate probe above leaves `markers` EMPTY, so nothing in this crate had
    // ever observed a marker entry on the real wire -- and the marker family is the
    // one whose encoding is the trap this batch exists to document (T14/T15). This
    // probe is what puts a bare-string template on an actual HTTP response.

    /// `{4}{R}{R}`, Legendary Creature — Dragon 4/4, `Completeness::Complete` by
    /// derive (`crates/card-defs/src/defs/lathliss_dragon_queen.rs`). Mono-red, so
    /// CR 903.5c colour identity admits a Mountain deck and a red spell; the most
    /// expensive `Complete` mono-red legend in the corpus, enumerated over the defs
    /// directory rather than guessed. Its trigger is a battlefield ability and it is
    /// never cast here, so it cannot perturb the board.
    const DX29_RED_COMMANDER: &str = "lathliss-dragon-queen";

    /// `{3}{R}`, Sorcery, Entwine `{2}{R}`, `Completeness::Complete` by derive
    /// (`crates/card-defs/src/defs/goblin_war_party.rs`). Modal, `min_modes: 1,
    /// max_modes: 1` — mode 0 creates three 1/1 red Goblin tokens, mode 1 gives
    /// creatures you control +1/+1 and haste until end of turn. Paying entwine
    /// (CR 702.42a) chooses BOTH, and because the modes execute in order the tokens
    /// are on the battlefield in time to be pumped — which is what makes "did the
    /// second mode run" readable off the board rather than argued from the cost.
    const DX29_ENTWINE_SPELL: &str = "goblin-war-party";
    const DX29_ENTWINE_SPELL_NAME: &str = "Goblin War Party";
    /// The token mode 0 creates.
    const DX29_GOBLIN_TOKEN_NAME: &str = "Goblin";

    /// CR 903.5c: [`DX29_RED_COMMANDER`] plus 99 Mountains, `overrides` written over
    /// the named positions. [`dx29_island_deck_with`]'s twin; see
    /// [`dx29_train_of_thought_deck`] for why position 0 is in the opening hand.
    fn dx29_mountain_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("mountain".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX29_RED_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Out-of-band oracle: the LAYER-RESOLVED power of every battlefield permanent
    /// `controller` controls whose rendered name is `name`.
    ///
    /// Layer-resolved rather than printed, deliberately: mode 1's `+1/+1` is a
    /// continuous effect (CR 613.4c), so `obj.characteristics.power` would report the
    /// printed 1 whether or not the mode ran, and the probe would pass either way.
    fn dx29_resolved_powers_by_name(
        state: &SharedState,
        controller: mtg_engine::PlayerId,
        name: &str,
    ) -> Vec<i32> {
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
                if obj.controller != controller || obj.characteristics.name != name {
                    return None;
                }
                let chars = mtg_engine::rules::layers::calculate_characteristics(gs, id)
                    .unwrap_or_else(|| obj.characteristics.clone());
                chars.power
            })
            .collect()
    }

    /// **CR 702.42a — Entwine, offered as a MARKER and PAID over HTTP.**
    ///
    /// The marker family's end-to-end probe, and the only place a bare-string
    /// `template` reaches an actual HTTP response. Same three beats as the Replicate
    /// probe: read the descriptor, submit an ILLEGAL answer and require 400
    /// `bad_params`, then submit the NON-DEFAULT answer (checking the marker — the
    /// decline here is literally sending nothing) and verify the board.
    ///
    /// The verification is deliberately two-sided, because either half alone is
    /// ambiguous: three Goblin tokens prove mode 0 ran, and they prove nothing about
    /// entwine, since mode 0 is also `spell_default_modes`' own fallback pick. The
    /// tokens' layer-resolved POWER is what proves mode 1 ran as well — i.e. that
    /// the marker was read (CR 702.42b: an entwined modal spell executes every mode).
    ///
    /// # The two halves are keyed on DIFFERENT things, and the revert matrix found it
    ///
    /// Row R20 set `casting.rs`'s `entwine_paid` to `false` expecting the tokens to
    /// come back 1/1. They came back **2/2** and only the mana-pool assertion
    /// reddened, because `resolution.rs` does not read that flag at all: it
    /// re-derives the decision by scanning `stack_obj.additional_costs` for
    /// `AdditionalCost::Entwine` (`resolution.rs`'s `stack_entwine_paid`). So the
    /// CHARGE is decided by `casting.rs`'s validated flag and the EFFECT by an
    /// independent rescan of the announced list. They agree today only because
    /// `casting.rs` errors out before a stack object exists when the spell has no
    /// entwine — a latent duplication, not a live defect, but the reason this probe
    /// asserts the pool total AND the board rather than treating either as a proxy
    /// for the other. Row R20b (`stack_entwine_paid && false`) is what actually
    /// reddens the power assertion, at `got [1, 1, 1]`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx29_entwine_is_offered_as_a_marker_and_paid_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let state = shared_state();
        ui2_install(
            &state,
            dx29_mountain_deck_with(&[(0, DX29_ENTWINE_SPELL)]),
            dx29_mountain_deck_with(&[]),
        );

        // 7 Mountains: base {3}{R} (MV 4) + Entwine {2}{R} (MV 3) = 7, exactly.
        let view = ui2_drive_playing_lands(&state, 7, UI2_LAND_DRIVE_MAX_STEPS).await;
        let cast_label = format!("Cast {DX29_ENTWINE_SPELL_NAME}");
        let action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!("{DX29_ENTWINE_SPELL_NAME} must be offered with 7 Mountains out: {view}")
            });
        let index = action["index"].as_u64().expect("index is a number");

        // ── the descriptor ────────────────────────────────────────────────────
        let costs = &action["costs"];
        assert!(
            !costs.is_null(),
            "an Entwine spell must carry a costs descriptor: {action}"
        );
        assert_eq!(costs["answer_field"], "additional_costs");
        assert_eq!(
            costs["counts"].as_array().map(|a| a.len()),
            Some(0),
            "Goblin War Party has no pay-N-times rider, and `counts` must still be \
             present-and-empty: {costs}"
        );
        let markers = costs["markers"].as_array().expect("markers is an array");
        assert_eq!(markers.len(), 1, "exactly one marker rider: {markers:?}");
        let entwine = &markers[0];
        assert_eq!(entwine["kind"], "Entwine");
        // Goblin War Party prints "Entwine {2}{R}"; `format_mana_cost_compact` emits
        // the generic component first.
        assert_eq!(entwine["cost_label"], "{2}{R}");
        // **The wire fact this batch exists to document, observed on a real HTTP
        // response**: the template is a bare JSON STRING, because
        // `AdditionalCost::Entwine` is a unit variant and serde's externally-tagged
        // encoding renders unit variants as strings. A picker that cloned it and
        // wrote `Object.keys(entry)[0]` would throw here.
        assert_eq!(entwine["template"], json!("Entwine"));
        assert!(
            entwine["template"].is_string(),
            "a marker template is a bare string, never an object: {entwine}"
        );
        assert!(
            entwine.get("count_key").is_none() && entwine.get("ids_key").is_none(),
            "a marker view names no fill key: {entwine}"
        );

        let wire_seq = seq(&view);

        // ── the illegal answer: a marker this offer never carried ─────────────
        //
        // Fuse rather than a malformed Entwine, because it is the same wire SHAPE
        // (a bare string in the same array) and differs only in being an
        // announcement the offer never made — so the 400 cannot be attributed to
        // the encoding.
        let (status, refused) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": ["Fuse"]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{refused}");
        assert_eq!(refused["kind"], "bad_params", "{refused}");

        // ── the real, NON-DEFAULT answer ──────────────────────────────────────
        assert!(
            dx29_resolved_powers_by_name(&state, p1, DX29_GOBLIN_TOKEN_NAME).is_empty(),
            "sanity: no Goblin token exists yet"
        );
        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": ["Entwine"]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        ui2_drain_stack(&state, after_cast, 60).await;

        // ── the result, read out of band and BY NAME (CR 400.7) ───────────────
        let powers = dx29_resolved_powers_by_name(&state, p1, DX29_GOBLIN_TOKEN_NAME);
        assert_eq!(
            powers.len(),
            3,
            "CR 702.42b: mode 0 creates three 1/1 red Goblin tokens. got {powers:?}"
        );
        assert!(
            powers.iter().all(|p| *p == 2),
            "CR 702.42b / CR 613.4c: entwine chooses BOTH modes, so mode 1's +1/+1 \
             must also have applied -- a 1/1 here would mean only the default mode \
             ran and the marker was announced but never read. got {powers:?}"
        );

        let graveyard = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard.contains(&DX29_ENTWINE_SPELL_NAME.to_string()),
            "the resolved sorcery itself must be in the graveyard: {graveyard:?}"
        );
        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "all 7 available mana must have been spent -- base {{3}}{{R}} plus \
             Entwine {{2}}{{R}}; a leftover pool would mean the marker was announced \
             and never charged"
        );
    }

    /// **CR 702.42a / SR-38 — the INVERTED form of a deviation pin, and the inversion
    /// happened inside the same batch.**
    ///
    /// This test was written wrong-way-round, asserting the behaviour PB-DX29's first
    /// draft shipped: a marker rider offered with **no affordability bound**, so paying
    /// an unaffordable one came back as a bare **422** from the engine. Its own message
    /// instructed a successor batch to invert it. There was no successor — the defect was
    /// this batch's own, so this batch fixed it and inverted the test.
    ///
    /// **Why it was wrong.** Every other mana-bearing rider carries a bound the client is
    /// held to: `SquadCostOption::max_count` and `CountCostOption::max_count` (Replicate,
    /// Escalate) come from `repeated_cost_max_count`, and
    /// `validate_additional_cost_params` turns exceeding one into a 400 before any command
    /// reaches the engine. `MarkerCostOption` had no such field, so Entwine / Offspring /
    /// Fuse were offered whenever the spell's BASE cost was affordable.
    ///
    /// `SpliceCostOption` is still unbounded, and its own doc gives the reason: bounding it
    /// is a subset-sum over `eligible`, because each spliced card costs a different amount.
    /// **That reason does not extend to a marker** — each is a single yes/no payment, so
    /// the bound is one `can_afford(base + rider)` call, which is what
    /// `marker_rider_is_affordable` now does. Fuse included: CR 702.102b makes its cost the
    /// two halves summed, and `effective_cast_cost_with_additional`'s Fuse arm computes
    /// exactly that.
    ///
    /// Measured, not argued, in both directions: with 4 Mountains — Goblin War Party's base
    /// `{3}{R}` affordable, its Entwine `{2}{R}` not — the offer now carries
    /// `affordable: false` and paying it is refused at the **400** boundary naming the
    /// offer, instead of by the engine with `"player does not have enough mana"`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx29_an_unaffordable_entwine_is_offered_disabled_and_refused_at_400() {
        let state = shared_state();
        ui2_install(
            &state,
            dx29_mountain_deck_with(&[(0, DX29_ENTWINE_SPELL)]),
            dx29_mountain_deck_with(&[]),
        );

        // 4 Mountains: enough for the base {3}{R} (MV 4), not for the Entwine
        // {2}{R} (MV 3) on top of it.
        let view = ui2_drive_playing_lands(&state, 4, UI2_LAND_DRIVE_MAX_STEPS).await;
        let cast_label = format!("Cast {DX29_ENTWINE_SPELL_NAME}");
        let action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!("the base cost is affordable, so the cast must be offered: {view}")
            });

        // Half 1: the rider is still SHOWN -- `affordable: false` is the marker analogue
        // of `max_count: 0`, not a suppression. A human is told the rider exists and is
        // not payable right now, which is strictly more information than an absence.
        // Non-vacuity: `counts` is empty here, so this is genuinely the marker family and
        // not a count rider whose `max_count` would have carried the bound instead.
        assert!(
            action["costs"]["counts"]
                .as_array()
                .is_none_or(|c| c.is_empty()),
            "non-vacuity: this must be the MARKER family, not a count rider: {action}"
        );
        let markers = action["costs"]["markers"]
            .as_array()
            .expect("markers is an array");
        assert_eq!(markers.len(), 1, "{action}");
        assert_eq!(markers[0]["kind"], "Entwine");
        assert_eq!(
            markers[0]["affordable"], false,
            "SR-38: 4 Mountains pay the base {{3}}{{R}} and cannot also pay Entwine \
             {{2}}{{R}}, so the offer must say so rather than rendering a tickable box. \
             marker: {}",
            markers[0]
        );

        // Half 2: submitting it anyway is refused at the 400 boundary, naming the offer --
        // NOT by the engine as a bare 422. That difference is the whole point: a 400 says
        // "your answer contradicts the payload you are holding", which needs no game state
        // to see; a 422 says "the engine looked at your command and said no", which is what
        // an offer the server should never have made looks like.
        let index = action["index"].as_u64().expect("index is a number");
        let wire_seq = seq(&view);
        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"additional_costs": ["Entwine"]}
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "SR-38: an unaffordable marker must be refused by \
             `validate_additional_cost_params` with a 400, exactly like an \
             over-`max_count` Replicate. A 422 here means the affordability bound was \
             removed and the server is again offering something it will not accept. \
             body: {body}"
        );
        assert_eq!(body["kind"], "bad_params", "{body}");
    }

    // ═══════════════════════════════════════════════════════════════════
    // PB-DX44 — the casts you cannot make: Spree mode costs (`OOS-DX29-14`),
    // the split-card right half (`OOS-DX29-9`), and Fuse's own target slots
    // (`OOS-DX29-12`), each driven through the SAME HTTP channel a browser
    // client uses.
    // ═══════════════════════════════════════════════════════════════════

    const DX44_SPREE_SPELL: &str = "insatiable-avarice";
    const DX44_SPREE_SPELL_NAME: &str = "Insatiable Avarice";
    /// Mono-black legendary creature, `Complete` -- covers Insatiable
    /// Avarice's `{B}` color identity (CR 903.4). Already proven legal as a
    /// commander fixture ([`UI1_COMMANDER`] / [`SIM6_COMMANDER`] both use it).
    const DX44_BLACK_COMMANDER: &str = "razaketh-the-foulblooded";

    const DX44_SPLIT_CARD: &str = "turn";
    const DX44_SPLIT_CARD_NAME: &str = "Turn // Burn";
    /// UR legendary creature, `Complete` by derive -- covers `Turn // Burn`'s
    /// {U}{R} color identity (CR 903.4 counts BOTH halves' mana symbols, so
    /// the mono-red [`DX29_RED_COMMANDER`] cannot legally include this card
    /// even though the RIGHT half alone is mono-red).
    const DX44_UR_COMMANDER: &str = "niv-mizzet-the-firemind";

    /// PB-DX44: [`DX44_BLACK_COMMANDER`] plus 99 Swamps, `overrides` written
    /// over specific indices -- the black-mana sibling of
    /// [`dx29_mountain_deck_with`].
    fn dx44_swamp_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = (0..99).map(|_| CardId("swamp".to_string())).collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX44_BLACK_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// PB-DX44: [`DX44_UR_COMMANDER`] plus alternating Mountain/Island,
    /// `overrides` written over specific indices -- needed only by the
    /// right-half `Turn // Burn` probe, whose CARD (not its castable right
    /// half alone) has a two-colour identity.
    fn dx44_mountain_island_deck_with(overrides: &[(usize, &str)]) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let lands = ["mountain", "island"];
        let mut main_deck: Vec<CardId> = (0..99)
            .map(|i| CardId(lands[i % lands.len()].to_string()))
            .collect();
        for (index, card) in overrides {
            main_deck[*index] = CardId(card.to_string());
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX44_UR_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Out-of-band oracle: `player`'s current life total.
    fn dx44_life_total(state: &SharedState, player: mtg_engine::PlayerId) -> i32 {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .player(player)
            .expect("player exists")
            .life_total
    }

    /// Out-of-band oracle: how many cards are in `player`'s hand right now.
    fn dx44_hand_count(state: &SharedState, player: mtg_engine::PlayerId) -> usize {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .zones()
            .get(&mtg_engine::ZoneId::Hand(player))
            .map(|z| z.object_ids().len())
            .unwrap_or(0)
    }

    /// **JOB 4.1 — the Spree channel, over HTTP, with a NON-DEFAULT mode
    /// selection.** `spell_default_modes` picks the FIRST `min_modes`
    /// indices -- mode 0 ("+{2}": search library) for Insatiable Avarice --
    /// so choosing mode 1 alone ("+{B}{B}": target player draws three and
    /// loses 3 life) is an answer neither a bot nor an unparameterized
    /// client would ever submit. Asserted on the RESOLUTION EFFECT (P2's
    /// hand size and life total), not on the offer.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx44_spree_mode_is_offered_and_resolved_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let state = shared_state();
        ui2_install(
            &state,
            dx44_swamp_deck_with(&[(0, DX44_SPREE_SPELL)]),
            dx29_mountain_deck_with(&[]),
        );

        // Base {B} + mode 1's {B}{B} = {B}{B}{B}, exactly 3 Swamps.
        let view = ui2_drive_playing_lands(&state, 3, UI2_LAND_DRIVE_MAX_STEPS).await;
        let cast_label = format!("Cast {DX44_SPREE_SPELL_NAME}");
        let action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!("{DX44_SPREE_SPELL_NAME} must be offered with 3 Swamps out: {view}")
            });
        let index = action["index"].as_u64().expect("index is a number");
        assert_eq!(
            action["modes"].as_array().map(|a| a.len()),
            Some(2),
            "Insatiable Avarice declares two Spree modes: {action}"
        );

        let wire_seq = seq(&view);
        let p2_hand_before = dx44_hand_count(&state, p2);
        let p2_life_before = dx44_life_total(&state, p2);

        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {
                    "modes_chosen": [1],
                    "targets": [{"Player": p2.0}]
                }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        ui2_drain_stack(&state, after_cast, 60).await;

        assert_eq!(
            dx44_hand_count(&state, p2),
            p2_hand_before + 3,
            "CR 702.172a mode 1: target player draws 3 cards"
        );
        assert_eq!(
            dx44_life_total(&state, p2),
            p2_life_before - 3,
            "CR 702.172a mode 1: target player loses 3 life"
        );
        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "all 3 Swamps must have been spent -- base {{B}} plus mode 1's {{B}}{{B}}"
        );
    }

    /// **JOB 4.3 (right half, browser channel)** — `Turn // Burn`'s right
    /// half (Burn, `{1}{R}`) is offered and resolved over HTTP as its OWN
    /// action, distinct from the ordinary (left-half, `{2}{U}`) cast.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx44_right_half_cast_is_offered_and_resolved_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let state = shared_state();
        ui2_install(
            &state,
            dx44_mountain_island_deck_with(&[(0, DX44_SPLIT_CARD)]),
            dx29_mountain_deck_with(&[]),
        );

        // Burn's own cost {1}{R} needs at least one Mountain. The deck is a
        // strict Mountain/Island ALTERNATION pre-shuffle, but CR 103.3's real
        // shuffle (seeded by `UI2_SEED`) does not preserve that order --
        // empirically, at this seed, a Mountain is not drawn until the 5th
        // land; 5 is pinned rather than 2 for exactly that reason (mirrors
        // this file's own `UI3_SPLIT_COMBAT_SEED` precedent: re-observe
        // rather than assume, and say so where the number is chosen).
        let view = ui2_drive_playing_lands(&state, 5, UI2_LAND_DRIVE_MAX_STEPS).await;
        let cast_label = format!("Cast {DX44_SPLIT_CARD_NAME} (right half only)");
        let action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "{DX44_SPLIT_CARD_NAME}'s right-half offer must be present with 2 \
                     Mountains out: {view}"
                )
            });
        let index = action["index"].as_u64().expect("index is a number");
        assert_eq!(
            action["target_min"], 1,
            "Burn (right half alone) declares exactly one target: {action}"
        );
        assert_eq!(
            action["target_max"], 1,
            "Burn (right half alone) declares exactly one target: {action}"
        );

        let wire_seq = seq(&view);
        let p2_life_before = dx44_life_total(&state, p2);

        let (status, after_cast) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"targets": [{"Player": p2.0}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_cast}");

        ui2_drain_stack(&state, after_cast, 60).await;

        assert_eq!(
            dx44_life_total(&state, p2),
            p2_life_before - 2,
            "Burn deals 2 damage to any target -- the right half's own effect \
             must have resolved on the announced target"
        );
        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "Burn's own cost ({{1}}{{R}}) must be charged exactly, not Turn's \
             ({{2}}{{U}})"
        );
        let graveyard = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard.contains(&DX44_SPLIT_CARD_NAME.to_string()),
            "the resolved card must be in the graveyard: {graveyard:?}"
        );
    }

    /// **JOB 4.4 — the fused target-slot regression, over the CHANNEL.**
    ///
    /// Stage 1 left a hole: `ActionOptionView.target_slots` was the UN-fused
    /// list even when the option ALSO offered a Fuse marker, so a human who
    /// ticked Fuse in `CostPicker` was then asked for the wrong number of
    /// targets in `TargetPicker` -- a clean offer followed by a guaranteed
    /// 422 (SR-38). This probe is a DIFFERENTIAL over the DTO the browser
    /// actually receives (`ActionOptionView.fused_target_slots`), not over
    /// two arguments of one function call -- PB-DX20's lesson, cited in
    /// Stage 2b's own execution notes: a differential between two arguments
    /// of `spell_target_requirements` proves the function, not the channel.
    /// It then proves the CHANNEL's count is exactly what the ENGINE accepts
    /// by casting for real with that many targets.
    #[test]
    fn test_dx44_fused_target_slots_match_what_the_engine_actually_accepts() {
        use mtg_view_model::{StateViewModel, Viewer};

        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let defs: std::collections::HashMap<String, mtg_engine::CardDefinition> =
            mtg_engine::all_cards()
                .into_iter()
                .map(|d| (d.name.clone(), d))
                .collect();
        let card_def = |name: &str| {
            defs.get(name)
                .unwrap_or_else(|| panic!("{name:?} not in all_cards()"))
        };
        let obj = |owner, name: &str, zone| {
            mtg_engine::enrich_spec_from_def(
                mtg_engine::ObjectSpec::card(owner, name)
                    .with_card_id(card_def(name).card_id.clone())
                    .in_zone(zone),
                &defs,
            )
        };
        let state = mtg_engine::GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(mtg_simulator::build_registry())
            .active_player(p1)
            .at_step(mtg_engine::Step::PreCombatMain)
            .player_mana(
                p1,
                mtg_engine::ManaPool {
                    red: 1,
                    white: 1,
                    colorless: 1,
                    ..Default::default()
                },
            )
            .object(obj(p1, "Wear // Tear", mtg_engine::ZoneId::Hand(p1)))
            .object(
                mtg_engine::ObjectSpec::artifact(p2, "Doomed Artifact")
                    .in_zone(mtg_engine::ZoneId::Battlefield),
            )
            .object(
                mtg_engine::ObjectSpec::enchantment(p2, "Doomed Enchantment")
                    .in_zone(mtg_engine::ZoneId::Battlefield),
            )
            .build()
            .expect("state builds");

        use mtg_simulator::LegalActionProvider as _;
        let actions = mtg_simulator::StubProvider.legal_actions(&state, p1);
        let card_id = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "Wear // Tear")
            .map(|(id, _)| *id)
            .expect("Wear // Tear must be in the built state");
        let index = actions
            .iter()
            .position(|a| {
                matches!(
                    a,
                    mtg_simulator::LegalAction::CastSpell {
                        card,
                        alt_cost: None,
                        ..
                    } if *card == card_id
                )
            })
            .unwrap_or_else(|| panic!("the ordinary cast must be offered: {actions:?}"));

        let decision = mtg_simulator::PendingDecision {
            seq: 1,
            player: p1,
            kind: mtg_simulator::DecisionKind::Priority,
            actions,
        };
        let player_names: HashMap<mtg_engine::PlayerId, String> = HashMap::new();
        let state_view =
            StateViewModel::from_game_state_for(&state, &player_names, Viewer::Seat(p1));
        let names = crate::view::NameIndex::from_view(&state_view);
        let dview = crate::view::decision_view(&decision, 1, &state, &names, &player_names);
        let option = &dview.actions[index];

        // The CHANNEL's own count -- this is what Stage 1 left wrong.
        assert_eq!(
            option.fused_target_slots.len(),
            2,
            "the wire DTO must carry TWO fused target slots (Wear + Tear): {:?}",
            option.fused_target_slots
        );
        assert_eq!(
            option.fused_target_min, 2,
            "{:?}",
            option.fused_target_slots
        );
        assert_eq!(
            option.fused_target_max, 2,
            "{:?}",
            option.fused_target_slots
        );

        // Now let the ENGINE be the arbiter: cast for real with exactly that
        // many targets, paying the fused cost. If the channel's count ever
        // disagreed with what `casting.rs` accepts, THIS is what would go red
        // -- not a second call to the same query function.
        let artifact_id = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "Doomed Artifact")
            .map(|(id, _)| *id)
            .expect("artifact exists");
        let enchantment_id = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "Doomed Enchantment")
            .map(|(id, _)| *id)
            .expect("enchantment exists");
        let card = card_id;
        mtg_engine::process_command(
            state,
            mtg_engine::Command::CastSpell(Box::new(mtg_engine::rules::command::CastSpellData {
                player: p1,
                card,
                targets: vec![
                    mtg_engine::Target::Object(artifact_id),
                    mtg_engine::Target::Object(enchantment_id),
                ],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![mtg_engine::AdditionalCost::Fuse],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap_or_else(|e| {
            panic!(
                "the engine must accept a fused cast announcing exactly the \
                 CHANNEL's own fused_target_slots count (2): {e:?}"
            )
        });
    }

    /// **PB-DX44 `/review` finding 1 (HIGH) — the browser half of this batch was
    /// ungated.**
    ///
    /// Two lines in `ActionBar.svelte` are load-bearing and nothing pinned either:
    ///
    /// 1. `resolvedTargetSlots`/`resolvedTargetRange` must open with the
    ///    Fuse-first branch (Stage 1's execution notes §4.3) — deleting it does
    ///    not fail a single test, and it recreates the EXACT SR-38 defect PB-DX44
    ///    exists to delete: a human ticks Fuse in `CostPicker`, and `TargetPicker`
    ///    then asks for the UN-fused slot count. Clean offer, server 422.
    /// 2. `pitch={activeOption.costs.pitch}` must be threaded into `CostPicker` —
    ///    `test_frontend_cost_picker_never_fills_a_unit_variant_marker_template`
    ///    proves the COMPONENT handles a `pitch` prop correctly, but nothing
    ///    proved `ActionBar` ever GIVES it one. Dropping the prop is silent:
    ///    `CostPicker` renders with `pitch: null`, its `{#if pitch}` branch never
    ///    opens, and `merge_required_additional_costs` (server-side) silently
    ///    substitutes `plan.pitch.default` — a human never chooses which card to
    ///    pitch, which is the acceptance criterion's own "NON-DEFAULT pitched
    ///    card" requirement made unreachable from the browser with everything
    ///    green.
    ///
    /// Source-level, for the standing reason this file states everywhere else —
    /// there is no frontend test harness (plan §8 R7). Both needles below are
    /// checked against `ActionBar.svelte`'s own text as returned by
    /// `collect_frontend_files(frontend_src, ..)`, and `ActionBar.svelte` lives
    /// directly under `frontend/src/lib/` — inside that walk's root, not behind
    /// the `$viewer` alias `vite.config.js` resolves to a sibling tool's source
    /// tree (the UI-4 gap: a needle satisfied from a file the walk never visits).
    /// Both functions this test pins are defined in `ActionBar.svelte` itself,
    /// not imported from `$viewer`, so that gap does not apply here.
    #[test]
    fn test_frontend_action_bar_keeps_the_fused_slot_and_pitch_wiring() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);
        let action_bar = sources
            .iter()
            .find(|(p, _)| p.ends_with("ActionBar.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ActionBar.svelte is in the frontend walk");

        // (1) The Fuse-first branch, counted rather than merely found: this
        //     exact string (with the `if (` prefix) appears ONLY in the two
        //     branches themselves — the doc comments above each function quote
        //     `option.fused_target_slots` alone, never the full condition, so a
        //     doc-comment-satisfiable false pass is not possible here.
        const FUSE_BRANCH: &str =
            "if (isFusedCast(paramsSoFar) && (option.fused_target_slots?.length ?? 0) > 0) {";
        assert_eq!(
            action_bar.matches(FUSE_BRANCH).count(),
            2,
            "`ActionBar.svelte` must open BOTH `resolvedTargetSlots` and `resolvedTargetRange` \
             with the Fuse-first branch ({FUSE_BRANCH:?}). Losing either one reopens the exact \
             SR-38 defect PB-DX44 Stage 2b closed (execution notes §4.3): a human ticks Fuse in \
             `CostPicker`, and `TargetPicker` then asks for the wrong (UN-fused) target count."
        );

        // (2) The pitch prop, threaded exactly like every other `costs.*` prop
        //     `CostPicker` is given.
        assert!(
            action_bar.contains("pitch={activeOption.costs.pitch}"),
            "`ActionBar` never threads `costs.pitch` into `CostPicker` — the pitch stage \
             renders with no candidates and the server silently substitutes the default \
             exiled card, making CR 118.9's choice unreachable from the browser"
        );
    }

    const DX44_PITCH_SPELL: &str = "force-of-will";
    const DX44_PITCH_SPELL_NAME: &str = "Force of Will";
    const DX44_PITCH_VICTIM: &str = "lightning-bolt";
    const DX44_PITCH_VICTIM_NAME: &str = "Lightning Bolt";
    /// [`DX44_PITCH_SPELL`]'s two pitch-eligible blue cards -- neither is ever
    /// cast, only exiled. `eligible[0]` (lower `ObjectId`, built/drawn first)
    /// is the provider's own default; this probe deliberately answers with
    /// the OTHER one, mirroring `pb_dx44_pitch_channel.rs`'s T1.
    const DX44_PITCH_DEFAULT_CANDIDATE: &str = "brainstorm";
    const DX44_PITCH_DEFAULT_CANDIDATE_NAME: &str = "Brainstorm";
    const DX44_PITCH_NON_DEFAULT_CANDIDATE: &str = "counterspell";
    const DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME: &str = "Counterspell";

    /// Drive P1 -- playing a land, else passing priority -- until ALL FOUR of
    /// [`DX44_PITCH_VICTIM_NAME`]/[`DX44_PITCH_SPELL_NAME`]/
    /// [`DX44_PITCH_DEFAULT_CANDIDATE_NAME`]/[`DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME`]
    /// are in hand AND at least one Mountain is on the battlefield.
    ///
    /// `ui2_drive_playing_lands`'s land-COUNT contract (used by the right-half
    /// probe above) is not reused here: that contract only pins WHEN the
    /// target's own land type is drawn, and this fixture additionally needs
    /// two DISTINCT other non-land cards in hand at the same moment, which
    /// [`UI2_SEED`]'s pinned-position doc (`ui2_lifes_legacy_two_elves_deck`)
    /// only maps out for the forest-based [`UI2_COMMANDER`] deck, not this
    /// mountain/island one -- so this drives on the STATE itself (never
    /// casting anything, never tapping), rather than assuming a position.
    async fn dx44_drive_until_pitch_fixture_ready(state: &SharedState, max_steps: usize) -> Value {
        let p1 = mtg_engine::PlayerId(1);
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            let hand = ui2_zone_names(state, mtg_engine::ZoneId::Hand(p1));
            let ready = [
                DX44_PITCH_VICTIM_NAME,
                DX44_PITCH_SPELL_NAME,
                DX44_PITCH_DEFAULT_CANDIDATE_NAME,
                DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME,
            ]
            .iter()
            .all(|name| hand.contains(&name.to_string()))
                && ui2_battlefield_count_by_name(state, p1, "Mountain") > 0;
            if ready {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before the pitch fixture (all four cards in \
                 hand, a Mountain in play) was ready: {view}"
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
        panic!("the pitch fixture was never ready within {max_steps} steps: {view}");
    }

    /// **`/review` finding 4 (MEDIUM) — JOB 4.5 (pitch, browser channel).**
    ///
    /// Spree and the split-card right half each got a full HTTP drive (the two
    /// tests above); the pitch channel had only a 400-boundary unit test on a
    /// synthetic action, so the chain `additional_costs_view` -> `PitchCostView`
    /// -> `CostPicker` render -> submit -> engine was never exercised end to end
    /// and `view.rs`'s `pitch_prompt`/`color_word` had no coverage at all.
    ///
    /// Force of Will is offered and resolved over HTTP as a SEPARATE pitch
    /// action (`AltCostKind::Pitch`), paying CR 118.9's alternative cost (1
    /// life + a NON-DEFAULT exiled blue card) rather than its printed
    /// `{3}{U}{U}`. Real deck, real HTTP router, no direct engine construction
    /// -- mirroring the right-half probe above exactly.
    ///
    /// P1 casts Lightning Bolt (targeting P2) first and, per CR 117.3c
    /// (PB-DP1: the actor keeps priority), is offered a fresh decision in
    /// response to their OWN spell -- no bot cooperation needed to put a
    /// counterable target on the stack.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx44_pitch_cast_is_offered_and_resolved_over_http() {
        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let state = shared_state();
        ui2_install(
            &state,
            dx44_mountain_island_deck_with(&[
                (0, DX44_PITCH_VICTIM),
                (1, DX44_PITCH_SPELL),
                // [`UI2_SEED`]'s pinned positions for THIS deck arrangement (mountain/
                // island, [`DX44_UR_COMMANDER`]) are NOT the forest-deck ones
                // `ui2_lifes_legacy_two_elves_deck` documents -- re-measured for this
                // arrangement by a throwaway library-order dump (never committed): 19
                // is in the OPENING hand and 22 is the very TOP of the post-shuffle
                // library (drawn on the first draw step), so both candidates are in
                // hand well within [`dx44_drive_until_pitch_fixture_ready`]'s budget.
                (19, DX44_PITCH_DEFAULT_CANDIDATE),
                (22, DX44_PITCH_NON_DEFAULT_CANDIDATE),
            ]),
            dx29_mountain_deck_with(&[]),
        );

        // Lightning Bolt's own {R} needs at least one Mountain, and the pitch
        // action needs the two candidate blue cards actually in hand.
        let view = dx44_drive_until_pitch_fixture_ready(&state, UI2_LAND_DRIVE_MAX_STEPS).await;

        let cast_bolt_label = format!("Cast {DX44_PITCH_VICTIM_NAME}");
        let bolt_action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == cast_bolt_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "{DX44_PITCH_VICTIM_NAME}'s ordinary offer must be present with a \
                     Mountain out: {view}"
                )
            });
        let bolt_index = bolt_action["index"].as_u64().expect("index is a number");

        let wire_seq = seq(&view);
        let (status, after_bolt) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": bolt_index,
                "params": {"targets": [{"Player": p2.0}]}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_bolt}");

        // CR 117.3c (PB-DP1): the actor keeps priority after casting, so P1
        // is offered a NEW decision here -- still their own priority window,
        // in response to their OWN Lightning Bolt on the stack.
        let view = after_bolt;
        let pitch_label = format!("Cast {DX44_PITCH_SPELL_NAME} via its pitch cost");
        let pitch_action = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["kind"] == "CastSpell" && a["label"] == pitch_label.as_str())
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "{DX44_PITCH_SPELL_NAME}'s pitch offer must be present in response to \
                     P1's own Lightning Bolt: {view}"
                )
            });
        let pitch_index = pitch_action["index"].as_u64().expect("index is a number");

        // The served `PitchCostView` itself -- candidates and prompt, not just
        // presence. Both blue candidates must be named (by id, via `NameIndex`).
        let costs = &pitch_action["costs"];
        assert_eq!(costs["answer_field"], "additional_costs", "{pitch_action}");
        let pitch_view = &costs["pitch"];
        assert!(
            !pitch_view.is_null(),
            "the pitch offer must carry its own `PitchCostView`: {pitch_action}"
        );
        assert!(
            pitch_view["prompt"]
                .as_str()
                .expect("prompt is a string")
                .contains("CR 118.9"),
            "the served prompt must cite CR 118.9: {pitch_view}"
        );
        let candidate_labels: Vec<String> = pitch_view["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["label"].as_str().expect("label is a string").to_string())
            .collect();
        let mut sorted_labels = candidate_labels.clone();
        sorted_labels.sort();
        let mut expected_labels = vec![
            DX44_PITCH_DEFAULT_CANDIDATE_NAME.to_string(),
            DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME.to_string(),
        ];
        expected_labels.sort();
        assert_eq!(
            sorted_labels, expected_labels,
            "the served candidate list must name both eligible blue cards: {candidate_labels:?}"
        );

        let non_default_id = pitch_view["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .find(|c| c["label"] == DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME)
            .and_then(|c| c["id"].as_u64())
            .expect("Counterspell must be among the served candidates");

        // CR 400.7: casting Lightning Bolt minted it a FRESH `ObjectId` on the
        // move to the stack -- re-resolve by name and zone, mirroring T1's own
        // note in `pb_dx44_pitch_channel.rs`.
        let bolt_stack_id = {
            let guard = state.session.lock().expect("lock");
            let session = guard.as_ref().expect("a session is installed");
            session
                .game
                .state()
                .objects()
                .iter()
                .find(|(_, o)| {
                    o.characteristics.name == DX44_PITCH_VICTIM_NAME
                        && o.zone == mtg_engine::ZoneId::Stack
                })
                .map(|(id, _)| id.0)
                .expect("Lightning Bolt must be on the stack")
        };

        let p1_life_before = dx44_life_total(&state, p1);
        let wire_seq = seq(&view);
        let (status, after_pitch) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": pitch_index,
                "params": {
                    "targets": [{"Object": bolt_stack_id}],
                    "additional_costs": [{"ExileFromHand": {"card": non_default_id}}]
                }
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after_pitch}");

        // CR 119.4: exactly 1 life paid.
        assert_eq!(
            dx44_life_total(&state, p1),
            p1_life_before - 1,
            "CR 118.9: pitching Force of Will pays exactly 1 life"
        );
        // CR 118.9a: the printed {3}{U}{U} must never be charged.
        assert_eq!(
            ui2_mana_pool_total(&state, p1),
            0,
            "{DX44_PITCH_SPELL_NAME}'s printed mana cost must never be paid"
        );

        // The NON-DEFAULT (Counterspell) card, and only it, is exiled. The
        // DEFAULT (Brainstorm) must still be in hand -- never touched.
        let exile_names = ui2_zone_names(&state, mtg_engine::ZoneId::Exile);
        assert_eq!(
            exile_names,
            vec![DX44_PITCH_NON_DEFAULT_CANDIDATE_NAME.to_string()],
            "exactly the chosen (non-default) blue card must be exiled: {exile_names:?}"
        );
        let hand_names = ui2_zone_names(&state, mtg_engine::ZoneId::Hand(p1));
        assert!(
            hand_names.contains(&DX44_PITCH_DEFAULT_CANDIDATE_NAME.to_string()),
            "the default (never chosen) card must still be in hand: {hand_names:?}"
        );

        ui2_drain_stack(&state, after_pitch, 60).await;

        // CR 608: resolution. Bolt is countered -- to its OWNER's graveyard
        // (CR 800.4a), which is P1: P1 both cast and owns Lightning Bolt (it
        // came from P1's own deck; P2 is merely the spell's TARGET).
        let graveyard = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard.contains(&DX44_PITCH_VICTIM_NAME.to_string()),
            "{DX44_PITCH_SPELL_NAME}'s CounterSpell must have sent {DX44_PITCH_VICTIM_NAME} \
             to its owner's graveyard: {graveyard:?}"
        );
    }

    // ── PB-DX15a (CR 608.2e / CR 101.4): APNAP order over a real HTTP round trip ──
    //
    // `scutemob-216`, the reachability half of `OOS-DP9-8`. The engine-side probe lives
    // in `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` and the offer-layer /
    // `LocalGame` probes in `crates/simulator/tests/pb_dx15a_apnap_channel.rs`; this is
    // the browser channel, driven through the real router.
    //
    // # THE fixture rule
    //
    // **A fixture whose active player is the LOWEST `PlayerId` cannot tell CR 608.2e
    // APNAP order from ascending `PlayerId` order**, because `GameStateBuilder` seeds
    // `turn_order` in `add_player` order (ascending everywhere in this tree), so
    // "rotate to start at the active player" is the identity. That is why `OOS-DP9-8`
    // survived behind a test whose doc said it pinned the deviation. `session::new_game`
    // always starts turn 1 with seat 1 active, so this fixture does not *build* a
    // non-lowest active player — it **drives the real game to turn 5**, where the active
    // player is seat 2, and asserts that fact before asserting anything about order.
    //
    // # What the play server can and cannot show, stated rather than glossed
    //
    // The HTTP surface exposes exactly ONE seat's questions, by construction and on
    // purpose: `seat_view` filters `pending.player == human` and `post_action` refuses a
    // submission whose `pending.player != play.human` (both cited in their own comments
    // above). So the *sequence of `PendingDecision::player`s* — which is what the
    // simulator-level `c1`/`c3` probes assert — is **not observable over HTTP**, and a
    // second human seat would not merely be unobservable but would deadlock the session.
    // This probe therefore does not claim to observe that sequence. It asserts the two
    // order-dependent facts that ARE observable here, both of which invert under
    // ascending `PlayerId` order:
    //
    // 1. **Which seat the server had already asked** when it handed the human its own
    //    question — `PlayerId(3)`, the APNAP-correct predecessor, and exactly one of
    //    them. Under ascending order the human is asked FIRST and this list is empty.
    //    Read through the same out-of-band oracle `ui1_zone` uses (the session's own
    //    `LocalGame`), never used to build a payload.
    // 2. **The order of the `CardDiscarded` lines in the HTTP `events` payload** —
    //    `["Bot-3", "Human-1"]`. Under ascending order it is the exact reversal.
    //
    // Both were executed red by reverting `resolve_player_target_list`'s
    // `EachPlayer`/`EachOpponent` arms to `state.players.keys()`.
    //
    // # Fixture choice
    //
    // `burglar_rat` — `{1}{B}` Creature — Rat, `Complete`, deck-legal: "When this
    // creature enters, each opponent discards a card." Its ETB is an
    // `Effect::ForEach { over: ForEachTarget::EachOpponent, .. }` around
    // `Effect::DiscardCards`, and `ForEach`'s player arm resolves through the same
    // `resolve_player_target_list` this batch rewired. Three seats, the human passive at
    // seat 1, the Rat in seat **2**'s deck alone — so the caster is the active player of
    // turn 5 and the two opponents are seats 3 and 1, whose APNAP order `[3, 1]` is the
    // exact reversal of ascending `[1, 3]`. A set assertion would not discriminate; an
    // ordered one does.

    /// **Read off a real run, not reasoned to** (the [`UI1_SEED`] / [`ENG1_SEED`]
    /// convention): at this seed [`DX15A_BURGLAR_RAT`] at `main_deck[0]` of seat 2's
    /// deck lands in that bot's OPENING hand, and the bot casts it on turn 5 with the
    /// human having done nothing but pass. A completeness flip in any card-def batch
    /// re-deals every seat and moves this; re-observe it off a real run rather than
    /// guessing.
    const DX15A_SEED: u64 = 7;

    /// CR 701.9b: "When this creature enters, each opponent discards a card."
    /// `{1}{B}`, `Complete`, and the only non-Swamp in seat 2's deck.
    const DX15A_BURGLAR_RAT: &str = "burglar-rat";

    /// The turn the drive reaches before the Rat resolves, and the whole reason this
    /// fixture can express the deviation: on turn 5 of a three-seat game the active
    /// player is seat 2, **not** the lowest `PlayerId`.
    const DX15A_EXPECTED_TURN: u32 = 5;

    /// Install the PB-DX15a fixture: three seats, human passive at seat 1, the Rat in
    /// seat 2's deck only. Built through `session::new_game` — the same constructor
    /// `post_game` uses, running the same two Invariant-9 gates — because
    /// `session::config_for` hard-codes two things this fixture must override
    /// (`DeckSource::RandomPerSeat`, and a player count taken from `NewGameDefaults`).
    /// Nothing about the HTTP path is stubbed; only the decks the game starts from.
    fn dx15a_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 3,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX15A_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), eng1_deck_with(&[])),
                (
                    mtg_engine::PlayerId(2),
                    eng1_deck_with(&[(0, DX15A_BURGLAR_RAT)]),
                ),
                (mtg_engine::PlayerId(3), eng1_deck_with(&[])),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX15a fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// The out-of-band oracle: the seats whose CR 608.2d questions the engine has
    /// already accepted an answer for, in application order. Read straight off the
    /// session's `LocalGame` journal — the same role `ui1_zone` plays, and used only to
    /// verify, never to build a payload.
    fn dx15a_answered_seats(state: &SharedState) -> Vec<u64> {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .journal()
            .iter()
            .filter_map(|record| match &record.command {
                mtg_engine::Command::AnswerEffectChoice { player, .. } => Some(player.0),
                _ => None,
            })
            .collect()
    }

    /// The active player of the session's current turn, and its turn number. Same
    /// out-of-band oracle; used to prove the fixture is *capable* of expressing the
    /// deviation before any order is asserted.
    fn dx15a_turn(state: &SharedState) -> (u32, u64) {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let turn = session.game.state().turn();
        (turn.turn_number, turn.active_player.0)
    }

    /// **CR 608.2e / CR 101.4 / CR 701.9b — "each opponent discards a card" is asked
    /// and resolved in APNAP order, over a real HTTP round trip.**
    ///
    /// See the block comment above for what this probe can and cannot observe, and why.
    /// The short version: the play server shows one seat's questions, so the evidence
    /// here is (a) which seat the server had already asked when it asked the human, and
    /// (b) the order of the resolution's own `CardDiscarded` lines in the payload. Both
    /// invert under the pre-PB-DX15a ascending-`PlayerId` walk.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx15a_each_opponent_discard_is_apnap_ordered_over_http() {
        let state = shared_state();
        dx15a_install(&state);

        let view = eng1_drive_pass_only(&state, "Discard", 400).await;

        // The fixture rule, checked on the REAL session before any order is asserted:
        // if the active player were seat 1 (the lowest id), APNAP and ascending
        // PlayerId would be the same list and everything below would be vacuous.
        let (turn, active) = dx15a_turn(&state);
        assert_eq!(
            (turn, active),
            (DX15A_EXPECTED_TURN, 2),
            "the drive must reach a turn whose active player is NOT the lowest \
             PlayerId, or this probe cannot tell APNAP from ascending PlayerId order"
        );

        // (1) The seat the server had ALREADY asked. CR 608.2e puts p3 (the active
        // player's first opponent in turn order) ahead of p1; ascending PlayerId puts
        // p1 first, in which case this list is EMPTY at this moment.
        assert_eq!(
            dx15a_answered_seats(&state),
            vec![3],
            "CR 608.2e / CR 101.4: with seat 2 active and casting, its opponents are \
             asked [3, 1]. So by the time the server hands the human (seat 1) its own \
             question, seat 3 has already answered exactly one. Ascending PlayerId -- \
             what the engine did before PB-DX15a -- asks seat 1 FIRST, leaving this \
             list empty."
        );

        let index = ui1_question_index(&view, "Discard").expect("just found");
        let option = view["decision"]["actions"]
            .as_array()
            .expect("actions is an array")
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let decision = &option["decision"];
        assert_eq!(decision["answer_field"], "effect_choice_answer");
        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "PickN");
        assert_eq!(
            answer["count"], 1,
            "Burglar Rat discards exactly one: {answer}"
        );

        // A NON-DEFAULT pick, so the resolution below distinguishes the human's answer
        // from the engine's fallback (`default_discard_answer` takes the LOWEST ids).
        let mut candidates: Vec<u64> = answer["candidates"]
            .as_array()
            .expect("candidates is an array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        candidates.sort_unstable();
        assert!(
            candidates.len() > 1,
            "the human must hold more than one card, or there is no choice to make: \
             {answer}"
        );
        let chosen = *candidates.last().expect("non-empty");
        assert_eq!(
            answer["template"],
            json!({"Discard": {"chosen": [candidates[0]]}}),
            "the offered default is the LOWEST ObjectId; the pick below is a different \
             card, so the hand check at the end is about the human's answer"
        );

        let wire_seq = seq(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": wire_seq,
                "action_index": index,
                "params": {"effect_choice_answer": {"Discard": {"chosen": [chosen]}}},
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{after}");

        // (2) The RESOLUTION's own events, straight off the wire payload.
        let discarders: Vec<String> = after["events"]
            .as_array()
            .expect("events is an array")
            .iter()
            .filter(|e| e["kind"] == "CardDiscarded")
            .map(|e| {
                e["player"]
                    .as_str()
                    .expect("a CardDiscarded line names its player")
                    .to_string()
            })
            .collect();
        assert_eq!(
            discarders,
            vec!["Bot-3".to_string(), "Human-1".to_string()],
            "CR 608.2e: the resolution applies each opponent's discard in APNAP order, \
             and the browser's own event feed shows it. Ascending PlayerId is the exact \
             reversal, [\"Human-1\", \"Bot-3\"]."
        );

        // ...and the human's non-default answer is what actually happened.
        let hand = ui1_hand(&state);
        assert!(
            !hand.contains(&chosen),
            "the card the human chose must be gone from hand: {hand:?}"
        );
        assert!(
            hand.contains(&candidates[0]),
            "the card the DEFAULT would have discarded must still be in hand, or this \
             probe is measuring the engine's fallback and not the human: {hand:?}"
        );
    }

    // ── PB-DX45 (`scutemob-217`, closing `OOS-DX24-9` == `OOS-DX27-5`) ────────
    //
    // CR 118.12 makes an optional cost a PLAYER decision, answered over PB-DP9's
    // CR 608.2d suspend-and-replay channel:
    // `EffectChoiceQuestion::PayOptionalCost { cost }` /
    // `EffectChoiceAnswer::PayOptionalCost { pay }`. The engine- and
    // simulator-level probes live in
    // `crates/engine/tests/primitives/pb_dx45_optional_cost.rs` and
    // `crates/simulator/tests/pb_dx45_optional_cost_channel.rs`; this section is
    // the play-server's own end of the same channel.
    //
    // **A real, reproducible gap was found while writing these, in a file this
    // task may not edit** -- see
    // `test_dx45_the_http_validator_currently_refuses_every_explicit_pay_
    // optional_cost_answer`'s doc immediately below the fixture helpers. `T1`
    // (the offer) is a genuine, passing, real-HTTP test. `T2`/`T3` (the
    // decline/accept) drive the SAME real-HTTP fixture up to the offer and then
    // answer through `PlaySession::submit` directly rather than through `POST
    // /api/game/action`, because that endpoint currently 400s on every explicit
    // `PayOptionalCost` answer -- their own doc comments say so again, so a
    // reader who only sees one of the two places still gets the caveat.

    /// [`DX45H_SEED`] reproduces the same "index 0 and index 1 land in the
    /// opening hand" property [`UI1_SEED`]'s doc describes, for a completely
    /// different deck -- the shuffle is a permutation of POSITIONS, so it does
    /// not depend on which cards occupy them. Confirmed by a real run, not
    /// assumed: at this seed p1's opening hand is `[Birthing Ritual, Forest x5,
    /// Arbor Elf]`.
    const DX45H_SEED: u64 = UI1_SEED;

    /// `{5}{G}{G}`, mono-green, unreachable inside this fixture's drive window --
    /// the same [`UI1_COMMANDER`] trick (fix the deck's colour identity with the
    /// most expensive card in it, and make sure neither seat can afford it).
    const DX45H_COMMANDER: &str = "old-gnawbone";

    /// `with_probe_cards` lets p2's deck skip both probe cards entirely, so the
    /// bot seat never has anything to decide with either one.
    ///
    /// Arbor Elf (`{G}`, no explicit `completeness:` field, `Complete` by the
    /// `#[default]` derive) satisfies Birthing Ritual's intervening-if ("if you
    /// control a creature") AND is the only sacrifice candidate once it is
    /// cast, with no combat step needed anywhere in the drive. Birthing Ritual
    /// (`{1}{G}`, `Completeness::Complete`) is the CR 118.12 question source:
    /// `AtBeginningOfYourEndStep` with `place_cost: Some(Cost::Sacrifice(..))`
    /// on `Effect::LookAtTopThenPlace` -- so the human's OWN end step, the same
    /// turn Birthing Ritual resolves, asks a genuine, reachable question.
    fn dx45h_deck(with_probe_cards: bool) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = Vec::new();
        if with_probe_cards {
            main_deck.push(CardId("arbor-elf".to_string()));
            main_deck.push(CardId("birthing-ritual".to_string()));
        }
        while main_deck.len() < 99 {
            main_deck.push(CardId("forest".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX45H_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install a two-player fixed-deck session through `session::new_game` --
    /// the `ui1_install` / `ui2_install` precedent. `POST /api/game` cannot
    /// build this fixture: `session::config_for` hard-codes
    /// `DeckSource::RandomPerSeat`, and `NewGameDefaults` carries no room for a
    /// decklist. Nothing about the HTTP path itself is stubbed -- only the deck
    /// the game starts from, and both of Architecture Invariant 9's gates
    /// (`validate_deck` inside `build_initial_state`, then
    /// `check_all_defs_complete` inside `LocalGame::start`) still run for real.
    fn dx45h_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX45H_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx45h_deck(true)),
                (mtg_engine::PlayerId(2), dx45h_deck(false)),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX45 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive p1 -- a land drop, then Arbor Elf, then Birthing Ritual, else pass
    /// priority -- until the offered decision IS the CR 118.12 offer
    /// (`kind == "AnswerEffectChoice"` and `decision.question ==
    /// "PayOptionalCost"`). Returns that view WITHOUT answering it, so callers
    /// can inspect the offer (T1) or go on to answer it (T2/T3, the finding
    /// test).
    ///
    /// A real run reaches the offer at turn 3's end step, 30 decisions in (see
    /// this function's own commit history for the exploratory drive that
    /// established that number); `max_steps` is generous above it so a small
    /// deal shift turns into a loud failure rather than a silent hang.
    async fn dx45h_drive_to_pay_optional_cost_offer(
        state: &SharedState,
        max_steps: usize,
    ) -> Value {
        let p1 = mtg_engine::PlayerId(1);
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            let elf_out = ui2_battlefield_count_by_name(state, p1, "Arbor Elf") > 0;
            let ritual_out = ui2_battlefield_count_by_name(state, p1, "Birthing Ritual") > 0;
            let actions = view["decision"]["actions"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if actions.iter().any(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "PayOptionalCost"
            }) {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "step {step}: the game ended before the CR 118.12 offer was reached: {view}"
            );
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    if elf_out {
                        None
                    } else {
                        actions.iter().find(|a| {
                            a["kind"] == "CastSpell"
                                && a["label"].as_str().unwrap_or("").contains("Arbor Elf")
                        })
                    }
                })
                .or_else(|| {
                    if elf_out && !ritual_out {
                        actions.iter().find(|a| {
                            a["kind"] == "CastSpell"
                                && a["label"]
                                    .as_str()
                                    .unwrap_or("")
                                    .contains("Birthing Ritual")
                        })
                    } else {
                        None
                    }
                })
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .unwrap_or_else(|| {
                    panic!("step {step}: no PlayLand/CastSpell/PassPriority offered: {actions:?}")
                });
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": seq(&view), "action_index": index, "params": {}}),
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
        panic!("the CR 118.12 offer was not reached within {max_steps} steps: {view}");
    }

    /// **T1 -- the browser is offered the `Confirm` shape (CR 118.12), over
    /// real HTTP.**
    ///
    /// A real deck, driven through the real router (`PlayLand` / `CastSpell` /
    /// `PassPriority`, exactly the calls a browser makes) to the human's own
    /// end step, where Birthing Ritual's `Effect::LookAtTopThenPlace{
    /// place_cost: Some(Sacrifice), .. }` suspends and asks. Asserts
    /// `view::blocking_decision_view`'s `EffectChoiceQuestion::PayOptionalCost`
    /// arm end to end: the question tag, the answer_field, the `Confirm` shape,
    /// a non-empty `cost_label`, `pay_key == "pay"`, `default == true`, and
    /// that `template` carries the `PayOptionalCost` variant KEY -- checked by
    /// presence (the key exists, and only that key exists), not by a
    /// hard-coded JSON string, so this does not pass against `template: {}`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx45_the_browser_is_offered_the_confirm_shape_over_http() {
        let state = shared_state();
        dx45h_install(&state);
        let view = dx45h_drive_to_pay_optional_cost_offer(&state, 200).await;
        let action = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "PayOptionalCost"
            })
            .expect("dx45h_drive_to_pay_optional_cost_offer just found this");
        let decision = &action["decision"];

        assert_eq!(decision["question"], "PayOptionalCost");
        assert_eq!(decision["answer_field"], "effect_choice_answer");

        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "Confirm");
        assert_eq!(answer["pay_key"], "pay");
        assert_eq!(
            answer["default"], true,
            "the engine's own default is the pre-PB-DX45 auto-pay: {answer}"
        );
        let cost_label = answer["cost_label"]
            .as_str()
            .expect("cost_label is a string");
        assert!(
            !cost_label.is_empty(),
            "Birthing Ritual's sacrifice cost must render a real label: {answer}"
        );

        // The variant KEY exists -- asserted by presence, not by a hard-coded
        // string, so a regression to `template: {}` fails here instead of
        // sailing through a `template.to_string().contains(...)` check.
        let template = answer["template"]
            .as_object()
            .expect("template is a JSON object");
        assert_eq!(
            template.len(),
            1,
            "an externally-tagged Rust enum serializes to exactly one key: {template:?}"
        );
        assert!(
            template.contains_key("PayOptionalCost"),
            "template must carry the PayOptionalCost variant key: {template:?}"
        );
        assert_eq!(
            answer["template"]["PayOptionalCost"]["pay"], true,
            "the engine's own default answer, serialized verbatim: {answer}"
        );
    }

    /// **T2 -- the HTTP validator accepts both `pay` and `decline`, and still
    /// rejects a genuine mismatch.**
    ///
    /// # History
    ///
    /// This test used to be named
    /// `test_dx45_the_http_validator_currently_refuses_every_explicit_pay_
    /// optional_cost_answer` and pinned a real defect found while writing this
    /// file: `validate_decision_params` (`tools/play-server/src/api.rs`) had an
    /// arm for every OTHER `EffectChoiceQuestion` variant and none for
    /// `PayOptionalCost`, so the trailing wildcard was doing double duty as
    /// BOTH "reject an unknown variant" and "reject a `PayOptionalCost` answer
    /// that is not unknown at all" -- every `POST /api/game/action` 400'd on
    /// ANY explicit `{"PayOptionalCost": {...}}`, for `pay: true` (the engine's
    /// OWN default!) exactly as for `pay: false`. That is the exact
    /// clean-offer-then-guaranteed-refusal shape SR-38 exists to catch: `T1`
    /// still passed throughout, because it only reads the offer and never
    /// submits an answer.
    ///
    /// PB-DX45 fixed it structurally, not by appending one arm:
    /// `validate_decision_params` now dispatches on `question` ALONE, an
    /// exhaustive match over `EffectChoiceQuestion` with no wildcard, so a
    /// SEVENTH variant is a compile error there instead of a silent 400 --
    /// `EffectChoiceQuestion::PayOptionalCost { .. }` returns `Ok(())`, with an
    /// in-source comment explaining why (CR 118.12's answer space is `{pay,
    /// decline}`; there is no membership to check).
    ///
    /// # What this version asserts
    ///
    /// Both `pay: true` and `pay: false` are accepted (no `"a different kind"`
    /// 400) -- proving the defect above is gone. A `SearchLibrary` answer
    /// against the SAME `PayOptionalCost` offer is then submitted and MUST
    /// still 400 with `"a different kind"`, so this test is not merely
    /// "everything is accepted now" -- the arm this test exercises is proven to
    /// still reject a genuine variant mismatch.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx45_the_http_validator_accepts_both_pay_and_decline() {
        let state = shared_state();
        dx45h_install(&state);
        let view = dx45h_drive_to_pay_optional_cost_offer(&state, 200).await;
        let index = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "PayOptionalCost"
            })
            .and_then(|a| a["index"].as_u64())
            .expect("dx45h_drive_to_pay_optional_cost_offer just found this");

        // Both bools of the real answer variant are accepted -- submitted on a
        // FRESH game each time, since a successful submission consumes the
        // decision and the second iteration would otherwise be answering
        // something that no longer exists.
        for pay in [true, false] {
            let state = shared_state();
            dx45h_install(&state);
            let view = dx45h_drive_to_pay_optional_cost_offer(&state, 200).await;
            let (status, body) = post_json(
                &state,
                "/api/game/action",
                json!({
                    "seq": seq(&view),
                    "action_index": index,
                    "params": {"effect_choice_answer": {"PayOptionalCost": {"pay": pay}}}
                }),
            )
            .await;
            assert_eq!(
                status,
                StatusCode::OK,
                "pay={pay}: an explicit PayOptionalCost answer must be accepted now that \
                 `validate_decision_params` dispatches on `question` alone: {body}"
            );
        }

        // The genuine-mismatch control: a `SearchLibrary` answer against a
        // `PayOptionalCost` offer must still be refused, and refused for the
        // SAME reason as before ("a different kind") -- proving the exhaustive
        // per-question dispatch still discriminates rather than having become
        // an accept-anything gate.
        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": index,
                "params": {"effect_choice_answer": {"SearchLibrary": {"found": null}}}
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a SearchLibrary answer against a PayOptionalCost offer must still 400, got \
             {status:?}: {body}"
        );
        assert_eq!(body["kind"], "bad_params");
        assert!(
            body["error"]
                .as_str()
                .unwrap_or_default()
                .contains("a different kind"),
            "expected the genuine-mismatch message naming the wrong kind, got {body}"
        );
    }

    /// **This pair drives `birthing_ritual`, not `nether_traitor`, and AC 7241 names
    /// the latter — disclosed rather than glossed** (PB-DX45 `/review`, Issue 6). A
    /// play-server session is installed from a DECK through `session::new_game`, and
    /// that path cannot be asked for "a `nether_traitor` in a graveyard with a creature
    /// dying on top of it"; `birthing_ritual`'s end-step trigger reaches a CR 118.12
    /// offer from an ordinary deck at turn 3. So this pair covers the OTHER pay site
    /// (`Effect::LookAtTopThenPlace`'s `place_cost`) and the OTHER cost kind
    /// (`Cost::Sacrifice`) — more coverage than the criterion asked for, and not the
    /// coverage it named. `nether_traitor`'s `{B}` is driven with a non-default answer
    /// through `LocalGame`/`HumanChoice` and the bot path by
    /// `crates/simulator/tests/pb_dx45_optional_cost_channel.rs`. What no probe covers
    /// is site 1 or a `Cost::Mana` optional cost over the HTTP TRANSPORT; the handler's
    /// own answer path is `PlaySession::submit` -> `LocalGame::submit`, which is the
    /// exact entry point those probes drive, so the untested layer is the JSON
    /// encode/decode four other question variants already exercise.
    /// **T2b -- a human DECLINE is answered over real HTTP and Arbor Elf
    /// survives (CR 118.12).**
    ///
    /// Reached over real HTTP up to the offer
    /// (`dx45h_drive_to_pay_optional_cost_offer`), then answered with a real
    /// `POST /api/game/action` carrying `{"PayOptionalCost": {"pay": false}}`
    /// -- no `PlaySession::submit` workaround is needed any more (see
    /// `test_dx45_the_http_validator_accepts_both_pay_and_decline`'s history
    /// section for why one used to be). Asserted by the RESOLUTION EFFECT,
    /// never by the offer (AC 7241's standard): before PB-DX45 a decline was
    /// not a reachable state through ANY channel -- the old `MayPayThenEffect`
    /// arm paid whenever `can_pay_optional_cost` was true, which it already is
    /// by this point in the drive -- so an offer-shaped assertion would pass on
    /// an engine that asks and pays anyway.
    ///
    /// This test and `test_dx45_a_human_accept_is_answered_over_http_and_
    /// arbor_elf_is_sacrificed` below are a DISCRIMINATING PAIR by
    /// construction, differing in exactly one JSON bool: a decline-only probe
    /// cannot tell "the decline works" from "this fixture never sacrifices
    /// Arbor Elf at all, so of course it survived".
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx45_a_human_decline_is_answered_and_arbor_elf_survives() {
        let state = shared_state();
        dx45h_install(&state);
        let view = dx45h_drive_to_pay_optional_cost_offer(&state, 200).await;
        let index = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "PayOptionalCost"
            })
            .and_then(|a| a["index"].as_u64())
            .expect("dx45h_drive_to_pay_optional_cost_offer just found this");
        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": index,
                "params": {"effect_choice_answer": {"PayOptionalCost": {"pay": false}}}
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "declining the CR 118.12 offer: {body}"
        );

        let p1 = mtg_engine::PlayerId(1);
        assert_eq!(
            ui2_battlefield_count_by_name(&state, p1, "Arbor Elf"),
            1,
            "CR 118.12: declined, so the sacrifice cost is never paid and Arbor Elf survives"
        );
        let graveyard = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            !graveyard.contains(&"Arbor Elf".to_string()),
            "Arbor Elf must not be in the graveyard after a decline: {graveyard:?}"
        );
    }

    /// **T3 -- the ACCEPT half, over real HTTP, the identical fixture and
    /// drive, differing only in the `pay` bool in the POST body.**
    ///
    /// This test and `test_dx45_a_human_decline_is_answered_and_arbor_elf_
    /// survives` above are a DISCRIMINATING PAIR: paid, so CR 118.12's
    /// sacrifice happens: Arbor Elf leaves the battlefield (CR 400.7 -- a NEW
    /// graveyard object with a new `ObjectId`, so it is checked by NAME, the
    /// `ui2_zone_names` convention this file uses throughout, never by the
    /// battlefield `ObjectId`) and the "put a creature card ... onto the
    /// battlefield" continuation resolves without a further blocking question
    /// (this deck's remaining library is 92 Forests and nothing else, so
    /// `Effect::LookAtTopThenPlace`'s optional placement has no legal
    /// candidate among the seven looked at and no-ops deterministically).
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx45_a_human_accept_is_answered_over_http_and_arbor_elf_is_sacrificed() {
        let state = shared_state();
        dx45h_install(&state);
        let view = dx45h_drive_to_pay_optional_cost_offer(&state, 200).await;
        let index = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "PayOptionalCost"
            })
            .and_then(|a| a["index"].as_u64())
            .expect("dx45h_drive_to_pay_optional_cost_offer just found this");
        let (status, body) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": index,
                "params": {"effect_choice_answer": {"PayOptionalCost": {"pay": true}}}
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "accepting the CR 118.12 offer: {body}"
        );

        let p1 = mtg_engine::PlayerId(1);
        assert_eq!(
            ui2_battlefield_count_by_name(&state, p1, "Arbor Elf"),
            0,
            "CR 118.12: paid, so the sacrifice cost is paid and Arbor Elf leaves the \
             battlefield"
        );
        let graveyard = ui2_zone_names(&state, mtg_engine::ZoneId::Graveyard(p1));
        assert!(
            graveyard.contains(&"Arbor Elf".to_string()),
            "the sacrificed Arbor Elf must be in the graveyard: {graveyard:?}"
        );
    }

    /// **T4 -- a frontend source gate: the client answers the `Confirm` shape
    /// without ever spelling the engine's variant name.**
    ///
    /// Source-level, for the standing reason this file states everywhere else
    /// -- there is no frontend test harness (plan §8 R7). Both files pinned
    /// here live directly under `frontend/src/lib/`, inside
    /// `collect_frontend_files`'s walk root, not behind the `$viewer` alias
    /// (the UI-4 gap `test_frontend_action_bar_keeps_the_fused_slot_and_
    /// pitch_wiring`'s own doc names) -- so that gap does not apply and no
    /// second walk of the shared library is needed here.
    #[test]
    fn test_dx45_frontend_answers_the_confirm_shape_without_spelling_the_variant() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);

        let action_bar = sources
            .iter()
            .find(|(p, _)| p.ends_with("ActionBar.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ActionBar.svelte is in the frontend walk");
        let confirm_picker = sources
            .iter()
            .find(|(p, _)| p.ends_with("ConfirmPicker.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ConfirmPicker.svelte is in the frontend walk");

        assert!(
            action_bar.contains("currentShape?.shape === 'Confirm'"),
            "ActionBar.svelte must dispatch on the Confirm shape"
        );
        assert!(
            action_bar.contains("<ConfirmPicker"),
            "ActionBar.svelte must mount ConfirmPicker for the Confirm shape"
        );

        // The rule is about CODE, not PROSE: `ConfirmPicker.svelte`'s own JSDoc
        // header explains the never-respell-the-variant discipline by naming
        // the variant it is avoiding ("`template` arrives as
        // `{\"PayOptionalCost\":{\"pay\":true}}`. ... It never spells
        // `\"PayOptionalCost\"`") -- so a bare substring ban over the whole file
        // text is vacuously red on the very file it is meant to pass. Strip
        // `/* ... */` blocks first (this file's only comment form is JSDoc,
        // never `//`), then check the code that remains.
        fn strip_block_comments(text: &str) -> String {
            let mut out = String::with_capacity(text.len());
            let mut rest = text;
            while let Some(start) = rest.find("/*") {
                out.push_str(&rest[..start]);
                match rest[start..].find("*/") {
                    Some(end) => rest = &rest[start + end + 2..],
                    None => {
                        rest = "";
                        break;
                    }
                }
            }
            out.push_str(rest);
            out
        }
        let confirm_picker_code = strip_block_comments(confirm_picker);
        assert!(
            confirm_picker_code.contains("payKey"),
            "the comment-stripping above must not have eaten the real code too -- \
             `payKey` is a real identifier this component reads: {confirm_picker_code}"
        );
        assert!(
            !confirm_picker_code.contains("PayOptionalCost"),
            "ConfirmPicker.svelte's CODE (comments stripped) must never spell the engine's \
             variant name -- it clones `template` and writes only `payKey`, the same \
             never-respell-the-variant discipline every other picker in this client follows \
             (see `AnswerShapeView::Partition::template`'s own doc). The JSDoc header is \
             allowed to name it in prose; this check does not run over that."
        );
    }

    /// **PB-DX50 -- a frontend source gate: the client answers the `BinaryChoice`
    /// shape, with a picker of its OWN, without ever spelling the engine's variant
    /// name.**
    ///
    /// Source-level, for the standing reason this file states everywhere else --
    /// there is no frontend test harness (plan §8 R7). All three files pinned here
    /// live directly under `frontend/src/lib/`, inside `collect_frontend_files`'s
    /// walk root, not behind the `$viewer` alias.
    ///
    /// **The third assertion is the one worth having.** CR 702.140c's answer space
    /// is a bool with a template and a key, i.e. structurally identical to
    /// CR 118.12's, so `AnswerShapeView::Confirm` would have WORKED -- and
    /// `ConfirmPicker` renders "Pay {cost}" / "Decline", which is a false label on
    /// a truthful payload. This gate fails if a later edit collapses the two.
    #[test]
    fn test_dx50_frontend_answers_the_binary_choice_shape_without_spelling_the_variant() {
        let frontend_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("frontend")
            .join("src");
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_frontend_files(&frontend_src, &mut sources);

        let action_bar = sources
            .iter()
            .find(|(p, _)| p.ends_with("ActionBar.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("ActionBar.svelte is in the frontend walk");
        let picker = sources
            .iter()
            .find(|(p, _)| p.ends_with("BinaryChoicePicker.svelte"))
            .map(|(_, t)| t.as_str())
            .expect("BinaryChoicePicker.svelte is in the frontend walk");

        assert!(
            action_bar.contains("currentShape?.shape === 'BinaryChoice'"),
            "ActionBar.svelte must dispatch on the BinaryChoice shape, or a human who \
             casts a mutate spell gets the visible \"unknown shape\" fallback and the \
             game DEADLOCKS on a resolution nobody can answer"
        );
        assert!(
            action_bar.contains("<BinaryChoicePicker"),
            "ActionBar.svelte must mount BinaryChoicePicker for the BinaryChoice shape"
        );
        // The two shapes must stay two components. `ConfirmPicker`'s markup is
        // `Pay {costLabel}` / `Decline`; mounting it for CR 702.140c would put a
        // pay/decline label on an over/under question.
        let bc_arm_start = action_bar
            .find("currentShape?.shape === 'BinaryChoice'")
            .expect("checked above");
        let bc_arm = &action_bar[bc_arm_start..];
        let bc_arm_end = bc_arm.find("{:else").unwrap_or(bc_arm.len());
        let bc_arm = &bc_arm[..bc_arm_end];
        assert!(
            !bc_arm.contains("<ConfirmPicker"),
            "the BinaryChoice arm must NOT mount ConfirmPicker: its buttons read \
             \"Pay {{cost}}\" and \"Decline\", and CR 702.140c's answers are over and \
             under -- neither is a payment and neither is the passive one. Arm: {bc_arm}"
        );

        // The rule is about CODE, not PROSE -- `ConfirmPicker`'s gate above states
        // why. Same stripping, same reason.
        fn strip_block_comments(text: &str) -> String {
            let mut out = String::with_capacity(text.len());
            let mut rest = text;
            while let Some(start) = rest.find("/*") {
                out.push_str(&rest[..start]);
                match rest[start..].find("*/") {
                    Some(end) => rest = &rest[start + end + 2..],
                    None => {
                        rest = "";
                        break;
                    }
                }
            }
            out.push_str(rest);
            out
        }
        // **The SERVER half, and it is the half the first draft of this gate missed.**
        // Everything above pins the FRONTEND. If `view.rs` started emitting
        // `AnswerShapeView::Confirm` for the CR 702.140c question, the browser would
        // faithfully render `ConfirmPicker` -- "Pay {host}" / "Decline" -- and every
        // assertion above would stay GREEN, because the `BinaryChoice` arm they check
        // would simply never be reached.
        //
        // **Measured, not reasoned.** The revert was executed: swapping the arm to
        // `Confirm` and neutralising the `dead_code` error that the now-unconstructed
        // variant raises left all 121 play-server tests passing. The `dead_code` error
        // is NOT a substitute for this check -- it is an artefact of `BinaryChoice`
        // having exactly one construction site today, and it would vanish the moment a
        // second question used the shape. (PB-DX32 §7 R7's class: a revert that fails to
        // COMPILE is not a revert that DISCRIMINATES.)
        //
        // Brace-matched from the arm head, never a fixed window (PB-DX49 `/review`).
        {
            let view_src = std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/view.rs"),
            )
            .expect("src/view.rs is readable");
            let arm = view_src
                .find("EffectChoiceQuestion::MutateOnTop { host } => {")
                .expect("view.rs must have a MutateOnTop arm in the shape dispatch");
            let bytes = view_src.as_bytes();
            let mut i = arm + "EffectChoiceQuestion::MutateOnTop { host } => {".len();
            let mut depth = 1usize;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            assert!(depth == 0, "unbalanced braces in view.rs's MutateOnTop arm");
            let body = &view_src[arm..i];
            // Non-vacuity: the extracted region really is that arm.
            assert!(
                body.contains("CR 702.140c") && body.len() < 3000,
                "the extracted arm must be the MutateOnTop one and must not have \
                 over-scanned; got {} bytes",
                body.len()
            );
            assert!(
                body.contains("AnswerShapeView::BinaryChoice"),
                "view.rs's MutateOnTop arm must build `BinaryChoice`. Arm: {body}"
            );
            assert!(
                !body.contains("AnswerShapeView::Confirm"),
                "view.rs's MutateOnTop arm must NOT build `Confirm`: `ConfirmPicker` \
                 renders \"Pay {{cost}}\" and \"Decline\", and CR 702.140c's two answers \
                 are over and under -- neither is a payment and neither is the passive \
                 one. That would be a truthful payload behind a false label. Arm: {body}"
            );
        }

        let picker_code = strip_block_comments(picker);
        assert!(
            picker_code.contains("choiceKey"),
            "the comment-stripping above must not have eaten the real code too -- \
             `choiceKey` is a real identifier this component reads: {picker_code}"
        );
        assert!(
            !picker_code.contains("MutateOnTop"),
            "BinaryChoicePicker.svelte's CODE (comments stripped) must never spell the \
             engine's variant name -- it clones `template` and writes only `choiceKey`, \
             the same never-respell-the-variant discipline every other picker in this \
             client follows. The JSDoc header is allowed to name it in prose; this check \
             does not run over that."
        );
    }

    // ── PB-DX35 Half B (`scutemob-227`, `OOS-DX4-5`) ──────────────────────────
    //
    // `Effect::LookAtTopThenPlace.optional` -- CR 118.12's "you may put ... onto
    // the battlefield" -- asked over PB-DP9's CR 608.2d channel through the SAME
    // `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true, .. }` /
    // `EffectChoiceAnswer::ChooseObject { chosen }` pair PB-DX28 built. The
    // engine- and simulator-level probes live in
    // `crates/engine/tests/primitives/pb_dx35_optional_placement.rs` and
    // `crates/simulator/tests/pb_dx35_optional_placement_channel.rs`; this
    // section is the play-server's own end of the same channel, over real HTTP.
    //
    // `api::validate_decision_params`'s `ChooseObject` arm and `view::blocking_
    // decision_view`'s `ChooseObject` arm needed ZERO code changes for this --
    // both already handle the variant generically since PB-DX28, and this
    // section proves that by execution rather than by reading the source.
    //
    // Satyr Wayfinder digs the top FOUR cards (`count: EffectAmount::Fixed(4)`),
    // not one -- with the fixture's near-all-Forest deck all four are legal
    // Land candidates, which makes this a genuine "which of several" choice
    // (Satyr Wayfinder's `destination` is HAND, `rest_to` is GRAVEYARD -- unlike
    // Risen Reef, there is no battlefield leg here at all).

    /// Seed 1 against a mono-green, 99-Forest-plus-one deck deals p1 an opening
    /// hand of `[Satyr Wayfinder, Forest x6]` -- found by an executed scan over
    /// seeds 0..200 (this file's own commit history for the scan), not guessed.
    /// `old-gnawbone` ({5}{G}{G}) is the [`DX45H_COMMANDER`] trick again:
    /// unreachable inside this fixture's drive window, so it never competes with
    /// the probe.
    const DX35H_SEED: u64 = 1;

    /// One `Satyr Wayfinder` ({1}{G}), the rest Forests. Its OWN ETB fires its
    /// own trigger -- `Effect::LookAtTopThenPlace { filter: Land, .. }` -- so
    /// casting the one probe card is the whole drive; no second creature and no
    /// combat step is needed anywhere.
    fn dx35h_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = Vec::new();
        main_deck.push(CardId("satyr-wayfinder".to_string()));
        while main_deck.len() < 99 {
            main_deck.push(CardId("forest".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX45H_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// Install a two-player fixed-deck session -- the `dx45h_install` precedent,
    /// same reasons: `POST /api/game` cannot build a fixed decklist, and nothing
    /// about the HTTP path itself is stubbed, only the deck the game starts
    /// from.
    fn dx35h_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX35H_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx35h_deck()),
                (mtg_engine::PlayerId(2), dx35h_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 5,
                max_commands: 2_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX35 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Drive p1 -- play a Forest, then cast Satyr Wayfinder, else pass priority
    /// -- until the offered decision IS the CR 118.12/608.2d `ChooseObject`
    /// offer (`kind == "AnswerEffectChoice"` and `decision.question ==
    /// "ChooseObject"`). Returns that view WITHOUT answering it, so callers can
    /// inspect the offer or go on to answer it.
    async fn dx35h_drive_to_choose_object_offer(state: &SharedState, max_steps: usize) -> Value {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            let actions = view["decision"]["actions"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if actions.iter().any(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "ChooseObject"
            }) {
                return view;
            }
            assert!(
                !view["decision"].is_null(),
                "step {step}: the game ended before the CR 118.12 offer was reached: {view}"
            );
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    actions.iter().find(|a| {
                        a["kind"] == "CastSpell"
                            && a["label"]
                                .as_str()
                                .unwrap_or("")
                                .contains("Satyr Wayfinder")
                    })
                })
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .unwrap_or_else(|| {
                    panic!("step {step}: no PlayLand/CastSpell/PassPriority offered: {actions:?}")
                });
            let index = pick["index"].as_u64().expect("index is a number");
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": seq(&view), "action_index": index, "params": {}}),
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
        panic!("the CR 118.12 offer was not reached within {max_steps} steps: {view}");
    }

    /// Whether object `id` (raw `u64`) still exists in `state.objects()`. A
    /// zone-changing move (CR 400.7) retires the OLD id and mints a NEW one, so
    /// this is the "did it leave its original zone" half of the proof; the
    /// COUNT helpers below are the "and it landed in the right one" half.
    fn dx35h_object_retired(state: &SharedState, id: u64) -> bool {
        state
            .session
            .lock()
            .expect("lock")
            .as_ref()
            .is_none_or(|s| {
                !s.game
                    .state()
                    .objects()
                    .contains_key(&mtg_engine::ObjectId(id))
            })
    }

    fn dx35h_zone_count(state: &SharedState, zone: mtg_engine::ZoneId) -> usize {
        state
            .session
            .lock()
            .expect("lock")
            .as_ref()
            .map(|s| {
                s.game
                    .state()
                    .objects()
                    .values()
                    .filter(|o| o.zone == zone)
                    .count()
            })
            .unwrap_or(0)
    }

    #[tokio::test(flavor = "multi_thread")]
    /// **T1 -- the browser is offered the `PickN` shape (CR 115.10/118.12), over
    /// real HTTP.** A real fixed deck, driven through the real router
    /// (`PlayLand` / `CastSpell` / `PassPriority`, exactly the calls a browser
    /// makes) to Satyr Wayfinder's own ETB, where `Effect::LookAtTopThenPlace`
    /// suspends and asks. Asserts `view::blocking_decision_view`'s
    /// `ChooseObject` arm end to end: the question tag, the `PickN` answer
    /// shape, `min_count == 0` (`up_to: true`), `count == 1`, the four real
    /// Land candidates (the top of a near-all-Forest library), and that the
    /// DEFAULT answer is PB-DX35's take-the-winner (lowest id) behaviour.
    async fn test_dx35_the_choose_object_offer_over_http() {
        let state = shared_state();
        dx35h_install(&state);
        let view = dx35h_drive_to_choose_object_offer(&state, 40).await;

        let action = view["decision"]["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["kind"] == "AnswerEffectChoice" && a["decision"]["question"] == "ChooseObject"
            })
            .expect("dx35h_drive_to_choose_object_offer just found this");
        let decision = &action["decision"];

        assert_eq!(decision["question"], "ChooseObject");
        assert_eq!(decision["answer_field"], "effect_choice_answer");

        let answer = &decision["answer"];
        assert_eq!(answer["shape"], "PickN");
        assert_eq!(
            answer["count"], 1,
            "CR 118.12 places AT MOST ONE, regardless of how many were looked at: {answer}"
        );
        assert_eq!(
            answer["min_count"], 0,
            "up_to: true means the minimum legal answer is ZERO -- declining is real: {answer}"
        );
        let candidates = answer["candidates"]
            .as_array()
            .expect("candidates is an array");
        assert_eq!(
            candidates.len(),
            4,
            "Satyr Wayfinder digs the top FOUR cards, and the fixture deck makes all four \
             legal Land candidates: {candidates:?}"
        );
        let mut ids: Vec<u64> = candidates
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        ids.sort_unstable();
        let default = answer["default"].as_array().expect("default is an array");
        assert_eq!(
            default,
            &vec![serde_json::json!(ids[0])],
            "the DEFAULT answer must be PB-DX35's take-the-winner (lowest id) behaviour: \
             {answer}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    /// **T2 -- the DECLINE, end to end over real HTTP.**
    ///
    /// This outcome was unreachable from every channel before PB-DX35 -- the
    /// old `LookAtTopThenPlace` arm destructured `optional: _` and always
    /// placed the best candidate when one existed, so a pre-batch engine put
    /// the winning card into hand with no question asked and no way to say no.
    /// A decline routes ALL FOUR looked-at cards to `rest_to` (the graveyard,
    /// Satyr Wayfinder's printed fallback) -- none reaches hand. Asserted by
    /// GRAVEYARD COUNT DELTA rather than by the original candidate ids staying
    /// put: CR 400.7 retires each id on its zone-changing move and mints a new
    /// one, so `dx35h_object_retired` proves EACH candidate left its original
    /// zone and the count delta proves where all four landed.
    async fn test_dx35_a_declined_choose_object_answer_over_http() {
        let state = shared_state();
        dx35h_install(&state);
        let view = dx35h_drive_to_choose_object_offer(&state, 40).await;
        let index = ui1_question_index(&view, "ChooseObject")
            .expect("the ChooseObject offer must be present");
        let actions = view["decision"]["actions"]
            .as_array()
            .expect("actions array");
        let action = actions
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let candidate_ids: Vec<u64> = action["decision"]["answer"]["candidates"]
            .as_array()
            .expect("candidates array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        assert_eq!(
            candidate_ids.len(),
            4,
            "sanity: the same four candidates as T1"
        );

        let p1_graveyard = mtg_engine::ZoneId::Graveyard(mtg_engine::PlayerId(1));
        let p1_hand = mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1));
        let graveyard_before = dx35h_zone_count(&state, p1_graveyard);
        let hand_before = dx35h_zone_count(&state, p1_hand);

        let (status, next) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": index,
                "params": {"effect_choice_answer": {"ChooseObject": {"chosen": []}}}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{next}");

        for id in &candidate_ids {
            assert!(
                dx35h_object_retired(&state, *id),
                "CR 400.7: candidate {id} must have LEFT the library (a new object minted \
                 in its destination zone) once the effect resolved"
            );
        }
        assert_eq!(
            dx35h_zone_count(&state, p1_graveyard),
            graveyard_before + 4,
            "CR 118.12's printed fallback: declined, so all FOUR looked-at cards are \
             routed to rest_to (the graveyard)"
        );
        assert_eq!(
            dx35h_zone_count(&state, p1_hand),
            hand_before,
            "a decline must add NOTHING to hand"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    /// **T3 -- the ACCEPT, on the identical fixture and drive.** T2 and T3
    /// differ in exactly one value: whether `chosen` names a candidate. Both
    /// halves are asserted because a decline-only probe cannot distinguish "the
    /// decline works" from "this fixture never places any card at all". The
    /// CHOSEN card's arrival in hand and the other three's arrival in the
    /// graveyard are BOTH asserted by count delta (CR 400.7 -- see T2's doc),
    /// proving the choice is real: CR 118.12 places AT MOST ONE, never all
    /// four.
    async fn test_dx35_an_accepted_choose_object_answer_over_http() {
        let state = shared_state();
        dx35h_install(&state);
        let view = dx35h_drive_to_choose_object_offer(&state, 40).await;
        let index = ui1_question_index(&view, "ChooseObject")
            .expect("the ChooseObject offer must be present");
        let actions = view["decision"]["actions"]
            .as_array()
            .expect("actions array");
        let action = actions
            .iter()
            .find(|a| a["index"] == index)
            .expect("the option with that index");
        let candidate_ids: Vec<u64> = action["decision"]["answer"]["candidates"]
            .as_array()
            .expect("candidates array")
            .iter()
            .map(|c| c["id"].as_u64().expect("id is a number"))
            .collect();
        let chosen_id = candidate_ids[0];

        let p1_graveyard = mtg_engine::ZoneId::Graveyard(mtg_engine::PlayerId(1));
        let p1_hand = mtg_engine::ZoneId::Hand(mtg_engine::PlayerId(1));
        let graveyard_before = dx35h_zone_count(&state, p1_graveyard);
        let hand_before = dx35h_zone_count(&state, p1_hand);

        let (status, next) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": index,
                "params": {"effect_choice_answer": {"ChooseObject": {"chosen": [chosen_id]}}}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{next}");

        for id in &candidate_ids {
            assert!(
                dx35h_object_retired(&state, *id),
                "CR 400.7: candidate {id} must have LEFT the library once the effect \
                 resolved"
            );
        }
        assert_eq!(
            dx35h_zone_count(&state, p1_hand),
            hand_before + 1,
            "CR 118.12: accepted, so exactly ONE card (the chosen one) reaches hand"
        );
        assert_eq!(
            dx35h_zone_count(&state, p1_graveyard),
            graveyard_before + 3,
            "the three non-chosen candidates must still be routed to rest_to -- \
             CR 118.12 places AT MOST ONE, not all four"
        );
    }

    // ── PB-DX52 (`OOS-DX25b-1`) -- Bolt Bend's "or ability" half over HTTP ──────
    //
    // The simulator-side halves (offer/accept via `LocalGame`/`HumanChoice`, the
    // resolution effect, the bot layer, and a control proving `Target::StackObject`
    // does not spuriously appear for a spell) are
    // `crates/simulator/tests/pb_dx52_stack_target_channel.rs`. This is the HTTP
    // half: a real `POST /api/game/action` through `app(state.clone()).oneshot(..)`,
    // the same router `main()` serves.
    //
    // # What this fixture drives, and what it deliberately does NOT
    //
    // A `play-server` session installs from a DECK and plays it out (PB-DX45's
    // disclosure standard), so there is no way to hand-place an OPPONENT's ability
    // on the stack at a chosen moment the way the engine-side fixture does. The
    // human's own deck therefore carries BOTH `Bolt Bend` and `Goblin Sharpshooter`
    // (real, `Complete`, deck-legal), and the drive has the human activate their
    // OWN Sharpshooter targeting THEMSELVES, then redirect it onto the bot with
    // Bolt Bend -- CR 115.7a's own candidate order (the redirecting player is tried
    // first, but is excluded because it equals the current target) makes this land
    // on the bot deterministically, without needing to script the bot's behaviour
    // at all. **The untested-over-HTTP combination is named rather than left
    // implied**: an OPPONENT-controlled ability being redirected, driven end to end
    // through a genuine bot-opponent HTTP session, is exercised only on the
    // simulator side (`c1`/`c2`/`c3` there drive p2's own ability); this probe
    // covers the WIRE round trip (a brand-new `kind: "stack_object"` value, a real
    // POST, a real resolution effect) on the reachable combination instead of not
    // testing it at all.

    /// Mono-red, `Complete` -- every card this fixture needs (`Bolt Bend`, `Goblin
    /// Sharpshooter`, `Mountain`) is red or colourless, so CR 903.5c color identity
    /// is satisfied trivially. **Not Krenko, Mob Boss** -- that was this fixture's
    /// first draft, and execution refuted it: its `{T}: create X 1/1 Goblins`
    /// activated ability needs only a tap once it resolves, `HeuristicBot` scores
    /// it highly, and the resulting snowballing Goblin army killed the human via
    /// combat well before Bolt Bend was ever drawn (measured: game over at step
    /// 225 of 4,000, no card ever offered). Karlach's own payload needs her to be
    /// CAST ({4}{R}) and then ATTACK before it does anything, which is far slower
    /// to bite and does not itself create any permanent.
    const DX52_COMMANDER: &str = "karlach-fury-of-avernus";

    /// **Read off an executed sweep, not reasoned to** (the `UI1_SEED` /
    /// `DX20B_SEED` precedent). A throwaway scratch test -- written, run, deleted,
    /// never committed -- swept seeds 1..=1500 directly against
    /// `setup::build_initial_state`'s dealt hand for this exact
    /// `DeckSource::Fixed` pair, looking for a seat-1 opening where BOTH `bolt-bend`
    /// and `goblin-sharpshooter` are in the OPENING SEVEN. The qualifying set —
    /// `[87, 146, 184, 752, 778, 863, 931, 1030]` — **agrees with `DX20B_SEED`'s own
    /// five-of-eleven set on every member the two sweeps share**, for a structurally
    /// identical deck shape (one two-card pair plus 97 copies of one basic land): the
    /// shuffle algorithm is a pure function of POSITION, not of WHICH two non-basic
    /// cards are in the 99, so the same seeds qualify regardless of which pair it is.
    ///
    /// **The first draft of that sentence said "byte-identical", and PB-DX52's `/review`
    /// refuted it.** `DX20B_SEED` swept `1..=800` and records five; this sweep ran
    /// `1..=1500` and lists eight. An eight-element list is not byte-identical to a
    /// five-element one. The claim that actually supports the inference — and the one
    /// meant — is narrower: **the members at or below 800 are exactly DX20B's five, and
    /// the extra three lie beyond DX20B's sweep bound.** Corrected in place rather than
    /// deleted, because the wrong version is the kind a later reader would reuse.
    /// 87 is chosen for the same reason `DX20B_SEED` was: it is the first of the
    /// set.
    const DX52_SEED: u64 = 87;

    /// `Bolt Bend` + `Goblin Sharpshooter` + 97 Mountains. Two action cards, one
    /// basic land type -- the `dx20b_human_deck` shape, so the drive's "play a
    /// land, else act on whichever action card is offered, else pass" policy is
    /// never ambiguous about what to do.
    fn dx52_human_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = vec![
            CardId("bolt-bend".to_string()),
            CardId("goblin-sharpshooter".to_string()),
        ];
        while main_deck.len() < 99 {
            main_deck.push(CardId("mountain".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX52_COMMANDER.to_string()),
            main_deck,
        }
    }

    fn dx52_bot_deck() -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        mtg_simulator::DeckConfig {
            commander: CardId(DX52_COMMANDER.to_string()),
            main_deck: (0..99).map(|_| CardId("mountain".to_string())).collect(),
        }
    }

    /// Install through `session::new_game` -- the same constructor the real handler
    /// uses, running the same two Invariant-9 gates (`validate_deck`,
    /// `check_all_defs_complete`). `POST /api/game` cannot express a
    /// `DeckSource::Fixed` game (`ui1_install`'s doc), which is why every PB-DX
    /// fixed-deck HTTP probe installs this way.
    fn dx52_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX52_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx52_human_deck()),
                (mtg_engine::PlayerId(2), dx52_bot_deck()),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX52 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// Out-of-band oracle: `player`'s current life total, read straight off the
    /// engine state rather than the wire (mirrors `dx44_life_total`).
    fn dx52_life_total(state: &SharedState, player: mtg_engine::PlayerId) -> i32 {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .player(player)
            .expect("player exists")
            .life_total
    }

    /// Out-of-band oracle: is the stack empty right now?
    fn dx52_stack_empty(state: &SharedState) -> bool {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session.game.state().stack_objects().is_empty()
    }

    /// Drive the human seat: play a land every chance, activate Goblin Sharpshooter
    /// (targeting THEMSELVES, p1) the moment it is offered, cast Goblin Sharpshooter
    /// the moment it is offered, otherwise pass -- UNTIL "Cast Bolt Bend" is offered
    /// with a `"stack_object"` candidate. Returns the view immediately before that
    /// POST (so the caller can inspect the offer) and the candidate itself.
    ///
    /// Never falls back to casting Bolt Bend with no target: the generic
    /// "anything not Concede" catch-all explicitly excludes it, because c4's own
    /// simulator-side finding is that `CastSpell(Bolt Bend)` is offered even with
    /// no legal target (SR-38 suppresses it by COST shape only, not by target
    /// shape) -- an accidental catch-all pick here would 4xx or (worse) silently
    /// consume the one Bolt Bend copy this deck has.
    async fn dx52_drive_to_bolt_bend_offer(
        state: &SharedState,
        max_steps: usize,
    ) -> (Value, Value) {
        let (status, mut view) = get_json(state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");
        for step in 0..max_steps {
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before Bolt Bend was ever offered against \
                 a stack_object candidate: {view}"
            );
            let actions = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .clone();

            if let Some(bb) = actions.iter().find(|a| {
                a["kind"] == "CastSpell"
                    && a["label"] == "Cast Bolt Bend"
                    && a["target_slots"][0]["candidates"]
                        .as_array()
                        .is_some_and(|cs| cs.iter().any(|c| c["kind"] == "stack_object"))
            }) {
                return (view.clone(), bb.clone());
            }

            // CR 302.6: Goblin Sharpshooter's `{T}` ability is offered the instant
            // summoning sickness clears -- which can be as early as THIS turn's
            // Upkeep, well before this turn's land drop, and therefore before Bolt
            // Bend's `{3}{R}` is affordable. Activating on sight (this probe's
            // first draft) fires the ability into a window where redirecting it is
            // impossible, and it never comes back (`Doesn't Untap`, no creature
            // deaths on this board). So the ability is deliberately GATED on Bolt
            // Bend ALREADY being co-offered in the SAME decision -- i.e. p1 can pay
            // for both right now -- rather than being taken the moment it exists.
            let bolt_bend_is_affordable_now = actions
                .iter()
                .any(|a| a["kind"] == "CastSpell" && a["label"] == "Cast Bolt Bend");

            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    if bolt_bend_is_affordable_now {
                        actions.iter().find(|a| {
                            a["kind"] == "ActivateAbility"
                                && a["label"]
                                    .as_str()
                                    .unwrap_or_default()
                                    .contains("Sharpshooter")
                        })
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    actions.iter().find(|a| {
                        a["kind"] == "CastSpell" && a["label"] == "Cast Goblin Sharpshooter"
                    })
                })
                .or_else(|| actions.iter().find(|a| a["kind"] == "KeepHand"))
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| a["kind"] != "Concede" && a["label"] != "Cast Bolt Bend")
                })
                .unwrap_or_else(|| {
                    panic!("only Concede/Bolt Bend were offered at step {step}: {view}")
                });

            let wire_seq = seq(&view);
            let params = if pick["kind"] == "ActivateAbility" {
                // CR 601.2c: target SELF -- p1 -- so the redirect below has
                // somewhere to move the target FROM (CR 115.7a's own candidate
                // order excludes the current target, so this makes the eventual
                // Bolt Bend land on the bot deterministically).
                let candidates = pick["target_slots"][0]["candidates"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                let self_candidate = candidates
                    .iter()
                    .find(|c| c["kind"] == "player" && c["id"] == 1)
                    .unwrap_or_else(|| {
                        panic!(
                            "Goblin Sharpshooter's offer must include p1 as a legal \
                             target: {pick}"
                        )
                    });
                json!({ "targets": [self_candidate["value"].clone()] })
            } else {
                json!({})
            };
            let (status, next) = post_json(
                state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": pick["index"], "params": params}),
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
            "Bolt Bend was never offered against a stack_object candidate within \
             {max_steps} steps: {view}"
        );
    }

    /// **h1** -- a genuine `POST /api/game/action` casting Bolt Bend at an
    /// activated ability's stack entry, using the `value` the server itself sent
    /// in the target candidate (echoed back verbatim, per the UI-4/SIM-6 standard
    /// -- `test_dx20b_imprisoned_offer_excludes_the_artifact_over_http`'s own
    /// `chosen["value"].clone()` idiom).
    ///
    /// The VERDICT is the RESOLUTION EFFECT (an out-of-band life-total read), not
    /// merely the HTTP 200 -- a clean offer followed by a guaranteed silent
    /// no-op is the SR-38 shape this project has shipped three times
    /// (`OOS-DX29`, PB-DX44, PB-DX45), and a 200-alone assertion cannot catch its
    /// mirror image.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx52_bolt_bend_redirects_an_ability_over_http() {
        let state = shared_state();
        dx52_install(&state);

        let (view, bolt_bend) = dx52_drive_to_bolt_bend_offer(&state, 4_000).await;

        let candidates = bolt_bend["target_slots"][0]["candidates"]
            .as_array()
            .unwrap_or_else(|| panic!("Bolt Bend must carry one target slot: {bolt_bend}"))
            .clone();
        assert_eq!(
            candidates.len(),
            1,
            "exactly one legal candidate on this board -- the ability itself: {candidates:?}"
        );
        assert_eq!(
            candidates[0]["kind"], "stack_object",
            "OOS-DX25b-1: the sole candidate must be the ability's stack entry: {:?}",
            candidates[0]
        );

        let p1 = mtg_engine::PlayerId(1);
        let p2 = mtg_engine::PlayerId(2);
        let p1_life_before = dx52_life_total(&state, p1);
        let p2_life_before = dx52_life_total(&state, p2);

        let before_commands = command_count(&view);
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": bolt_bend["index"],
                "params": {"auto_tap": true, "targets": [candidates[0]["value"].clone()]},
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "POST /api/game/action naming the ability's own stack entry must return 200: \
             {after}"
        );
        assert!(
            command_count(&after) > before_commands,
            "the command count should have advanced past {before_commands}, got {:?}",
            command_count(&after)
        );

        // Resolve the rest of the stack: pass priority until it is empty.
        let mut view = after;
        for step in 0..200 {
            if dx52_stack_empty(&state) {
                break;
            }
            assert!(
                !view["decision"].is_null(),
                "the game ended at step {step} before the stack finished resolving: {view}"
            );
            let wire_seq = seq(&view);
            let pass_index = view["decision"]["actions"]
                .as_array()
                .expect("actions is an array")
                .iter()
                .find(|a| a["kind"] == "PassPriority")
                .unwrap_or_else(|| {
                    panic!("no PassPriority while resolving the stack at step {step}: {view}")
                })["index"]
                .clone();
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": wire_seq, "action_index": pass_index, "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "step {step}: {next}");
            view = next;
        }
        assert!(
            dx52_stack_empty(&state),
            "the stack must have resolved within 200 passes"
        );

        // THE VERDICT: CR 115.7a redirected Goblin Sharpshooter's target off p1
        // (the caster of Bolt Bend, excluded because it was the CURRENT target)
        // and onto p2 (the next candidate in `retarget_candidates`' order), so
        // p1 takes none of the 1 damage and p2 takes all of it.
        let p1_life_after = dx52_life_total(&state, p1);
        let p2_life_after = dx52_life_total(&state, p2);
        assert_eq!(
            p1_life_after, p1_life_before,
            "CR 115.7a: p1 must take none of Goblin Sharpshooter's damage -- it was \
             redirected off them (before {p1_life_before}, after {p1_life_after})"
        );
        assert_eq!(
            p2_life_after,
            p2_life_before - 1,
            "CR 115.7a: p2 must take the printed 1 damage, as the ability's NEW \
             target (before {p2_life_before}, after {p2_life_after})"
        );
    }

    /// **h2** -- a wire-shape pin. `TargetOptionView` for a stack entry serialises
    /// with `kind == "stack_object"`, an `id` equal to the stack-entry's OWN id
    /// (read from the out-of-band oracle, not merely self-consistent with the
    /// candidate), a non-empty label, and a `value` that round-trips back to
    /// `Target::StackObject(that id)` when parsed by the engine's own
    /// `serde_json::from_value`.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx52_stack_object_candidate_wire_shape_round_trips() {
        let state = shared_state();
        dx52_install(&state);

        let (_view, bolt_bend) = dx52_drive_to_bolt_bend_offer(&state, 4_000).await;
        let candidate = bolt_bend["target_slots"][0]["candidates"][0].clone();

        assert_eq!(candidate["kind"], "stack_object", "{candidate}");

        // Out-of-band oracle: the ability's stack entry ACTUALLY has this id --
        // not merely "the wire says so".
        let ability_id: u64 = {
            let guard = state.session.lock().expect("lock");
            let session = guard.as_ref().expect("a session is installed");
            session
                .game
                .state()
                .stack_objects()
                .iter()
                .find(|so| {
                    matches!(
                        so.kind,
                        mtg_engine::StackObjectKind::ActivatedAbility { .. }
                    )
                })
                .unwrap_or_else(|| panic!("no ActivatedAbility on the stack"))
                .id
                .0
        };
        assert_eq!(
            candidate["id"].as_u64(),
            Some(ability_id),
            "the candidate's wire `id` must equal the ability's REAL stack-entry id: \
             {candidate}"
        );

        let label = candidate["label"].as_str().unwrap_or_default();
        assert!(
            !label.is_empty() && label != "Unknown",
            "the label must not be a placeholder: {candidate}"
        );

        // The round trip: parse the wire `value` back through the ENGINE's own
        // `Target` type -- not string-matched, actually deserialized.
        let parsed: mtg_engine::Target = serde_json::from_value(candidate["value"].clone())
            .unwrap_or_else(|e| {
                panic!("candidate.value must deserialize as mtg_engine::Target: {e}: {candidate}")
            });
        assert_eq!(
            parsed,
            mtg_engine::Target::StackObject(mtg_engine::ObjectId(ability_id)),
            "the wire value must round-trip to Target::StackObject(the real id), got \
             {parsed:?}"
        );
    }

    // ── PB-DX55 Half 1 (`OOS-SIM6-3`) — the browser half ─────────────────────
    //
    // The engine/simulator half is proven in
    // `crates/simulator/tests/pb_dx55_activation_auto_tap.rs` on a real
    // `LocalGame`/`HumanChoice` drive. This section is the OTHER channel the
    // acceptance criterion names, and it is the one the seed is actually ABOUT:
    // `OOS-SIM6-3` says *"a browser human activating a mana-cost ability gets a
    // 422 unless they happened to have floating mana"*. A 422 is an HTTP fact,
    // so it is refuted by an HTTP probe or not at all.

    const DX55H_SEED: u64 = UI1_SEED;

    /// `old-gnawbone`'s colour identity admits Forests, and the two probe cards
    /// are colourless or mono-green, so the deck is `validate_deck`-legal
    /// without any special pleading. Swiftfoot Boots is `Complete` by derive
    /// (no `Completeness` marker in its def at all) and prints **Equip {1}** —
    /// a `Cost::Mana` activation with no `{T}` component, which is exactly the
    /// shape `auto_tap_commands_for` could not fund before this batch.
    fn dx55h_deck(with_probe_cards: bool) -> mtg_simulator::DeckConfig {
        use mtg_engine::CardId;
        let mut main_deck: Vec<CardId> = Vec::new();
        if with_probe_cards {
            main_deck.push(CardId("arbor-elf".to_string()));
            main_deck.push(CardId("swiftfoot-boots".to_string()));
        }
        while main_deck.len() < 99 {
            main_deck.push(CardId("forest".to_string()));
        }
        mtg_simulator::DeckConfig {
            commander: CardId(DX45H_COMMANDER.to_string()),
            main_deck,
        }
    }

    /// The `dx45h_install` precedent, one deck over. `POST /api/game` cannot
    /// build this fixture (`session::config_for` hard-codes
    /// `DeckSource::RandomPerSeat`), so the DECK is installed directly and
    /// nothing else is: the router, `session::submit`, `LocalGame::submit`,
    /// `auto_tap_commands_for` and both of Architecture Invariant 9's gates all
    /// run for real.
    fn dx55h_install(state: &SharedState) {
        let cfg = mtg_simulator::LocalGameConfig {
            player_count: 2,
            human_seats: [mtg_engine::PlayerId(1)].into_iter().collect(),
            bot_kind: BotKind::Heuristic,
            seed: DX55H_SEED,
            decks: mtg_simulator::DeckSource::Fixed(vec![
                (mtg_engine::PlayerId(1), dx55h_deck(true)),
                (mtg_engine::PlayerId(2), dx55h_deck(false)),
            ]),
            limits: mtg_simulator::LocalGameLimits {
                max_turns: 200,
                max_commands: 40_000,
                max_consecutive_passes: 500,
                record_journal: true,
            },
        };
        let session = session::new_game(cfg, 0).expect("the PB-DX55 fixture deck must be legal");
        *state.session.lock().expect("fresh lock") = Some(session);
    }

    /// p1's floating mana, read out of band. The precondition this whole probe
    /// rests on: **zero**. Funding an activation out of a pool that was already
    /// full proves nothing about auto-tap.
    fn dx55h_pool_total(state: &SharedState) -> u32 {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        session
            .game
            .state()
            .players()
            .get(&mtg_engine::PlayerId(1))
            .expect("p1 exists")
            .mana_pool
            .total()
    }

    /// How many UNTAPPED lands p1 controls. The other half of the precondition:
    /// the cost must be payable *with taps* and not otherwise.
    fn dx55h_untapped_lands(state: &SharedState) -> usize {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let gs = session.game.state();
        gs.objects_in_zone(&mtg_engine::ZoneId::Battlefield)
            .into_iter()
            .filter(|o| {
                o.controller == mtg_engine::PlayerId(1)
                    && !o.status.tapped
                    && o.characteristics
                        .card_types
                        .contains(&mtg_engine::CardType::Land)
            })
            .count()
    }

    /// Is the Equipment actually attached to the Elf? The RESOLUTION EFFECT, and
    /// the only thing this probe accepts as proof — a 200 on the POST would be
    /// satisfied by an activation that paid and fizzled.
    fn dx55h_boots_are_attached(state: &SharedState) -> bool {
        let guard = state.session.lock().expect("lock");
        let session = guard.as_ref().expect("a session is installed");
        let gs = session.game.state();
        let elf = gs
            .objects()
            .values()
            .find(|o| {
                o.characteristics.name == "Arbor Elf" && o.zone == mtg_engine::ZoneId::Battlefield
            })
            .map(|o| o.id);
        gs.objects().values().any(|o| {
            o.characteristics.name == "Swiftfoot Boots"
                && o.zone == mtg_engine::ZoneId::Battlefield
                && o.attached_to.is_some()
                && o.attached_to == elf
        })
    }

    /// **PB-DX55 Half 1 over real HTTP — `OOS-SIM6-3`'s own headline sentence,
    /// refuted in the channel it was written about.**
    ///
    /// CR 602.2a/602.2b: an activated ability's activation cost is paid as it is
    /// activated, and CR 605/CR 601.2f let the player produce that mana first.
    /// Before this batch `LocalGame::auto_tap_commands_for` opened with
    /// `let Command::CastSpell(cast) = command else { return None; }`, so the
    /// browser's `POST /api/game/action` for an `ActivateAbility` was applied
    /// against whatever was already floating and the engine refused it — the
    /// 422 the seed describes.
    ///
    /// The drive is the calls a browser makes and nothing else: `PlayLand`,
    /// `CastSpell`, `PassPriority`, then the Equip activation. **No
    /// `TapForMana` is ever submitted** — asserted, not merely omitted, by
    /// counting the kind across every action this test posts. At the moment of
    /// activation p1's pool is asserted EMPTY and their untapped land count
    /// asserted non-zero, so the cost is payable with taps and in no other way.
    /// The verdict is the attachment (CR 702.6a), not the status code.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_dx55_browser_activates_a_mana_cost_ability_with_an_empty_pool() {
        let state = shared_state();
        dx55h_install(&state);
        let p1 = mtg_engine::PlayerId(1);

        let (status, mut view) = get_json(&state, "/api/game").await;
        assert_eq!(status, StatusCode::OK, "{view}");

        let mut posted_kinds: Vec<String> = Vec::new();
        let mut equip: Option<Value> = None;
        for step in 0..400usize {
            assert!(
                !view["decision"].is_null(),
                "step {step}: the game ended before the Equip activation was reachable: {view}"
            );
            let actions = view["decision"]["actions"]
                .as_array()
                .cloned()
                .unwrap_or_default();

            // The Equip activation, recognised by its own target slots rather
            // than by a label substring: an `ActivateAbility` on the Boots that
            // offers at least one candidate is the one we can actually drive.
            if let Some(a) = actions.iter().find(|a| {
                a["kind"] == "ActivateAbility"
                    && a["label"]
                        .as_str()
                        .unwrap_or("")
                        .contains("Swiftfoot Boots")
                    && a["target_slots"][0]["candidates"][0]["value"] != Value::Null
            }) {
                equip = Some(a.clone());
                break;
            }

            let boots_out = ui2_battlefield_count_by_name(&state, p1, "Swiftfoot Boots") > 0;
            let elf_out = ui2_battlefield_count_by_name(&state, p1, "Arbor Elf") > 0;
            let pick = actions
                .iter()
                .find(|a| a["kind"] == "PlayLand")
                .or_else(|| {
                    if elf_out {
                        None
                    } else {
                        actions.iter().find(|a| {
                            a["kind"] == "CastSpell"
                                && a["label"].as_str().unwrap_or("").contains("Arbor Elf")
                        })
                    }
                })
                .or_else(|| {
                    if boots_out {
                        None
                    } else {
                        actions.iter().find(|a| {
                            a["kind"] == "CastSpell"
                                && a["label"]
                                    .as_str()
                                    .unwrap_or("")
                                    .contains("Swiftfoot Boots")
                        })
                    }
                })
                .or_else(|| actions.iter().find(|a| a["kind"] == "PassPriority"))
                .unwrap_or_else(|| panic!("step {step}: nothing drivable offered: {actions:?}"));
            posted_kinds.push(pick["kind"].as_str().unwrap_or_default().to_string());
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": seq(&view), "action_index": pick["index"], "params": {}}),
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

        let equip = equip.expect(
            "the Equip {1} activation was never offered with a candidate — the drive needs \
             re-observing, not the assertion relaxing",
        );

        // THE PRECONDITIONS. Without both of these the 200 below means nothing.
        assert_eq!(
            dx55h_pool_total(&state),
            0,
            "precondition: p1's pool must be EMPTY at the moment of activation — a funded \
             pool proves nothing about auto-tap"
        );
        assert!(
            dx55h_untapped_lands(&state) > 0,
            "precondition: p1 must control an untapped land, or the cost is unpayable by \
             any means and the refusal would be correct"
        );
        assert!(
            !posted_kinds.iter().any(|k| k == "TapForMana"),
            "this probe must never tap manually — the whole claim is that the browser does \
             not have to. Posted kinds were {posted_kinds:?}"
        );
        assert!(
            !dx55h_boots_are_attached(&state),
            "precondition: the Boots must not already be attached, or the assertion below \
             is satisfied by the fixture rather than by the activation"
        );

        let target = equip["target_slots"][0]["candidates"][0]["value"].clone();
        let (status, after) = post_json(
            &state,
            "/api/game/action",
            json!({
                "seq": seq(&view),
                "action_index": equip["index"],
                "params": { "targets": [target] },
            }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "OOS-SIM6-3's own sentence: the browser activation was refused: {after}"
        );

        // CR 602.2b/CR 608: an activated ability USES THE STACK, so acceptance is
        // not resolution. Pass priority until it resolves. **The first draft of
        // this probe asserted the attachment immediately after the 200 and failed
        // — correctly, and that failure is the reason the attachment assertion is
        // the verdict rather than the status code.**
        view = after;
        for step in 0..40usize {
            if dx55h_boots_are_attached(&state) {
                break;
            }
            assert!(
                !view["decision"].is_null(),
                "resolution step {step}: the game ended before the Equip ability resolved"
            );
            let actions = view["decision"]["actions"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let Some(pass) = actions.iter().find(|a| a["kind"] == "PassPriority") else {
                break;
            };
            posted_kinds.push("PassPriority".to_string());
            let (status, next) = post_json(
                &state,
                "/api/game/action",
                json!({"seq": seq(&view), "action_index": pass["index"], "params": {}}),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "resolution step {step}: {next}");
            view = next;
        }
        assert!(
            !posted_kinds.iter().any(|k| k == "TapForMana"),
            "still no manual tap after the resolution passes: {posted_kinds:?}"
        );

        // THE VERDICT — the resolution effect (CR 702.6a), not the status code.
        assert!(
            dx55h_boots_are_attached(&state),
            "the activation was accepted but the Equipment never attached — a 200 on a \
             fizzle is exactly what this assertion exists to catch"
        );
    }
}

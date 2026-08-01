//! Axum route handlers — the **only** async code in this crate.
//!
//! This is not the only place tokio is *named* (`main.rs` builds the runtime and
//! every inline test carries a `#[tokio::test]` attribute). The accurate
//! statement is the one `session.rs` already makes: **nothing below `api.rs`
//! references tokio.** (S5 re-review LOW 8.)
//!
//! M11-local Session 5, plan items 5 and 6 (`memory/m11-session-plan.md` §4).
//!
//! # Route surface
//!
//! Deliberately disjoint from the replay viewer's `/api/step/...`, so the two
//! servers could one day be merged without a route collision:
//!
//! | Route | Meaning |
//! |---|---|
//! | `POST /api/game` | start a new game (optionally overriding players/bot/seed) |
//! | `GET /api/game` | this seat's view, its pending decision, and new events |
//! | `POST /api/game/action` | answer the pending decision |
//! | `POST /api/game/mulligan` | CR 103.5 pregame redeal, pregame only |
//! | `GET /api/healthz` | liveness |
//!
//! # No WebSocket, no SSE (M11-local decision)
//!
//! Bots act **synchronously inside the same request** that carries the human's
//! action: `POST /api/game/action` calls `submit` then `advance`, and returns
//! the seat view that results. There is therefore never a moment where the
//! server knows something the client has not been told in a response it is
//! already waiting for, so request/response is sufficient and push
//! infrastructure is M10a's problem. (Session 8 records this in
//! `memory/decisions.md` and the crate README.)
//!
//! # `block_in_place`
//!
//! Every call into `session.rs` runs inside `tokio::task::block_in_place`. The
//! engine is synchronous and a single `advance()` can run a long chain of bot
//! commands, a deep trigger cascade, or a 100-card shuffle; without this a
//! single request would starve the reactor of a worker thread's cooperative
//! yields. **`block_in_place` panics on a current-thread runtime**, which is why
//! `main` hand-builds a multi-thread runtime and why the inline tests must use
//! `#[tokio::test(flavor = "multi_thread")]`.

use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use mtg_simulator::{AdvanceOutcome, BotKind, HumanChoice, LocalGameError, SetupError};
use mtg_view_model::{event_view_for, StateViewModel, Viewer};

use crate::session::{self, AppState, NewGameDefaults, PlaySession, SessionError, SharedState};
use crate::view::{
    self, ActionRequest, GameSummary, MulliganRequest, NameIndex, NewGameRequest, SeatView,
};

// ── Error envelope ────────────────────────────────────────────────────────────

/// The single JSON error shape every failing handler returns.
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Human-readable detail. For an engine rejection this is the
    /// `GameStateError` rendered as text, per plan item 6.
    pub error: String,
    /// Stable machine tag, so the client can branch without parsing prose.
    pub kind: &'static str,
}

/// An error response: a status plus the envelope above.
#[derive(Debug)]
pub struct ApiFailure {
    pub status: StatusCode,
    pub body: ApiError,
}

impl ApiFailure {
    fn new(status: StatusCode, kind: &'static str, error: impl Into<String>) -> Self {
        ApiFailure {
            status,
            body: ApiError {
                error: error.into(),
                kind,
            },
        }
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Plan item 6, verbatim — the whole `LocalGameError` -> HTTP mapping, in one
/// place so no handler can invent its own.
///
/// | variant | status | reasoning |
/// |---|---|---|
/// | `StaleDecision` | **409 Conflict** | the client answered a superseded action list; retrying against the current `seq` will work, which is exactly what 409 means. The body carries `expected` and `got` so the client can resync without a second round trip. |
/// | `NoPendingDecision` | **409 Conflict** | same shape: the request is well-formed but conflicts with the current state of the resource. |
/// | `UnknownAction(i)` | **400 Bad Request** | the index is not in the list the server just sent; the request itself is malformed. |
/// | `Rejected(GameStateError)` | **422 Unprocessable Entity** | syntactically fine and semantically addressed to a real action, but the *engine* refused it (an illegal target, an unpayable cost). 422 is precisely "understood, but could not be processed". The `GameStateError` is rendered as text. |
/// | `BadParams(String)` | **400 Bad Request** | the client supplied a param this action has no channel for (`ParamError::UnsupportedParam`) or omitted a required one. Nothing about the game state makes it invalid — it is wrong on its face, and would be wrong against any state. That is a client error, not an engine rejection, so it is 400 and not 422. |
/// | `Engine(GameStateError)` | **500 Internal Server Error** | **currently unreachable through this impl** — see below. Kept as the correct mapping should it ever become reachable: the failure would be on the server's side of the boundary and the client could do nothing about it, so 4xx would blame the caller for a server bug. |
///
/// # `Engine` is unreachable here, and the row above must not be read as a live contract
///
/// S5 re-review MEDIUM 3, verified rather than assumed:
///
/// * `LocalGameError::Engine` is constructed at **exactly one** site in the
///   workspace, `LocalGame::start`. The play server routes that through
///   `SessionError::Start` -> **500 `start_failed`**, never through this impl.
/// * The only expression in this crate that feeds `From<LocalGameError>` is the
///   `play.submit(..)?` in [`post_action`]. `PlaySession::submit` delegates to
///   `LocalGame::submit`, which returns only `NoPendingDecision`,
///   `StaleDecision`, `UnknownAction`, `BadParams` and `Rejected`.
/// * `LocalGame::advance` does not return a `Result` at all. An engine failure
///   **while advancing a bot seat** — the thing the row used to describe — becomes
///   `AdvanceOutcome::Halted(HaltReason::EngineError(..))`, which [`seat_view`]
///   renders as `game_over { halted: true, .. }` and answers with **200**.
///
/// The arm stays because the `match` is exhaustive over a plain (non-`#[non_exhaustive]`)
/// enum: dropping it would need a wildcard, which would silently swallow a variant
/// added later. It is a compile-forced classification, not a route a client can take.
impl From<LocalGameError> for ApiFailure {
    fn from(err: LocalGameError) -> Self {
        match err {
            LocalGameError::StaleDecision { expected, got } => ApiFailure::new(
                StatusCode::CONFLICT,
                "stale_decision",
                format!(
                    "decision seq mismatch: expected {expected}, got {got}; \
                     re-read GET /api/game and retry"
                ),
            ),
            LocalGameError::NoPendingDecision => ApiFailure::new(
                StatusCode::CONFLICT,
                "no_pending_decision",
                "there is no decision awaiting an answer",
            ),
            LocalGameError::UnknownAction(index) => ApiFailure::new(
                StatusCode::BAD_REQUEST,
                "unknown_action",
                format!("action_index {index} is not in the pending decision's action list"),
            ),
            LocalGameError::BadParams(message) => {
                ApiFailure::new(StatusCode::BAD_REQUEST, "bad_params", message)
            }
            LocalGameError::Rejected(e) => ApiFailure::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "rejected",
                // Rendered as TEXT (`Display`), per plan item 6 — not `Debug`.
                e.to_string(),
            ),
            // Unreachable through this impl today — see the doc block above.
            LocalGameError::Engine(e) => ApiFailure::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "engine_error",
                e.to_string(),
            ),
        }
    }
}

/// `SessionError` -> HTTP.
///
/// # Why `InvalidDeck` is 422 and not 400 (S5 re-review MEDIUM 4)
///
/// The re-review flagged 422 here against the crate's previously-stated rule,
/// *"400 means the request never reached the engine, 422 means the engine looked
/// at it and said no"* — pregame deck assembly never calls `process_command`.
/// The **rule** was the thing that was wrong, and it has been restated (here and
/// in the README) rather than the status changed:
///
/// > **400 means this crate refused the request before any engine code judged
/// > it. 422 means engine code looked at what the request asked for and said no
/// > — `process_command` for a command, `validate_deck` for a pregame table.**
///
/// A `POST /api/game {"seed": 17}` is well-formed in every syntactic sense; the
/// seed is a legal `u64` and nothing about the request is malformed on its face.
/// It fails because the table that seed *builds* is illegal — `deck::
/// basics_for_colors` pads a colourless commander's deck with Forests and
/// `mtg_engine::validate_deck` refuses them under CR 903.5c. That is the textbook
/// 422: understood, syntactically correct, semantically unprocessable. Reporting
/// it as 400 would tell the client to fix its request shape, which is not the
/// problem.
///
/// `BadPlayerCount` stays **400** and the contrast is the point: a count outside
/// `2..=6` is wrong against every state and never reaches engine code at all.
///
/// # The other three `SetupError` variants are 500, not 422 (S5 third audit LOW 3)
///
/// The rule above grounds 422 in `validate_deck`, and only
/// [`SetupError::InvalidDeck`] is a `validate_deck` judgment. The rest are
/// server-side faults by the same reasoning that puts `Start` at 500:
///
/// * `NoDeckForSeat` — `random_deck` found no legendary creature in the card
///   pool *this server chose*. Nothing the client sent caused it.
/// * `MissingCardDefinition` — `crates/simulator/src/setup.rs` documents this as
///   "a defensive check at spec-build time, in case a `DeckSource::Fixed` deck
///   was assembled against a different card pool". This crate only ever passes
///   `DeckSource::RandomPerSeat`, so reaching it would mean the pool and the
///   builder disagree — an internal inconsistency.
/// * `Builder(GameStateError)` — `GameStateBuilder::build()` refusing the table
///   the server assembled, exactly parallel to `Start`.
///
/// **None of the three is reachable today** (`players` is range-checked to
/// `2..=6` before this point, and the pool always contains a legendary
/// creature), so no status currently lies either way. Matching the variant makes
/// the *rule* true rather than merely narrowed, which is the point of the
/// finding. They carry a distinct `kind` so the README's kind→status table stays
/// one-to-one.
impl From<SessionError> for ApiFailure {
    fn from(err: SessionError) -> Self {
        let (status, kind) = match &err {
            SessionError::Setup(SetupError::InvalidDeck { .. }) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "setup_failed")
            }
            // Exhaustive rather than a wildcard: `SetupError` is a plain enum, so
            // a variant added later is a compile error here and has to be
            // classified rather than silently inheriting a status.
            SessionError::Setup(
                SetupError::NoDeckForSeat { .. }
                | SetupError::MissingCardDefinition { .. }
                | SetupError::Builder(_),
            ) => (StatusCode::INTERNAL_SERVER_ERROR, "setup_internal"),
            // `LocalGame::start` failing is `check_all_defs_complete` refusing the
            // table the server itself assembled — a server-side fault.
            SessionError::Start(_) => (StatusCode::INTERNAL_SERVER_ERROR, "start_failed"),
            SessionError::BadPlayerCount(_) => (StatusCode::BAD_REQUEST, "bad_player_count"),
            SessionError::NotPregame => (StatusCode::CONFLICT, "not_pregame"),
        };
        ApiFailure::new(status, kind, err.to_string())
    }
}

/// Re-wrap one of **axum's own** extractor rejections into this crate's
/// envelope (S5 review MEDIUM 2 / LOW 5).
///
/// Without this, a body axum could not deserialize escapes every handler before
/// the handler runs and axum answers it directly — with a `text/plain` body and
/// no `kind` field. Two things were wrong with that:
///
/// 1. It falsified the README's "one JSON envelope for every failure": a client
///    branching on `err.kind` read `undefined`.
/// 2. `JsonDataError`'s own status is **422**, which *collides* with this
///    crate's documented meaning for 422 ("the **engine** refused the command").
///    A client-side typo — `"target"` for `"targets"` — would be reported to the
///    user as an engine rejection.
///
/// So a data error is remapped to **400**, per this crate's own stated rule:
/// 400 means this crate refused the request before any engine code judged it;
/// 422 means engine code looked at what it asked for and said no. A
/// deserialization failure is refused by the extractor, before a handler exists.
/// Syntax (400) and content-type (415) keep axum's status, which already agrees.
fn json_rejection(rejection: JsonRejection) -> ApiFailure {
    let (status, kind) = match &rejection {
        // Syntactically valid JSON, wrong shape: a missing field, a wrong type,
        // or — because every request DTO here is `deny_unknown_fields` — a
        // misspelled one. axum says 422; this crate says 400.
        JsonRejection::JsonDataError(_) => (StatusCode::BAD_REQUEST, "invalid_body"),
        JsonRejection::JsonSyntaxError(_) => (StatusCode::BAD_REQUEST, "malformed_json"),
        JsonRejection::MissingJsonContentType(_) => {
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type")
        }
        // `JsonRejection` is `#[non_exhaustive]`, so this arm is mandatory rather
        // than defensive. Anything else (a body that could not be read at all)
        // keeps axum's own status and is still answered in the envelope.
        _ => (rejection.status(), "invalid_body"),
    };
    ApiFailure::new(status, kind, rejection.body_text())
}

// ── Combat submission validation (plan item 2) ────────────────────────────────

/// CR 508.1 / CR 509.1: check a submitted combat declaration against the very
/// lists the client was shown.
///
/// # Why this exists at all, when the engine validates too
///
/// The engine does check attacker and blocker legality, and it would refuse an
/// ineligible pair — but as a **422** carrying a `GameStateError`, i.e. "the
/// engine looked at your command and said no". That is the wrong report for this
/// case. The action list the server sent already enumerated exactly which
/// creatures may attack and what they may attack; a submission naming something
/// outside those lists is wrong *against the response the client is holding*,
/// with no reference to game state needed. That is this crate's own definition of
/// a 400 (`ApiFailure`'s doc: "400 means this crate refused the request before
/// any engine code judged it").
///
/// It also closes a hole the engine cannot see. `params.rs` maps
/// `LegalAction::DeclareAttackers` with **default** params to
/// `Command::DeclareAttackers { attackers: vec![] }` — a legal, irreversible "I
/// attack with nothing" that the engine accepts silently. The S6 handoff flagged
/// that as its second review MEDIUM. It is not fixed by rejecting the empty set
/// (declaring no attackers is legal under CR 508.1 and rejecting it would
/// deadlock a combat); it is fixed by the client now having a picker, which is
/// item 4 of this session. What this function adds is that a picker bug cannot
/// silently produce a *different* legal declaration than the one the human made.
///
/// # What is checked, and what is deliberately left to the engine
///
/// Checked here: membership in the provider's own `eligible` / `targets` /
/// `attackers` lists, and duplicate declarations of the same creature. Both are
/// decidable from the response alone.
///
/// Left to the engine: everything that needs rules reasoning — menace (CR
/// 702.110b) needing two blockers, evasion, requirements and restrictions (CR
/// 509.1c/d), banding. Re-deriving any of those here would be the OOS-RS-2 drift
/// class the whole `queries.rs` delegation exists to avoid.
fn validate_combat_params(
    action: &mtg_simulator::LegalAction,
    params: &crate::view::ActionParamsDto,
) -> Result<(), ApiFailure> {
    use mtg_simulator::LegalAction;

    let bad = |message: String| ApiFailure::new(StatusCode::BAD_REQUEST, "bad_params", message);

    match action {
        LegalAction::DeclareAttackers { eligible, targets } => {
            let mut seen = std::collections::BTreeSet::new();
            for (attacker, target) in &params.attackers {
                if !eligible.contains(attacker) {
                    return Err(bad(format!(
                        "object {} is not an eligible attacker (CR 508.1a); this decision \
                         offered {:?}",
                        attacker.0,
                        eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                    )));
                }
                if !targets.contains(target) {
                    return Err(bad(format!(
                        "{target:?} is not a legal attack target for this combat (CR 508.1a); \
                         this decision offered {targets:?}"
                    )));
                }
                if !seen.insert(attacker.0) {
                    return Err(bad(format!(
                        "object {} is declared as an attacker more than once (CR 508.1a: a \
                         creature attacks one player or planeswalker)",
                        attacker.0
                    )));
                }
            }
            Ok(())
        }
        LegalAction::DeclareBlockers {
            eligible,
            attackers,
        } => {
            let mut seen = std::collections::BTreeSet::new();
            for (blocker, attacker) in &params.blockers {
                if !eligible.contains(blocker) {
                    return Err(bad(format!(
                        "object {} is not an eligible blocker (CR 509.1a); this decision \
                         offered {:?}",
                        blocker.0,
                        eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                    )));
                }
                if !attackers.contains(attacker) {
                    return Err(bad(format!(
                        "object {} is not an attacking creature in this combat (CR 509.1a); \
                         this decision offered {:?}",
                        attacker.0,
                        attackers.iter().map(|o| o.0).collect::<Vec<_>>()
                    )));
                }
                // CR 509.1a: a creature blocks one attacker unless something says
                // otherwise. The exceptions ("can block an additional creature")
                // are real, so this rejects only the *identical* pair twice, which
                // is meaningless under every rule rather than merely unusual.
                if !seen.insert((blocker.0, attacker.0)) {
                    return Err(bad(format!(
                        "object {} is assigned to block object {} more than once",
                        blocker.0, attacker.0
                    )));
                }
            }
            Ok(())
        }
        // Every other variant: `params.rs` already refuses `attackers`/`blockers`
        // on it with `ParamError::UnsupportedParam` -> 400, so there is nothing to
        // add and nothing to duplicate.
        _ => Ok(()),
    }
}

/// **404**, deliberately. "No game exists" is the absence of the resource the
/// route names, which is what 404 means; 409 would imply the resource exists in
/// a conflicting state. The client's remedy is `POST /api/game`.
fn no_session() -> ApiFailure {
    ApiFailure::new(
        StatusCode::NOT_FOUND,
        "no_session",
        "no game is in progress; POST /api/game to start one",
    )
}

/// A poisoned mutex means a previous handler panicked mid-mutation. The session
/// is not trustworthy after that, so this is a 500 rather than a silent
/// `into_inner()` recovery.
fn poisoned() -> ApiFailure {
    ApiFailure::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "session_poisoned",
        "the session lock was poisoned by an earlier panic; restart the server",
    )
}

// ── Seat view assembly ────────────────────────────────────────────────────────

/// Build the payload for the human seat from whatever `advance()` last said.
///
/// Architecture Invariant 7 chokepoint: the state view is built with
/// `from_game_state_for(.., Viewer::Seat(human))` and every event line through
/// `event_view_for(.., Viewer::Seat(human))`. Neither omniscient entry point
/// (`from_game_state`, `Viewer::Omniscient`) is reachable from the **production
/// paths** of this crate. (The qualifier is load-bearing and matches the README:
/// `main.rs`'s test module does reach `from_game_state` deliberately, as the
/// out-of-band oracle the seat-redaction test checks this payload against. A
/// comment that claimed otherwise would be false.)
fn seat_view(session: &mut PlaySession, outcome: &AdvanceOutcome) -> SeatView {
    let human = session.human;
    let viewer = Viewer::Seat(human);

    // Events first: they are rendered against the CURRENT state, after every
    // command in the batch has been applied. (`event_view_for` uses the state
    // only to resolve object identity for the entitlement check, so a card that
    // has since changed zones renders name-free rather than wrongly named.)
    let records = session.take_new_records();
    let state = session.game.state();
    // The client-facing `seq`, which is NOT `pending.seq` — see
    // `PlaySession::wire_seq`.
    let wire_seq = session.pending_wire_seq();
    let events = records
        .iter()
        .flat_map(|record| record.events.iter())
        .filter_map(|ev| event_view_for(ev, state, &session.names, viewer))
        .collect();

    let state_view = StateViewModel::from_game_state_for(state, &session.names, viewer);
    let names = NameIndex::from_view(&state_view);

    // `zip` rather than two `unwrap`s: the pending decision and its wire `seq`
    // are produced together and are structurally inseparable here.
    let decision = session
        .pending
        .as_ref()
        .zip(wire_seq)
        .map(|(pending, seq)| view::decision_view(pending, seq, state, &names, &session.names));

    let game_over = match outcome {
        AdvanceOutcome::AwaitingHuman(_) => None,
        AdvanceOutcome::GameOver(result) => Some(view::game_over_view(result, &session.names)),
        AdvanceOutcome::Halted(reason) => Some(view::halted_view(
            reason,
            state.turn().turn_number,
            session.game.command_count(),
        )),
    };

    let summary = GameSummary {
        players: session.cfg.player_count,
        human: human.0,
        bot: format!("{:?}", session.cfg.bot_kind),
        seed: session.cfg.seed,
        turn: state.turn().turn_number,
        command_count: session.game.command_count(),
        mulligan_count: session.mulligan_count,
        pregame: session.is_pregame(),
    };

    SeatView {
        summary,
        state: state_view,
        decision,
        events,
        game_over,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `POST /api/game` — start (or restart) the game.
///
/// An **absent** body means "use the CLI defaults", and every field of a present
/// body is optional too. A *malformed* body is a 400 and no longer silently
/// yields a default game — see the extractor comment below. `advance()` runs
/// before returning, so the response already carries the human's first decision.
///
/// This is also the one route that **recovers from a poisoned session mutex** —
/// see the comment on the lock below.
pub async fn post_game(
    State(state): State<SharedState>,
    // `Result<_, JsonRejection>`, NOT `Option<Json<_>>` (S5 review LOW 5).
    // `Option<T>`'s `FromRequest` impl is literally `T::from_request(..).ok()`,
    // so it maps *every* rejection to `None` — `POST /api/game {"playerz": 9}`
    // used to answer **200 with a default four-player game** instead of
    // reporting the typo. A `Result` keeps the two apart.
    body: Result<Json<NewGameRequest>, JsonRejection>,
) -> Result<Json<SeatView>, ApiFailure> {
    let req = match body {
        Ok(Json(b)) => b,
        // The deliberate, tested case: no body at all. axum reports a request
        // with no `Content-Type: application/json` as `MissingJsonContentType`,
        // and a body-less POST carries no content type, so this arm *is* "the
        // body was omitted". (A body sent under some other content type lands
        // here too and is treated as omitted — the same thing the previous
        // `Option<Json<_>>` did, so no behaviour is lost.) Every other rejection
        // is now reported.
        Err(JsonRejection::MissingJsonContentType(_)) => NewGameRequest::default(),
        Err(rejection) => return Err(json_rejection(rejection)),
    };
    let defaults = merge_defaults(state.defaults, &req)?;
    let cfg = session::config_for(defaults)?;

    // See the module doc: synchronous engine work, off the reactor.
    tokio::task::block_in_place(|| {
        // **The one route that recovers from poisoning** (S5 review LOW 6).
        //
        // Every other handler that *takes the lock* answers 500 for the life of
        // the process once a handler has panicked mid-mutation. That is right
        // for them — they read the session and must not read a half-mutated one
        // — but it means a single engine panic costs a process restart on a
        // surface whose whole job is to *find* engine panics
        // (`check_invariants: true` and debug `debug_assert!`s are live here).
        //
        // "That takes the lock" is the accurate qualifier (S5 re-review LOW 12):
        // `get_healthz` never locks and keeps answering 200, and the checks that
        // run *before* the lock — a rejected body, a bad `players`, an unknown
        // `bot`, a non-empty `cards_to_bottom` — keep their 400.
        //
        // The asymmetry is sound because this handler **discards the corrupt
        // session outright** and never plays on with it. The one thing it does
        // read is `next_seq_base()` — a plain `u64` counter that is a copy, not
        // an invariant, and that no panic can leave inconsistent. Preserving it
        // across the recovery is what keeps MEDIUM 1's monotonic-`seq`
        // guarantee true through a panic, so reading it is a feature rather
        // than a leak of corrupt state.
        //
        // # The recovery is atomic (S5 re-review MEDIUM 1)
        //
        // The corrupt session is `take()`n out of the `Option` in the **same
        // straight-line block** that clears the poison flag, with no fallible
        // operation between the two. That is what makes "the flag is clear" and
        // "there is no untrustworthy session left to read" one fact rather than
        // two facts held together by statement order.
        //
        // The previous shape cleared the flag here and relied on the
        // `*guard = Some(play)` at the bottom to remove the corrupt value — but
        // `session::new_game` sits between them and is fallible on a
        // **client-supplied seed** (`deck::basics_for_colors` pads a colourless
        // commander's deck with Forests, whose green identity `validate_deck`
        // refuses under CR 903.5c; a sweep found 7 such tables in 180
        // `(players, seed)` pairs). Its `?` skipped the assignment and left the
        // half-mutated session readable at 200.
        // `test_poison_recovery_is_atomic_when_the_rebuild_fails` pins it.
        let (mut guard, seq_base) = match state.session.lock() {
            Ok(guard) => {
                // Healthy: only *peek* at the counter. A rebuild that fails
                // below must leave a running game exactly as it was.
                let base = guard.as_ref().map_or(0, |prev| prev.next_seq_base());
                (guard, base)
            }
            Err(poison) => {
                let mut guard = poison.into_inner();
                let base = guard.take().map_or(0, |corrupt| corrupt.next_seq_base());
                state.session.clear_poison();
                (guard, base)
            }
        };
        // PB-DX4 (2026-08-01, `scutemob-168`): the test-only rebuild-failure injection.
        //
        // The two tests that pin this block's atomicity
        // (`test_poison_recovery_is_atomic_when_the_rebuild_fails`,
        // `test_a_failed_rebuild_leaves_a_running_game_untouched`) need
        // `session::new_game` to fail RIGHT HERE — after the lock is taken and after the
        // poison recovery has run — because the subject is the `?` on this line skipping
        // the `*guard = Some(play)` below. Until now their only trigger was OOS-M11-6, the
        // CR 903.5c colorless-commander Forest padding, which PB-DX4 closed in
        // `crates/simulator/src/deck.rs`. Their own maintenance note predicted exactly
        // this ("whoever closes OOS-M11-6 needs a new way to fail a rebuild inside the
        // lock; there is no other one today") and it was right: a probe of `players` 0 / 1
        // / 2 / 5 / 200 and the previously-failing `seed: 17` finds only 400
        // `bad_player_count` (checked BEFORE the lock, so it never reaches this line) or
        // 200. No client input reaches this `?` any more.
        //
        // So the trigger is now explicit rather than incidental, and it is carried by the
        // REQUEST rather than by process state. That second property was learned the hard
        // way: the first version of this injection was a global `AtomicBool`, and because
        // `cargo test` runs this binary's tests in parallel and many of them POST
        // `/api/game`, a flag set by one test could be consumed by another's request —
        // giving that test a spurious 422 and this one a 200. It passed under
        // `-p play-server` and failed under `--workspace`, twice, before the cause was
        // found. A sentinel seed has no such coupling: it is scoped to exactly the one
        // request that carries it, so there is nothing to leak and nothing to serialize.
        // Scope note (fix cycle, review Finding 12): this returns ABOVE `session::new_game`,
        // so the two tests prove "a rebuild that fails anywhere between the recovery and the
        // assignment leaves no readable corrupt session" rather than specifically "a rebuild
        // that fails INSIDE `session::new_game` does". That is the property the block is
        // written to have — the recovery `take()`s in the same straight-line stretch with no
        // fallible step between — and every statement from here to `*guard = Some(play)` is
        // covered by it. It is a slightly weaker statement than the test names suggest, and
        // saying so is cheaper than moving the injection past a call whose failure modes are
        // no longer client-reachable at all.
        #[cfg(test)]
        if cfg.seed == crate::tests::REBUILD_FAILURE_SEED {
            return Err(ApiFailure::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "setup_failed",
                "deliberate test-only rebuild failure (PB-DX4 injection point)",
            ));
        }
        let mut play = session::new_game(cfg, seq_base)?;
        let outcome = play.advance();
        let view = seat_view(&mut play, &outcome);
        *guard = Some(play);
        Ok(Json(view))
    })
}

/// `GET /api/game` — this seat's view, its pending decision, and every event
/// since the last read.
///
/// Calls `advance()` even though it is a read: `LocalGame::advance` is
/// **idempotent while a decision is outstanding** (it re-issues the same `seq`
/// rather than minting a new one), and every request leaves the game parked at
/// `AwaitingHuman` / `GameOver` / `Halted`, so this is a no-op in the common
/// case. It is what lets a plain `GET` report a concluded game consistently
/// without the session having to cache the last outcome.
///
/// It does advance `journal_cursor` — reading the events consumes them.
pub async fn get_game(State(state): State<SharedState>) -> Result<Json<SeatView>, ApiFailure> {
    tokio::task::block_in_place(|| {
        let mut guard = state.session.lock().map_err(|_| poisoned())?;
        let play = guard.as_mut().ok_or_else(no_session)?;
        let outcome = play.advance();
        Ok(Json(seat_view(play, &outcome)))
    })
}

/// `POST /api/game/action` — answer the pending decision, then let the bots act.
///
/// `submit` then `advance`, in one request: the bots play out their whole turn
/// synchronously and the response carries the state the human next has to act
/// on. This is why M11-local needs no push channel.
pub async fn post_action(
    State(state): State<SharedState>,
    // `Result<_, JsonRejection>` so a body axum cannot deserialize is answered in
    // this crate's envelope with a 400, not by axum with a bare `text/plain` 422
    // that a client would read as an engine rejection (S5 review MEDIUM 2).
    body: Result<Json<ActionRequest>, JsonRejection>,
) -> Result<Json<SeatView>, ApiFailure> {
    let Json(req) = body.map_err(json_rejection)?;
    tokio::task::block_in_place(|| {
        let mut guard = state.session.lock().map_err(|_| poisoned())?;
        let play = guard.as_mut().ok_or_else(no_session)?;

        // CR 508.1 / CR 509.1 (plan item 2): check a combat declaration against
        // the lists this decision actually offered, before anything is applied.
        //
        // Only when the `seq` matches: a stale `seq` must keep answering **409
        // `stale_decision`** (the client's remedy is to re-read and retry), and
        // validating an old submission against the *current* decision's action
        // list would report the mismatch as a 400 instead — a worse diagnosis for
        // the same fault. `submit` re-checks `seq` itself, so this is a guard on
        // *whether to speak*, not a duplicate of the staleness check.
        //
        // An out-of-range `action_index` falls through untouched and `submit`
        // reports it as `UnknownAction` -> 400, which is already the right answer.
        if play.pending_wire_seq() == Some(req.seq) {
            if let Some(action) = play
                .pending
                .as_ref()
                .and_then(|pending| pending.actions.get(req.action_index))
            {
                validate_combat_params(action, &req.params)?;
            }
        }

        // `LegalAction` never crossed the wire: the index is resolved against
        // the `PendingDecision` the server is still holding.
        play.submit(
            req.seq,
            HumanChoice {
                action_index: req.action_index,
                params: req.params.into(),
            },
        )?;

        let outcome = play.advance();
        Ok(Json(seat_view(play, &outcome)))
    })
}

/// `POST /api/game/mulligan` — CR 103.5 pregame redeal. **Pregame only.**
///
/// `take: true` rebuilds the table through `setup::redeal`; `take: false` keeps
/// the hand as dealt and is a no-op.
///
/// **Two honest limitations, both inherited from `setup::redeal` and neither
/// papered over** (see `PlaySession::mulligan`'s doc comment for the full
/// argument):
///
/// 1. The redeal rebuilds the **whole table**, not one seat. It re-rolls every
///    seat's commander, and the command zone is public (CR 903.6), so it is not
///    invisible to the other players; and it cannot represent a partially
///    decided table (CR 103.5c gives each player their own mulligan count).
///    `redeal`'s own doc says the per-seat model "belongs with the Session 5
///    play-server pregame flow" — this session keeps the whole-table rebuild,
///    because a per-seat model needs each bot seat to be *asked*, which is a new
///    decision channel rather than a small addition.
/// 2. CR 103.5's bottoming half is not expressible on this path at all:
///    `handle_keep_hand` checks `cards_to_bottom.len()` against
///    `PlayerState::mulligan_count`, which a rebuild always leaves at 0 because
///    no `Command::TakeMulligan` is ever issued. A non-empty `cards_to_bottom`
///    is therefore **refused with 400** rather than accepted and silently
///    discarded — loud, not silently wrong.
pub async fn post_mulligan(
    State(state): State<SharedState>,
    // See `post_action` — same reason (S5 review MEDIUM 2).
    body: Result<Json<MulliganRequest>, JsonRejection>,
) -> Result<Json<SeatView>, ApiFailure> {
    let Json(req) = body.map_err(json_rejection)?;
    if !req.cards_to_bottom.is_empty() {
        return Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "bad_params",
            "cards_to_bottom is not supported by the pregame redeal path (CR 103.5's \
             bottoming half needs an engine-side mulligan count, which a whole-table \
             rebuild leaves at 0); send an empty list",
        ));
    }

    tokio::task::block_in_place(|| {
        let mut guard = state.session.lock().map_err(|_| poisoned())?;
        let play = guard.as_mut().ok_or_else(no_session)?;

        if !play.is_pregame() {
            return Err(ApiFailure::from(SessionError::NotPregame));
        }
        if req.take {
            play.mulligan()?;
        }
        let outcome = play.advance();
        Ok(Json(seat_view(play, &outcome)))
    })
}

/// `GET /api/healthz` — liveness. Never touches the session lock, so it answers
/// even while a long resolution holds it.
pub async fn get_healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "play-server",
    }))
}

// ── Request-body merging ──────────────────────────────────────────────────────

/// Overlay a `POST /api/game` body onto the CLI defaults.
fn merge_defaults(
    defaults: NewGameDefaults,
    req: &NewGameRequest,
) -> Result<NewGameDefaults, ApiFailure> {
    let bot = match req.bot.as_deref() {
        None => defaults.bot,
        Some(s) => parse_bot_kind(s)?,
    };
    Ok(NewGameDefaults {
        players: req.players.unwrap_or(defaults.players),
        bot,
        seed: req.seed.unwrap_or(defaults.seed),
    })
}

/// `"heuristic"` / `"random"`, case-insensitive. Anything else is a 400.
fn parse_bot_kind(s: &str) -> Result<BotKind, ApiFailure> {
    match s.to_ascii_lowercase().as_str() {
        "heuristic" => Ok(BotKind::Heuristic),
        "random" => Ok(BotKind::Random),
        other => Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "bad_bot_kind",
            format!("unknown bot kind {other:?}; expected \"heuristic\" or \"random\""),
        )),
    }
}

// A compile-time reminder that `AppState` is the axum `State` type and must stay
// `Clone` — axum's `State` extractor requires it, and the whole point of putting
// the session behind an `Arc<Mutex<..>>` is that cloning the handle is cheap.
const _: fn() = || {
    fn assert_clone<T: Clone>() {}
    assert_clone::<AppState>();
};

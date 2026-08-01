//! Axum route handlers — the **only** async code in this crate, and the only
//! place tokio is named.
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
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use mtg_simulator::{AdvanceOutcome, BotKind, HumanChoice, LocalGameError};
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
/// | `Engine(GameStateError)` | **500 Internal Server Error** | the engine failed while advancing *bot* seats. The human's request was valid; the failure is on the server's side of the boundary and the client can do nothing about it. Reporting it as 4xx would blame the caller for a server bug. |
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
            LocalGameError::Engine(e) => ApiFailure::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "engine_error",
                e.to_string(),
            ),
        }
    }
}

impl From<SessionError> for ApiFailure {
    fn from(err: SessionError) -> Self {
        let (status, kind) = match err {
            SessionError::Setup(_) => (StatusCode::UNPROCESSABLE_ENTITY, "setup_failed"),
            // `LocalGame::start` failing is `check_all_defs_complete` refusing the
            // table the server itself assembled — a server-side fault.
            SessionError::Start(_) => (StatusCode::INTERNAL_SERVER_ERROR, "start_failed"),
            SessionError::BadPlayerCount(_) => (StatusCode::BAD_REQUEST, "bad_player_count"),
            SessionError::NotPregame => (StatusCode::CONFLICT, "not_pregame"),
        };
        ApiFailure::new(status, kind, err.to_string())
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
/// (`from_game_state`, `Viewer::Omniscient`) is reachable from this crate.
fn seat_view(session: &mut PlaySession, outcome: &AdvanceOutcome) -> SeatView {
    let human = session.human;
    let viewer = Viewer::Seat(human);

    // Events first: they are rendered against the CURRENT state, after every
    // command in the batch has been applied. (`event_view_for` uses the state
    // only to resolve object identity for the entitlement check, so a card that
    // has since changed zones renders name-free rather than wrongly named.)
    let records = session.take_new_records();
    let state = session.game.state();
    let events = records
        .iter()
        .flat_map(|record| record.events.iter())
        .filter_map(|ev| event_view_for(ev, state, &session.names, viewer))
        .collect();

    let state_view = StateViewModel::from_game_state_for(state, &session.names, viewer);
    let names = NameIndex::from_view(&state_view);

    let decision = session
        .pending
        .as_ref()
        .map(|pending| view::decision_view(pending, state, &names));

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
/// The body is optional and every field in it is optional; anything omitted
/// falls back to the CLI defaults. `advance()` runs before returning, so the
/// response already carries the human's first decision.
pub async fn post_game(
    State(state): State<SharedState>,
    body: Option<Json<NewGameRequest>>,
) -> Result<Json<SeatView>, ApiFailure> {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let defaults = merge_defaults(state.defaults, &req)?;
    let cfg = session::config_for(defaults)?;

    // See the module doc: synchronous engine work, off the reactor.
    tokio::task::block_in_place(|| {
        let mut guard = state.session.lock().map_err(|_| poisoned())?;
        let mut play = session::new_game(cfg)?;
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
    Json(req): Json<ActionRequest>,
) -> Result<Json<SeatView>, ApiFailure> {
    tokio::task::block_in_place(|| {
        let mut guard = state.session.lock().map_err(|_| poisoned())?;
        let play = guard.as_mut().ok_or_else(no_session)?;

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
    Json(req): Json<MulliganRequest>,
) -> Result<Json<SeatView>, ApiFailure> {
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

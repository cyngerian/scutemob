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
//! | `GET /api/game/report` | the bug-report / repro artefact (S8 item 5) |
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
///   pool *this server chose*, or (since `scutemob-187`) `setup::dealt_decks`
///   could not read a seat back out of a table this server had just built.
///   Nothing the client sent caused either.
/// * `MissingCardDefinition` — `crates/simulator/src/setup.rs` documents this as
///   "a defensive check at spec-build time, in case a `DeckSource::Fixed` deck
///   was assembled against a different card pool". This crate does pass
///   `DeckSource::Fixed` since `scutemob-187`, but only lists it read out of a
///   state built from `all_cards()` moments earlier in the same process — so
///   reaching it would still mean the pool and the builder disagree, an internal
///   inconsistency.
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
/// `Command::DeclareAttackers { attackers: vec![] }` — a legal "I attack with
/// nothing" that the engine accepts silently. The S6 handoff flagged that as its
/// second review MEDIUM. It is not fixed by rejecting the empty set (declaring no
/// attackers is legal under CR 508.1 and rejecting it would deadlock a combat); it
/// is fixed by the client now having a picker, which is item 4 of this session.
/// What this function adds is that a picker bug cannot silently produce a
/// *different* legal declaration than the one the human made.
///
/// **The README's word *irreversible* (`tools/play-server/README.md:297`, review
/// finding L4 -- it does not appear in THIS file) was aspirational until PB-DX21
/// (`OOS-M11-9`, `scutemob-200`) and is now true.** CR 508.1 makes declaring
/// attackers a once-per-combat turn-based action; before PB-DX21 the engine
/// accepted a second `DeclareAttackers` in the same combat without limit, so an
/// empty declaration could always be followed by a real one. `combat.rs::
/// handle_declare_attackers` now rejects any second declaration with
/// `GameStateError::AlreadyDeclaredAttackers`, and `legal_actions.rs` stops
/// offering the action once `CombatState::attackers_declared` is set — so the
/// empty declaration this function guards really is the player's one shot at
/// CR 508.1 for the combat.
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
        // CR 509.2 (M11-local S8, item 2). Checked here for the same reason the two
        // above are: a submitted order naming something outside the candidate list
        // the server just sent is wrong against *that response*, with no game state
        // needed to see it.
        //
        // The completeness half (CR 509.2 requires ALL of an attacker's blockers to
        // be ordered) is deliberately left to the engine, which reports it as
        // `GameStateError::IncompleteBlockerOrder` -> 422. It is a rules judgment
        // about the combat, not about the response — and re-deriving it here would
        // be the drift class the delegation exists to avoid.
        LegalAction::OrderBlockers { blockers, .. } => {
            let mut seen = std::collections::BTreeSet::new();
            for blocker in &params.blocker_order {
                if !blockers.contains(blocker) {
                    return Err(bad(format!(
                        "object {} is not blocking this attacker (CR 509.2); this decision \
                         offered {:?}",
                        blocker.0,
                        blockers.iter().map(|o| o.0).collect::<Vec<_>>()
                    )));
                }
                if !seen.insert(blocker.0) {
                    return Err(bad(format!(
                        "object {} appears more than once in the damage assignment order \
                         (CR 509.2: the order is a permutation of the blockers)",
                        blocker.0
                    )));
                }
            }
            Ok(())
        }
        // Every other variant: `params.rs` already refuses
        // `attackers`/`blockers`/`blocker_order` on it with
        // `ParamError::UnsupportedParam` -> 400, so there is nothing to add and
        // nothing to duplicate.
        _ => Ok(()),
    }
}

/// UI-1: check a **blocking-decision** answer against the candidate lists this
/// same decision sent, before anything is applied.
///
/// # Same argument as [`validate_combat_params`], same boundary
///
/// The engine checks all of this too, and would refuse it — but as a **422**
/// carrying a `GameStateError`, i.e. "the engine looked at your answer and said
/// no". An answer naming a card the response never offered is wrong *against the
/// response the client is holding*, with no game state needed to see it, which is
/// this crate's own definition of a 400.
///
/// It also matters more here than it does for combat, because the answer types are
/// richer: a client that got `bottom`/`top` the wrong way round, or that filled in
/// the wrong `EffectChoiceAnswer` variant, gets a message naming the mismatch
/// instead of `InvalidCommand("CR 608.2d: answer ... does not answer question ...")`
/// rendered through the engine-rejection path.
///
/// # What is deliberately left to the engine
///
/// * **CR 601.2c cross-slot distinctness** for two `TargetPermanentDistinctFrom`
///   slots naming the same permanent. That is a rules judgment about the
///   requirement, not about the response, and re-deriving it here would be the
///   OOS-RS-2 drift class.
/// * **CR 514.1's step gate and the `choice_id` moment guard.** Both are facts
///   about the game's state at apply time, not about the response — and the wire
///   `seq` check already covers the staleness those protect against.
/// * Everything about whether the decision is still the pending one at all.
fn validate_decision_params(
    action: &mtg_simulator::LegalAction,
    params: &crate::view::ActionParamsDto,
) -> Result<(), ApiFailure> {
    use mtg_engine::{EffectChoiceAnswer, EffectChoiceQuestion};
    use mtg_simulator::LegalAction;

    let bad = |message: String| ApiFailure::new(StatusCode::BAD_REQUEST, "bad_params", message);

    /// Membership + duplication for one flat id list against one candidate set.
    fn check_ids(
        submitted: &[mtg_engine::ObjectId],
        allowed: &[mtg_engine::ObjectId],
        what: &str,
        cr: &str,
    ) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        for id in submitted {
            if !allowed.contains(id) {
                return Err(format!(
                    "object {} is not among the cards this {what} offered ({cr}); this \
                     decision offered {:?}",
                    id.0,
                    allowed.iter().map(|o| o.0).collect::<Vec<_>>()
                ));
            }
            if !seen.insert(id.0) {
                return Err(format!(
                    "object {} appears in the {what} answer twice",
                    id.0
                ));
            }
        }
        Ok(())
    }

    match action {
        // CR 514.1 / CR 701.9b. Empty means "accept the engine's default" (see
        // `ActionParams::discard_cards`), so an empty list skips every check here —
        // the default is by construction exactly `count` cards from the hand.
        LegalAction::DiscardToHandSize { count, hand, .. } => {
            if params.discard_cards.is_empty() {
                return Ok(());
            }
            check_ids(&params.discard_cards, hand, "cleanup discard", "CR 514.1").map_err(bad)?;
            if params.discard_cards.len() != *count as usize {
                return Err(bad(format!(
                    "CR 514.1 requires exactly {count} card(s) to be discarded, got {}",
                    params.discard_cards.len()
                )));
            }
            Ok(())
        }
        // CR 608.2d. The variant check comes first: answering a scry with a search
        // answer is the mistake most likely to be a client bug rather than a typo,
        // and reporting it as "you answered the wrong question" beats reporting the
        // membership failure that would follow from it.
        LegalAction::AnswerEffectChoice { question, .. } => {
            let Some(answer) = params.effect_choice_answer.as_ref() else {
                return Ok(());
            };
            match (question, answer) {
                (
                    EffectChoiceQuestion::SearchLibrary {
                        candidates,
                        may_fail_to_find,
                    },
                    EffectChoiceAnswer::SearchLibrary { found },
                ) => {
                    match found {
                        // CR 701.23a.
                        Some(id) => {
                            check_ids(std::slice::from_ref(id), candidates, "search", "CR 701.23a")
                                .map_err(bad)?
                        }
                        // CR 701.23d: an unrestricted search MUST find. The picker
                        // is told this through `may_decline: false` and should not
                        // offer the button at all.
                        None if !may_fail_to_find => {
                            return Err(bad(
                                "CR 701.23d: this search is not for a card with a stated \
                                 quality, so failing to find is not legal"
                                    .to_string(),
                            ))
                        }
                        None => {}
                    }
                    Ok(())
                }
                (
                    EffectChoiceQuestion::Scry { looked_at },
                    EffectChoiceAnswer::Scry { bottom, top },
                ) => check_partition(looked_at, bottom, top, "scry", "CR 701.22a").map_err(bad),
                (
                    EffectChoiceQuestion::Surveil { looked_at },
                    EffectChoiceAnswer::Surveil { graveyard, top },
                ) => {
                    check_partition(looked_at, graveyard, top, "surveil", "CR 701.25a").map_err(bad)
                }
                // CR 701.9b (ENG-1): exactly `count`, no duplicates, every one from
                // the hand this decision offered.
                (
                    EffectChoiceQuestion::Discard { hand, count },
                    EffectChoiceAnswer::Discard { chosen },
                ) => {
                    check_ids(chosen, hand, "discard", "CR 701.9b").map_err(bad)?;
                    if chosen.len() != *count as usize {
                        return Err(bad(format!(
                            "CR 701.9b: this effect discards exactly {count} card(s), got {}",
                            chosen.len()
                        )));
                    }
                    Ok(())
                }
                // PB-DX28 (CR 115.10 / CR 608.2): every id drawn from `candidates`,
                // no duplicates, exactly `min(count, candidates.len())` when
                // `!up_to` ("as much as possible"), `<= count` when `up_to`.
                (
                    EffectChoiceQuestion::ChooseObject {
                        candidates,
                        count,
                        up_to,
                    },
                    EffectChoiceAnswer::ChooseObject { chosen },
                ) => {
                    check_ids(chosen, candidates, "choose object", "CR 115.10").map_err(bad)?;
                    let expected = (*count as usize).min(candidates.len());
                    if *up_to {
                        if chosen.len() > *count as usize {
                            return Err(bad(format!(
                                "CR 115.10 / CR 608.2: this choice picks UP TO {count}, got {}",
                                chosen.len()
                            )));
                        }
                    } else if chosen.len() != expected {
                        return Err(bad(format!(
                            "CR 115.10 / CR 608.2: this choice picks exactly {expected} \
                             object(s) (as much as possible, of {count} wanted), got {}",
                            chosen.len()
                        )));
                    }
                    Ok(())
                }
                _ => Err(bad(format!(
                    "CR 608.2d: this decision asked a {} question; the answer given is a \
                     different kind",
                    question_kind(question)
                ))),
            }
        }
        // CR 603.3d / CR 601.2c (OOS-DP8-2).
        LegalAction::ChooseTriggerTargets { slots, .. } => {
            if params.trigger_targets.is_empty() {
                return Ok(());
            }
            if params.trigger_targets.len() != slots.len() {
                return Err(bad(format!(
                    "CR 601.2c: this trigger has {} target slot(s), the answer names {}",
                    slots.len(),
                    params.trigger_targets.len()
                )));
            }
            for (i, (submitted, slot)) in params.trigger_targets.iter().zip(slots).enumerate() {
                // CR 601.2c: an `optional` slot is `UpToN` and takes 0..=max; every
                // other slot takes exactly one. `handle_choose_trigger_targets`'
                // own step-6 check, read off the same two fields.
                let ok = if slot.optional {
                    submitted.len() <= slot.max as usize
                } else {
                    submitted.len() == 1
                };
                if !ok {
                    return Err(bad(format!(
                        "CR 601.2c: slot {i} takes {}, the answer names {}",
                        if slot.optional {
                            format!("up to {}", slot.max)
                        } else {
                            "exactly 1".to_string()
                        },
                        submitted.len()
                    )));
                }
                let mut seen = Vec::new();
                for target in submitted {
                    if !slot.candidates.iter().any(|c| &c.target == target) {
                        return Err(bad(format!(
                            "{target:?} is not a legal choice for trigger target slot {i} \
                             (CR 603.3d)"
                        )));
                    }
                    if seen.contains(&target) {
                        return Err(bad(format!(
                            "trigger target slot {i} names the same target twice (CR 601.2c)"
                        )));
                    }
                    seen.push(target);
                }
            }
            Ok(())
        }
        // Every other variant: `params.rs` refuses these three fields on it with
        // `ParamError::UnsupportedParam` -> 400, so there is nothing to add.
        _ => Ok(()),
    }
}

/// UI-2 (CR 118.8 / CR 702.157): check a submitted `AdditionalCost::Sacrifice` /
/// `AdditionalCost::Squad` against the `AdditionalCostPlan` this decision's
/// `CastSpell` option actually offered, before anything is applied.
///
/// SIM-6 (CR 602.2) added the second half: a submitted `cost_sacrifice_target` /
/// `cost_discard_card` is checked against the `ActivationCostPlan` this decision's
/// `ActivateAbility` option offered. The name is now half a misnomer — an
/// activation cost is not an `AdditionalCost` and never becomes one — but the two
/// halves answer the same question at the same boundary, and one function that
/// sees every cost answer is worth more than two that each see half.
///
/// # Same boundary as `validate_combat_params` / `validate_decision_params`
///
/// The engine checks all of this too, and would refuse an out-of-set sacrifice
/// or an over-count Squad payment — but as a **422** carrying a
/// `GameStateError`, i.e. "the engine looked at your command and said no". An
/// answer naming a permanent this decision never offered as eligible, or a
/// Squad count this decision never said was affordable, is wrong *against the
/// response the client is holding*, with no game state needed to see it — this
/// crate's own definition of a 400.
///
/// # Nine of `AdditionalCost`'s FIFTEEN variants are surfaced
///
/// **The count in this heading was wrong until PB-DX29 and the correction is worth
/// keeping**: the enum has **15** variants, not sixteen, and Kicker is not one of them
/// (it is `CastSpellData.kicker_times`). UI-2's own count was off by one and its list
/// named a variant that does not exist.
///
/// UI-2 rendered a picker for `Sacrifice` and `Squad`. PB-DX29 added `Replicate`,
/// `EscalateModes`, `Entwine`, `Fuse`, `Offspring`, `Gift` and `Splice`, so this
/// function now speaks for **nine**.
///
/// The remaining six fall through to `Ok(())` and are judged, if at all, by the
/// **engine's** 422 — and every one of them has a stated reason rather than being
/// residue:
///
/// * `Mutate` — carried per-ACTION (`LegalAction::CastWithMutate`), not announced here.
/// * `Assist` — deliberately NOT surfaced: CR 702.132a needs the assisting player's
///   agreement and no cross-seat decision machinery exists, so a picker would let one
///   human spend another's floating mana without asking.
/// * `CollectEvidenceExile` — reachable but has zero deck-legal members, and is the
///   only kind whose mandatory/optional status is a per-def flag.
/// * `Discard` (Retrace / Jump-Start), `EscapeExile`, `ExileFromHand` — **unreachable
///   by construction**: `StubProvider`'s cast loops walk Hand and Command zone only
///   (no graveyard), and `params.rs` hard-codes `alt_cost: None`.
///
/// The rule this function still obeys is unchanged: it can only speak authoritatively
/// about kinds it renders an offer for. A check written against no offer is a check
/// against nothing.
///
/// `pub(crate)` rather than private: unit-tested directly from the crate's
/// `tests` module (`main.rs`), which is a sibling module and cannot see a plain
/// private item here — the same reason `REBUILD_FAILURE_SEED` (`main.rs`'s
/// `tests` module) is `pub(crate)` rather than private.
pub(crate) fn validate_additional_cost_params(
    action: &mtg_simulator::LegalAction,
    params: &crate::view::ActionParamsDto,
    state: &mtg_engine::GameState,
    player: mtg_engine::PlayerId,
) -> Result<(), ApiFailure> {
    use mtg_engine::AdditionalCost;
    use mtg_simulator::legal_actions::{CountCostKind, MarkerCostKind};
    use mtg_simulator::LegalAction;

    let bad = |message: String| ApiFailure::new(StatusCode::BAD_REQUEST, "bad_params", message);

    // CR 602.2 (SIM-6): the ACTIVATION half. Same boundary and same reason as the
    // cast half below -- an id this decision never offered is wrong against the
    // response the client is holding, with no game state needed to see it.
    //
    // Checked BEFORE the `CastSpell` destructure, and the two are disjoint: an
    // activation answer on a `CastSpell` (or on any third variant) falls through to
    // the `_` arm here and is refused as "no such cost to pay", rather than being
    // silently ignored the way an unread param would be.
    if let LegalAction::ActivateAbility {
        activation_costs, ..
    } = action
    {
        if let Some(chosen) = params.cost_sacrifice_target {
            let Some(sac) = activation_costs.sacrifice.as_ref() else {
                return Err(bad(
                    "CR 602.2: this ability has no sacrifice cost to pay".to_string()
                ));
            };
            if !sac.eligible.contains(&chosen) {
                return Err(bad(format!(
                    "object {} is not among the permanents this sacrifice cost offered \
                     (CR 602.2); this decision offered {:?}",
                    chosen.0,
                    sac.eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                )));
            }
        }
        if let Some(chosen) = params.cost_discard_card {
            let Some(dis) = activation_costs.discard.as_ref() else {
                return Err(bad(
                    "CR 602.2 / CR 111.10g: this ability has no discard cost to pay".to_string(),
                ));
            };
            if !dis.eligible.contains(&chosen) {
                return Err(bad(format!(
                    "object {} is not among the cards this discard cost offered (CR 602.2); \
                     this decision offered {:?}",
                    chosen.0,
                    dis.eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                )));
            }
        }
        // The mirror image of the guard below (`/review` finding 2). CR 118.8's
        // `additional_costs` array belongs to a `CastSpell`; `params.rs`'s
        // `ActivateAbility` arm never reads it, and `ActivateAbility` IS in that
        // function's consuming allowlist, so `first_announced_field` will not catch
        // it either — an array sent here would be dropped in silence. Refusing both
        // directions is the only symmetric answer.
        if !params.additional_costs.is_empty() {
            return Err(bad(
                "CR 118.8: `additional_costs` describes a SPELL's additional costs; an \
                 activated ability's costs are answered with cost_sacrifice_target / \
                 cost_discard_card"
                    .to_string(),
            ));
        }
        return Ok(());
    }

    // An activation-cost answer on anything that is not an `ActivateAbility`. The
    // simulator's own `first_announced_field` would reject it for every
    // non-consuming variant, but `CastSpell` IS a consuming variant, so a
    // `cost_sacrifice_target` sent alongside a cast would otherwise be dropped in
    // silence (`params.rs`' own "residual, deliberately not guarded" note).
    if params.cost_sacrifice_target.is_some() || params.cost_discard_card.is_some() {
        return Err(bad(
            "CR 602.2: activation-cost answers (cost_sacrifice_target / cost_discard_card) \
             belong to an ActivateAbility decision, and this decision is not one"
                .to_string(),
        ));
    }

    let LegalAction::CastSpell {
        additional_costs: plan,
        ..
    } = action
    else {
        return Ok(());
    };

    // **At most one entry of each kind**, checked before anything else.
    //
    // The offer carries at most one `SacrificeCostView` and at most one
    // `SquadCostView`, so a second entry of either kind is an announcement this
    // response never made — a 400 by this crate's own definition. It is also the
    // only reading that is unambiguous, because the engine resolves duplicates
    // SILENTLY and the two kinds resolve them DIFFERENTLY: `casting.rs`'s
    // destructuring loop assigns `squad_count` (so the LAST `Squad` wins) while its
    // sacrifice extraction is a `find_map` over `ids.first()` (so the FIRST
    // `Sacrifice` wins and the rest are dropped with no error and no diagnostic).
    // A client that sent two would get one of them applied and never be told which.
    //
    // `legal_actions::effective_cast_cost_with_additional` mirrors the engine's
    // last-wins arithmetic for Squad regardless, so a non-HTTP caller cannot make
    // the auto-tap and the engine disagree; this guard is about not letting the
    // ambiguity onto the wire in the first place.
    let sacrifice_entries = params
        .additional_costs
        .iter()
        .filter(|c| matches!(c, AdditionalCost::Sacrifice { .. }))
        .count();
    if sacrifice_entries > 1 {
        return Err(bad(format!(
            "CR 118.8: this spell has at most one additional sacrifice cost, but {sacrifice_entries} \
             were announced; the engine would silently apply only the first"
        )));
    }
    let squad_entries = params
        .additional_costs
        .iter()
        .filter(|c| matches!(c, AdditionalCost::Squad { .. }))
        .count();
    if squad_entries > 1 {
        return Err(bad(format!(
            "CR 702.157a: announce the squad count once, not {squad_entries} times; the engine \
             would silently apply only the last"
        )));
    }

    // PB-DX29: the same at-most-one rule for the seven kinds this batch surfaced, and
    // for the same reason — `casting.rs`'s destructuring loop is a plain assignment for
    // every one of them, so a second entry silently overwrites the first with no error
    // and no diagnostic. Table-driven so a kind cannot be added to the offer and
    // forgotten here; the discriminant is matched rather than the payload, because two
    // `Gift`s naming DIFFERENT opponents is exactly the ambiguity being refused.
    for (label, cr, is_kind) in DUPLICABLE_COST_KINDS {
        let n = params
            .additional_costs
            .iter()
            .filter(|c| is_kind(c))
            .count();
        if n > 1 {
            return Err(bad(format!(
                "{cr}: announce the {label} cost once, not {n} times; the engine's own \
                 destructuring loop assigns rather than accumulates, so it would silently \
                 apply only the last"
            )));
        }
    }

    for cost in &params.additional_costs {
        match cost {
            // CR 118.8: exactly one id, and it must be among the permanents this
            // decision offered as eligible.
            AdditionalCost::Sacrifice { ids, .. } => {
                let Some(sac) = plan.sacrifice.as_ref() else {
                    return Err(bad(
                        "CR 118.8: this spell has no additional sacrifice cost to pay".to_string(),
                    ));
                };
                if ids.len() != 1 {
                    return Err(bad(format!(
                        "CR 118.8: this spell's additional cost is a single mandatory \
                         sacrifice, got {} id(s)",
                        ids.len()
                    )));
                }
                if !sac.eligible.contains(&ids[0]) {
                    return Err(bad(format!(
                        "object {} is not among the permanents this sacrifice cost offered \
                         (CR 118.8); this decision offered {:?}",
                        ids[0].0,
                        sac.eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                    )));
                }
            }
            // CR 702.157a: the spell must actually have a Squad cost, and the
            // count must be within what this decision said was affordable.
            AdditionalCost::Squad { count } => {
                let Some(squad) = plan.squad.as_ref() else {
                    return Err(bad(
                        "CR 702.157a: this spell has no squad cost to pay".to_string()
                    ));
                };
                if *count > squad.max_count {
                    return Err(bad(format!(
                        "CR 702.157a: at most {} squad payment(s) are affordable right now, \
                         got {count}",
                        squad.max_count
                    )));
                }
            }
            // ── PB-DX29: the seven kinds this batch surfaced ──────────────────
            //
            // Each check is the same shape as Squad's: the OFFER must have carried this
            // kind, and the answer must be within what the offer said. Nothing is
            // re-derived — every bound is read off the plan the same response carried.
            AdditionalCost::Replicate { count } => {
                let Some(opt) = count_option(plan, CountCostKind::Replicate) else {
                    return Err(bad(
                        "CR 702.56a: this spell has no replicate cost to pay".to_string()
                    ));
                };
                if *count > opt.max_count {
                    return Err(bad(format!(
                        "CR 702.56a: at most {} replicate payment(s) are affordable right now, \
                         got {count}",
                        opt.max_count
                    )));
                }
            }
            AdditionalCost::EscalateModes { count } => {
                let Some(opt) = count_option(plan, CountCostKind::Escalate) else {
                    return Err(bad(
                        "CR 702.120a: this spell has no escalate cost to pay".to_string()
                    ));
                };
                if *count > opt.max_count {
                    return Err(bad(format!(
                        "CR 702.120a: at most {} additional mode(s) are affordable right now, \
                         got {count}",
                        opt.max_count
                    )));
                }
            }
            AdditionalCost::Entwine if !marker_is_affordable(plan, MarkerCostKind::Entwine) => {
                return Err(bad(marker_refusal(
                    "entwine",
                    "CR 702.42a",
                    plan,
                    MarkerCostKind::Entwine,
                )));
            }
            AdditionalCost::Fuse if !marker_is_affordable(plan, MarkerCostKind::Fuse) => {
                return Err(bad(marker_refusal(
                    "fuse",
                    "CR 702.102a/b",
                    plan,
                    MarkerCostKind::Fuse,
                )));
            }
            AdditionalCost::Offspring if !marker_is_affordable(plan, MarkerCostKind::Offspring) => {
                return Err(bad(marker_refusal(
                    "offspring",
                    "CR 702.175a",
                    plan,
                    MarkerCostKind::Offspring,
                )));
            }
            AdditionalCost::Gift { opponent } => {
                let Some(gift) = plan.gift.as_ref() else {
                    return Err(bad(
                        "CR 702.174a: this spell has no gift to give".to_string()
                    ));
                };
                if !gift.eligible.contains(opponent) {
                    return Err(bad(format!(
                        "player {} is not among the opponents this gift offered (CR 702.174a); \
                         this decision offered {:?}",
                        opponent.0,
                        gift.eligible.iter().map(|p| p.0).collect::<Vec<_>>()
                    )));
                }
            }
            AdditionalCost::Splice { cards } => {
                let Some(splice) = plan.splice.as_ref() else {
                    return Err(bad(
                        "CR 702.47a: no card in your hand can be spliced onto this spell"
                            .to_string(),
                    ));
                };
                for card in cards {
                    if !splice.eligible.contains(card) {
                        return Err(bad(format!(
                            "card {} is not among the cards this splice offer accepted \
                             (CR 702.47a); this decision offered {:?}",
                            card.0,
                            splice.eligible.iter().map(|o| o.0).collect::<Vec<_>>()
                        )));
                    }
                }
                // CR 702.47b: "one or more OTHER cards" -- each may be spliced once. The
                // engine refuses a repeat too, but as a 422; a list containing the same
                // id twice is wrong against the offer itself.
                let mut seen: std::collections::BTreeSet<mtg_engine::ObjectId> =
                    std::collections::BTreeSet::new();
                for card in cards {
                    if !seen.insert(*card) {
                        return Err(bad(format!(
                            "CR 702.47b: card {} is spliced twice; each card may be spliced at \
                             most once",
                            card.0
                        )));
                    }
                }
            }
            // **PB-DX29 `/review` M1: DEFAULT-DENY, not default-allow.**
            //
            // This arm was `_ => {}` — a fall-through that let the six unrendered kinds
            // reach the engine unchecked. The doc above argued that Assist is safe
            // because PB-DX29 "deliberately did not surface" it; **not surfacing closes
            // the picker and does not close the wire.** Proven by execution during the
            // review: a raw POST casting Huddle Up with
            // `additional_costs: [{"Assist":{"player":2,"amount":2}}]` passed this
            // boundary and the engine ACCEPTED it, moving P2's mana pool 5 → 3 without
            // P2 ever being asked (CR 702.132a). Every argument in that doc about why a
            // kind is deferred is an argument about the PICKER; this is the wire.
            //
            // Refusing here is exactly this function's own stated rule: an answer naming
            // something *this decision never offered* is wrong against the payload the
            // client is holding, with no game state needed to see it. None of the six is
            // ever rendered, so none of them is ever an offer.
            // The three marker arms above are `if !marker_is_affordable(..)` GUARDS, so a
            // marker the plan DID offer and DID call payable falls through them. These
            // three arms accept it. (A first draft of the default-deny below omitted
            // them and every legal marker answer became a 400 — caught by
            // `test_dx29_validate_accepts_one_legal_answer_of_every_family`, which exists
            // precisely so a blanket refusal cannot masquerade as a check.)
            AdditionalCost::Entwine | AdditionalCost::Fuse | AdditionalCost::Offspring => {}

            // **PB-DX29 `/review` M1: the six unsurfaced kinds are NAMED and REFUSED,
            // not waved through.**
            //
            // This was `_ => {}` — a fall-through that let them reach the engine
            // unchecked. The doc above argued Assist is safe because PB-DX29
            // "deliberately did not surface" it; **not surfacing closes the picker and
            // does not close the wire.** Proven by execution during the review: a raw
            // POST casting Huddle Up with
            // `additional_costs: [{"Assist":{"player":2,"amount":2}}]` passed this
            // boundary and the engine ACCEPTED it, moving P2's mana pool 5 -> 3 without
            // P2 ever being asked (CR 702.132a). Every argument in that doc about why a
            // kind is deferred is an argument about the PICKER; this is the wire.
            //
            // Written as six NAMED arms rather than a wildcard, so a sixteenth
            // `AdditionalCost` variant is a compile error here — which is the class
            // `OOS-UI2-4` describes: a kind arriving with nobody noticing.
            AdditionalCost::Assist { .. }
            | AdditionalCost::Mutate { .. }
            | AdditionalCost::Discard(_)
            | AdditionalCost::EscapeExile { .. }
            | AdditionalCost::CollectEvidenceExile { .. }
            | AdditionalCost::ExileFromHand { .. } => {
                return Err(bad(format!(
                    "this decision offered no such additional cost, so it cannot be \
                     answered: {}. PB-DX29 surfaces nine of `AdditionalCost`'s fifteen \
                     kinds; the other six are deferred with a stated reason each (see \
                     `validate_additional_cost_params`' doc) and are refused here rather \
                     than forwarded to the engine unchecked",
                    cost_kind_name(cost)
                )));
            }
        }
    }

    // ── PB-DX29 `/review` H1: the WHOLE ANSWER must be affordable ────────────────
    //
    // Every per-kind bound above is computed for that kind ALONE — `max_count` for a
    // count, `affordable` for a marker — and `SpliceCostOption` carries no bound at all,
    // for the reason its own doc gives: bounding the OFFER would be a subset-sum over
    // `eligible`, because each spliced card costs a different amount.
    //
    // **That is a reason not to publish a maximum in the offer. It is not a reason to
    // skip the check here**, where the chosen list is known and the arithmetic is one
    // call. Proven by execution during the review: with `Reach Through Mists` and
    // `Glacial Ray` (both `Complete`, both deck-legal) and one blue mana, the splice
    // offer was made, this boundary accepted the answer, and the engine returned
    // **422 `InsufficientMana`** — a clean offer followed by a server rejection, the
    // exact SR-38 shape this batch exists to delete, one family over from the marker
    // affordability the batch had already fixed.
    //
    // Checking the whole announced vector rather than each rider closes a second gap in
    // the same move: two riders each affordable alone and unaffordable together. No
    // corpus def carries two mana-bearing riders today (pinned by
    // `core::pb_dx29_additional_cost_roster`), so that half is latent — but it is closed
    // by construction rather than by the corpus happening to be small.
    //
    // The arithmetic is `effective_cast_cost_with_additional` + `can_afford`, the same
    // pair the provider's own bounds walk, so this cannot disagree with them and
    // inherits their stated `OOS-UI2-3` under-report and no new one.
    if !params.additional_costs.is_empty() {
        if let LegalAction::CastSpell { card, .. } = action {
            // **Fails OPEN when the cost cannot be computed at all**, and that is the
            // correct direction rather than a convenience.
            // `effective_cast_cost_with_additional` returns `None` for a card this state
            // does not hold, or a rider whose cost the def does not declare — neither is
            // evidence of UNaffordability, and this boundary's whole contract is that it
            // refuses only what it can positively judge against the payload the client is
            // holding. The engine still judges the rest.
            if let Some(cost) = mtg_simulator::legal_actions::effective_cast_cost_with_additional(
                state,
                player,
                *card,
                &params.additional_costs,
            ) {
                if !mtg_simulator::legal_actions::can_afford(state, player, &cost) {
                    return Err(bad(
                        "CR 601.2f-h: this answer's additional costs are not payable on top of \
                         the spell's own cost. Every rider this decision offered is bounded \
                         individually; a combination of them, or a splice list, can still \
                         exceed what you can pay -- refused here rather than as an engine \
                         rejection, so the refusal names the offer rather than the game state"
                            .to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

/// CR 606.6 (PB-DX29 `/review` M2): an announced `{X}` on a `-X` loyalty ability must
/// not exceed the loyalty counters the permanent actually has.
///
/// # This check exists because PB-DX29 opened the channel and bounded nothing
///
/// `x_value` was hard-coded `None` before this batch, so nothing could over-announce it.
/// The batch made it announceable and stopped there: `legal_actions.rs` offers a
/// `LoyaltyCost::MinusX` ability unconditionally (correct — CR 606.6 permits X = 0),
/// `view.rs::action_needs_x` tells the client an `{X}` exists, and `ValuePrompt.svelte`
/// renders a bare number input with **no ceiling**. Measured during the review on
/// `chandra_flamecaller` (`Complete`, deck-legal, 4 loyalty): announcing X = 9 reached
/// the engine and came back
/// `422 InvalidCommand("ActivateLoyaltyAbility: insufficient loyalty counters (4
/// available, 9 needed) (CR 606.6)")`.
///
/// A clean offer followed by a server rejection — on the half the batch was primarily
/// dispatched for, while it was building `max_count` bounds for counts and an
/// `affordable` bound for markers. The bound is one integer the server already holds.
///
/// **Only `MinusX` is bounded**, because it is the only loyalty cost whose paid amount
/// is the activator's choice; `Plus`/`Minus`/`Zero` are fixed numbers and an `x_value`
/// announced alongside one is simply unread by the engine (this function leaves that
/// alone rather than inventing a second rule the engine does not have).
pub(crate) fn validate_loyalty_x_value(
    action: &mtg_simulator::LegalAction,
    params: &crate::view::ActionParamsDto,
    state: &mtg_engine::GameState,
) -> Result<(), ApiFailure> {
    use mtg_simulator::LegalAction;
    let LegalAction::ActivateLoyaltyAbility {
        source,
        ability_index,
    } = action
    else {
        return Ok(());
    };
    if !mtg_engine::loyalty_ability_needs_x(state, *source, *ability_index) {
        return Ok(());
    }
    // CR 606.6: the engine reads `x_value.unwrap_or(0)` and refuses when the resulting
    // negative cost exceeds the counters present. Same number, read the same way.
    let available = state
        .objects()
        .get(source)
        .and_then(|o| o.counters.get(&mtg_engine::CounterType::Loyalty).copied())
        .unwrap_or(0);
    if params.x_value > available {
        return Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "bad_params",
            format!(
                "CR 606.6: this planeswalker has {available} loyalty counter(s), so a -X                  ability cannot be activated for X = {}. The engine would refuse this too,                  but as a 422 -- refused here so the message names the offer rather than the                  game state",
                params.x_value
            ),
        ));
    }
    Ok(())
}

/// PB-DX29 `/review` L3: say WHICH of the two things went wrong.
///
/// `marker_is_affordable` folds two questions into one boolean — "did the plan offer this
/// rider at all?" and "did it say the rider is payable?" — which is right for the guard
/// and wrong for the message. The first draft told a human casting Goblin War Party with
/// four Mountains that *"this spell has no entwine cost to pay"*, on a card that plainly
/// prints Entwine {2}{R}. Behaviour right, diagnosis wrong — and a 400's whole job is to
/// name the part of the payload the client is holding that its answer contradicts.
fn marker_refusal(
    word: &str,
    cr: &str,
    plan: &mtg_simulator::legal_actions::AdditionalCostPlan,
    kind: mtg_simulator::legal_actions::MarkerCostKind,
) -> String {
    if plan.markers.iter().any(|m| m.kind == kind) {
        format!(
            "{cr}: this spell's {word} cost is not payable right now -- this decision offered \
             it and marked it unaffordable on top of the spell's own cost. Tap more mana, or \
             cast without it"
        )
    } else {
        format!(
            "{cr}: this decision offered no {word} cost to pay -- this spell has none, or (for \
             fuse, CR 702.102a/d) it cannot be fused from this zone or with a targeted right \
             half"
        )
    }
}

/// PB-DX29 `/review` M1: the printed name of a cost kind, for the default-deny message.
///
/// Exhaustive with **no wildcard arm**: a sixteenth `AdditionalCost` variant must be
/// named here or this crate stops compiling, which is the point — the class
/// `OOS-UI2-4` describes is exactly a kind arriving with nobody noticing.
fn cost_kind_name(cost: &mtg_engine::AdditionalCost) -> &'static str {
    use mtg_engine::AdditionalCost as A;
    match cost {
        A::Sacrifice { .. } => "Sacrifice (CR 118.8)",
        A::Discard(_) => "Discard (CR 702.15a Retrace / CR 702.133a Jump-Start)",
        A::EscapeExile { .. } => "EscapeExile (CR 702.138a)",
        A::CollectEvidenceExile { .. } => "CollectEvidenceExile (CR 701.59a)",
        A::Assist { .. } => "Assist (CR 702.132a)",
        A::Replicate { .. } => "Replicate (CR 702.56a)",
        A::Squad { .. } => "Squad (CR 702.157a)",
        A::EscalateModes { .. } => "EscalateModes (CR 702.120a)",
        A::Splice { .. } => "Splice (CR 702.47a)",
        A::Entwine => "Entwine (CR 702.42a)",
        A::Fuse => "Fuse (CR 702.102a)",
        A::Offspring => "Offspring (CR 702.175a)",
        A::Gift { .. } => "Gift (CR 702.174a)",
        A::Mutate { .. } => "Mutate (CR 702.140a)",
        A::ExileFromHand { .. } => "ExileFromHand (CR 118.9)",
    }
}

/// PB-DX29: the cost kinds whose second entry silently overwrites the first.
///
/// `Sacrifice` and `Squad` are checked above with their own bespoke messages (UI-2
/// wrote those and they name the first-wins/last-wins asymmetry); this table covers the
/// seven PB-DX29 added. `Mutate` and the unreachable kinds are absent because no offer
/// carries them, so "the offer never made this announcement" is not a claim this
/// function is entitled to make about them.
#[allow(clippy::type_complexity)]
const DUPLICABLE_COST_KINDS: &[(&str, &str, fn(&mtg_engine::AdditionalCost) -> bool)] = &[
    ("replicate", "CR 702.56a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Replicate { .. })
    }),
    ("escalate", "CR 702.120a", |c| {
        matches!(c, mtg_engine::AdditionalCost::EscalateModes { .. })
    }),
    ("entwine", "CR 702.42a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Entwine)
    }),
    ("fuse", "CR 702.102a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Fuse)
    }),
    ("offspring", "CR 702.175a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Offspring)
    }),
    ("gift", "CR 702.174a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Gift { .. })
    }),
    ("splice", "CR 702.47a", |c| {
        matches!(c, mtg_engine::AdditionalCost::Splice { .. })
    }),
];

/// PB-DX29: the plan's descriptor for a pay-N-times rider, if it offered one.
fn count_option(
    plan: &mtg_simulator::legal_actions::AdditionalCostPlan,
    kind: mtg_simulator::legal_actions::CountCostKind,
) -> Option<&mtg_simulator::legal_actions::CountCostOption> {
    plan.counts.iter().find(|c| c.kind == kind)
}

/// PB-DX29: did the plan offer this pay-or-not rider, AND say it was payable?
///
/// **The affordability half is not decoration and its absence was a live defect.** A
/// count rider is bounded by `max_count` and an over-count is a 400; a marker has no
/// count, and the first draft checked presence alone. Measured: with the base cost
/// affordable and the rider not, the offer carried a tickable Entwine and ticking it
/// returned `422 "player does not have enough mana to pay the cost"` — a clean offer
/// followed by a server rejection, on the batch that exists to delete them. Folding
/// affordability in here turns that into a 400 that names the offer.
///
/// One function rather than a presence check plus a separate affordability check,
/// because two call sites that must agree are the drift class `OOS-RS-2` names.
fn marker_is_affordable(
    plan: &mtg_simulator::legal_actions::AdditionalCostPlan,
    kind: mtg_simulator::legal_actions::MarkerCostKind,
) -> bool {
    plan.markers.iter().any(|m| m.kind == kind && m.affordable)
}

/// CR 701.22a / CR 701.25a: the two piles must PARTITION `whole` — same multiset,
/// no duplicates, nothing else. `effects::validate_partition`'s check, restated at
/// the response boundary for the 400/422 reason in [`validate_decision_params`]'s
/// doc; the engine still runs its own.
fn check_partition(
    whole: &[mtg_engine::ObjectId],
    a: &[mtg_engine::ObjectId],
    b: &[mtg_engine::ObjectId],
    what: &str,
    cr: &str,
) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for id in a.iter().chain(b.iter()) {
        if !whole.contains(id) {
            return Err(format!(
                "object {} is not among the {} card(s) this {what} looked at ({cr}); this \
                 decision offered {:?}",
                id.0,
                whole.len(),
                whole.iter().map(|o| o.0).collect::<Vec<_>>()
            ));
        }
        if !seen.insert(id.0) {
            return Err(format!(
                "object {} is put in both piles of this {what} ({cr})",
                id.0
            ));
        }
    }
    if a.len() + b.len() != whole.len() {
        return Err(format!(
            "{cr}: a {what} answer must account for all {} card(s) looked at; this one \
             accounts for {}",
            whole.len(),
            a.len() + b.len()
        ));
    }
    Ok(())
}

/// Display tag for an [`mtg_engine::EffectChoiceQuestion`], for the wrong-variant
/// message.
///
/// Enumerated rather than `Debug`-ed for **message quality**, not for redaction —
/// and the distinction is worth stating, because an earlier version of this comment
/// claimed the latter and was contradicted by `check_ids` (nested inside
/// [`validate_decision_params`]) and [`check_partition`], which both format the
/// candidate ids straight into their own 400 bodies.
///
/// Those ids are fine there and a `Debug` here would be fine too: every id in the
/// question was just sent to this seat on the wire, in this decision, as the answer
/// space it is being asked to choose from. What a `Debug` would *not* be is
/// readable — "this decision asked a scry question" is a diagnosis; a dumped
/// `EffectChoiceQuestion::Scry { looked_at: [ObjectId(97), ObjectId(98)] }` is a
/// thing the client has to parse to learn the same fact.
fn question_kind(question: &mtg_engine::EffectChoiceQuestion) -> &'static str {
    use mtg_engine::EffectChoiceQuestion;
    match question {
        EffectChoiceQuestion::SearchLibrary { .. } => "library search",
        EffectChoiceQuestion::Scry { .. } => "scry",
        EffectChoiceQuestion::Surveil { .. } => "surveil",
        EffectChoiceQuestion::Discard { .. } => "discard",
        EffectChoiceQuestion::ChooseObject { .. } => "choose object",
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
    //
    // **The `filter` is Architecture Invariant 7, made structural** (UI-1 review,
    // HIGH 2). `view::question_card_label` renders the REAL name of a library card
    // — a scry's `looked_at`, a search's `candidates` — and its whole safety
    // argument is that the ids come out of an `EffectChoiceQuestion` the engine
    // minted *for the seat this payload is being rendered for*
    // (`GameEvent::EffectChoiceRequired::private_to()` names exactly one player).
    //
    // Nothing enforced the emphasised half. It held only by arithmetic on a
    // one-element set: `session::config_for` hard-codes `human_seats: [HUMAN_SEAT]`,
    // so `session.human` is the only seat `LocalGame` ever parks a decision for, so
    // `pending.player` happened to always equal it. Break that assumption and seat
    // A's scry candidates would render, **with their real names**, into seat B's
    // payload, through a channel both older Invariant-7 gates are blind to (one
    // scans for omniscient view-model entry points, the other for another seat's
    // *hand* names).
    //
    // A decision addressed to another seat is now simply absent from this one's
    // payload: a seat is not entitled to know what another seat is being asked.
    // [`post_action`] refuses the matching write for the same reason — see the
    // guard there, and note that the two are NOT redundant, because
    // `PlaySession::pending_wire_seq` does not read `human` at all and the 409 body
    // discloses the current `seq` verbatim, so a client that guessed could
    // otherwise have acted on a decision this filter had hidden from it.
    //
    // **This does not make the crate M10a-ready, and should not be read as
    // claiming to.** `PlaySession::human` is a single `PlayerId` taken from
    // `cfg.human_seats`' lowest element, so a genuine second human seat would find
    // its decisions withheld with no channel to answer them — correct redaction,
    // deadlocked gameplay. The actual missing piece is a per-request viewer. What
    // this pair of guards buys is that the failure is fail-closed rather than a
    // leak.
    let decision = session
        .pending
        .as_ref()
        .filter(|pending| pending.player == human)
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
        // Read from the SAME `session.names` map `StateViewModel` is built with, so
        // the string is guaranteed to be a key of `state_view.players` rather than a
        // reconstruction that merely ought to be. See `GameSummary::human_name`.
        human_name: view::display_name(human, &session.names),
        bot: format!("{:?}", session.cfg.bot_kind),
        // NO `seed` — review MR-M11-01 (HIGH). See `GameSummary`'s doc: the seed plus
        // these fields reconstruct every other seat's opening hand and library order,
        // which is exactly what Architecture Invariant 7 forbids this payload from
        // carrying. It lives on `BugReportView` alone, which is opt-in and documented
        // as the one deliberate exception.
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

        // **Architecture Invariant 7's write half** (UI-1 re-review, LOW 3). The
        // read half — `seat_view`'s `pending.player == human` filter — hides a
        // decision addressed to another seat. Hiding it does not stop this seat
        // *answering* it: `LocalGame::submit` builds the command for
        // `pending.player` and has no notion of a viewer, so a submission naming a
        // hidden decision's `seq` was **accepted**. Not reasoned to — with this
        // guard deleted the re-review's probe gets HTTP 200 and the other seat's
        // scry is applied.
        //
        // The `seq` was obtainable, too: `PlaySession::pending_wire_seq` does not
        // read `human`, and a 409 `stale_decision` body carries the current `seq`
        // verbatim, so a deliberate stale post would have disclosed it. **This
        // guard's own placement closes that**, which is why it sits above the `seq`
        // check rather than beside it: a foreign decision now answers 409
        // `no_pending_decision` with no `expected` field, so the seq is never
        // handed out in the first place. (`seq` is a small monotonic integer and
        // guessable regardless — the disclosure was never the load-bearing part.)
        //
        // So the two guards are a pair, not a duplicate. 409 rather than 403,
        // matching the sibling case below it: from this seat's point of view there
        // is genuinely nothing outstanding to answer.
        //
        // Unreachable today for the same reason the read half is — `config_for`
        // hard-codes one human seat. Both exist so that stops being load-bearing.
        if let Some(pending) = play.pending.as_ref() {
            if pending.player != play.human {
                return Err(ApiFailure::new(
                    StatusCode::CONFLICT,
                    "no_pending_decision",
                    "the outstanding decision belongs to another seat; there is \
                     nothing for this seat to answer",
                ));
            }
        }

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
                // UI-1 (CR 514.1 / CR 608.2d / CR 603.3d): same boundary, same
                // reason — an answer naming something the response never offered
                // is a 400, not an engine rejection.
                validate_decision_params(action, &req.params)?;
                // UI-2 (CR 118.8 / CR 702.157): same boundary again, for the two
                // additional-cost kinds this crate renders a picker for.
                // PB-DX29 `/review` H1/M1/M2: the state and the deciding seat are
                // threaded in, because three of this function's checks need them.
                // `pending.player` rather than a request field — the seat is the
                // server's, never the client's (Architecture Invariant 7).
                let deciding_seat = play
                    .pending
                    .as_ref()
                    .map(|pending| pending.player)
                    .unwrap_or(mtg_engine::PlayerId(0));
                validate_additional_cost_params(
                    action,
                    &req.params,
                    play.game.state(),
                    deciding_seat,
                )?;
                // PB-DX29 `/review` M2 (CR 606.6): the loyalty `{X}` this batch opened.
                validate_loyalty_x_value(action, &req.params, play.game.state())?;
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
/// 1. The redeal rebuilds the **whole table**, not one seat: every seat's
///    library is reshuffled and every seat's hand redrawn, and it cannot
///    represent a partially decided table (CR 103.5c gives each player their own
///    mulligan count). `redeal`'s own doc says the per-seat model "belongs with
///    the play-server pregame flow" — this session keeps the whole-table
///    rebuild, because a per-seat model needs each bot seat to be *asked*, which
///    is a new decision channel rather than a small addition.
///
///    What it no longer does is re-roll the **decklists and commanders**. It did
///    until `scutemob-187` (G2 of `memory/playtest-triage-2026-08-02b.md`): the
///    session held `DeckSource::RandomPerSeat`, a recipe keyed on the seed, so a
///    perturbed seed rebuilt all four decks and all four commanders — visible to
///    everyone, because CR 903.6 puts the commander in the public command zone.
///    `session::new_game` now resolves the decks once and stores
///    `DeckSource::Fixed`, so a redeal permutes a fixed multiset (CR 103.5).
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
        } else {
            // CR 103.5: "Once a player chooses not to take a mulligan, the remaining
            // cards become that player's opening hand." Terminal — record it, so a
            // second request cannot redeal the hand this one accepted (review
            // MR-M11-10). Before this the choice lived only in the browser's own
            // `keptHand` flag.
            play.keep_hand();
        }
        let outcome = play.advance();
        Ok(Json(seat_view(play, &outcome)))
    })
}

/// `GET /api/game/report` — the bug-report / repro artefact (M11-local S8, plan
/// item 5; `docs/mtg-engine-runtime-integrity.md` Layer 3).
///
/// `{seed, config, protocol/hash versions, final state hash, journal}` as JSON. See
/// [`crate::view::BugReportView`] for the shape, how to replay it, and — the part
/// worth reading before adding a second consumer — why this is the **one** payload
/// in this crate that is not seat-redacted, and what has to change about that at
/// M10a.
///
/// **A pure read.** Unlike [`get_game`] it does not call `advance()` and does not
/// move `journal_cursor`, so requesting a report can neither change the game nor
/// consume event lines the live feed has not shipped yet. It therefore takes the
/// lock immutably — a report can be pulled from a game parked on a decision without
/// disturbing the decision.
pub async fn get_report(
    State(state): State<SharedState>,
) -> Result<Json<view::BugReportView>, ApiFailure> {
    tokio::task::block_in_place(|| {
        let guard = state.session.lock().map_err(|_| poisoned())?;
        let play = guard.as_ref().ok_or_else(no_session)?;
        Ok(Json(view::bug_report_view(play)))
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

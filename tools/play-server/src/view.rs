//! Wire DTOs for the play client, and the server-side rendering that produces
//! them.
//!
//! M11-local Session 5, plan item 4 (`memory/m11-session-plan.md` §4).
//!
//! # `LegalAction` is NEVER serialized
//!
//! This is a hard rule of the design, not an optimisation. The client receives
//! an [`ActionOptionView`] carrying an `index` and a rendered `label`; to act it
//! posts that `index` plus [`ActionParamsDto`]. The server maps the index back
//! through the `PendingDecision` it is still holding (`session.pending`), and
//! `LocalGame::submit` builds the `Command` for `pending.player`. Consequences:
//!
//! * the browser never needs an engine type, so no engine enum is a wire type
//!   and no `Command`/`GameEvent`/`Effect` variant is added anywhere (plan §3,
//!   "Wire-format impact — none");
//! * a stale tab cannot act on a superseded action list — the `seq` check
//!   catches it. That holds *across a session rebuild* too, which needs more
//!   than `LocalGame`'s own counter: `POST /api/game` and
//!   `POST /api/game/mulligan` both call `LocalGame::start`, which restarts
//!   `decision_seq` at 0, so the play server adds `PlaySession::seq_base` to
//!   make the **wire** `seq` monotonic for the life of the process. A `seq` from
//!   a superseded game is therefore strictly below the current base and is
//!   rejected as stale rather than matched by coincidence (S5 review MEDIUM 1);
//! * a command naming another seat is structurally unrepresentable.
//!
//! # Every name rendered here comes from the seat-redacted view
//!
//! Architecture Invariant 7. The state view is built with
//! `StateViewModel::from_game_state_for(.., Viewer::Seat(human))` — **never**
//! `from_game_state`, which is the omniscient replay-viewer path — and every
//! label is rendered from a [`NameIndex`] derived from *that already-redacted
//! view*, never from `state.objects()`.
//!
//! This is the S4 handoff's HIGH finding applied forward: **redaction follows
//! the rendering site, not the zone.** A face-down attacker is on the
//! battlefield, so a zone-shaped redaction "covers" it while a label that names
//! it leaks. An id the redacted view does not identify (a hidden hand card, a
//! face-down permanent, an object in a zone the seat may not read) renders as
//! [`HIDDEN_LABEL`], never as its printed name. `redact::viewer_may_identify` is
//! `pub(crate)` to `mtg-view-model` and deliberately not re-exported, so the
//! redacted view really is the only channel.

use std::collections::HashMap;

use mtg_engine::{
    AbilityDefinition, AdditionalCost, AttackTarget, Effect, EffectChoiceAnswer,
    EffectChoiceQuestion, GameState, ModeSelection, ObjectId, PlayerId, Target, TargetRequirement,
};
use mtg_simulator::{
    ActionParams, DecisionKind, GameResult, HaltReason, LegalAction, PendingDecision,
};
use mtg_view_model::{EventView, StateViewModel};
use serde::{Deserialize, Serialize};

/// What a label says about an object the viewing seat may not identify.
pub const HIDDEN_LABEL: &str = "(hidden card)";

/// What a label says about an object that is not in the seat's view at all
/// (already left the zone the action referenced, or lives in a zone the view
/// model does not model). Distinct from [`HIDDEN_LABEL`] so a UI bug and a
/// redaction are not confused for each other.
pub const UNKNOWN_LABEL: &str = "(unknown card)";

// ── Top-level payloads ────────────────────────────────────────────────────────

/// Static-ish facts about the game in progress, so the client does not have to
/// infer them from the state view.
#[derive(Debug, Serialize)]
pub struct GameSummary {
    pub players: u32,
    /// The seat this payload is redacted for.
    pub human: u64,
    /// `"Heuristic"` or `"Random"`.
    pub bot: String,
    /// **The seed is NOT here, and its absence is Architecture Invariant 7** (review
    /// MR-M11-01, HIGH).
    ///
    /// S5 shipped a `seed: u64` on this struct, and S5's own review LOW 7 noted —
    /// approvingly, as a *reproducibility* property — that `(seed, players, bot,
    /// mulligan_count)` reproduce the table exactly. That is the same sentence read
    /// the other way round: `setup::build_initial_state` is deterministic in its
    /// `LocalGameConfig` alone, and `session::config_for` fixes every other input
    /// (`human_seats = {PlayerId(1)}`, `DeckSource::RandomPerSeat`, the limits), so
    /// those fields rebuild **every other seat's opening hand and library order** —
    /// precisely the pair Invariant 7 names. It shipped on the *default* payload, on
    /// every response, and the frontend rendered it in the header.
    ///
    /// Neither of the milestone's two Invariant-7 gates could see it, and that is the
    /// durable part: the HTTP leak scan looks for another seat's card **names**, and
    /// the source gate looks for omniscient **view-model entry points**. A seed is
    /// neither. **A redaction gate checks the channel it was written for; a
    /// reconstruction key is a different channel.**
    ///
    /// The seed still exists on [`BugReportView`], which is opt-in, is documented as
    /// the one deliberately unredacted payload, and carries the M10a re-scope
    /// obligation. Putting it there and nowhere else is what makes the exception
    /// contained rather than nominal.
    pub turn: u32,
    /// Commands applied so far — 0 exactly while the game is still pregame.
    pub command_count: u32,
    /// CR 103.5: pregame redeals this seat has taken. Kept because the client shows
    /// it, and harmless on its own — it is only half a reproduction key, and the other
    /// half (the seed) is no longer here. See the note above.
    pub mulligan_count: u32,
    /// True while `POST /api/game/mulligan` is still accepted.
    pub pregame: bool,
}

/// Everything one seat is entitled to know right now.
///
/// The plan's field list is `{ state, decision, events, game_over }`; `summary`
/// is additive (the plan names `GameSummary` as a DTO but gives it no home).
#[derive(Debug, Serialize)]
pub struct SeatView {
    pub summary: GameSummary,
    pub state: StateViewModel,
    pub decision: Option<DecisionView>,
    /// Rendered, already-redacted history lines since the client last read.
    pub events: Vec<EventView>,
    pub game_over: Option<GameOverView>,
}

/// The decision the human must answer, if any.
#[derive(Debug, Serialize)]
pub struct DecisionView {
    /// Echo this back in `POST /api/game/action`. A mismatch is a 409.
    pub seq: u64,
    /// [`DecisionKind`] rendered as a tag string. `DecisionKind` is
    /// `#[non_exhaustive]`, so [`decision_kind_tag`] carries a wildcard arm and
    /// this is a `String`, never a client-side enum.
    pub kind: String,
    /// Always the human seat — a decision is only ever handed out for a seat the
    /// server is holding.
    ///
    /// **Additive to plan item 4's field list** (S5 re-review LOW 12): the plan
    /// names `{ seq, kind, actions }`. Kept because a client that renders "it is
    /// *your* turn to act" should not have to infer the seat from `summary.human`
    /// and trust that the two agree.
    pub player: u64,
    pub actions: Vec<ActionOptionView>,
}

/// One selectable action. `index` is the only thing the client sends back.
#[derive(Debug, Serialize)]
pub struct ActionOptionView {
    /// Index into the pending decision's `actions`. The whole submission
    /// protocol.
    pub index: usize,
    /// Stable machine tag for the `LegalAction` variant, for client-side
    /// grouping and icon choice. Not a serialized `LegalAction`.
    pub kind: String,
    /// Rendered server-side from the seat-redacted view — see the module doc.
    pub label: String,
    /// The primary object this action is about, when it has one (the card being
    /// cast, the permanent being tapped). Purely so the client can highlight it.
    pub object_id: Option<u64>,
    /// CR 601.2c: per-slot legal target candidates, populated from
    /// `crates/engine/src/rules/queries.rs` (`spell_target_requirements` /
    /// `ability_target_requirements` + `legal_targets_per_slot`).
    ///
    /// Outer index = slot, in the same order the engine's requirement list is in,
    /// which is the order `Command::CastSpell`'s / `Command::ActivateAbility`'s
    /// `targets` vector must be in. Per the S4 handoff, these labels are a fifth
    /// rendering site and are built from the seat-redacted view exactly as
    /// [`NameIndex`] does for action labels.
    ///
    /// **Empty for a modal spell whose targets are per-mode** (CR 700.2c): the
    /// slots then live on each [`ModeOptionView`] instead, because which slots
    /// exist depends on which modes the human picks. See [`ModeOptionView::target_slots`].
    pub target_slots: Vec<TargetSlotView>,
    /// CR 601.2c: how many targets the engine will accept in total, from
    /// `mtg_engine::target_count_range`. `TargetRequirement::UpToN` is why the
    /// two can differ — "up to N target creatures" legally takes fewer.
    pub target_min: usize,
    pub target_max: usize,
    /// CR 107.3 / 601.2b: this action needs an `x_value` announced.
    ///
    /// Answered for **both** `CastSpell` (the spell's own `{X}`) and
    /// `ActivateAbility` (the `{X}` in its `ActivationCost`) — see
    /// [`action_needs_x`]. The ability half is S7 closing the README's
    /// Limitation 5, which had let an `{X}` ability be announced as 0 in silence.
    pub needs_x: bool,
    /// CR 700.2: selectable modes, empty for a non-modal action.
    pub modes: Vec<ModeOptionView>,
    /// CR 700.2a: how many modes must be chosen. Both 0 when [`Self::modes`] is
    /// empty.
    pub mode_min: usize,
    pub mode_max: usize,
    /// CR 508.1: present exactly on a `DeclareAttackers` option.
    pub attack: Option<AttackOptionsView>,
    /// CR 509.1: present exactly on a `DeclareBlockers` option.
    pub block: Option<BlockOptionsView>,
    /// CR 509.2: present exactly on an `OrderBlockers` option.
    pub order: Option<OrderBlockersOptionsView>,
    /// The full answer space of a **blocking decision** — a cleanup discard (CR
    /// 514.1), a resolution-time effect choice (CR 608.2d: search / scry /
    /// surveil), or a trigger's target announcement (CR 603.3d).
    ///
    /// `None` on every other action. See [`BlockingDecisionView`].
    pub decision: Option<BlockingDecisionView>,
}

/// One blocking decision, as everything a client needs to render a real picker
/// (UI-1; `memory/playtest-triage-2026-08-02.md` F8).
///
/// # Why this exists
///
/// `StubProvider` bakes the engine's own deterministic default into the
/// `LegalAction` — cleanup discard = the `count` highest `ObjectId`s, scry/surveil
/// = the identity partition, search = `candidates.first()` — precisely so a *bot*
/// can submit it and always be accepted (SR-38). The candidate data rides along on
/// the same action so a *human* client can render a choice instead
/// (`legal_actions.rs`'s own doc says so). Until UI-1 this view layer threw that
/// data away, so the browser rendered one bare button that submitted the default:
/// the game discarded the right-hand cards for you and resolved every scry as a
/// no-op.
///
/// # Generic on purpose
///
/// The three questions are one *shape* problem, not three: pick a subset, pick one,
/// split a pile, fill some slots. So a client renders [`Self::answer`]'s **shape**,
/// not [`Self::question`] — and a fourth question that reuses an existing shape
/// needs no new client code at all. That claim is not aspirational here:
/// `ChooseTriggerTargets` (OOS-DP8-2, filed as the *identical* gap) is carried by
/// [`AnswerShapeView::Slots`], whose payload is the same `Vec<TargetSlotView>` the
/// CR 601.2c target picker already renders.
///
/// # Hidden information (Architecture Invariant 7)
///
/// Every label here is seat-redacted, but **not all of them come from
/// [`NameIndex`]**, and that difference is the interesting part — see
/// [`question_card_label`].
#[derive(Debug, Serialize)]
pub struct BlockingDecisionView {
    /// Stable machine tag for the question class: `"CleanupDiscard"`,
    /// `"SearchLibrary"`, `"Scry"`, `"Surveil"`, `"TriggerTargets"`. For display
    /// and telemetry; a client that switches on this rather than on
    /// [`Self::answer`]'s shape is doing more work than it needs to.
    pub question: String,
    /// One-line prompt, rendered server-side from the seat-redacted labels.
    pub prompt: String,
    /// Which [`ActionParamsDto`] field carries the answer to this question:
    /// `"discard_cards"`, `"effect_choice_answer"` or `"trigger_targets"`.
    ///
    /// Sent rather than inferred from the shape because the mapping is
    /// many-to-one (`PickOne` and `Partition` both answer through
    /// `effect_choice_answer`), and a client that inferred it would be a second
    /// place that has to know the params schema.
    pub answer_field: String,
    /// The answer's shape — which picker to render, and what to render it over.
    pub answer: AnswerShapeView,
}

/// How a [`BlockingDecisionView`] is answered. One variant per *shape*, not per
/// question — see that type's doc.
///
/// Externally tagged under a `shape` key, so the client's dispatch is
/// `decision.answer.shape`.
#[derive(Debug, Serialize)]
#[serde(tag = "shape")]
pub enum AnswerShapeView {
    /// CR 514.1 / CR 701.9b: choose **exactly** [`Self::Subset::count`] of
    /// `candidates`. The answer is the chosen ids, into
    /// `ActionParamsDto::discard_cards`.
    Subset {
        candidates: Vec<CardOptionView>,
        /// `handle_discard_to_hand_size` requires exactly this many, with no
        /// duplicates, each in the player's own hand.
        count: usize,
        /// The engine's own default subset — the `count` highest `ObjectId`s
        /// (`default_cleanup_discard`). Sent so a client can offer "accept the
        /// default" as one click, and so a test can assert it drove something else.
        default: Vec<u64>,
    },
    /// CR 701.23a/b: choose one of `candidates`, or none iff `may_decline`. The
    /// answer goes into `ActionParamsDto::effect_choice_answer`.
    PickOne {
        candidates: Vec<CardOptionView>,
        /// CR 701.23b vs CR 701.23d. `false` for an unrestricted "search your
        /// library for a card", where finding is MANDATORY and a `found: null`
        /// answer is refused by the engine — so a client must not offer the button.
        may_decline: bool,
        /// See [`AnswerShapeView::Partition::template`].
        template: EffectChoiceAnswer,
        /// The key inside `template`'s single variant object that the chosen id
        /// goes in (`"found"`). See `template`.
        found_key: String,
    },
    /// CR 701.22a / CR 701.25a: split `looked_at` into two piles. The answer goes
    /// into `ActionParamsDto::effect_choice_answer`.
    Partition {
        /// The cards this player is looking at, **top-first**
        /// (`Zone::top_n`'s own order).
        looked_at: Vec<CardOptionView>,
        /// The key inside `template`'s variant object for the pile that stays on
        /// the library — always `"top"`, and top-first. See `template`.
        kept_key: String,
        /// The key for the other pile: `"bottom"` for a scry, `"graveyard"` for a
        /// surveil.
        moved_key: String,
        /// Prose for the other pile, for the picker's own heading.
        moved_label: String,
        /// The engine's own default answer, **serialized verbatim** — the same
        /// argument as [`TargetOptionView::value`], and load-bearing here.
        ///
        /// `EffectChoiceAnswer` is an externally-tagged enum, so this arrives as
        /// `{"Scry":{"bottom":[],"top":[20,21]}}`. A client answers by cloning it,
        /// keeping its single key, and replacing the arrays named by `kept_key` /
        /// `moved_key`. It therefore **never spells the variant name itself**, and
        /// the wire encoding of `EffectChoiceAnswer` stays known in exactly one
        /// place (the engine). A client that built `{"Scry": ...}` from scratch
        /// would be a second place for that encoding to drift.
        template: EffectChoiceAnswer,
    },
    /// CR 603.3d / CR 601.2c: one target list per slot. The answer goes into
    /// `ActionParamsDto::trigger_targets`, outer index = slot.
    ///
    /// **The extension proof for OOS-DP8-2** ([`BlockingDecisionView`]'s doc):
    /// this reuses [`TargetSlotView`] unchanged, which is what the CR 601.2c
    /// target picker already renders — so a trigger's target announcement needs no
    /// new picker component and no new answer encoding.
    Slots {
        slots: Vec<TargetSlotView>,
        /// The engine's own default announcement
        /// (`abilities::default_trigger_targets`), one entry per slot.
        default: Vec<Vec<Target>>,
    },
}

/// One card in a blocking decision's answer space, with a seat-redacted label.
///
/// Shaped like [`CombatantOptionView`] and deliberately not the same type: that
/// one names a creature on the battlefield, which is public (CR 400.1), and this
/// one can name a card in a **hidden** zone that the effect has told this seat to
/// look at. Sharing a type would invite sharing the labelling path, and the two
/// labelling paths are exactly what must not be confused — see
/// [`question_card_label`].
#[derive(Debug, Serialize)]
pub struct CardOptionView {
    pub id: u64,
    pub label: String,
}

/// One target slot: its own count range, plus every candidate legal for it.
///
/// # Why this is a struct and not a bare `Vec<TargetOptionView>`
///
/// A slot is **one `TargetRequirement`**, and a requirement is not always worth
/// one target. `TargetRequirement::UpToN { count }` is a single requirement that
/// admits up to `count` targets — `casting::target_count_range` adds `count` to
/// the maximum for it, and `validate_targets_inner`'s second pass assigns
/// several announced targets to that one slot.
///
/// So a client holding only `Vec<Vec<TargetOptionView>>` plus a *collective*
/// `(min, max)` cannot tell **which** slot the slack belongs to, and the obvious
/// reading — one pick per slot — silently caps an "up to two" spell at one
/// target. That is not hypothetical: `force_of_vigor` is `Complete` and
/// deck-legal with exactly one `UpToN { count: 2 }` requirement, so "destroy up
/// to two target artifacts and/or enchantments" would have destroyed at most
/// one. Caught in review, not by a test — no seeded game in the S7 fixture sweep
/// dealt such a card.
///
/// [`Self::min`] / [`Self::max`] are this slot's own contribution, computed by
/// handing `mtg_engine::target_count_range` a one-element slice, so they cannot
/// drift from the collective [`ActionOptionView::target_min`]/`target_max`
/// (which is the same function over the whole list).
#[derive(Debug, Serialize)]
pub struct TargetSlotView {
    /// 0 for an `UpToN` slot, 1 for every other requirement.
    pub min: usize,
    /// `count` for an `UpToN { count }` slot, 1 for every other requirement.
    pub max: usize,
    /// Every `Target` the engine would accept in this slot, in the engine's own
    /// order.
    pub candidates: Vec<TargetOptionView>,
}

/// One legal target for one slot.
#[derive(Debug, Serialize)]
pub struct TargetOptionView {
    /// `"object"` or `"player"`.
    pub kind: String,
    pub id: u64,
    /// Redacted display text.
    pub label: String,
    /// The engine's own `Target`, serialized verbatim.
    ///
    /// **Additive to the plan's `{kind, id, label}` sketch, and deliberately so.**
    /// `ActionParamsDto::targets` is `Vec<Target>`, i.e. `{"Object": 12}` /
    /// `{"Player": 2}`. A client that had only `kind` and `id` would have to
    /// *reconstruct* that encoding, which is a second place for the wire shape to
    /// be known and therefore a place for it to drift. With this field the client
    /// echoes back exactly what the server sent and `kind`/`id` are for display
    /// and highlighting only.
    pub value: Target,
}

/// One selectable mode of a modal spell or ability (CR 700.2).
#[derive(Debug, Serialize)]
pub struct ModeOptionView {
    pub index: usize,
    pub label: String,
    /// CR 700.2c/700.2f: this mode's own target slots, when the card declares
    /// per-mode target requirements (`ModeSelection.mode_targets`). Empty when
    /// the card's targets are flat, in which case [`ActionOptionView::target_slots`]
    /// carries them instead.
    pub target_slots: Vec<TargetSlotView>,
    /// CR 601.2c for *this mode's* slots, from `mtg_engine::target_count_range`.
    ///
    /// The option-level [`ActionOptionView::target_min`]/`target_max` are
    /// computed with an empty `modes_chosen` and are therefore `(0, 0)` for a
    /// per-mode-targeting card (`action_target_requirements`' doc says why), so
    /// a client that has just let the human pick modes has nothing usable at the
    /// option level. It sums these instead.
    ///
    /// Added because the S7 frontend agent found the gap and reported it rather
    /// than guessing a range — its first pass had to approximate every per-mode
    /// slot as mandatory, which is wrong for a mode carrying `UpToN`.
    pub target_min: usize,
    pub target_max: usize,
}

/// CR 508.1: what a `DeclareAttackers` option may declare.
#[derive(Debug, Serialize)]
pub struct AttackOptionsView {
    /// Creatures that may be declared as attackers, from the provider's
    /// `LegalAction::DeclareAttackers { eligible, .. }`.
    pub eligible: Vec<CombatantOptionView>,
    /// CR 508.1a: what each attacker may be declared as attacking — a player or
    /// a planeswalker — from the same `LegalAction`'s `targets`.
    pub targets: Vec<AttackTargetOptionView>,
}

/// CR 509.1: what a `DeclareBlockers` option may declare.
#[derive(Debug, Serialize)]
pub struct BlockOptionsView {
    /// Creatures that may be declared as blockers, from the provider's
    /// `LegalAction::DeclareBlockers { eligible, .. }`.
    pub eligible: Vec<CombatantOptionView>,
    /// CR 509.1a: the attacking creatures a blocker may be assigned to, from the
    /// same `LegalAction`'s `attackers`.
    pub attackers: Vec<CombatantOptionView>,
}

/// CR 509.2: what an `OrderBlockers` option may reorder (M11-local S8, item 2).
///
/// `blockers` is the candidate set **in the engine's own default order** — that is
/// exactly what `apply_combat_damage` uses when no order has been set, so a client
/// that echoes it back unchanged (or sends an empty `blocker_order`) changes
/// nothing. The client's job is to let the human permute this list; the first entry
/// is assigned damage first and must be dealt lethal before damage flows to the next.
#[derive(Debug, Serialize)]
pub struct OrderBlockersOptionsView {
    /// The attacking creature whose damage order this is.
    pub attacker: CombatantOptionView,
    /// Every creature blocking [`Self::attacker`], in the engine's default order.
    pub blockers: Vec<CombatantOptionView>,
}

/// One creature in a combat declaration, with a seat-redacted label.
#[derive(Debug, Serialize)]
pub struct CombatantOptionView {
    pub id: u64,
    pub label: String,
}

/// CR 508.1a: one thing an attacker may attack.
#[derive(Debug, Serialize)]
pub struct AttackTargetOptionView {
    /// `"player"` or `"planeswalker"`.
    pub kind: String,
    pub id: u64,
    pub label: String,
    /// The engine's own `AttackTarget`, serialized verbatim — same argument as
    /// [`TargetOptionView::value`].
    pub value: AttackTarget,
}

/// Why the game stopped. Covers both `AdvanceOutcome::GameOver` (CR 104) and
/// `AdvanceOutcome::Halted` (a `LocalGameLimits` safety valve), because from the
/// client's point of view both mean "no further decision is coming".
#[derive(Debug, Serialize)]
pub struct GameOverView {
    /// Display name of the last player standing, if there is one.
    pub winner: Option<String>,
    pub turn_count: u32,
    pub total_commands: usize,
    /// `true` when a safety valve tripped rather than the game concluding.
    pub halted: bool,
    /// Human-readable reason; `None` for a clean win.
    ///
    /// **Redaction-safe by construction, not by filtering** (review MR-M11-08): this
    /// is built from [`halt_reason_summary`], which reads only the *variant* and its
    /// numeric fields. It is never a `Debug` of an engine error — see that function.
    pub reason: Option<String>,
    /// Simulator invariant violations observed during the game: **check name and turn
    /// only**, never the description.
    ///
    /// # Why the description is dropped (review MR-M11-08)
    ///
    /// `InvariantViolation::description` is free-form text produced by
    /// `crates/simulator/src/invariants.rs`, and at least one check interpolates a card
    /// name directly off `GameState` — `check_no_orphaned_tokens` formats
    /// `obj.characteristics.name`. That is a rendering site, and it is **outside both**
    /// of this crate's Invariant-7 chokepoints: it never passes through
    /// `StateViewModel::from_game_state_for` and never through [`NameIndex`], so a
    /// redaction that follows the rendering site (the S4 lesson) does not cover it.
    ///
    /// The description is not lost — it is on [`BugReportView::violations`], the opt-in
    /// route that is documented as the one deliberately unredacted payload. What the
    /// seat view needs is *that a violation happened and which check fired*, which is
    /// enough to tell the play-tester to export a report; the detail belongs in the
    /// report.
    pub violations: Vec<String>,
}

// ── Bug-report export (M11-local S8, plan item 5) ─────────────────────────────

/// The self-contained reproduction artefact `GET /api/game/report` returns, per
/// `docs/mtg-engine-runtime-integrity.md` Layer 3.
///
/// # This is the ONE payload in this crate that is not seat-redacted
///
/// Every other response goes through the Architecture Invariant 7 chokepoint
/// (`StateViewModel::from_game_state_for(.., Viewer::Seat(human))` and
/// `event_view_for(.., Viewer::Seat(human))`). This one deliberately does not, and
/// the reason is that a redacted repro is not a repro: [`Self::journal`] carries raw
/// `Command`s and raw `GameEvent`s, which is exactly what a maintainer needs to
/// replay a defect, and a redacted `Command::AnswerEffectChoice` naming a searched
/// library card would be unusable.
///
/// That is **safe only because of what M11-local is**: one human, three bots, one
/// process, no networking (see the crate README's scope note). The only "other
/// players" whose hidden information this exposes are simulator bots in the same
/// process as the person requesting the file. **When M10a puts a real opponent on
/// the other end of a socket this endpoint must be re-scoped** — either redacted, or
/// restricted to a single-player game, or authenticated. That is recorded here, in
/// the README, and in `memory/decisions.md` rather than left to be rediscovered.
///
/// # Reproducing from it
///
/// [`Self::seed`] plus [`Self::config`] rebuild the exact table:
/// `mtg_simulator::setup::build_initial_state` is deterministic in `cfg.seed`
/// (`test_setup_same_seed_same_state_hash`), and after `mulligan_count` redeals the
/// effective seed is `redeal_seed(seed, human_seat, mulligan_count)`. Replaying
/// [`Self::journal`]'s commands in order from that state reaches
/// [`Self::state_hash`].
///
/// [`Self::protocol_version`] / [`Self::hash_schema_version`] are what make that
/// claim checkable rather than hopeful: a repro is only valid against an engine
/// build with the same two numbers (`crates/simulator/src/bin/fuzzer.rs`'s "repro
/// seeds are not portable across engine changes").
#[derive(Debug, Serialize)]
pub struct BugReportView {
    /// The **base** seed — see [`GameSummary::seed`]; combine with
    /// [`ReportConfigView::mulligan_count`] for the effective one.
    pub seed: u64,
    pub config: ReportConfigView,
    /// `mtg_engine::PROTOCOL_VERSION` at capture time.
    pub protocol_version: u32,
    /// `mtg_engine::PROTOCOL_SCHEMA_FINGERPRINT` at capture time.
    pub protocol_fingerprint: String,
    /// `mtg_engine::HASH_SCHEMA_VERSION` at capture time.
    pub hash_schema_version: u8,
    /// Lowercase hex of `GameState::public_state_hash()` for the final state.
    pub state_hash: String,
    pub turn: u32,
    pub command_count: u32,
    /// Simulator invariant violations seen across the whole game, stringified.
    pub violations: Vec<String>,
    /// Every command applied, with the events it produced, in order. **Raw** — see
    /// the type doc.
    ///
    /// Empty when `LocalGameLimits::record_journal` is off; the play server sets it
    /// on (`session::config_for`), which is what makes this endpoint useful at all.
    pub journal: Vec<JournalEntryView>,
}

/// The half of the reproduction key that is not the seed.
#[derive(Debug, Serialize)]
pub struct ReportConfigView {
    pub players: u32,
    /// The seat the human occupies.
    pub human_seat: u64,
    /// `"Heuristic"` or `"Random"`.
    pub bot: String,
    /// CR 103.5 pregame redeals taken — part of the effective seed.
    pub mulligan_count: u32,
    pub max_turns: u32,
    pub max_commands: u32,
    pub max_consecutive_passes: u32,
}

/// One applied command and its events. Both are the engine's own wire types,
/// serialized verbatim rather than re-encoded, so a consumer parses exactly what
/// `crates/engine` defines.
#[derive(Debug, Serialize)]
pub struct JournalEntryView {
    pub turn: u32,
    pub command: mtg_engine::Command,
    pub events: Vec<mtg_engine::GameEvent>,
}

// ── Request DTOs ──────────────────────────────────────────────────────────────

/// Optional overrides for `POST /api/game`. Every field falls back to the CLI
/// default when omitted, and the whole body may be omitted.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NewGameRequest {
    pub players: Option<u32>,
    /// `"heuristic"` or `"random"`, case-insensitive.
    pub bot: Option<String>,
    pub seed: Option<u64>,
}

/// `POST /api/game/action` body.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRequest {
    pub seq: u64,
    pub action_index: usize,
    #[serde(default)]
    pub params: ActionParamsDto,
}

/// Client-facing mirror of `mtg_simulator::ActionParams`.
///
/// `ActionParams` itself derives neither `Serialize` nor `Deserialize` — it is a
/// simulator-internal assembly type that deliberately never crosses the wire
/// (`params.rs`'s module doc). Its *field* types all do (`Target`,
/// `AttackTarget`, `AdditionalCost`, `ObjectId` are all `Serialize +
/// Deserialize` in `crates/card-types`), so this DTO reuses them verbatim rather
/// than inventing a parallel encoding that could drift from the engine's.
///
/// Every field defaults, so `{}` is a valid params object for an action that
/// announces nothing.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ActionParamsDto {
    /// CR 601.2c.
    pub targets: Vec<Target>,
    /// CR 601.2b / 107.3m.
    pub x_value: u32,
    /// CR 700.2.
    pub modes_chosen: Vec<usize>,
    /// CR 508.1.
    pub attackers: Vec<(ObjectId, AttackTarget)>,
    /// CR 509.1.
    pub blockers: Vec<(ObjectId, ObjectId)>,
    /// CR 509.2: the chosen damage-assignment order for an `OrderBlockers` action,
    /// front to back. Empty means "keep the engine's default order", which is the
    /// order [`OrderBlockersOptionsView::blockers`] was sent in.
    pub blocker_order: Vec<ObjectId>,
    /// CR 103.5.
    pub cards_to_bottom: Vec<ObjectId>,
    pub additional_costs: Vec<AdditionalCost>,
    /// CR 514.1 / CR 701.9b (UI-1): the cards chosen for a cleanup discard.
    /// Empty means "accept the engine's default" — see `ActionParams::discard_cards`.
    pub discard_cards: Vec<ObjectId>,
    /// CR 608.2d (UI-1): the answer to a search / scry / surveil, as the engine's
    /// own `EffectChoiceAnswer` **verbatim** rather than a re-encoding. `null`
    /// means "accept the engine's default".
    ///
    /// This is `Target`'s argument applied to a second type
    /// (`TargetOptionView::value`): the client echoes back a mutated copy of the
    /// `template` the server sent on `AnswerShapeView::PickOne`/`Partition`, so the
    /// externally-tagged encoding of this enum is known in exactly one place.
    pub effect_choice_answer: Option<EffectChoiceAnswer>,
    /// CR 603.3d / CR 601.2c (UI-1, OOS-DP8-2): per-slot targets for a trigger's
    /// announcement, outer index = slot. Empty means "accept the engine's default".
    /// Each `Target` is echoed verbatim from a `TargetSlotView` candidate's `value`.
    pub trigger_targets: Vec<Vec<Target>>,
    /// Tap mana sources on the human's behalf before a `CastSpell`, but only
    /// when the existing pool cannot already cover the cost (see
    /// `LocalGame::auto_tap_commands_for`). Defaults to `true`: a browser client
    /// has no mana-tapping UI yet, and a cast that silently cannot be paid for
    /// is the single most confusing failure on this surface.
    #[serde(default = "default_auto_tap")]
    pub auto_tap: bool,
}

fn default_auto_tap() -> bool {
    true
}

/// Hand-written, **not** `#[derive(Default)]` (review MR-M11-05).
///
/// `ActionRequest::params` is `#[serde(default)]`, so an **omitted** `params` key built
/// `ActionParamsDto::default()` — and a derived `Default` gives `auto_tap: false`,
/// while a present-but-empty `"params": {}` runs serde's field defaults and gives
/// `auto_tap: true` (`default_auto_tap`). Two spellings a client would reasonably
/// consider identical produced *different game behaviour*: with `auto_tap: false` a
/// `CastSpell` whose cost is not already in the pool is refused 422, and the difference
/// is invisible in the request.
///
/// Writing the impl by hand rather than annotating the field is deliberate: it makes
/// the two paths share one source of truth, so a field added later cannot reintroduce
/// the divergence by being defaulted in only one of them.
impl Default for ActionParamsDto {
    fn default() -> Self {
        ActionParamsDto {
            targets: Vec::new(),
            x_value: 0,
            modes_chosen: Vec::new(),
            attackers: Vec::new(),
            blockers: Vec::new(),
            blocker_order: Vec::new(),
            cards_to_bottom: Vec::new(),
            additional_costs: Vec::new(),
            discard_cards: Vec::new(),
            effect_choice_answer: None,
            trigger_targets: Vec::new(),
            auto_tap: default_auto_tap(),
        }
    }
}

impl From<ActionParamsDto> for ActionParams {
    fn from(dto: ActionParamsDto) -> Self {
        ActionParams {
            targets: dto.targets,
            x_value: dto.x_value,
            modes_chosen: dto.modes_chosen,
            attackers: dto.attackers,
            blockers: dto.blockers,
            blocker_order: dto.blocker_order,
            cards_to_bottom: dto.cards_to_bottom,
            additional_costs: dto.additional_costs,
            discard_cards: dto.discard_cards,
            effect_choice_answer: dto.effect_choice_answer,
            trigger_targets: dto.trigger_targets,
            auto_tap: dto.auto_tap,
        }
    }
}

/// `POST /api/game/mulligan` body. CR 103.5.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MulliganRequest {
    /// `true` to take another mulligan, `false` to keep the hand as dealt.
    pub take: bool,
    /// CR 103.5's bottoming half. **Must be empty** on this path — see
    /// `PlaySession::mulligan`'s doc comment; a non-empty value is refused with
    /// 400 rather than accepted and discarded.
    #[serde(default)]
    pub cards_to_bottom: Vec<u64>,
}

// ── Name index ────────────────────────────────────────────────────────────────

/// `ObjectId` -> display text, built **only** from an already-redacted
/// [`StateViewModel`].
///
/// Architecture Invariant 7 (see the module doc): this is the sole source of
/// card names for every label this crate renders. An id that is absent, or
/// present but flagged `hidden`, resolves to a placeholder.
pub struct NameIndex {
    names: HashMap<u64, String>,
}

impl NameIndex {
    /// Walk every entry of the redacted view that carries an object id.
    ///
    /// Order matters only in that a later insert wins; the sources are disjoint
    /// by construction (an object is in exactly one zone).
    pub fn from_view(view: &StateViewModel) -> Self {
        let mut names = HashMap::new();

        for permanents in view.zones.battlefield.values() {
            for p in permanents {
                // A face-down permanent comes back from the layer system with an
                // empty name (CR 708.2a); the redactor blanks the rest. Either
                // way an empty name is not identifiable.
                names.insert(p.object_id, non_empty(&p.name));
            }
        }

        let card_zones = [
            &view.zones.hand,
            &view.zones.graveyard,
            &view.zones.command_zone,
        ];
        for zone in card_zones {
            for cards in zone.values() {
                for c in cards {
                    names.insert(c.object_id, card_label(c.hidden, &c.name));
                }
            }
        }
        for c in &view.zones.exile {
            names.insert(c.object_id, card_label(c.hidden, &c.name));
        }
        for item in &view.zones.stack {
            // **`source_object_id`, not `id`** — and the difference is a bug fix,
            // not a preference.
            //
            // `StackItemView::id` is a **`StackObject`** id. This map is keyed by
            // `ObjectId` (that is what [`NameIndex::label`] takes, and every
            // caller — action objects, target candidates, combatants — holds
            // one). The two are different id spaces that both count from small
            // integers, so the previous `names.insert(item.id, ..)` was writing a
            // foreign key into an `ObjectId` map: at best dead (nothing ever
            // looks a stack id up here), at worst it **overwrote a real
            // permanent's name**, because the stack is inserted last and a
            // `StackObject` id can numerically equal an `ObjectId`.
            //
            // `source_object_id` is the id `mtg_engine::Target::Object` actually
            // names for a spell on the stack, which is the key a target lookup
            // arrives with. Indexing the wrong one is how a counterspell's target
            // came out as [`UNKNOWN_LABEL`] — observed on a real S7 payload
            // (Dispel targeting an instant on the stack), not reasoned about.
            //
            // The value is the already-redacted `source_name`, which
            // `redact::redact_stack` has blanked if the seat may not identify the
            // source (CR 405.1 makes *that* a spell is on the stack public; CR
            // 702.36b keeps a face-down one's identity private).
            if let Some(source) = item.source_object_id {
                names.insert(source, non_empty(&item.source_name));
            }
        }

        NameIndex { names }
    }

    /// Display text for `id`, or [`UNKNOWN_LABEL`] if the redacted view has no
    /// entry for it. Never reads `GameState`.
    pub fn label(&self, id: ObjectId) -> String {
        self.names
            .get(&id.0)
            .cloned()
            .unwrap_or_else(|| UNKNOWN_LABEL.to_string())
    }
}

fn card_label(hidden: bool, name: &str) -> String {
    if hidden {
        HIDDEN_LABEL.to_string()
    } else {
        non_empty(name)
    }
}

fn non_empty(name: &str) -> String {
    if name.is_empty() {
        HIDDEN_LABEL.to_string()
    } else {
        name.to_string()
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// `DecisionKind` -> stable tag string.
///
/// `DecisionKind` is `#[non_exhaustive]` (decision-point audit §9.4 rec 1), so
/// this match **must** carry a wildcard arm — a new engine decision class must
/// not fail to compile a downstream binary, and `"Unknown"` is a correct if
/// unhelpful floor. Every named arm is a kind the client can render a picker
/// for.
pub fn decision_kind_tag(kind: DecisionKind) -> String {
    let tag = match kind {
        DecisionKind::Priority => "Priority",
        DecisionKind::Mulligan => "Mulligan",
        DecisionKind::CommanderZoneChoice => "CommanderZoneChoice",
        DecisionKind::DeclareAttackers => "DeclareAttackers",
        DecisionKind::DeclareBlockers => "DeclareBlockers",
        DecisionKind::CleanupDiscard => "CleanupDiscard",
        DecisionKind::TriggerTargets => "TriggerTargets",
        DecisionKind::EffectChoice => "EffectChoice",
        _ => "Unknown",
    };
    tag.to_string()
}

/// Stable machine tag for a `LegalAction` variant.
fn action_kind(action: &LegalAction) -> &'static str {
    match action {
        LegalAction::PassPriority => "PassPriority",
        LegalAction::Concede => "Concede",
        LegalAction::PlayLand { .. } => "PlayLand",
        LegalAction::CastSpell { .. } => "CastSpell",
        LegalAction::TapForMana { .. } => "TapForMana",
        LegalAction::ActivateAbility { .. } => "ActivateAbility",
        LegalAction::DeclareAttackers { .. } => "DeclareAttackers",
        LegalAction::DeclareBlockers { .. } => "DeclareBlockers",
        LegalAction::OrderBlockers { .. } => "OrderBlockers",
        LegalAction::TakeMulligan => "TakeMulligan",
        LegalAction::KeepHand => "KeepHand",
        LegalAction::ReturnCommanderToCommandZone { .. } => "ReturnCommanderToCommandZone",
        LegalAction::LeaveCommanderInZone { .. } => "LeaveCommanderInZone",
        LegalAction::ActivateBloodrush { .. } => "ActivateBloodrush",
        LegalAction::SaddleMount { .. } => "SaddleMount",
        LegalAction::CastWithMutate { .. } => "CastWithMutate",
        LegalAction::TurnFaceUp { .. } => "TurnFaceUp",
        LegalAction::ActivateLoyaltyAbility { .. } => "ActivateLoyaltyAbility",
        LegalAction::CastMorphFaceDown { .. } => "CastMorphFaceDown",
        LegalAction::PayEcho { .. } => "PayEcho",
        LegalAction::PayCumulativeUpkeep { .. } => "PayCumulativeUpkeep",
        LegalAction::PayRecover { .. } => "PayRecover",
        LegalAction::DiscardToHandSize { .. } => "DiscardToHandSize",
        LegalAction::ChooseTriggerTargets { .. } => "ChooseTriggerTargets",
        LegalAction::AnswerEffectChoice { .. } => "AnswerEffectChoice",
    }
}

/// The primary object an action is about, if it has one.
fn action_object(action: &LegalAction) -> Option<ObjectId> {
    match action {
        LegalAction::PlayLand { card }
        | LegalAction::CastSpell { card, .. }
        | LegalAction::ActivateBloodrush { card, .. }
        | LegalAction::CastWithMutate { card, .. }
        | LegalAction::CastMorphFaceDown { card, .. } => Some(*card),
        LegalAction::TapForMana { source, .. }
        | LegalAction::ActivateAbility { source, .. }
        | LegalAction::ActivateLoyaltyAbility { source, .. }
        | LegalAction::ChooseTriggerTargets { source, .. }
        | LegalAction::AnswerEffectChoice { source, .. } => Some(*source),
        LegalAction::ReturnCommanderToCommandZone { object_id }
        | LegalAction::LeaveCommanderInZone { object_id } => Some(*object_id),
        LegalAction::SaddleMount { mount, .. } => Some(*mount),
        LegalAction::TurnFaceUp { permanent, .. }
        | LegalAction::PayEcho { permanent, .. }
        | LegalAction::PayCumulativeUpkeep { permanent, .. } => Some(*permanent),
        LegalAction::PayRecover { recover_card, .. } => Some(*recover_card),
        // CR 509.2: the attacker is what the client should highlight — it is the
        // creature whose damage is being ordered, and the blockers are all listed
        // in `ActionOptionView::order`.
        LegalAction::OrderBlockers { attacker, .. } => Some(*attacker),
        LegalAction::PassPriority
        | LegalAction::Concede
        | LegalAction::DeclareAttackers { .. }
        | LegalAction::DeclareBlockers { .. }
        | LegalAction::TakeMulligan
        | LegalAction::KeepHand
        | LegalAction::DiscardToHandSize { .. } => None,
    }
}

/// Human-readable label, rendered from the seat-redacted [`NameIndex`] only.
fn action_label(action: &LegalAction, names: &NameIndex) -> String {
    let card = |id: ObjectId| names.label(id);
    match action {
        LegalAction::PassPriority => "Pass priority".to_string(),
        LegalAction::Concede => "Concede".to_string(),
        LegalAction::PlayLand { card: c } => format!("Play {}", card(*c)),
        LegalAction::CastSpell { card: c, .. } => format!("Cast {}", card(*c)),
        LegalAction::TapForMana { source, .. } => format!("Tap {} for mana", card(*source)),
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => format!("Activate ability {ability_index} of {}", card(*source)),
        LegalAction::DeclareAttackers { eligible, .. } => {
            format!("Declare attackers ({} eligible)", eligible.len())
        }
        LegalAction::DeclareBlockers { eligible, .. } => {
            format!("Declare blockers ({} eligible)", eligible.len())
        }
        LegalAction::OrderBlockers { attacker, blockers } => format!(
            "Order the {} blockers of {} (CR 509.2)",
            blockers.len(),
            card(*attacker)
        ),
        LegalAction::TakeMulligan => "Take a mulligan".to_string(),
        LegalAction::KeepHand => "Keep this hand".to_string(),
        LegalAction::ReturnCommanderToCommandZone { object_id } => {
            format!("Return {} to the command zone", card(*object_id))
        }
        LegalAction::LeaveCommanderInZone { object_id } => {
            format!("Leave {} where it is", card(*object_id))
        }
        LegalAction::ActivateBloodrush { card: c, target } => {
            format!("Bloodrush {} onto {}", card(*c), card(*target))
        }
        LegalAction::SaddleMount { mount, .. } => format!("Saddle {}", card(*mount)),
        LegalAction::CastWithMutate {
            card: c,
            mutate_target,
        } => format!("Mutate {} onto {}", card(*c), card(*mutate_target)),
        LegalAction::TurnFaceUp { permanent, .. } => format!("Turn {} face up", card(*permanent)),
        LegalAction::ActivateLoyaltyAbility {
            source,
            ability_index,
        } => format!("Loyalty ability {ability_index} of {}", card(*source)),
        LegalAction::CastMorphFaceDown { card: c, .. } => {
            format!("Cast {} face down", card(*c))
        }
        LegalAction::PayEcho { permanent, pay } => {
            format!("{} echo for {}", pay_verb(*pay), card(*permanent))
        }
        LegalAction::PayCumulativeUpkeep { permanent, pay } => {
            format!(
                "{} cumulative upkeep for {}",
                pay_verb(*pay),
                card(*permanent)
            )
        }
        LegalAction::PayRecover { recover_card, pay } => {
            format!("{} recover for {}", pay_verb(*pay), card(*recover_card))
        }
        LegalAction::DiscardToHandSize { count, .. } => {
            format!("Discard {count} card(s) to hand size")
        }
        LegalAction::ChooseTriggerTargets { source, .. } => {
            format!("Choose targets for {}'s trigger", card(*source))
        }
        LegalAction::AnswerEffectChoice { source, .. } => {
            format!("Answer {}'s choice", card(*source))
        }
    }
}

fn pay_verb(pay: bool) -> &'static str {
    if pay {
        "Pay"
    } else {
        "Decline"
    }
}

/// CR 107.3 / 601.2b: does this action need an `x_value`?
///
/// Read through `calculate_characteristics`, never off `obj.characteristics`
/// directly — the standing layer-correctness gotcha.
///
/// **Both halves are answered as of S7.** `CastSpell` reads the spell's own
/// `mana_cost.x_count`. `ActivateAbility` reads the `{X}` in the *ability's*
/// `ActivationCost`, reached through the same layer-resolved
/// `activated_abilities` list `ability_index` indexes into (`abilities.rs`) —
/// which is what the S6 handoff's "`LegalAction::ActivateAbility` does not carry
/// it" note missed: the action does not carry the cost, but the *state* does, and
/// `source` + `ability_index` are enough to find it.
///
/// That closes the README's Limitation 5. Before it, a deck-legal
/// `mirror_entity` ({X}: creatures become X/X) was announced as X = 0 with no
/// error and no way for the client to know an `{X}` existed.
///
/// `TapForMana` is deliberately not answered: a mana ability with `{X}` in its
/// cost has no channel on `Command::TapForMana` at all, so reporting `true`
/// would offer the human a box whose value is discarded.
fn action_needs_x(action: &LegalAction, state: &GameState) -> bool {
    match action {
        LegalAction::CastSpell { card, .. } => mtg_engine::calculate_characteristics(state, *card)
            .and_then(|chars| chars.mana_cost)
            .is_some_and(|cost| cost.x_count > 0),
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => mtg_engine::calculate_characteristics(state, *source)
            .and_then(|chars| chars.activated_abilities.get(*ability_index).cloned())
            .and_then(|ability| ability.cost.mana_cost)
            .is_some_and(|cost| cost.x_count > 0),
        _ => false,
    }
}

/// CR 700.2a: the `ModeSelection` this action's object carries, if any.
///
/// Two different lookups, because the two live in different places:
///
/// * an **activated ability**'s modes are on the layer-resolved
///   `ActivatedAbility` that `ability_index` indexes (`Characteristics::
///   activated_abilities`), so this is the same read `handle_activate_ability`
///   makes and cannot drift from it;
/// * a **spell**'s modes live on the card definition's `AbilityDefinition::Spell
///   { modes }`. `rules::casting::spell_mode_selection` is the engine's own
///   accessor for exactly this and it is `pub(crate)`, so the four lines below
///   re-derive it through the public `GameState::card_registry`.
///
/// **That re-derivation is the one place in this file that restates an engine
/// rule rather than delegating**, and it is recorded as such rather than left
/// implicit: if `spell_mode_selection` ever stops being "the first
/// `AbilityDefinition::Spell` with `modes: Some(..)`", this goes stale silently.
/// It is confined to a *display* concern (which modes to offer); the engine
/// re-validates `modes_chosen` on the cast path regardless (CR 601.2b, PB-DP3),
/// so a drift here is a wrong picker, never a wrong game state.
fn action_modes(action: &LegalAction, state: &GameState) -> Option<ModeSelection> {
    match action {
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => mtg_engine::calculate_characteristics(state, *source)
            .and_then(|chars| chars.activated_abilities.get(*ability_index).cloned())
            .and_then(|ability| ability.modes),
        LegalAction::CastSpell { card, .. } => {
            let card_id = state
                .objects()
                .get(card)
                .and_then(|obj| obj.card_id.clone())?;
            let def = state.card_registry().get(card_id)?;
            def.abilities.iter().find_map(|a| match a {
                AbilityDefinition::Spell { modes: Some(m), .. } => Some(m.clone()),
                _ => None,
            })
        }
        _ => None,
    }
}

/// A one-line label for a mode.
///
/// `ModeSelection.modes` is a `Vec<Effect>` and an `Effect` carries **no** prose
/// — there is no oracle-text-per-mode field anywhere in the DSL. So the label is
/// the `Effect`'s `Debug` rendering, truncated: honest about what the mode does,
/// and visibly machine-shaped rather than pretending to be printed text.
fn mode_label(index: usize, effect: &Effect) -> String {
    const MAX: usize = 90;
    let mut detail = format!("{effect:?}");
    // Debug output is one line for these; collapse anyway so a nested pretty
    // formatter could never break the label across lines.
    detail = detail.split_whitespace().collect::<Vec<_>>().join(" ");
    if detail.chars().count() > MAX {
        detail = detail.chars().take(MAX - 1).collect::<String>() + "…";
    }
    format!("Mode {}: {detail}", index + 1)
}

/// CR 601.2c: the target requirements this action announces, from the engine's
/// own query surface — never re-derived here.
///
/// `modes_chosen` is `&[]` and `alt_cost` is `None` because that is exactly what
/// `mtg_simulator::params::action_to_command_with_params` builds the `Command`
/// with for these two variants: it passes `alt_cost: None` unconditionally, and
/// forwards `params.modes_chosen`, which is empty at render time (the human has
/// not answered yet). Any other pair of arguments here would advertise a target
/// set for a cast the client cannot actually make.
fn action_target_requirements(action: &LegalAction, state: &GameState) -> Vec<TargetRequirement> {
    match action {
        LegalAction::CastSpell { card, .. } => {
            mtg_engine::spell_target_requirements(state, *card, &[], None)
        }
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => mtg_engine::ability_target_requirements(state, *source, *ability_index),
        // Every other variant either takes no targets or carries them inside the
        // action itself (`ActivateBloodrush`, `CastWithMutate`,
        // `ChooseTriggerTargets`), and `params.rs` refuses a `targets` param on
        // all of them with `UnsupportedParam`. Offering a picker whose value the
        // mapping table would reject with a 400 is worse than offering none.
        _ => Vec::new(),
    }
}

/// The object whose controller/source context the target query runs against.
fn target_query_source(action: &LegalAction) -> Option<ObjectId> {
    match action {
        LegalAction::CastSpell { card, .. } => Some(*card),
        LegalAction::ActivateAbility { source, .. } => Some(*source),
        _ => None,
    }
}

/// Render one slot's worth of `Target`s for the wire, seat-redacted.
fn target_options(
    targets: &[Target],
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> Vec<TargetOptionView> {
    targets
        .iter()
        .map(|t| match t {
            Target::Object(id) => TargetOptionView {
                kind: "object".to_string(),
                id: id.0,
                label: names.label(*id),
                value: t.clone(),
            },
            Target::Player(p) => TargetOptionView {
                kind: "player".to_string(),
                id: p.0,
                label: display_name(*p, player_names),
                value: t.clone(),
            },
        })
        .collect()
}

/// CR 508.1 / CR 509.1: render the combat payloads straight out of the
/// `LegalAction` the provider emitted.
///
/// **Nothing is re-derived**: `eligible`, `targets` and `attackers` are the
/// provider's own `can_attack` / `can_block` verdicts (`legal_actions.rs`), and
/// [`crate::api::validate_combat_params`] validates a submission against these
/// same three lists, so the picker the human sees and the check the server makes
/// cannot disagree.
fn combat_options(
    action: &LegalAction,
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> (Option<AttackOptionsView>, Option<BlockOptionsView>) {
    let combatants = |ids: &[ObjectId]| -> Vec<CombatantOptionView> {
        ids.iter()
            .map(|id| CombatantOptionView {
                id: id.0,
                label: names.label(*id),
            })
            .collect()
    };
    match action {
        LegalAction::DeclareAttackers { eligible, targets } => (
            Some(AttackOptionsView {
                eligible: combatants(eligible),
                targets: targets
                    .iter()
                    .map(|t| match t {
                        AttackTarget::Player(p) => AttackTargetOptionView {
                            kind: "player".to_string(),
                            id: p.0,
                            label: display_name(*p, player_names),
                            value: t.clone(),
                        },
                        AttackTarget::Planeswalker(id) => AttackTargetOptionView {
                            kind: "planeswalker".to_string(),
                            id: id.0,
                            label: names.label(*id),
                            value: t.clone(),
                        },
                    })
                    .collect(),
            }),
            None,
        ),
        LegalAction::DeclareBlockers {
            eligible,
            attackers,
        } => (
            None,
            Some(BlockOptionsView {
                eligible: combatants(eligible),
                attackers: combatants(attackers),
            }),
        ),
        _ => (None, None),
    }
}

/// CR 509.2 (M11-local S8, item 2): render the damage-assignment-order payload.
///
/// Separate from [`combat_options`] rather than a third element of its tuple
/// because this action does not come from the provider at all — it is appended by
/// `mtg_simulator::local_game::human_only_actions` for a human seat only, so the
/// "nothing is re-derived, these are the provider's verdicts" argument on
/// `combat_options` does not apply verbatim and should not be implied by sharing
/// its body. The lists here are the *engine's* (`state.combat()`), read at the
/// moment the decision was minted.
fn order_options(action: &LegalAction, names: &NameIndex) -> Option<OrderBlockersOptionsView> {
    let LegalAction::OrderBlockers { attacker, blockers } = action else {
        return None;
    };
    Some(OrderBlockersOptionsView {
        attacker: CombatantOptionView {
            id: attacker.0,
            label: names.label(*attacker),
        },
        blockers: blockers
            .iter()
            .map(|id| CombatantOptionView {
                id: id.0,
                label: names.label(*id),
            })
            .collect(),
    })
}

/// Display text for a card the **engine has told this seat to look at** — and the
/// one place in this crate that reads a name from `GameState` rather than from the
/// seat-redacted [`NameIndex`].
///
/// # Why [`NameIndex`] cannot answer this
///
/// A scry, a surveil and a library search all name cards in the **library**, and
/// `StateViewModel` does not model library contents at all — it carries
/// `library_size` and nothing else. So `NameIndex::label` answers
/// [`UNKNOWN_LABEL`] for every one of these ids, and a picker built on it would
/// offer the human three buttons all reading "(unknown card)".
///
/// That is not a *safer* outcome, which is the point. The ids are already on the
/// wire and must be — they are what the answer is expressed in — so rendering them
/// nameless leaks exactly as much and delivers nothing. The redacted view is the
/// wrong instrument here, not an instrument that was bypassed.
///
/// # The entitlement is the engine's, not this crate's
///
/// CR 701.22a / CR 701.23a / CR 701.25a each instruct **this player** to look at
/// **these cards**. The engine encodes that structurally rather than in prose:
/// the ids live inside a `PendingEffectChoice { player, question, .. }` minted for
/// exactly one seat, and `GameEvent::EffectChoiceRequired::private_to()` returns
/// `Some(player)` for that seat and nobody else (`crates/card-types/src/state/
/// stubs.rs`, whose own doc says "Every `ObjectId` in every variant names a card in
/// a HIDDEN zone -- the library. That is why ... `private_to()` returns
/// `Some(player)`").
///
/// So the safety argument is a *structural* one and it has **two** premises:
///
/// 1. this function is only ever called with an id drawn out of an
///    `EffectChoiceQuestion` — every call site below maps over `candidates` /
///    `looked_at` directly for that reason, and none takes an id from anywhere
///    else; and
/// 2. that question is the one the engine minted **for the seat this payload is
///    being rendered for**.
///
/// The second premise used to hold only by arithmetic on a one-element set
/// (`session::config_for` hard-codes a single human seat, so `pending.player`
/// happened to always equal `session.human`). It is enforced as of the UI-1 review:
/// `api.rs::seat_view` filters the pending decision on `pending.player == human`,
/// so a decision addressed to another seat is absent from this one's payload
/// rather than rendered into it. Read that filter's comment before weakening it.
///
/// # This is a fourth channel, and a new channel is invisible to an old gate
///
/// Review MR-M11-01's durable lesson, stated in the crate README: *a redaction gate
/// checks the channel it was written for.* `GameSummary.seed` shipped for three
/// sessions past two green Invariant-7 gates because it was a reconstruction key
/// and both gates watched names. This is the fourth: a **look entitlement**, a real
/// card name from a hidden zone, deliberately rendered.
///
/// # What actually holds it — two gates, and what neither of them covers
///
/// * `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload` is the
///   behavioural one. It drives a real scry, confirms this function is returning
///   real names for it, then moves the **viewer** (`PlaySession::human`) to the
///   other seat — not the decision, which `advance()` would refresh straight back
///   — and asserts the payload loses the decision and the `looked_at` field that
///   carried its cards. It also asserts the matching *write* is refused. It is
///   two-sided on both halves: removing either `api.rs` guard turns it red.
///
///   Precisely what the raw-body half asserts is the absence of the `looked_at`
///   **key**, not of the card names. The names cannot be asserted absent — seat 2
///   legitimately holds Swamps of its own — so the key is the right needle, and
///   saying "every name it carried" would overstate it.
/// * `test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places` pins the
///   number of raw `GameState` object-table reads in this file's production code at
///   two, so a *third* look channel cannot be opened in silence.
///
/// Neither covers a hidden-information channel that reads `GameState` some other
/// way — `zones()`, `card_registry()`, `player()`. The count gate watches one
/// needle, which is the same shape of limitation MR-M11-01 is about, and is said
/// here rather than left to be rediscovered.
fn question_card_label(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|obj| non_empty(&obj.characteristics.name))
        .unwrap_or_else(|| UNKNOWN_LABEL.to_string())
}

/// Render the ids of an [`EffectChoiceQuestion`] as labelled options. The single
/// call path into [`question_card_label`] — see that function's premise.
fn question_cards(state: &GameState, ids: &[ObjectId]) -> Vec<CardOptionView> {
    ids.iter()
        .map(|id| CardOptionView {
            id: id.0,
            label: question_card_label(state, *id),
        })
        .collect()
}

/// UI-1: render a blocking decision's answer space, or `None` for an action that
/// is not one.
///
/// Nothing here is re-derived. `count`, `hand`, `candidates`, `looked_at`, `slots`
/// and every `default` come off the `LegalAction` the provider emitted, which is
/// the same data `handle_discard_to_hand_size` / `handle_answer_effect_choice` /
/// `handle_choose_trigger_targets` will validate the answer against — so the
/// picker the human sees and the check the engine makes cannot disagree.
fn blocking_decision_view(
    action: &LegalAction,
    state: &GameState,
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> Option<BlockingDecisionView> {
    match action {
        // CR 514.1 / CR 701.9b. Hand cards ARE in the seat-redacted view (they are
        // this seat's own), so these labels come through `NameIndex` like every
        // other label in this file — the `question_card_label` channel is not
        // involved and must not be.
        LegalAction::DiscardToHandSize { count, hand, cards } => Some(BlockingDecisionView {
            question: "CleanupDiscard".to_string(),
            prompt: format!(
                "Discard {count} card{} to maximum hand size (CR 514.1)",
                plural(*count as usize)
            ),
            answer_field: "discard_cards".to_string(),
            answer: AnswerShapeView::Subset {
                candidates: hand
                    .iter()
                    .map(|id| CardOptionView {
                        id: id.0,
                        label: names.label(*id),
                    })
                    .collect(),
                count: *count as usize,
                default: cards.iter().map(|id| id.0).collect(),
            },
        }),
        // CR 608.2d. `source` is a spell or ability on the stack, which is public
        // (CR 405.1) and therefore in `NameIndex`; the CANDIDATES are library cards
        // and are not — see `question_card_label`.
        LegalAction::AnswerEffectChoice {
            source,
            question,
            answer,
            ..
        } => {
            let src = names.label(*source);
            let (question_tag, prompt, shape) = match question {
                EffectChoiceQuestion::SearchLibrary {
                    candidates,
                    may_fail_to_find,
                } => (
                    "SearchLibrary",
                    format!(
                        "{src}: search your library — choose a card{} (CR 701.23a)",
                        if *may_fail_to_find {
                            ", or fail to find"
                        } else {
                            ""
                        }
                    ),
                    AnswerShapeView::PickOne {
                        candidates: question_cards(state, candidates),
                        may_decline: *may_fail_to_find,
                        template: answer.clone(),
                        found_key: "found".to_string(),
                    },
                ),
                EffectChoiceQuestion::Scry { looked_at } => (
                    "Scry",
                    format!(
                        "{src}: scry {} — put any of these on the bottom of your library \
                         (CR 701.22a)",
                        looked_at.len()
                    ),
                    AnswerShapeView::Partition {
                        looked_at: question_cards(state, looked_at),
                        kept_key: "top".to_string(),
                        moved_key: "bottom".to_string(),
                        moved_label: "bottom of library".to_string(),
                        template: answer.clone(),
                    },
                ),
                EffectChoiceQuestion::Surveil { looked_at } => (
                    "Surveil",
                    format!(
                        "{src}: surveil {} — put any of these into your graveyard \
                         (CR 701.25a)",
                        looked_at.len()
                    ),
                    AnswerShapeView::Partition {
                        looked_at: question_cards(state, looked_at),
                        kept_key: "top".to_string(),
                        moved_key: "graveyard".to_string(),
                        moved_label: "graveyard".to_string(),
                        template: answer.clone(),
                    },
                ),
            };
            Some(BlockingDecisionView {
                question: question_tag.to_string(),
                prompt,
                answer_field: "effect_choice_answer".to_string(),
                answer: shape,
            })
        }
        // CR 603.3d / CR 601.2c (OOS-DP8-2). Trigger-target candidates are
        // `SpellTarget`s naming permanents, players and stack objects — all public,
        // all in `NameIndex`.
        LegalAction::ChooseTriggerTargets {
            source,
            slots,
            targets,
            ..
        } => Some(BlockingDecisionView {
            question: "TriggerTargets".to_string(),
            prompt: format!(
                "Choose target{} for {}'s triggered ability (CR 603.3d)",
                plural(slots.len()),
                names.label(*source)
            ),
            answer_field: "trigger_targets".to_string(),
            answer: AnswerShapeView::Slots {
                slots: slots
                    .iter()
                    .map(|slot| TargetSlotView {
                        // CR 601.2c: an `optional` slot is `TargetRequirement::UpToN`
                        // and may legally be answered with zero; every other slot
                        // takes exactly one. This is `handle_choose_trigger_targets`'
                        // own per-slot cardinality check (its step 6), read off the
                        // same two fields rather than re-derived.
                        min: if slot.optional { 0 } else { 1 },
                        max: slot.max as usize,
                        candidates: target_options(
                            &slot
                                .candidates
                                .iter()
                                .map(|c| c.target.clone())
                                .collect::<Vec<_>>(),
                            names,
                            player_names,
                        ),
                    })
                    .collect(),
                default: targets.clone(),
            },
        }),
        _ => None,
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Render a `PendingDecision` for the wire.
///
/// `wire_seq` is supplied by the caller rather than read off `decision.seq`:
/// `LocalGame`'s counter restarts at 0 on every rebuild, so the value a client
/// may echo back is `PlaySession::wire_seq(decision.seq)` and nothing else. See
/// `session::PlaySession::wire_seq`.
///
/// **This is a convention, not a guarantee** (S5 re-review LOW 12). The parameter
/// is a bare `u64`; nothing in the type system stops a caller passing
/// `decision.seq`. What actually holds it is that `api.rs::seat_view` is the
/// only caller and builds the `(pending, wire_seq)` pair with a `zip`, so the
/// two are produced together or not at all. Making it structural would take a
/// newtype, which this session did not add.
pub fn decision_view(
    decision: &PendingDecision,
    wire_seq: u64,
    state: &GameState,
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> DecisionView {
    let actions = decision
        .actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            action_option_view(index, action, decision.player, state, names, player_names)
        })
        .collect();

    DecisionView {
        seq: wire_seq,
        kind: decision_kind_tag(decision.kind),
        player: decision.player.0,
        actions,
    }
}

/// Render one `LegalAction` for the wire.
///
/// # Cost, stated with both terms and measured rather than argued
///
/// `legal_targets_per_slot` is one `validate_targets_inner` per (slot ×
/// candidate) and each object candidate costs a `calculate_characteristics`.
/// `queries.rs` asks in terms that this be **measured** before a browser polls
/// it, so it was.
///
/// The candidate sweep runs once per option-level requirement **plus once per
/// declared mode** of a modal action. The second term matters and an earlier
/// draft of this comment denied it: for a per-mode-targeting card the
/// option-level requirement list is empty *by design*
/// (`action_target_requirements`' doc), so exactly the actions that sentence
/// called free are the ones that pay `modes × slots × candidates`. An action
/// declaring neither — a land drop, a mana ability, a pass, which is most of a
/// priority list — touches no candidate at all.
///
/// **Measured** (M11-local S7, temporary `#[ignore]`d probe through `oneshot`,
/// since removed), and the numbers are the probe's rather than an estimate — a
/// first draft of this paragraph carried invented ones and the probe
/// contradicted them:
///
/// > 4 players, seed 9, **turn 17** (the deepest board the S7 fixtures reach):
/// > **12** actions offered, of which **1** carries a target requirement, over
/// > **22** candidates across Battlefield / Stack / Graveyard. One whole
/// > [`decision_view`] costs **≈ 201 µs**, mean of 20 renders, **debug build**.
///
/// That is against a `POST /api/game/action` that also runs every bot seat
/// forward to the human's next decision, so it is nowhere near the dominant
/// term, and the per-`(state, source)` cache `queries.rs` suggests is not needed
/// at this board size.
///
/// What the number does **not** establish: the cost is roughly linear in
/// (targeted actions × slots × candidates), and this board exercises exactly one
/// targeted action. A hand full of removal on a wide board — say ten targeted
/// options over a hundred candidates — is ~50× this and is worth re-measuring
/// before assuming it is still fine. No seeded fixture has produced one.
fn action_option_view(
    index: usize,
    action: &LegalAction,
    player: PlayerId,
    state: &GameState,
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> ActionOptionView {
    let requirements = action_target_requirements(action, state);
    let (target_min, target_max) = mtg_engine::target_count_range(&requirements);
    let source = target_query_source(action);

    let slots = |reqs: &[TargetRequirement]| -> Vec<TargetSlotView> {
        let Some(src) = source else {
            return Vec::new();
        };
        if reqs.is_empty() {
            return Vec::new();
        }
        // `legal_targets_per_slot` returns one entry per requirement, in the same
        // order — its own doc says "parallel to `requirements`" — so zipping the
        // two is index-correspondent by the engine's construction rather than by
        // an assumption made here.
        mtg_engine::legal_targets_per_slot(state, player, src, reqs)
            .iter()
            .zip(reqs.iter())
            .map(|(candidates, req)| {
                // Per-slot range from the same function that computes the
                // collective one, handed a one-element slice. `UpToN { count }`
                // is the only requirement whose min and max differ.
                let (min, max) = mtg_engine::target_count_range(std::slice::from_ref(req));
                TargetSlotView {
                    min,
                    max,
                    candidates: target_options(candidates, names, player_names),
                }
            })
            .collect()
    };

    let target_slots = slots(&requirements);

    // CR 700.2a/700.2c. A modal action's per-mode target requirements are only
    // knowable once the modes are chosen, and the human has not chosen yet, so
    // each mode carries its own slots and the client picks the ones for the modes
    // it selects. `spell_target_requirements` with an empty `modes_chosen`
    // deliberately returns `vec![]` for such a card (its own doc, divergence 1),
    // which is why `target_slots` above is empty for them rather than wrong.
    let mode_selection = action_modes(action, state);
    let (modes, mode_min, mode_max) = match &mode_selection {
        None => (Vec::new(), 0, 0),
        Some(ms) => {
            let rendered = ms
                .modes
                .iter()
                .enumerate()
                .map(|(i, effect)| {
                    let reqs = ms
                        .mode_targets
                        .as_ref()
                        .and_then(|mt| mt.get(i))
                        .cloned()
                        .unwrap_or_default();
                    let (mode_target_min, mode_target_max) = mtg_engine::target_count_range(&reqs);
                    ModeOptionView {
                        index: i,
                        label: mode_label(i, effect),
                        target_slots: slots(&reqs),
                        target_min: mode_target_min,
                        target_max: mode_target_max,
                    }
                })
                .collect();
            (rendered, ms.min_modes, ms.max_modes)
        }
    };

    let (attack, block) = combat_options(action, names, player_names);

    ActionOptionView {
        index,
        kind: action_kind(action).to_string(),
        label: action_label(action, names),
        object_id: action_object(action).map(|id| id.0),
        target_slots,
        target_min,
        target_max,
        needs_x: action_needs_x(action, state),
        modes,
        mode_min,
        mode_max,
        attack,
        block,
        order: order_options(action, names),
        decision: blocking_decision_view(action, state, names, player_names),
    }
}

/// One violation, reduced to what a seat may see — see [`GameOverView::violations`].
fn violation_summary(v: &mtg_simulator::InvariantViolation) -> String {
    format!("{} (turn {})", v.check, v.turn_number)
}

/// A `HaltReason` reduced to its variant and numbers (review MR-M11-08).
///
/// `HaltReason::EngineError(String)` is the one that matters: it carries a
/// `format!("{:?}", GameStateError)` produced while advancing a **bot** seat, which
/// can name a bot's object. Every other variant holds only a player id and integers.
/// So the engine text is replaced with a pointer to the export rather than forwarded.
///
/// This is deliberately not a `Debug` with a filter over it: a filter has to know every
/// shape the text can take, and a new `GameStateError` variant would silently defeat it.
/// Enumerating the variants here means a new `HaltReason` is a compile error.
fn halt_reason_summary(reason: &HaltReason) -> String {
    match reason {
        HaltReason::MaxTurns { max_turns, turn } => {
            format!("the {max_turns}-turn limit was reached (turn {turn})")
        }
        HaltReason::InfiniteLoop { turn } => {
            format!("a stall guard tripped on turn {turn} (command or consecutive-pass limit)")
        }
        HaltReason::NoLegalActions { player, turn } => format!(
            "seat {} had no legal action on turn {turn} and could not even pass",
            player.0
        ),
        HaltReason::EngineError(_) => "the engine rejected a bot seat's command and its \
             fallback; the detail is in GET /api/game/report"
            .to_string(),
    }
}

/// Render a concluded game (CR 104).
pub fn game_over_view(
    result: &GameResult,
    player_names: &HashMap<PlayerId, String>,
) -> GameOverView {
    GameOverView {
        winner: result.winner.map(|p| display_name(p, player_names)),
        turn_count: result.turn_count,
        total_commands: result.total_commands,
        halted: false,
        // `GameResult::error` is a `GameDriverError`, which `HaltReason` converts into
        // — same argument as `halt_reason_summary`, and it is `None` on every path
        // `LocalGame::advance` takes to `GameOver` today.
        reason: result
            .error
            .as_ref()
            .map(|_| "the game ended with a driver error; see GET /api/game/report".to_string()),
        violations: result.violations.iter().map(violation_summary).collect(),
    }
}

/// Render a halted game (a `LocalGameLimits` safety valve tripped). Presented
/// through the same DTO because the client's question is the same: is another
/// decision coming?
pub fn halted_view(reason: &HaltReason, turn_count: u32, total_commands: u32) -> GameOverView {
    GameOverView {
        winner: None,
        turn_count,
        total_commands: total_commands as usize,
        halted: true,
        reason: Some(halt_reason_summary(reason)),
        violations: Vec::new(),
    }
}

/// Assemble the bug-report artefact (M11-local S8, plan item 5). See
/// [`BugReportView`] for what it is for and why it is not seat-redacted.
pub fn bug_report_view(session: &crate::session::PlaySession) -> BugReportView {
    let state = session.game.state();
    BugReportView {
        seed: session.cfg.seed,
        config: ReportConfigView {
            players: session.cfg.player_count,
            human_seat: session.human.0,
            bot: format!("{:?}", session.cfg.bot_kind),
            mulligan_count: session.mulligan_count,
            max_turns: session.cfg.limits.max_turns,
            max_commands: session.cfg.limits.max_commands,
            max_consecutive_passes: session.cfg.limits.max_consecutive_passes,
        },
        protocol_version: mtg_engine::PROTOCOL_VERSION,
        protocol_fingerprint: mtg_engine::PROTOCOL_SCHEMA_FINGERPRINT.to_string(),
        hash_schema_version: mtg_engine::HASH_SCHEMA_VERSION,
        state_hash: hex_of(&state.public_state_hash()),
        turn: state.turn().turn_number,
        command_count: session.game.command_count(),
        violations: session
            .game
            .violations()
            .iter()
            .map(|v| format!("{v:?}"))
            .collect(),
        // `journal()`, NOT `take_new_records()`: the export is the WHOLE history and
        // must not move `journal_cursor`, or requesting a bug report would silently
        // consume the event lines the live feed has not delivered yet.
        journal: session
            .game
            .journal()
            .iter()
            .map(|record| JournalEntryView {
                turn: record.turn,
                command: record.command.clone(),
                events: record.events.clone(),
            })
            .collect(),
    }
}

fn hex_of(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Player display names are public information (they are shown to the whole
/// table), so this needs no entitlement check.
pub fn display_name(player: PlayerId, player_names: &HashMap<PlayerId, String>) -> String {
    player_names
        .get(&player)
        .cloned()
        .unwrap_or_else(|| format!("player_{}", player.0))
}

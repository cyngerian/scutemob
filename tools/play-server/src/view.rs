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

use mtg_engine::cards::card_definition::Cost;
use mtg_engine::cards::card_definition::GiftType;
use mtg_engine::{
    AbilityDefinition, AdditionalCost, AltCostKind, AttackTarget, Effect, EffectChoiceAnswer,
    EffectChoiceQuestion, GameState, HybridMana, ManaColor, ManaCost, ModeSelection, ObjectId,
    PhyrexianMana, PlayerId, SpellAdditionalCost, Target, TargetRequirement,
};
use mtg_simulator::legal_actions::{CountCostKind, MarkerCostKind};
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
    /// That seat's **display name**, the key it appears under in
    /// `StateViewModel::players` / `zones.*` (UI-3, `scutemob-180`).
    ///
    /// Additive, and it exists to delete a client-side re-derivation rather than
    /// to add information. The play surface needs this name in four places (the
    /// own-hand bar, the "you" badges on the seat card and the battlefield cell,
    /// and the board's human-seat outline), and without it the client's only
    /// route is to rebuild `format!("Human-{}", summary.human)` — i.e. to keep a
    /// second copy of `mtg_simulator::setup::seat_name`'s convention, in another
    /// language, which would go silently wrong the day that function changes.
    ///
    /// **Not an Invariant-7 concern**: player display names are public, stated in
    /// as many words by `event_view_for`'s own doc ("Player display names are
    /// public information, so the extra parameter carries no hidden data"), and
    /// this exact string is already on every payload as a key of
    /// `StateViewModel::players` and as `TurnView::active_player`. This says
    /// *which of those already-visible names is yours*, which the recipient
    /// necessarily knows.
    pub human_name: String,
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
    /// PB-DX44 (`OOS-DX29-12`), CR 702.102a/d: this option's FUSED target slots —
    /// what [`Self::target_slots`] would be if the human answers this cast's
    /// `CostPicker` stage with the Fuse marker ticked. `spell_target_requirements`
    /// with `fuse: true` is a no-op for a spell without the Fuse keyword, so this
    /// is populated for every `CastSpell` option and equals [`Self::target_slots`]
    /// (in candidates, not necessarily in count) for the overwhelming majority.
    ///
    /// Populated as its OWN field rather than folded into `target_slots`, mirroring
    /// [`ModeOptionView::target_slots`]'s precedent ("which slots exist depends on
    /// an earlier stage's answer"): fusing is decided at the `CostPicker` stage,
    /// and by the time a client reaches `TargetPicker` it already knows whether
    /// Fuse was ticked (`ActionBar.svelte`'s `resolvedTargetSlots`).
    ///
    /// **This closes the hole Stage 1 left open**: before this field existed, a
    /// human who ticked Fuse was shown the UN-fused (one-target) slot list while
    /// `casting.rs` demanded the fused (two-target) one — a clean offer followed
    /// by a guaranteed 422 (SR-38), created by Stage 1 and closed here.
    pub fused_target_slots: Vec<TargetSlotView>,
    pub fused_target_min: usize,
    pub fused_target_max: usize,
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
    /// CR 118.8 / CR 702.157 (UI-2, `memory/playtest-triage-2026-08-02.md` F9): the
    /// additional costs this `CastSpell` option must or may pay. `None` on every
    /// other action, and `None` on a `CastSpell` whose `AdditionalCostPlan` is
    /// `Default` (both fields `None`) -- which is nearly every spell in the corpus.
    /// See [`AdditionalCostsView`].
    pub costs: Option<AdditionalCostsView>,
}

/// CR 118.8 / CR 702.157 (UI-2): the additional-cost descriptor for one
/// `CastSpell` option, rendered from the provider's own
/// `mtg_simulator::legal_actions::AdditionalCostPlan`.
///
/// Nothing here is re-derived: `sacrifice.eligible` / `sacrifice.default` /
/// `squad.cost` / `squad.max_count` are the plan's own fields, and
/// [`crate::api`]'s `validate_additional_cost_params` checks a submission
/// against these same fields -- so the picker the human sees and the check the
/// server makes cannot disagree (the same argument [`combat_options`]'s doc
/// makes for CR 508.1/509.1).
#[derive(Debug, Serialize)]
pub struct AdditionalCostsView {
    /// Which `ActionParamsDto` field carries the answer for the two `CastSpell`
    /// blocks -- `"additional_costs"`, an ARRAY of `AdditionalCost`. Sent rather
    /// than inferred, same argument as [`BlockingDecisionView::answer_field`].
    ///
    /// The two `ActivateAbility` blocks below do **not** use it: an activation cost
    /// reaches the engine as a scalar `Command::ActivateAbility` field, never as an
    /// `AdditionalCost`, so each names its own `ActionParamsDto` field in its own
    /// [`ActivationChoiceView::answer_field`]. The two kinds of block are never
    /// present at once (they come from different `LegalAction` variants).
    pub answer_field: String,
    pub prompt: String,
    /// CR 118.8: present when the SPELL declares a required sacrifice.
    pub sacrifice: Option<SacrificeCostView>,
    /// CR 702.157a: present when the SPELL has a Squad cost.
    pub squad: Option<SquadCostView>,
    /// CR 602.2 (SIM-6): present when the ACTIVATED ABILITY's cost includes
    /// "Sacrifice a/another <thing>".
    pub activation_sacrifice: Option<ActivationChoiceView>,
    /// CR 602.2 / CR 111.10g (SIM-6): present when the ACTIVATED ABILITY's cost
    /// includes "Discard a card".
    pub activation_discard: Option<ActivationChoiceView>,
    /// PB-DX29, CR 702.56a / CR 702.120a: the SPELL's pay-N-times riders — Replicate
    /// and Escalate. A list rather than two more `Option` fields, because the two are
    /// the same widget with a different label; a third such rider is a provider table
    /// entry, not a seventh field for a client to learn.
    ///
    /// **Always serialized, empty when there is nothing to ask.** An earlier draft
    /// carried `skip_serializing_if = "Vec::is_empty"`, which made these two fields
    /// ABSENT while every sibling was `null` — two presence conventions in one struct,
    /// which is a trap for the next client rather than a saving of six bytes.
    pub counts: Vec<CountCostView>,
    /// PB-DX29, CR 702.42a / CR 702.102a / CR 702.175a: the SPELL's pay-or-not riders —
    /// Entwine, Fuse and Offspring. Same argument as [`Self::counts`].
    pub markers: Vec<MarkerCostView>,
    /// PB-DX29, CR 702.174a: present when the SPELL has Gift.
    pub gift: Option<GiftCostView>,
    /// PB-DX29, CR 702.47a: present when this player holds a card that may be spliced
    /// onto this spell.
    pub splice: Option<SpliceCostView>,
    /// PB-DX44, CR 118.9: present only on the SEPARATE pitch `CastSpell` action --
    /// `None` on the ordinary cast, whichever spell it is. See [`PitchCostView`].
    pub pitch: Option<PitchCostView>,
}

/// PB-DX29, CR 702.56a / CR 702.120a: a rider paid a chosen number of times.
///
/// The [`SquadCostView`] shape, generalised — same `template` + `count_key`
/// template-copying idiom, so the client fills one named field of a cloned object and
/// never has to know the externally-tagged encoding of `AdditionalCost`.
#[derive(Debug, Serialize)]
pub struct CountCostView {
    /// `"Replicate"` or `"Escalate"` — the mechanic's printed name, for the label. Sent
    /// rather than derived from `template`'s tag, because the tag is
    /// `"EscalateModes"` and no printed card says that.
    pub kind: String,
    pub prompt: String,
    /// Compact MTG notation (`{1}{U}`), rendered server-side by
    /// [`format_mana_cost_compact`].
    pub cost_label: String,
    /// The largest N the provider will vouch for. `0` means "offerable but not payable
    /// right now"; every rider here is optional, so declining is always legal.
    pub max_count: u32,
    /// `Replicate { count: 0 }` / `EscalateModes { count: 0 }` — see
    /// [`SacrificeCostView::template`] for why this is sent verbatim.
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object the chosen count goes in —
    /// `"count"` for both.
    pub count_key: String,
}

/// PB-DX29, CR 702.42a / CR 702.102a / CR 702.175a: a rider that is simply paid or not.
///
/// # This one deliberately has NO `*_key`, and the reason is a wire fact worth stating
///
/// `AdditionalCost::Entwine`, `::Fuse` and `::Offspring` are **unit** variants, and
/// serde's externally-tagged encoding serialises a unit variant as a bare JSON
/// **string** (`"Entwine"`), not as an object with one key. So the `fillTemplate` idiom
/// every other cost picker uses — clone the object, write the field the server named —
/// has nothing to write into and would crash on `Object.keys(entry)[0]`.
///
/// The answer here IS the template: include it verbatim to pay, omit it to decline.
/// That is the same shape-of-JSON trap PB-DP10 measured on `Effect::Proliferate`, where
/// every prior gate's serde walk matched object keys only and was structurally blind to
/// a unit variant. Recording it here so the next author does not rediscover it from a
/// `DataCloneError` in a browser.
#[derive(Debug, Serialize)]
pub struct MarkerCostView {
    /// `"Entwine"`, `"Fuse"` or `"Offspring"`.
    pub kind: String,
    pub prompt: String,
    /// Compact MTG notation, or `None` for Fuse — CR 702.102b makes a fused spell's
    /// cost the two halves SUMMED, so there is no separate fuse cost to show and
    /// rendering `{0}` would be a lie. The client must say so in words instead.
    pub cost_label: Option<String>,
    /// SR-38: may this rider be TICKED right now?
    ///
    /// The marker analogue of [`SquadCostView::max_count`], and `false` is the analogue
    /// of `max_count: 0` — the rider is shown, disabled, with its reason, rather than
    /// hidden. `crate::api::validate_additional_cost_params` refuses a submission
    /// against `false` with a **400**, so a client that ticks it anyway is told which
    /// offer its answer contradicts instead of getting the engine's bare 422.
    ///
    /// PB-DX29's first draft had no such field, and the omission was live: the picker
    /// rendered a tickable Entwine on a board that could not pay it, and ticking it
    /// returned `422 "player does not have enough mana to pay the cost"` — a clean offer
    /// followed by a server rejection, on the very batch that exists to delete them.
    pub affordable: bool,
    /// The whole answer, verbatim: a bare `"Entwine"` / `"Fuse"` / `"Offspring"` string.
    pub template: AdditionalCost,
}

/// PB-DX29, CR 702.174a: the offer's Gift descriptor.
///
/// The only additional cost whose answer is a `PlayerId`. There is no mana component at
/// all — naming an opponent IS the cost.
#[derive(Debug, Serialize)]
pub struct GiftCostView {
    pub prompt: String,
    /// What the chosen player receives (CR 702.174d-i), as printed text.
    pub gift_label: String,
    /// Every other player still in the game, from the provider's own eligible set.
    /// Player identities are public (Architecture Invariant 7 is about hidden ZONES),
    /// so these carry display names.
    pub candidates: Vec<PlayerOptionView>,
    /// `Gift { opponent: <first eligible> }` — see [`SacrificeCostView::template`].
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object the chosen seat goes in —
    /// `"opponent"`.
    pub player_key: String,
}

/// PB-DX29: one seat a [`GiftCostView`] may name.
#[derive(Debug, Serialize)]
pub struct PlayerOptionView {
    pub id: u64,
    pub label: String,
}

/// PB-DX29, CR 702.47a: the offer's Splice descriptor.
///
/// The answer is a LIST of card ids, so this is the first cost picker that is
/// genuinely multi-select rather than pick-one. There is deliberately **no `default`**:
/// splice is optional and the empty list is the decline, so pre-selecting anything
/// would spend a human's mana for them.
#[derive(Debug, Serialize)]
pub struct SpliceCostView {
    pub prompt: String,
    /// Cards in this seat's own hand that `casting.rs`'s splice gate will accept for
    /// this spell, from the provider's own eligible set. Labelled through [`NameIndex`]
    /// for the same reason [`ActivationChoiceView::candidates`] is: these are cards in
    /// the ACTING seat's own hand (CR 108.4 / Architecture Invariant 7), and this view
    /// is built from the seat-redacted `StateViewModel`, so a card another seat holds
    /// could not be labelled here even if the provider offered it.
    pub candidates: Vec<CardOptionView>,
    /// `Splice { cards: [] }` — see [`SacrificeCostView::template`].
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object the chosen ids go in —
    /// `"cards"`.
    pub ids_key: String,
}

/// PB-DX44, CR 118.9 (`OOS-DX29-3`): the offer's pitch descriptor -- present only
/// on the SEPARATE pitch `CastSpell` action (`ActionOptionView.kind == "CastSpell"`
/// with `option.costs.pitch.is_some()`), never on the ordinary cast's own
/// [`AdditionalCostsView`] -- see [`mtg_simulator::legal_actions::
/// AdditionalCostPlan::pitch`]'s own doc for why Pitch is not a rider on the
/// ordinary cast.
#[derive(Debug, Serialize)]
pub struct PitchCostView {
    pub prompt: String,
    /// Cards in this seat's own hand that `casting.rs`'s pitch gate will accept,
    /// from the provider's own eligible set. Labelled through [`NameIndex`], same
    /// argument as [`SacrificeCostView::candidates`].
    pub candidates: Vec<CardOptionView>,
    pub default: u64,
    /// `ExileFromHand { card: <default> }` -- see [`SacrificeCostView::template`]
    /// for why this is sent verbatim rather than reconstructed client-side.
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object the chosen card id goes
    /// in -- `"card"`. A SCALAR key, unlike [`SacrificeCostView::ids_key`] /
    /// [`SpliceCostView::ids_key`]: `AdditionalCost::ExileFromHand` carries one
    /// `ObjectId` field, not a `Vec`, mirroring [`GiftCostView::player_key`]'s
    /// shape rather than the two array-answered kinds.
    pub card_key: String,
}

/// CR 602.2 (SIM-6): one object-naming component of an activated ability's cost.
///
/// One type for both components, because they are one shape: pick exactly one id
/// from a candidate list the server already judged eligible, and put it in the
/// named scalar field. There is no `template` here and no `ids_key` -- unlike
/// [`SacrificeCostView`], whose answer is an externally-tagged `AdditionalCost`
/// the client must clone and fill in, this answer is a bare `ObjectId`, so there
/// is no enum encoding for a client to know or get wrong.
#[derive(Debug, Serialize)]
pub struct ActivationChoiceView {
    pub prompt: String,
    /// The objects `handle_activate_ability`'s own cost gate will accept, from the
    /// provider's own eligible set -- **never** re-derived here. Labelled through
    /// [`NameIndex`]; see [`SacrificeCostView::candidates`] for why that channel and
    /// not [`question_card_label`].
    ///
    /// For a sacrifice these are battlefield permanents (public under CR 400.1). For
    /// a discard they are cards in the ACTIVATING SEAT's own hand, which that seat
    /// may look at (CR 108.4 / Architecture Invariant 7): this view is built from
    /// the seat-redacted `StateViewModel`, so a card another seat holds could not be
    /// labelled here even if the provider offered it -- and it cannot, because
    /// `StubProvider` enumerates `ZoneId::Hand(player)` for the acting player alone.
    pub candidates: Vec<CardOptionView>,
    /// The provider's own deterministic default -- what a bot submits, and what the
    /// picker pre-selects so Confirm alone is a legal play.
    pub default: u64,
    /// Which `ActionParamsDto` field carries the chosen id:
    /// `"cost_sacrifice_target"` or `"cost_discard_card"`. Sent rather than
    /// inferred, same argument as [`Self::prompt`]'s neighbour above.
    pub answer_field: String,
}

/// CR 118.8: the offer's required-sacrifice descriptor.
#[derive(Debug, Serialize)]
pub struct SacrificeCostView {
    pub prompt: String,
    /// Battlefield permanents this player controls that the engine's own
    /// sacrifice gate will accept, labelled through [`NameIndex`] -- **not**
    /// [`question_card_label`]. A sacrifice candidate is a permanent on the
    /// battlefield, public under CR 400.1, not a card in a hidden zone an effect
    /// has told this seat to look at (`question_card_label`'s whole reason to
    /// exist); routing it through that channel instead would open a FOURTH
    /// raw `GameState` read in this file, which
    /// `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`
    /// pins at exactly three.
    pub candidates: Vec<CardOptionView>,
    pub default: u64,
    /// The engine's own `AdditionalCost`, serialized verbatim, holding the
    /// default -- same argument as [`TargetOptionView::value`] /
    /// [`AnswerShapeView::Partition::template`]: the client clones it and
    /// replaces the array named by [`Self::ids_key`], so the externally-tagged
    /// encoding of `AdditionalCost` stays known in exactly one place.
    ///
    /// `lki` is deliberately EMPTY and must stay empty: `casting.rs`'s sacrifice
    /// site (CR 118.8) PATCHES it from the layer-resolved characteristics
    /// captured before the zone move (CR 608.2b/608.2h/608.2i). A
    /// client-supplied `lki` would be a second opinion about LKI the engine
    /// already owns and computes correctly on its own.
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object that the chosen id(s)
    /// go in -- `"ids"`.
    pub ids_key: String,
}

/// CR 702.157a: the offer's optional Squad descriptor.
#[derive(Debug, Serialize)]
pub struct SquadCostView {
    pub prompt: String,
    /// Compact MTG notation (`{1}{G}`), rendered server-side by
    /// [`format_mana_cost_compact`].
    pub cost_label: String,
    /// CR 702.157a: the largest N this player can currently afford on top of the
    /// spell's own cost, from the provider's `SquadCostOption::max_count`. `0`
    /// means "offerable but not payable right now" -- the spell is still legal
    /// to cast, declining is always legal (CR 702.157a "any number of times",
    /// including zero).
    pub max_count: u32,
    /// `Squad { count: 0 }` -- see [`SacrificeCostView::template`]'s argument for
    /// why this is sent verbatim rather than reconstructed client-side.
    pub template: AdditionalCost,
    /// The key inside `template`'s single variant object that the chosen count
    /// goes in -- `"count"`.
    pub count_key: String,
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
        /// **Look-only. Never an answer space** (UI-6, `scutemob-194`; G9 of
        /// `memory/playtest-triage-2026-08-02b.md`).
        ///
        /// CR 701.23a: *"To search for a card in a zone, look at **all** cards in
        /// that zone (even if it's a hidden zone)."* `candidates` is the engine's
        /// filtered answer space and stays exactly that — `handle_answer_effect_choice`
        /// refuses anything outside it, so offering a non-candidate as an answer
        /// would be offering an illegal one (SR-38). The playtest complaint ("only
        /// showed legal basic lands") is therefore about the *look*, not the *pick*,
        /// and the two are sent as two different lists precisely so a client cannot
        /// confuse them.
        ///
        /// This is the searcher's **own** library and nothing else — see
        /// [`library_look_cards`], which also explains why it is sorted by name
        /// rather than sent in library order.
        ///
        /// Not a superset by construction: a "search your library **and**
        /// graveyard" effect (`also_search_graveyard`, `effects/mod.rs`) puts
        /// graveyard cards in `candidates` that are in no library at all. A client
        /// renders the union and must not assume containment either way.
        ///
        /// Empty when the library is empty, and — deliberately — for every other
        /// `PickOne` question a future arm might route through this shape: the
        /// entitlement is CR 701.23a's, so it is filled in at the search arm alone.
        all_cards: Vec<CardOptionView>,
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
    /// CR 701.9b (ENG-1): choose **exactly** `count` of `candidates`, answered
    /// through a template. The answer goes into
    /// `ActionParamsDto::effect_choice_answer`.
    ///
    /// `PickOne` and `PickN` are the two cardinal-choice shapes that answer via
    /// `effect_choice_answer` and therefore carry a `template` + a key;
    /// [`Self::Subset`] is the one that answers via its own `discard_cards`
    /// field and carries no template. Splitting on that, rather than making
    /// `Subset`'s template optional, means a stale or malformed payload lands in
    /// `ActionBar`'s visible "unknown shape" fallback instead of posting a body
    /// the server will 400 (the UI-4 lesson).
    PickN {
        candidates: Vec<CardOptionView>,
        count: usize,
        /// PB-DX28: the fewest the client may submit. `== count` for every
        /// PRE-PB-DX28 use (CR 701.9b's discard is always exactly `count`) --
        /// play-server-LOCAL, not a wire change; the engine's own `up_to` bool
        /// lives on `EffectChoiceQuestion::ChooseObject`, this is just its DTO
        /// projection. `0` for a PB-DX28 "up to `count`" choice.
        min_count: usize,
        /// The key inside `template`'s single variant object that the chosen ids
        /// go in (`"chosen"`). See [`Self::Partition::template`].
        chosen_key: String,
        /// See [`Self::Partition::template`] — serialized verbatim, cloned by the
        /// client, never re-spelled.
        template: EffectChoiceAnswer,
        /// The engine's own default subset (`default_discard_answer`: the `count`
        /// LOWEST `ObjectId`s -- note that is the opposite end of the hand from
        /// `Subset`'s CR 514.1 default). Sent so "use the default" is one click
        /// and so a test can assert the human drove something else.
        default: Vec<u64>,
    },
    /// PB-DX45: CR 118.12 — pay an optional cost, or decline. The answer goes
    /// into `ActionParamsDto::effect_choice_answer`.
    ///
    /// **The first shape with no candidate list**, and that is a property of the
    /// rule rather than a thin DTO: CR 118.12 offers exactly two answers. The
    /// four id-bearing shapes above exist because their questions name an answer
    /// SPACE; this one's answer space is `{pay, decline}` and the engine only
    /// asks when the cost is already payable.
    ///
    /// A client renders two buttons and answers by cloning `template` and
    /// setting the key named by `pay_key` — the same never-respell-the-variant
    /// discipline as [`Self::Partition::template`], for the same reason.
    Confirm {
        /// The printed cost, formatted for display (`{2}{B}`, `Sacrifice a
        /// creature`, `Pay 2 life`, …). Display only: the engine validates the
        /// answer against its OWN recorded question and never against this
        /// string.
        cost_label: String,
        /// See [`Self::Partition::template`] — serialized verbatim, cloned by the
        /// client, never re-spelled.
        template: EffectChoiceAnswer,
        /// The key inside `template`'s single variant object that the boolean
        /// goes in (`"pay"`).
        pay_key: String,
        /// The engine's own default answer — `true`, the exact recovery of the
        /// pre-PB-DX45 auto-pay. Sent so "accept the default" is one click and so
        /// a test can assert the human drove the OTHER one, which is the only
        /// answer the old engine could not produce.
        default: bool,
    },
    /// PB-DX50: CR 702.140c — a two-way choice that is **not** a cost.
    ///
    /// **Deliberately NOT [`Self::Confirm`], and the reason is the label rather
    /// than the payload.** The two shapes carry the same information (a
    /// template, a boolean key and a default) and `ConfirmPicker` renders them
    /// as "Pay {cost}" / "Decline". CR 702.140c's question is *over or under*:
    /// nothing is paid, nothing is declined, and neither answer is the passive
    /// one. Reusing `Confirm` would put a truthful payload behind a false
    /// label — which is the defect class this queue keeps filing — so the two
    /// answers name themselves and the picker renders exactly what it is told.
    ///
    /// A client renders two buttons and answers by cloning `template` and
    /// setting the key named by `choice_key` — the same
    /// never-respell-the-variant discipline as [`Self::Partition::template`].
    BinaryChoice {
        /// The button that submits `true`.
        true_label: String,
        /// The button that submits `false`.
        false_label: String,
        /// See [`Self::Partition::template`] — serialized verbatim, cloned by the
        /// client, never re-spelled.
        template: EffectChoiceAnswer,
        /// The key inside `template`'s single variant object that the boolean
        /// goes in (`"on_top"`).
        choice_key: String,
        /// The engine's own default answer — `true` for CR 702.140c, the exact
        /// recovery of the pre-PB-DX50 hard-coded value. Sent so "accept the
        /// default" is one click and so a test can assert the human drove the
        /// OTHER one, which is the answer the old engine could not produce at
        /// resolution time.
        default: bool,
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
    /// Which seat this candidate currently belongs to, for **display grouping
    /// only** (UI-3, `scutemob-180`; playtest note "target selector should have
    /// segments broken up by player").
    ///
    /// Meaning, precisely, because "owner" is doing three jobs at once and the
    /// difference matters if anyone ever reaches for this field for anything
    /// else: it is the seat the *seat-redacted view model* associates the object
    /// with right now — `PermanentView::controller` for a battlefield permanent
    /// (CR 109.4, controller, not owner), `StackItemView::controller` for a spell
    /// or ability on the stack, and the **zone key** for a card in a per-player
    /// hand / graveyard / command zone (i.e. owner, CR 108.3). For
    /// `Target::Player` it is that player's own display name, so a player target
    /// sorts into their own segment.
    ///
    /// `None` when the redacted view has no entry for the id — the same
    /// condition under which [`Self::label`] is [`UNKNOWN_LABEL`]. A client
    /// groups those together under an "unknown" heading rather than guessing.
    ///
    /// **Never used for legality.** `legal_targets_per_slot` already decided what
    /// is targetable (`crates/engine/src/rules/queries.rs`), and the server
    /// re-validates on submission; this is a heading, and a wrong heading is a
    /// cosmetic defect where a wrong candidate list would be a rules one.
    ///
    /// Derived from the already-redacted [`NameIndex`] source, so it carries no
    /// information the same payload's `label` does not — an object whose identity
    /// this seat may not know is still an object it can *see*, in a zone whose
    /// ownership is public (CR 401.1/403.1/405.1).
    pub owner: Option<String>,
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
    /// Bot-seat commands the engine **refused**, oldest first (SIM-5 fix (3), G5).
    ///
    /// [`Self::journal`] records applied commands only, and that is exactly the limit
    /// the G5 triage ran into: with the rejection thrown away at
    /// `local_game.rs`'s auto-pass arm, "why did that bot waste six mana at upkeep?"
    /// could only be *inferred* from the surrounding commands. These are the engine's
    /// own refusals, so the next triage can classify instead.
    ///
    /// Truncated at `mtg_simulator::local_game::MAX_RETAINED_REJECTIONS`; compare the
    /// length against [`Self::rejection_count`] to see whether anything was dropped.
    pub rejections: Vec<RejectionView>,
    /// Total refusals over the whole game, never truncated.
    pub rejection_count: u32,
}

/// One refused bot command. `command` is the engine's own wire type, serialized
/// verbatim, exactly like [`JournalEntryView::command`].
#[derive(Debug, Serialize)]
pub struct RejectionView {
    pub turn: u32,
    pub player: u64,
    pub command: mtg_engine::Command,
    /// The engine's rejection reason, stringified (`GameStateError` is not a
    /// `Serialize` type).
    pub error: String,
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
    /// CR 602.2 (SIM-6): the permanent chosen to pay an `ActivateAbility`'s
    /// sacrifice cost. `null` means "accept the offer's own default" — see
    /// `ActionParams::cost_sacrifice_target`. Checked against the offer's own
    /// eligible set by `api::validate_additional_cost_params` before anything is
    /// applied.
    pub cost_sacrifice_target: Option<ObjectId>,
    /// CR 602.2 / CR 111.10g (SIM-6): the card chosen to pay an
    /// `ActivateAbility`'s discard cost. `null` means "accept the offer's own
    /// default". **Not [`Self::discard_cards`]**, which answers a CR 514.1 cleanup
    /// discard.
    pub cost_discard_card: Option<ObjectId>,
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
            cost_sacrifice_target: None,
            cost_discard_card: None,
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
            cost_sacrifice_target: dto.cost_sacrifice_target,
            cost_discard_card: dto.cost_discard_card,
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
    /// `ObjectId` -> the seat the redacted view associates it with. Populated
    /// alongside [`Self::names`] from the same walk, so the two cannot disagree
    /// about which objects exist. See [`TargetOptionView::owner`] for what
    /// "owner" means here and what it may be used for.
    ///
    /// Sparser than `names` by design: the shared exile pile is not per-player
    /// in the view model (`ZonesView::exile` is a flat `Vec`), so an exiled card
    /// contributes a name and no owner.
    owners: HashMap<u64, String>,
    /// PB-DX52 (`OOS-DX25b-1`): `StackObject` id -> display text, for
    /// `mtg_engine::Target::StackObject`.
    ///
    /// **A THIRD map, and the separation is the whole point.** `names`/`owners` are keyed
    /// by `ObjectId`; a stack-entry id is a DIFFERENT id space that also counts from small
    /// integers, so folding the two together is precisely the collision this file's own
    /// `from_view` comment records as a shipped bug ("the previous `names.insert(item.id,
    /// ..)` was writing a foreign key into an `ObjectId` map ... at worst it OVERWROTE a
    /// real permanent's name"). That insert was deleted for that reason; this map is how
    /// the same information comes back without the collision.
    stack_labels: HashMap<u64, String>,
    /// Companion of [`Self::stack_labels`] -- the seat each stack entry is controlled by.
    stack_owners: HashMap<u64, String>,
}

impl NameIndex {
    /// Walk every entry of the redacted view that carries an object id.
    ///
    /// Order matters only in that a later insert wins; the sources are disjoint
    /// by construction (an object is in exactly one zone).
    pub fn from_view(view: &StateViewModel) -> Self {
        let mut names = HashMap::new();
        let mut owners: HashMap<u64, String> = HashMap::new();
        let mut stack_labels: HashMap<u64, String> = HashMap::new();
        let mut stack_owners: HashMap<u64, String> = HashMap::new();

        for (controller, permanents) in &view.zones.battlefield {
            for p in permanents {
                // A face-down permanent comes back from the layer system with an
                // empty name (CR 708.2a); the redactor blanks the rest. Either
                // way an empty name is not identifiable.
                names.insert(p.object_id, non_empty(&p.name));
                // CR 109.4: the battlefield map is keyed by CONTROLLER, and
                // `PermanentView::controller` is the same string. Prefer the
                // permanent's own field over the map key so that if the two ever
                // disagree this follows the object, not the container.
                owners.insert(
                    p.object_id,
                    if p.controller.is_empty() {
                        controller.clone()
                    } else {
                        p.controller.clone()
                    },
                );
            }
        }

        let card_zones = [
            &view.zones.hand,
            &view.zones.graveyard,
            &view.zones.command_zone,
        ];
        for zone in card_zones {
            for (owner, cards) in zone.iter() {
                for c in cards {
                    names.insert(c.object_id, card_label(c.hidden, &c.name));
                    // **A hidden entry contributes no owner**, and that is not
                    // caution about leaking — a zone's ownership is public (CR
                    // 402.1: everyone knows whose hand it is). It is about the
                    // KEY: `redact::hidden_placeholder` rewrites a hidden card's
                    // `object_id` to **0**, so every redacted hand card at the
                    // table shares one id, and inserting them would make id 0
                    // resolve to whichever seat was walked last. `names` already
                    // lives with that collision harmlessly (every colliding entry
                    // maps to the same `HIDDEN_LABEL`); an owner map does not,
                    // because the values differ. Nothing is lost: a hidden card
                    // is never a legal target, so no `TargetOptionView` is ever
                    // built for one.
                    if !c.hidden {
                        owners.insert(c.object_id, owner.clone());
                    }
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
                // CR 405.1: the stack is public, and so is who controls each
                // object on it — `redact::redact_stack` blanks a face-down
                // source's *name* and never touches `controller`.
                owners.insert(source, item.controller.clone());
            }
            // PB-DX52: the STACK-entry id space, kept separate from `names` on purpose
            // (see [`NameIndex::stack_labels`]). `source_name` is already redacted by
            // `redact::redact_stack`, so a face-down source's ability reads as the
            // face-down placeholder here too, with no second entitlement decision made
            // in this file.
            stack_labels.insert(item.id, format!("{}'s ability", non_empty(&item.source_name)));
            stack_owners.insert(item.id, item.controller.clone());
        }

        NameIndex {
            names,
            owners,
            stack_labels,
            stack_owners,
        }
    }

    /// Display text for `id`, or [`UNKNOWN_LABEL`] if the redacted view has no
    /// entry for it. Never reads `GameState`.
    pub fn label(&self, id: ObjectId) -> String {
        self.names
            .get(&id.0)
            .cloned()
            .unwrap_or_else(|| UNKNOWN_LABEL.to_string())
    }

    /// Same lookup as [`Self::label`], but returns `None` on a miss instead of
    /// the generic [`UNKNOWN_LABEL`], so a caller can supply a
    /// candidate-specific fallback. Used by the `PickN` `Discard` arm
    /// (`OOS-ENG1-9`) to distinguish same-resolution-drawn candidates from each
    /// other rather than rendering two indistinguishable buttons.
    pub fn label_opt(&self, id: ObjectId) -> Option<String> {
        self.names.get(&id.0).cloned()
    }

    /// The seat the redacted view associates `id` with, or `None` when it has no
    /// per-player home there (an exiled card, or an id the view does not carry).
    ///
    /// Display grouping only — see [`TargetOptionView::owner`]. Never reads
    /// `GameState`, exactly as [`Self::label`] does not.
    pub fn owner(&self, id: ObjectId) -> Option<String> {
        self.owners.get(&id.0).cloned()
    }

    /// PB-DX52 (`OOS-DX25b-1`): display text for a `mtg_engine::Target::StackObject`.
    ///
    /// Deliberately NOT [`Self::label`] with a different map inline: the caller holds a
    /// `StackObject` id, and routing it through the `ObjectId`-keyed map is the exact
    /// id-space confusion `OOS-DX25-3`/`OOS-SIM3-5`/`OOS-DX25b-1` are all instances of.
    /// Two lookups, two names, no shared key.
    pub fn stack_entry_label(&self, id: ObjectId) -> String {
        self.stack_labels
            .get(&id.0)
            .cloned()
            .unwrap_or_else(|| UNKNOWN_LABEL.to_string())
    }

    /// The seat controlling the stack entry `id`, for display grouping only -- the
    /// stack-entry twin of [`Self::owner`]. CR 405.1: who controls each object on the
    /// stack is public.
    pub fn stack_entry_owner(&self, id: ObjectId) -> Option<String> {
        self.stack_owners.get(&id.0).cloned()
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
        LegalAction::ChooseDredge { .. } => "ChooseDredge",
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
        // PB-DX23 (CR 702.52a): `card` is already `Option<ObjectId>` -- a decline
        // correctly has no object, and a `Some(id)` names the dredge card itself.
        LegalAction::ChooseDredge { card, .. } => *card,
    }
}

/// Human-readable label, rendered from the seat-redacted [`NameIndex`] only.
fn action_label(action: &LegalAction, names: &NameIndex, state: &GameState) -> String {
    let card = |id: ObjectId| names.label(id);
    match action {
        LegalAction::PassPriority => "Pass priority".to_string(),
        LegalAction::Concede => "Concede".to_string(),
        LegalAction::PlayLand { card: c } => format!("Play {}", card(*c)),
        // PB-DX44 (`OOS-DX29-3`/`-9`): `alt_cost` distinguishes what would otherwise
        // be up to THREE identical "Cast <name>" buttons for the same card. String
        // suffixes only -- no new `GameState` read, so the Invariant-7 raw-read gate
        // (`test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`)
        // stays untouched; the RIGHT half's own sub-name (e.g. "Burn" for "Turn //
        // Burn") would need a registry lookup this file's pinned read count does not
        // have room for, so the label names the printed card and the half generically
        // rather than by its own name.
        LegalAction::CastSpell {
            card: c, alt_cost, ..
        } => match alt_cost {
            Some(AltCostKind::Pitch) => format!("Cast {} via its pitch cost", card(*c)),
            Some(AltCostKind::SplitRightHalf) => {
                format!("Cast {} (right half only)", card(*c))
            }
            _ => format!("Cast {}", card(*c)),
        },
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
        // CR 702.140a (PB-DX50): **no over/under in the label**, because there is no
        // over/under in the action any more. PB-DX29 rendered "Mutate X over/under Y"
        // to distinguish the two halves of its `(target, on_top)` pair; CR 702.140c
        // makes that a RESOLUTION choice, so a human now picks one action per host and
        // is asked over-or-under when the spell resolves
        // (`AnswerShapeView::BinaryChoice`). Keeping the word here would name a choice
        // this button no longer makes.
        LegalAction::CastWithMutate {
            card: c,
            mutate_target,
        } => format!("Mutate {} onto {}", card(*c), card(*mutate_target)),
        LegalAction::TurnFaceUp { permanent, .. } => format!("Turn {} face up", card(*permanent)),
        // CR 606.4 / CR 107.3m (PB-DX29 `/review` L9): name the loyalty COST, not the
        // slot index. Chandra has three loyalty abilities and the old label rendered
        // them as "Loyalty ability 0/1/2 of Chandra, Flamecaller" — three
        // indistinguishable buttons on the very card this batch was dispatched to make
        // usable. The batch filed the analogous modal-label opacity as `OOS-DX29-16`
        // arguing "this batch is what makes modal spells routinely clickable"; the
        // identical argument applies here and was missed.
        //
        // The cost is what a player says out loud ("Chandra plus one"), it is what the
        // printed card shows, and it is available from the same registry read the
        // engine's own handler makes. The index is kept as a disambiguator, because two
        // abilities may share a cost and the index is what the `Command` carries.
        LegalAction::ActivateLoyaltyAbility {
            source,
            ability_index,
        } => match loyalty_cost_label(state, *source, *ability_index) {
            Some(cost) => format!("{cost}: ability {ability_index} of {}", card(*source)),
            None => format!("Loyalty ability {ability_index} of {}", card(*source)),
        },
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
        // PB-DX23 (CR 702.52a).
        LegalAction::ChooseDredge {
            card: Some(c),
            mill,
        } => {
            format!("Dredge {} (mill {mill})", card(*c))
        }
        LegalAction::ChooseDredge { card: None, .. } => {
            "Decline dredge — draw normally".to_string()
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

/// CR 606.4 / CR 107.3m (PB-DX29 `/review` L9): the printed loyalty cost — `+1`, `−3`,
/// `0`, `−X` — of the loyalty ability at `ability_index`, as display text.
///
/// **The registry read lives in the ENGINE, not here**, and the batch's own Invariant-7
/// gate is why: a first draft did the lookup in this file and
/// `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places` went red on the
/// spot. A new raw `GameState` read in `view.rs` is a new hidden-information channel that
/// no other Invariant-7 gate can see, and the pin exists precisely so one cannot arrive
/// unnoticed. `mtg_engine::loyalty_ability_cost` returns the cost; this formats it.
///
/// The minus sign is U+2212 (−), matching the printed card rather than an ASCII hyphen.
fn loyalty_cost_label(state: &GameState, source: ObjectId, ability_index: usize) -> Option<String> {
    use mtg_engine::cards::card_definition::LoyaltyCost;
    mtg_engine::loyalty_ability_cost(state, source, ability_index).map(|cost| match cost {
        LoyaltyCost::Plus(n) => format!("+{n}"),
        LoyaltyCost::Minus(n) => format!("\u{2212}{n}"),
        LoyaltyCost::Zero => "0".to_string(),
        LoyaltyCost::MinusX => "\u{2212}X".to_string(),
    })
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
        // PB-DX29 (CR 606.4 / CR 107.3m): the third half, and it is not a mana `{X}`.
        // `LoyaltyCost::MinusX` spends X **loyalty counters**, so the `{X}` does not
        // live in a `ManaCost` at all and neither arm above could ever have found it.
        // `chandra_flamecaller` is `Complete` and deck-legal; before this arm its
        // printed "−X: deals X damage to each creature" was −0 for 0 damage in every
        // client, because `params.rs` hard-coded `x_value: None` and the engine reads
        // `x_value.unwrap_or(0)`.
        LegalAction::ActivateLoyaltyAbility {
            source,
            ability_index,
        } => mtg_engine::loyalty_ability_needs_x(state, *source, *ability_index),
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
/// `modes_chosen` is `&[]` because that is exactly what
/// `mtg_simulator::params::action_to_command_with_params` builds the `Command`
/// with for `CastSpell`: it forwards `params.modes_chosen`, which is empty at
/// render time (the human has not answered yet). Any other value here would
/// advertise a target set for a cast the client cannot actually make.
///
/// **`alt_cost` is READ FROM THE ACTION as of PB-DX44 (`OOS-DX29-9`), not hard-
/// coded.** This doc used to say `params.rs` hard-codes `alt_cost: None`
/// unconditionally; PB-DX44 made that sentence false — `LegalAction::CastSpell`
/// now carries its OWN `alt_cost` (Pitch and SplitRightHalf are each a separate
/// action from the ordinary cast, not a client-chosen param), and `params.rs`
/// forwards it verbatim. Reading `None` here for a right-half-only action would
/// surface the printed card's FLAT (left-half) target list while `casting.rs`
/// demands the right half's — the exact SR-38 defect Stage 2b exists to close.
fn action_target_requirements(action: &LegalAction, state: &GameState) -> Vec<TargetRequirement> {
    match action {
        // `fuse: false`, same reasoning as `modes_chosen: &[]` above: the human has
        // not decided whether to fuse at render time (`params.rs` builds the
        // `Command` with whatever `AdditionalCost::Fuse` the client separately submits
        // in its `additional_costs`, which this render precedes). See
        // `action_option_view`'s `fused_target_slots` for the FUSED case.
        LegalAction::CastSpell { card, alt_cost, .. } => {
            mtg_engine::spell_target_requirements(state, *card, &[], *alt_cost, false)
        }
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => mtg_engine::ability_target_requirements(state, *source, *ability_index),
        // PB-DX29 (`OOS-M11-10(loyalty)`, CR 606.3 / CR 601.2c). Deliberately NOT
        // `ability_target_requirements` — a loyalty `ability_index` indexes the
        // registry def's `AbilityDefinition::LoyaltyAbility` entries, not the
        // layer-resolved `activated_abilities` list, and on a planeswalker carrying
        // both the two name different abilities. See
        // `queries.rs::loyalty_ability_target_requirements`.
        LegalAction::ActivateLoyaltyAbility {
            source,
            ability_index,
        } => mtg_engine::loyalty_ability_target_requirements(state, *source, *ability_index),
        // Every other variant either takes no targets or carries them inside the
        // action itself (`ActivateBloodrush`, `CastWithMutate`,
        // `ChooseTriggerTargets`), and `params.rs` refuses a `targets` param on
        // all of them with `UnsupportedParam`. Offering a picker whose value the
        // mapping table would reject with a 400 is worse than offering none.
        _ => Vec::new(),
    }
}

/// The object whose controller/source context the target query runs against.
///
/// **This function is as load-bearing as [`action_target_requirements`] and is easy to
/// miss**: `action_option_view`'s `slots` closure returns `Vec::new()` the moment this
/// returns `None`, so an action whose requirements are surfaced but whose source is not
/// renders a picker with **zero candidates** — visibly broken rather than absent. The
/// loyalty arm was added to both together (PB-DX29).
fn target_query_source(action: &LegalAction) -> Option<ObjectId> {
    match action {
        LegalAction::CastSpell { card, .. } => Some(*card),
        LegalAction::ActivateAbility { source, .. } => Some(*source),
        LegalAction::ActivateLoyaltyAbility { source, .. } => Some(*source),
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
                owner: names.owner(*id),
                value: t.clone(),
            },
            // PB-DX52 (`OOS-DX25b-1`): an ABILITY on the stack, offered as a target for
            // the first time. `kind` is a NEW wire value, `"stack_object"` -- deliberately
            // not reusing `"object"`, because the browser must not look this id up in the
            // card index (it is not a `state.objects` key and `names.label` would render
            // a placeholder). The label is the entry's source, matching how the same
            // ability is already labelled in the stack panel.
            Target::StackObject(id) => TargetOptionView {
                kind: "stack_object".to_string(),
                id: id.0,
                label: names.stack_entry_label(*id),
                owner: names.stack_entry_owner(*id),
                value: t.clone(),
            },
            Target::Player(p) => TargetOptionView {
                kind: "player".to_string(),
                id: p.0,
                label: display_name(*p, player_names),
                // A player target belongs in that player's own segment, so the
                // grouping key is their own name — the same string the label is,
                // which is why this is not read off `names`.
                owner: Some(display_name(*p, player_names)),
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

/// CR 118.8 / CR 702.157 (UI-2): render a `CastSpell` option's additional-cost
/// descriptor, or `None` for every other action -- and `None` for a `CastSpell`
/// whose plan is `Default` (both fields `None`), which is nearly every spell.
///
/// Every candidate is labelled through [`NameIndex`], never through
/// [`question_card_label`] -- see [`SacrificeCostView::candidates`]'s doc for why
/// the two channels must not be confused here.
fn additional_costs_view(
    action: &LegalAction,
    names: &NameIndex,
    player_names: &HashMap<PlayerId, String>,
) -> Option<AdditionalCostsView> {
    // CR 602.2 (SIM-6, triage G4): the ACTIVATION arm. This early return used to be
    // `let LegalAction::CastSpell { .. } else { return None }`, which is precisely
    // why the browser never rendered a cost picker for Yahenni or Altar of
    // Dementia -- the provider's plan existed nowhere for a non-cast, so
    // `ActionBar` never entered its cost stage and the human's only path was a
    // command the engine refused with a 422.
    if let LegalAction::ActivateAbility {
        activation_costs, ..
    } = action
    {
        return activation_costs_view(activation_costs, names);
    }

    let LegalAction::CastSpell {
        additional_costs: plan,
        ..
    } = action
    else {
        return None;
    };
    // PB-DX29: the presence test is deliberately NOT here any more. It used to read the
    // PLAN and return early, which is one fact away from what the client needs: the plan
    // can carry a `gift` whose builder below then yields `None` (an empty eligible set
    // has no seat to put in the template), and the panel would open with a prompt and
    // nothing to answer. The test now runs on the BUILT views, at the bottom of this
    // function — one place, reading the same values the wire will carry.

    let sacrifice = plan.sacrifice.as_ref().map(|sac| SacrificeCostView {
        prompt: sacrifice_prompt(&sac.requirement),
        candidates: sac
            .eligible
            .iter()
            .map(|id| CardOptionView {
                id: id.0,
                label: names.label(*id),
            })
            .collect(),
        default: sac.default.0,
        // `lki: vec![]` -- see the field's own doc. `casting.rs`'s sacrifice site
        // patches this from LKI it captures itself; nothing here supplies it.
        template: AdditionalCost::Sacrifice {
            ids: vec![sac.default],
            lki: vec![],
        },
        ids_key: "ids".to_string(),
    });

    let squad = plan.squad.as_ref().map(|sq| SquadCostView {
        prompt: "Pay the squad cost any number of times to create token copies on entry \
                 (CR 702.157a)"
            .to_string(),
        cost_label: format_mana_cost_compact(&sq.cost),
        max_count: sq.max_count,
        template: AdditionalCost::Squad { count: 0 },
        count_key: "count".to_string(),
    });

    // PB-DX29 — the four new families. Every label is display text; every JUDGMENT
    // (what is eligible, what N is affordable, whether the rider exists at all) is the
    // provider's, read verbatim from the plan and never re-derived here.
    let counts: Vec<CountCostView> = plan
        .counts
        .iter()
        .map(|c| match c.kind {
            CountCostKind::Replicate => CountCostView {
                kind: "Replicate".to_string(),
                prompt: "Pay the replicate cost any number of times to copy this spell \
                         (CR 702.56a)"
                    .to_string(),
                cost_label: format_mana_cost_compact(&c.cost),
                max_count: c.max_count,
                template: AdditionalCost::Replicate { count: 0 },
                count_key: "count".to_string(),
            },
            CountCostKind::Escalate => CountCostView {
                kind: "Escalate".to_string(),
                prompt: "Pay the escalate cost once for each mode beyond the first \
                         (CR 702.120a)"
                    .to_string(),
                cost_label: format_mana_cost_compact(&c.cost),
                max_count: c.max_count,
                template: AdditionalCost::EscalateModes { count: 0 },
                count_key: "count".to_string(),
            },
        })
        .collect();

    let markers: Vec<MarkerCostView> = plan
        .markers
        .iter()
        .map(|m| match m.kind {
            MarkerCostKind::Entwine => MarkerCostView {
                kind: "Entwine".to_string(),
                prompt: "Pay the entwine cost to choose all modes (CR 702.42a)".to_string(),
                cost_label: m.cost.as_ref().map(format_mana_cost_compact),
                affordable: m.affordable,
                template: AdditionalCost::Entwine,
            },
            MarkerCostKind::Fuse => MarkerCostView {
                kind: "Fuse".to_string(),
                prompt: "Cast both halves of this split card, paying both costs \
                         (CR 702.102a)"
                    .to_string(),
                // Deliberately `None`: CR 702.102b makes the cost the two halves
                // summed, so there is no separate figure and `{0}` would be a lie.
                cost_label: None,
                affordable: m.affordable,
                template: AdditionalCost::Fuse,
            },
            MarkerCostKind::Offspring => MarkerCostView {
                kind: "Offspring".to_string(),
                prompt: "Pay the offspring cost to create a 1/1 token copy when this \
                         creature enters (CR 702.175a)"
                    .to_string(),
                cost_label: m.cost.as_ref().map(format_mana_cost_compact),
                affordable: m.affordable,
                template: AdditionalCost::Offspring,
            },
        })
        .collect();

    let gift = plan.gift.as_ref().and_then(|g| {
        // The provider guarantees a non-empty eligible set, but a template needs a
        // concrete seat and an `expect` here would be a panic in a request handler.
        let first = g.eligible.first().copied()?;
        Some(GiftCostView {
            prompt: format!(
                "Promise an opponent {} as you cast this spell (CR 702.174a)",
                gift_label(&g.gift_type)
            ),
            gift_label: gift_label(&g.gift_type),
            candidates: g
                .eligible
                .iter()
                .map(|p| PlayerOptionView {
                    id: p.0,
                    label: display_name(*p, player_names),
                })
                .collect(),
            template: AdditionalCost::Gift { opponent: first },
            player_key: "opponent".to_string(),
        })
    });

    let splice = plan.splice.as_ref().map(|s| SpliceCostView {
        prompt: "Splice cards from your hand onto this spell, paying each one's splice \
                 cost (CR 702.47a)"
            .to_string(),
        candidates: s
            .eligible
            .iter()
            .map(|id| CardOptionView {
                id: id.0,
                label: names.label(*id),
            })
            .collect(),
        template: AdditionalCost::Splice { cards: vec![] },
        ids_key: "cards".to_string(),
    });

    // PB-DX44, CR 118.9: present only on the SEPARATE pitch `CastSpell` action --
    // `plan.pitch` is `None` on the ordinary cast, whichever spell it is (see
    // `AdditionalCostPlan::pitch`'s own doc).
    let pitch = plan.pitch.as_ref().and_then(|p| {
        // The provider guarantees a non-empty eligible set, but a template needs a
        // concrete card id and an `expect` here would be a panic in a request
        // handler -- same defensive shape as `gift` above.
        let default = p.eligible.first().copied()?;
        Some(PitchCostView {
            prompt: pitch_prompt(p),
            candidates: p
                .eligible
                .iter()
                .map(|id| CardOptionView {
                    id: id.0,
                    label: names.label(*id),
                })
                .collect(),
            default: default.0,
            template: AdditionalCost::ExileFromHand { card: default },
            card_key: "card".to_string(),
        })
    });

    // PB-DX29: "is there anything to ask?", asked of the BUILT views rather than of the
    // plan. Forgetting a family here is invisible — the picker simply never opens and the
    // rider is silently lost, which is `OOS-UI2-4`'s exact symptom, so the list is
    // exhaustive over every field the struct carries on the cast side.
    if sacrifice.is_none()
        && squad.is_none()
        && counts.is_empty()
        && markers.is_empty()
        && gift.is_none()
        && splice.is_none()
        && pitch.is_none()
    {
        return None;
    }

    Some(AdditionalCostsView {
        answer_field: "additional_costs".to_string(),
        // PB-DX29: the CR citation names what is actually being asked. The old text
        // hard-coded "CR 118.8 / CR 702.157" for every cast, which after this batch would
        // have cited a required sacrifice and Squad on a spell whose only rider is
        // Replicate, Gift or Splice — two rules that have nothing to do with it.
        prompt: cast_cost_prompt(
            &sacrifice, &squad, &counts, &markers, &gift, &splice, &pitch,
        ),
        sacrifice,
        squad,
        activation_sacrifice: None,
        activation_discard: None,
        counts,
        markers,
        gift,
        splice,
        pitch,
    })
}

/// PB-DX44, CR 118.9: the pitch panel's prompt, composed from the printed
/// non-mana cost components -- life payment and/or an exiled colour -- rather
/// than a hard-coded sentence, so Force of Will's "pay 1 life and exile a blue
/// card" and Misdirection's plain "exile a blue card" read correctly from the
/// SAME function.
fn pitch_prompt(p: &mtg_simulator::legal_actions::PitchCostOption) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in &p.costs {
        match c {
            mtg_engine::Cost::PayLife(n) => parts.push(format!("pay {n} life")),
            mtg_engine::Cost::ExileFromHand { color } => parts.push(format!(
                "exile a {} card from your hand",
                color_word(*color)
            )),
            _ => {}
        }
    }
    let body = if parts.is_empty() {
        "pay this spell's alternative cost".to_string()
    } else {
        parts.join(" and ")
    };
    let restriction = if p.opponents_turn_only {
        " -- only when it isn't your turn"
    } else {
        ""
    };
    format!("You may {body} rather than pay this spell's mana cost (CR 118.9){restriction}")
}

/// PB-DX44: the printed colour word for a pitch cost's `Color` component.
/// Exhaustive with **no wildcard arm**, same rule as [`gift_label`]: a sixth
/// `Color` variant must be named here or the crate stops compiling.
fn color_word(color: mtg_engine::Color) -> &'static str {
    match color {
        mtg_engine::Color::White => "white",
        mtg_engine::Color::Blue => "blue",
        mtg_engine::Color::Black => "black",
        mtg_engine::Color::Red => "red",
        mtg_engine::Color::Green => "green",
    }
}

/// PB-DX29: the panel header, naming the rules the offer on screen actually invokes.
///
/// Display text only. Every per-rider block carries its own precise prompt and CR cite;
/// this is the one-line summary above them, and its whole job is not to cite a rule the
/// human is not being asked about.
#[allow(clippy::too_many_arguments)]
fn cast_cost_prompt(
    sacrifice: &Option<SacrificeCostView>,
    squad: &Option<SquadCostView>,
    counts: &[CountCostView],
    markers: &[MarkerCostView],
    gift: &Option<GiftCostView>,
    splice: &Option<SpliceCostView>,
    pitch: &Option<PitchCostView>,
) -> String {
    let mut cites: Vec<&str> = Vec::new();
    if sacrifice.is_some() {
        cites.push("CR 118.8");
    }
    if squad.is_some() {
        cites.push("CR 702.157a");
    }
    for c in counts {
        cites.push(match c.kind.as_str() {
            "Replicate" => "CR 702.56a",
            _ => "CR 702.120a",
        });
    }
    for m in markers {
        cites.push(match m.kind.as_str() {
            "Entwine" => "CR 702.42a",
            "Fuse" => "CR 702.102a",
            _ => "CR 702.175a",
        });
    }
    if gift.is_some() {
        cites.push("CR 702.174a");
    }
    if splice.is_some() {
        cites.push("CR 702.47a");
    }
    if pitch.is_some() {
        cites.push("CR 118.9");
    }
    // A required sacrifice is the only one of these a player cannot decline, so the
    // header says "must" only when one is present. Pitch is its own third phrasing:
    // it REPLACES the mana cost rather than adding to it (CR 118.9a), so "additional
    // cost" would be the wrong rule for the one block that can appear alone here
    // (`offerable_pitch_plan` never combines it with any other family today).
    let verb = if sacrifice.is_some() {
        "This spell has an additional cost to cast"
    } else if pitch.is_some() {
        "This spell may be cast via an alternative cost instead of its mana cost"
    } else {
        "This spell has optional additional costs you may pay"
    };
    format!("{verb} ({})", cites.join(" / "))
}

/// PB-DX29, CR 702.174d-i: the printed name of what a gift's chosen player receives.
///
/// Exhaustive with **no wildcard arm**: a new `GiftType` must be labelled here or the
/// crate stops compiling, rather than being rendered as something else.
fn gift_label(gift_type: &GiftType) -> String {
    match gift_type {
        GiftType::Food => "a Food".to_string(),
        GiftType::Card => "a card".to_string(),
        GiftType::TappedFish => "a tapped Fish".to_string(),
        GiftType::Treasure => "a Treasure".to_string(),
        GiftType::Octopus => "an Octopus".to_string(),
        GiftType::ExtraTurn => "an extra turn".to_string(),
    }
}

/// CR 602.2 (SIM-6): render an `ActivateAbility` option's non-mana cost
/// components, or `None` when the ability has none -- which is the overwhelming
/// majority of abilities, exactly as for spells.
///
/// Nothing is re-derived: `eligible` and `default` are the provider's own fields,
/// mirroring `handle_activate_ability`'s gate, and
/// [`crate::api::validate_additional_cost_params`] checks a submission against
/// these same fields -- so the picker the human sees and the check the server makes
/// cannot disagree.
fn activation_costs_view(
    plan: &mtg_simulator::legal_actions::ActivationCostPlan,
    names: &NameIndex,
) -> Option<AdditionalCostsView> {
    if plan.sacrifice.is_none() && plan.discard.is_none() {
        return None;
    }

    let label = |ids: &[mtg_engine::ObjectId]| -> Vec<CardOptionView> {
        ids.iter()
            .map(|id| CardOptionView {
                id: id.0,
                label: names.label(*id),
            })
            .collect()
    };

    let activation_sacrifice = plan.sacrifice.as_ref().map(|sac| ActivationChoiceView {
        prompt: activation_sacrifice_prompt(&sac.filter, sac.exclude_self),
        candidates: label(&sac.eligible),
        default: sac.default.0,
        answer_field: "cost_sacrifice_target".to_string(),
    });

    let activation_discard = plan.discard.as_ref().map(|dis| ActivationChoiceView {
        prompt: "Discard a card to activate this ability (CR 602.2)".to_string(),
        candidates: label(&dis.eligible),
        default: dis.default.0,
        answer_field: "cost_discard_card".to_string(),
    });

    Some(AdditionalCostsView {
        // Carried for shape compatibility with the `CastSpell` payload; the two
        // blocks below name their own scalar fields and the client uses those.
        // See [`AdditionalCostsView::answer_field`].
        answer_field: "additional_costs".to_string(),
        prompt: "This ability has a cost to pay before it goes on the stack (CR 602.2)".to_string(),
        sacrifice: None,
        squad: None,
        activation_sacrifice,
        activation_discard,
        // PB-DX29: all four are CAST-side riders (CR 601.2b). An activated ability's
        // cost is a `Command::ActivateAbility` scalar and never an `AdditionalCost`, so
        // these are structurally empty here rather than merely unpopulated — spelled
        // out because the two block families sharing one struct is the thing a reader
        // most easily mistakes for an omission.
        counts: Vec::new(),
        markers: Vec::new(),
        gift: None,
        splice: None,
        // PB-DX44: Pitch is CAST-side too (CR 118.9), and only ever present on the
        // separate pitch `CastSpell` action -- see the comment above.
        pitch: None,
    })
}

/// A one-line prompt naming what CR 602.2 requires, from the engine's own
/// `SacrificeFilter` plus the CR 109.1 "another" bit -- for display only, the same
/// argument [`sacrifice_prompt`] makes for the spell side: the CANDIDATES already
/// carry the judgment of who is eligible.
///
/// The `another` wording is not decoration. Yahenni prints "Sacrifice **another**
/// creature", and until this batch its own card definition did not carry that bit
/// at all -- so a human who saw "sacrifice a creature" and did not see Yahenni in
/// the list would have no way to tell a correct exclusion from a missing card.
fn activation_sacrifice_prompt(
    filter: &mtg_engine::state::game_object::SacrificeFilter,
    exclude_self: bool,
) -> String {
    use mtg_engine::state::game_object::SacrificeFilter;
    let what = match filter {
        SacrificeFilter::Creature => "creature".to_string(),
        SacrificeFilter::Land => "land".to_string(),
        SacrificeFilter::Artifact => "artifact".to_string(),
        SacrificeFilter::ArtifactOrCreature => "artifact or creature".to_string(),
        SacrificeFilter::Subtype(sub) => sub.0.clone(),
        SacrificeFilter::CreatureOfChosenType => "creature of the chosen type".to_string(),
    };
    let article = if exclude_self {
        "another"
    } else if what.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("Sacrifice {article} {what} to activate this ability (CR 602.2)")
}

/// A one-line prompt naming what CR 118.8 requires, from the engine's own
/// `SpellAdditionalCost` -- for display only; the CANDIDATES already carry the
/// engine's judgment of who is eligible, so this text cannot itself be wrong in
/// a way that matters.
fn sacrifice_prompt(requirement: &SpellAdditionalCost) -> String {
    let what = match requirement {
        SpellAdditionalCost::SacrificeCreature => "a creature".to_string(),
        SpellAdditionalCost::SacrificeLand => "a land".to_string(),
        SpellAdditionalCost::SacrificeArtifactOrCreature => "an artifact or creature".to_string(),
        SpellAdditionalCost::SacrificeSubtype(sub) => format!("a {}", sub.0),
        SpellAdditionalCost::SacrificeColorPermanent(color) => {
            format!("a {color:?} permanent").to_lowercase()
        }
    };
    format!("Sacrifice {what} as an additional cost (CR 118.8)")
}

/// Compact MTG notation for a mana cost: `{2}{W}{W}`, `{0}` for a zero cost.
/// The generic number FIRST, then coloured pips in WUBRG order, then `{C}`.
///
/// Modelled on `tools/tui/src/play/panels/card_detail.rs::format_mana_cost`, and
/// deliberately a duplicate rather than a shared helper: that copy lives in a
/// different binary crate (`tools/tui`) with no dependency relationship to this
/// one, and is itself `#[allow(dead_code)]` there. Like `sacrifice_prompt` above,
/// this renders **display text only** -- it does not decide what a Squad payment
/// costs; the engine's own `ManaCost` arithmetic does that untouched.
///
/// **The pip ORDER deliberately diverges from that copy**, which emits colours
/// first and so renders Galadhrim Brigade's printed "Squad {1}{G}" as `{G}{1}`.
/// Every real card prints the generic component first (CR 107.4 / the comprehensive
/// rules' own notation), so the TUI's order is simply wrong; it is dead code there
/// and fixing it is out of this batch's scope, but a label a human reads next to a
/// printed card must match the printing.
///
/// **The hybrid / Phyrexian / `{X}` limitation is CLOSED as of PB-DX29, and the way it
/// surfaced is worth keeping.** This function used to render the seven plain components
/// and nothing else, so a cost carrying a hybrid pip, a Phyrexian pip or an `{X}`
/// displayed as strictly cheaper than it is. UI-2 knew, and pinned the premise with
/// `core::ui2_additional_cost_roster` R4 — "no def in the corpus has a hybrid or
/// Phyrexian **Squad** cost, so the gate fails loudly the day one is authored".
///
/// The day arrived on a **different cost kind**. PB-DX29 renders six more kinds through
/// this same formatter, and its `core::pb_dx29_additional_cost_roster` R3 — the same
/// assertion widened past Squad — went red on its first run against
/// `brokkos_apex_of_forever`, whose mutate cost is `{2}{G}{G}{U/B}`. A gate scoped to
/// one kind measures that kind; the corpus had carried the counter-example the whole
/// time. Rendering all three forms is the fix, rather than narrowing the gate back.
///
/// CR 107.4e (`{W/U}`, `{2/W}`), CR 107.4f (`{W/P}`, `{G/W/P}`), CR 107.3 (`{X}`).
/// Symbol ORDER follows the printed convention: `{X}` first, then generic, then the
/// coloured pips in WUBRG order, then `{C}`, then the hybrid and Phyrexian pips.
/// PB-DX45 (CR 118.12): a human-readable label for an optional cost.
///
/// Display only — the engine validates a `PayOptionalCost` answer against its
/// own recorded question and never against this string, so a formatting change
/// here can never change what is legal.
///
/// **Not exhaustive over `Cost`, deliberately, and the `other` arm is PROVABLY
/// DEAD rather than merely unlikely.** The engine asks a `PayOptionalCost`
/// question ONLY when `can_pay_optional_cost` has already returned `true`, and
/// that function returns an unconditional `false` for every `Cost` variant
/// outside `Mana` / `PayLife` / `DiscardCard` / `Sacrifice` / `Sequence`. So no
/// other variant can reach this function at all — a stronger bound than the
/// corpus gate `pb_dx45_may_pay_roster.rs::r2_every_corpus_cost_is_decidable`
/// provides, and one that does not depend on which cards happen to be authored.
/// (The first draft of this doc cited the corpus gate as its bound and described
/// the tail as unconditional-`true`; both were corrected by PB-DX45's own
/// `/review`.) The arm stays anyway: it renders as a debug-ish name, which is
/// ugly and correct, and it cannot render as something misleadingly specific.
fn format_optional_cost(cost: &Cost) -> String {
    match cost {
        Cost::Mana(mc) => format_mana_cost_compact(mc),
        Cost::PayLife(n) => format!("{n} life"),
        Cost::DiscardCard => "a card from your hand".to_string(),
        Cost::Sacrifice(_) => "a permanent you sacrifice".to_string(),
        Cost::Sequence(parts) => parts
            .iter()
            .map(format_optional_cost)
            .collect::<Vec<_>>()
            .join(" and "),
        other => format!("{other:?}"),
    }
}
fn format_mana_cost_compact(cost: &ManaCost) -> String {
    let mut parts = Vec::new();
    // CR 107.3: `{X}` is printed before the rest of the cost.
    for _ in 0..cost.x_count {
        parts.push("{X}".to_string());
    }
    if cost.generic > 0 {
        parts.push(format!("{{{}}}", cost.generic));
    }
    for _ in 0..cost.white {
        parts.push("{W}".to_string());
    }
    for _ in 0..cost.blue {
        parts.push("{U}".to_string());
    }
    for _ in 0..cost.black {
        parts.push("{B}".to_string());
    }
    for _ in 0..cost.red {
        parts.push("{R}".to_string());
    }
    for _ in 0..cost.green {
        parts.push("{G}".to_string());
    }
    for _ in 0..cost.colorless {
        parts.push("{C}".to_string());
    }
    // CR 107.4e: a hybrid pip is either of two colours, or a colour or two generic.
    for pip in &cost.hybrid {
        parts.push(match pip {
            HybridMana::ColorColor(a, b) => {
                format!("{{{}/{}}}", mana_color_symbol(a), mana_color_symbol(b))
            }
            HybridMana::GenericColor(c) => format!("{{2/{}}}", mana_color_symbol(c)),
        });
    }
    // CR 107.4f: a Phyrexian pip is its colour(s) or 2 life.
    for pip in &cost.phyrexian {
        parts.push(match pip {
            PhyrexianMana::Single(c) => format!("{{{}/P}}", mana_color_symbol(c)),
            PhyrexianMana::Hybrid(a, b) => {
                format!("{{{}/{}/P}}", mana_color_symbol(a), mana_color_symbol(b))
            }
        });
    }
    if parts.is_empty() {
        "{0}".to_string()
    } else {
        parts.join("")
    }
}

/// The single printed letter for a mana colour (CR 107.4a).
///
/// Exhaustive with **no wildcard arm**: a new `ManaColor` variant must be given a symbol
/// here or the crate stops compiling, rather than silently rendering as something else.
fn mana_color_symbol(color: &ManaColor) -> &'static str {
    match color {
        ManaColor::White => "W",
        ManaColor::Blue => "U",
        ManaColor::Black => "B",
        ManaColor::Red => "R",
        ManaColor::Green => "G",
        ManaColor::Colorless => "C",
    }
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
/// # What actually holds it — three gates, and what none of them covers
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
/// * `test_ui6_a_foreign_seat_never_receives_the_whole_library_look` is the same
///   behavioural shape for the CR 701.23a channel [`library_look_cards`] opened
///   (UI-6). It gets its **own** gate rather than an extra assertion inside the
///   scry one, and that is the MR-M11-01 lesson applied rather than restated: a
///   redaction gate checks the channel it was written for, and the scry gate's
///   needle is the `looked_at` key, which the search payload does not contain.
/// * `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places` pins the
///   number of raw `GameState` reads in this file's production code at
///   three, so a *fourth* look channel cannot be opened in silence. It counts a
///   **set** of needles, not one: UI-6 added a read that spells `.zone(` rather
///   than `.objects()`, and against the pre-UI-6 single-needle gate that new
///   channel would have been invisible — the count would have stayed at 2 and
///   stayed green. That near-miss is why the gate is a needle set now.
///
/// None of them covers a hidden-information channel that reads `GameState` some
/// other way — `zones()`, `card_registry()`, `player()`. The count gate watches
/// an enumerated needle set, which is a weaker claim than "every raw read" and is
/// the same shape of limitation MR-M11-01 is about; it is said here rather than
/// left to be rediscovered.
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

/// **CR 701.23a: every card in the library being searched, look-only.**
/// The look entitlement's *third* raw `GameState` read, and the reason the
/// Invariant-7 count gate moved from two to three (UI-6, `scutemob-194`).
///
/// # The entitlement, stated as the rule and not as a preference
///
/// CR 701.23a: *"To search for a card in a zone, look at **all** cards in that
/// zone (even if it's a hidden zone)."* The searcher is entitled to see the whole
/// library — not the subset the effect can find. That is not a UX opinion the
/// playtest happened to hold; it is what the rule says, and the engine already
/// encodes the *decision* half of it structurally: this seat is the one
/// `GameEvent::EffectChoiceRequired::private_to()` addresses, and `api.rs`'s
/// `seat_view` has already refused to build this payload for any other seat
/// (`pending.player == human`).
///
/// # …but "all cards in that zone" is not always the whole library
///
/// **CR 121.1 / CR 614.1: a search-restriction replacement changes the zone
/// being searched, so it changes what may be looked at.** Under an opponent's
/// Aven Mindcensor this player *"searches the top four cards of that library
/// instead"* — the entitlement is four cards, and showing 99 with 95 marked
/// "look only" would be a real over-disclosure of this seat's own library, not a
/// cosmetic one. So the same `apply_search_library_replacement` the engine's
/// search path calls (`effects/mod.rs`, immediately before it builds candidates)
/// is called here, and the look is narrowed to `Zone::top_n` exactly as the
/// engine narrows the candidates.
///
/// That makes this the **second** place in this file that restates an engine rule
/// rather than delegating — [`action_modes`] is the first and says so. It is
/// recorded the same way and for the same reason: if the engine's search path
/// ever stops calling that function, or starts restricting by something other
/// than `top_n`, this goes stale silently. It is confined to a *display* concern
/// (what the look list holds), the engine re-derives the candidate set on its own
/// at resolution, and every divergence is in the **narrowing** direction — a stale
/// restriction here shows too few cards, never too many.
///
/// The returned events are discarded deliberately. They are the engine's own
/// replacement-applied log lines, emitted by the resolution path; re-emitting
/// them from a render would put a duplicate in the feed every time the client
/// polls. The function takes `&GameState` and cannot mutate anything, so
/// discarding them loses nothing but noise.
///
/// # Why the library and not "the zone being searched"
///
/// `EffectChoiceQuestion::SearchLibrary` carries only its candidates, not the
/// zone they came from. It does not have to: the search effect
/// (`effects/mod.rs`, the `for p in players` loop) builds candidates from
/// `ZoneId::Library(p)` — and, with `also_search_graveyard`, `ZoneId::Graveyard(p)`
/// — where `p` is the very player the question is asked of. So the searched
/// library is **always the answering seat's own**, and `player` here is
/// `PendingDecision::player`. There is no engine path today by which one seat
/// answers a search of another seat's library, and if one is ever added this
/// function is wrong rather than merely incomplete. That premise is **stated
/// here, in prose, and is not machine-checked** — `EffectChoiceQuestion` carries
/// no zone for a gate to compare against. Said plainly rather than dressed up:
/// an earlier draft of this paragraph called it "the assertion of the premise",
/// which reads as a code guarantee this function does not make.
///
/// # Sorted by NAME, never in library order — this is the load-bearing line
///
/// CR 701.23a entitles the searcher to *look at* the cards. It does not entitle
/// them to learn the library's **order**, which is hidden information the shuffle
/// afterwards (CR 701.23e) exists to protect — and Architecture Invariant 7 names
/// "library order" explicitly, alongside another player's hand, as the thing that
/// must never reach the wrong client. Sending `Zone::object_ids()` in its stored
/// order would leak exactly that to the *right* client, which is a subtler defect
/// and a real one: a search that fails to find, or one whose effect does not
/// shuffle, would leave the seat knowing its own draw order.
///
/// So the list is sorted by `(label, id)`. The tiebreak on `id` keeps it
/// deterministic across the five copies of Swamp that share a label — the replay
/// and the seeded probes depend on a stable rendering — while the primary key
/// being the *name* means the ordering carries no positional information at all.
///
/// # Cost
///
/// One `Vec` of the library's length per rendered search option, labelled through
/// [`question_card_label`] (an `OrdMap` lookup each) and sorted. A Commander
/// library is ≤ 99 cards and a search option is rendered at most once per
/// decision, so this is far below `decision_view`'s measured ≈ 201 µs
/// (see [`action_option_view`]) and nowhere near `legal_targets_per_slot`, which
/// pays a `calculate_characteristics` per candidate.
fn library_look_cards(state: &GameState, player: PlayerId) -> Vec<CardOptionView> {
    let Ok(library) = state.zone(&mtg_engine::ZoneId::Library(player)) else {
        // A player with no library zone is not a state this engine produces; an
        // empty look list degrades to "the client renders the candidates alone",
        // which is exactly the pre-UI-6 behaviour.
        return Vec::new();
    };
    // CR 121.1 / CR 614.1 — see the doc above. `top_n` is the same accessor the
    // engine's own restriction path uses, so a restricted look and a restricted
    // candidate set are drawn from the same cards by construction rather than by
    // two implementations agreeing.
    let (restriction, _replacement_events) =
        mtg_engine::rules::replacement::apply_search_library_replacement(state, player);
    let ids = match restriction {
        Some(n) => library.top_n(n as usize),
        None => library.object_ids(),
    };
    let mut cards: Vec<CardOptionView> = ids
        .iter()
        .map(|id| CardOptionView {
            id: id.0,
            label: question_card_label(state, *id),
        })
        .collect();
    cards.sort_by(|a, b| a.label.cmp(&b.label).then(a.id.cmp(&b.id)));
    cards
}

/// UI-1: render a blocking decision's answer space, or `None` for an action that
/// is not one.
///
/// Nothing here is re-derived. `count`, `hand`, `candidates`, `looked_at`, `slots`
/// and every `default` come off the `LegalAction` the provider emitted, which is
/// the same data `handle_discard_to_hand_size` / `handle_answer_effect_choice` /
/// `handle_choose_trigger_targets` will validate the answer against — so the
/// picker the human sees and the check the engine makes cannot disagree.
///
/// `player` is [`PendingDecision::player`] — the seat this decision belongs to,
/// which `api.rs::seat_view` has already checked equals the viewing seat. It is
/// used for one thing: the CR 701.23a whole-library look
/// ([`library_look_cards`]). Passing it rather than deriving it is what keeps
/// that look pinned to the *answering* seat's own library.
fn blocking_decision_view(
    action: &LegalAction,
    player: PlayerId,
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
        // (CR 405.1) and therefore in `NameIndex`. The CANDIDATE source is
        // PER-QUESTION: `SearchLibrary`/`Scry`/`Surveil` name LIBRARY cards,
        // which are not in `NameIndex` and go through `question_card_label`
        // instead; `Discard` (ENG-1, CR 701.9b) names the answerer's own HAND,
        // which IS in `NameIndex` (see that arm's own comment below) and
        // deliberately does not use `question_card_label` at all.
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
                        // CR 701.23a's look entitlement, kept strictly apart from
                        // the answer space above — see the field's own doc.
                        all_cards: library_look_cards(state, player),
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
                EffectChoiceQuestion::Discard { hand, count } => {
                    // CR 701.9b / CR 400.7 / `OOS-ENG1-9`: a card drawn EARLIER
                    // IN THIS RESOLUTION (e.g. a "draw two, discard one" effect's
                    // draw half) mints a fresh `ObjectId` that the restored view
                    // this decision was built from cannot contain, so `names`
                    // has no entry for it -- not a redaction gap, a same-
                    // resolution ordering gap (deferred, see `OOS-ENG1-9`).
                    // Give each such candidate a DISTINGUISHING placeholder
                    // rather than the bare `UNKNOWN_LABEL`: two same-resolution
                    // draws must never render as two buttons with identical
                    // text. The ordinal counts only the unlabelled candidates,
                    // is stable for a given question (hand order is fixed), and
                    // can never mislabel a card as a DIFFERENT one -- a
                    // same-resolution-drawn id is always strictly greater than
                    // every id the restored view can contain.
                    let mut undrawn_ordinal = 0usize;
                    let candidates = hand
                        .iter()
                        .map(|id| {
                            let label = match names.label_opt(*id) {
                                Some(name) => name,
                                None => {
                                    undrawn_ordinal += 1;
                                    format!("(card drawn this resolution #{undrawn_ordinal})")
                                }
                            };
                            CardOptionView { id: id.0, label }
                        })
                        .collect();
                    (
                        "Discard",
                        format!(
                            "{src}: discard {count} card{} — you choose (CR 701.9b). \
                             A card drawn earlier in this same resolution cannot be \
                             named yet (OOS-ENG1-9); it renders as \"(card drawn this \
                             resolution #N)\".",
                            plural(*count as usize)
                        ),
                        AnswerShapeView::PickN {
                            // CR 701.9b: these are the ANSWERER'S OWN hand cards, so
                            // they are already in the seat-redacted view and their
                            // labels come through `NameIndex` -- exactly as the
                            // CR 514.1 arm above does it, and deliberately NOT
                            // through `question_card_label`. That channel exists for
                            // LIBRARY cards the effect has granted a look at, and
                            // `test_ui6_view_rs_reads_game_state_in_exactly_the_
                            // three_known_places` pins its size; routing an owned-hand
                            // question through it would enlarge a channel for no
                            // reason and blur what that gate is counting.
                            candidates,
                            count: *count as usize,
                            // CR 701.9b's discard is always exactly `count`.
                            min_count: *count as usize,
                            chosen_key: "chosen".to_string(),
                            template: answer.clone(),
                            default: match answer {
                                EffectChoiceAnswer::Discard { chosen } => {
                                    chosen.iter().map(|id| id.0).collect()
                                }
                                _ => Vec::new(),
                            },
                        },
                    )
                }
                // PB-DX28 (CR 115.10): a resolution-time UNTARGETED object
                // choice. Through PB-DX35, `candidates` named only PUBLIC objects
                // (battlefield permanents / graveyard cards) -- that stopped being
                // true the moment `Effect::LookAtTopThenPlace`'s `optional`
                // placement (`OOS-DX4-5`) started asking this SAME variant with
                // LIBRARY ids, a hidden zone (CR 400.2). Redaction is unaffected
                // either way: `GameEvent::EffectChoiceRequired` is `private_to
                // (player)`, the same channel `SearchLibrary`/`Scry`/`Surveil`
                // already carry hidden library ids on. This still goes through
                // `question_cards` -- the SAME channel `SearchLibrary` above
                // already uses, not a new raw `GameState` read (see
                // `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_
                // places`) -- so no code change was owed for either population.
                EffectChoiceQuestion::ChooseObject {
                    candidates,
                    count,
                    up_to,
                } => (
                    "ChooseObject",
                    format!(
                        "{src}: choose {}{} (CR 115.10 -- not a targeted choice)",
                        if *up_to {
                            format!("up to {count}")
                        } else {
                            count.to_string()
                        },
                        if candidates.len() == 1 {
                            " object"
                        } else {
                            " objects"
                        }
                    ),
                    AnswerShapeView::PickN {
                        candidates: question_cards(state, candidates),
                        count: *count as usize,
                        min_count: if *up_to { 0 } else { *count as usize },
                        chosen_key: "chosen".to_string(),
                        template: answer.clone(),
                        default: match answer {
                            EffectChoiceAnswer::ChooseObject { chosen } => {
                                chosen.iter().map(|id| id.0).collect()
                            }
                            _ => Vec::new(),
                        },
                    },
                ),
                // PB-DX45 (CR 118.12): pay an optional cost, or decline. No
                // candidate ids at all, so neither `NameIndex` nor
                // `question_cards` is involved -- there is nothing to label. The
                // cost is rendered from the question's own `Cost`, which is a
                // printed characteristic of a public card.
                EffectChoiceQuestion::PayOptionalCost { cost } => (
                    "PayOptionalCost",
                    format!(
                        "{src}: you may pay {} (CR 118.12)",
                        format_optional_cost(cost)
                    ),
                    AnswerShapeView::Confirm {
                        cost_label: format_optional_cost(cost),
                        template: answer.clone(),
                        pay_key: "pay".to_string(),
                        default: match answer {
                            EffectChoiceAnswer::PayOptionalCost { pay } => *pay,
                            // Unreachable against the engine, which always pairs
                            // the variants; `true` mirrors
                            // `default_effect_choice_answer` rather than inventing
                            // a third behaviour.
                            _ => true,
                        },
                    },
                ),
                // PB-DX50 (CR 702.140c): over or under. Like `PayOptionalCost`
                // there is no candidate list, but unlike it there is also no
                // cost -- so this is `BinaryChoice`, not `Confirm`, and the two
                // labels say what the two answers actually are. `host` IS in
                // `NameIndex`: it is a battlefield permanent (CR 400.1, public)
                // and it is this spell's announced target, so no new
                // `GameState` read is involved (see `test_ui6_view_rs_reads_
                // game_state_in_exactly_the_three_known_places`).
                EffectChoiceQuestion::MutateOnTop { host } => {
                    let host_label = names.label(*host);
                    (
                        "MutateOnTop",
                        format!(
                            "{src}: put this on top of {host_label}, or under it? \
                             The topmost card supplies the merged permanent's name, \
                             mana cost, colours, types and power/toughness \
                             (CR 702.140c / CR 702.140e)."
                        ),
                        AnswerShapeView::BinaryChoice {
                            true_label: format!("On top of {host_label}"),
                            false_label: format!("Under {host_label}"),
                            template: answer.clone(),
                            choice_key: "on_top".to_string(),
                            default: match answer {
                                EffectChoiceAnswer::MutateOnTop { on_top } => *on_top,
                                // Unreachable against the engine, which always
                                // pairs the variants; `true` mirrors
                                // `default_effect_choice_answer` rather than
                                // inventing a third behaviour.
                                _ => true,
                            },
                        },
                    )
                }
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

    // PB-DX44 (`OOS-DX29-12`), CR 702.102a/d: the FUSED requirement list --
    // `fuse: true` is a no-op for any card without the Fuse keyword (`casting_
    // with_fuse`'s own gate), so this is safe and cheap to compute unconditionally
    // for every `CastSpell` option and stays empty for every other action kind.
    // See `ActionOptionView::fused_target_slots`'s own doc for why this is a
    // SEPARATE field rather than folded into `target_slots` above.
    let fused_requirements = match action {
        LegalAction::CastSpell { card, .. } => {
            mtg_engine::spell_target_requirements(state, *card, &[], None, true)
        }
        _ => Vec::new(),
    };
    let (fused_target_min, fused_target_max) = mtg_engine::target_count_range(&fused_requirements);
    let fused_target_slots = slots(&fused_requirements);

    // CR 700.2a/700.2c. A modal action's per-mode target requirements are only
    // knowable once the modes are chosen, and the human has not chosen yet, so
    // each mode carries its own slots and the client picks the ones for the modes
    // it selects. `spell_target_requirements` with an empty `modes_chosen`
    // deliberately returns `vec![]` for such a card (its own doc, divergence 1),
    // which is why `target_slots` above is empty for them rather than wrong.
    //
    // OOS-DX20-8: for a modal AURA specifically, this reasoning has a gap -- once
    // a mode is chosen, the cast path and this query path can diverge on the
    // per-mode target requirement (an unconditional cast rejection in one shape,
    // a query/cast disagreement in the other). Corpus exposure is 0 today (no Aura
    // def also carries `AbilityDefinition::Spell`), gated by a roster assertion.
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
        label: action_label(action, names, state),
        object_id: action_object(action).map(|id| id.0),
        target_slots,
        target_min,
        target_max,
        fused_target_slots,
        fused_target_min,
        fused_target_max,
        needs_x: action_needs_x(action, state),
        modes,
        mode_min,
        mode_max,
        attack,
        block,
        order: order_options(action, names),
        decision: blocking_decision_view(action, player, state, names, player_names),
        costs: additional_costs_view(action, names, player_names),
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
        // SIM-5 fix (3): see `BugReportView::rejections`.
        rejections: session
            .game
            .rejections()
            .iter()
            .map(|r| RejectionView {
                turn: r.turn,
                player: r.player.0,
                command: r.command.clone(),
                error: r.error.clone(),
            })
            .collect(),
        rejection_count: session.game.rejection_count(),
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

#[cfg(test)]
mod format_mana_cost_compact_tests {
    use super::*;
    use mtg_engine::{HybridMana, ManaColor, PhyrexianMana};

    /// PB-DX29 `/review` M3 — CR 107.3 / 107.4a / 107.4e / 107.4f.
    ///
    /// # Why these exist
    ///
    /// `format_mana_cost_compact` had **no test anywhere** — five call sites, zero
    /// assertions — and PB-DX29 taught it three new symbol families. The roster gate's
    /// non-vacuity floor was supposed to keep those arms honest, and the review proved
    /// it could not: the only corpus costs carrying a hybrid pip are `MutateCost`s,
    /// which this formatter is never handed. **A floor satisfied by a value the function
    /// never sees is not a floor**, so the correctness of the new arms rests here
    /// instead, on direct assertions rather than on the corpus happening to contain the
    /// right shape.
    ///
    /// The order is the printed one: `{X}`, then generic, then WUBRG, then `{C}`, then
    /// hybrid, then Phyrexian.
    #[test]
    fn every_symbol_family_renders_in_the_printed_order() {
        let cost = ManaCost {
            generic: 2,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            colorless: 1,
            x_count: 1,
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black),
                HybridMana::GenericColor(ManaColor::White),
            ],
            phyrexian: vec![
                PhyrexianMana::Single(ManaColor::Green),
                PhyrexianMana::Hybrid(ManaColor::Red, ManaColor::White),
            ],
        };
        assert_eq!(
            format_mana_cost_compact(&cost),
            "{X}{2}{W}{U}{B}{R}{G}{C}{U/B}{2/W}{G/P}{R/W/P}"
        );
    }

    /// CR 107.4e/107.4f in isolation — the two families PB-DX29 added, each alone, so a
    /// regression in one cannot be masked by the other being present.
    #[test]
    fn hybrid_and_phyrexian_render_alone() {
        let hybrid = ManaCost {
            hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
            ..Default::default()
        };
        assert_eq!(format_mana_cost_compact(&hybrid), "{G/W}");
        let phyrexian = ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
            ..Default::default()
        };
        assert_eq!(format_mana_cost_compact(&phyrexian), "{U/P}");
    }

    /// **The exact cost that reddened this batch's own R3 on its first run**, rendered.
    /// `brokkos_apex_of_forever`'s mutate cost is `{2}{G}{G}{U/B}`; before PB-DX29 this
    /// function would have printed `{2}{G}{G}` and told a human the cost was one pip
    /// cheaper than it is.
    #[test]
    fn the_cost_that_reddened_r3_renders_in_full() {
        let brokkos = ManaCost {
            generic: 2,
            green: 2,
            hybrid: vec![HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black)],
            ..Default::default()
        };
        assert_eq!(format_mana_cost_compact(&brokkos), "{2}{G}{G}{U/B}");
    }

    /// A zero cost still renders as `{0}` rather than as the empty string — the
    /// pre-existing behaviour, pinned so the new arms cannot have moved it.
    #[test]
    fn a_zero_cost_still_renders_as_zero() {
        assert_eq!(format_mana_cost_compact(&ManaCost::default()), "{0}");
    }
}

#[cfg(test)]
mod pb_dx50_binary_choice_wire_shape {
    use super::*;
    use mtg_engine::{
        CardType, EffectChoiceAnswer, EffectChoiceQuestion, GameStateBuilder, ObjectSpec, PlayerId,
        SubType,
    };
    use mtg_simulator::LegalAction;

    /// PB-DX50 `/review` — the `MutateOnTop` decision's **serialized JSON**, asserted
    /// key by key against what `BinaryChoicePicker.svelte` actually reads.
    ///
    /// # Why a wire-shape test and not another source gate
    ///
    /// Both halves of this surface — `view.rs`'s `blocking_decision_view` arm and
    /// `api::validate_decision_params`' arm — were pinned by SOURCE gates only, and
    /// nothing anywhere constructed the question and looked at the bytes. **PB-DX45
    /// shipped a 400-on-every-answer defect on exactly this surface**: a clean offer
    /// followed by a guaranteed refusal, with seven compile-forced consumers updated and
    /// the eighth — the one that broke — silently taking a wildcard arm. A source gate
    /// asserting that an arm exists cannot see a field whose name the client does not
    /// read.
    ///
    /// So this asserts the five keys `ActionBar.svelte` reads off the shape
    /// (`currentShape.true_label` / `.false_label` / `.template` / `.choice_key` /
    /// `.default`) plus the `shape` tag it dispatches on. The `template` is checked by
    /// PRESENCE of its single variant key, never against a hard-coded JSON string, so a
    /// regression to `template: {}` fails here rather than sailing through.
    ///
    /// **Revert to watch red**: rename any of those fields in
    /// [`AnswerShapeView::BinaryChoice`] (or its serde name), or change the `shape` tag.
    #[test]
    fn test_dx50_mutate_on_top_serializes_the_keys_the_picker_reads() {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut host = ObjectSpec::card(p1, "DX50 Wire Host")
            .in_zone(mtg_engine::ZoneId::Battlefield)
            .with_types(vec![CardType::Creature])
            .with_subtypes(vec![SubType("Wolf".to_string())]);
        host.power = Some(2);
        host.toughness = Some(3);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(host)
            .active_player(p1)
            .build()
            .expect("fixture builds");
        let host_id = *state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "DX50 Wire Host")
            .map(|(id, _)| id)
            .expect("the host is in the state");

        let player_names: HashMap<PlayerId, String> =
            [(p1, "P1".to_string()), (p2, "P2".to_string())]
                .into_iter()
                .collect();
        let names = NameIndex::from_view(&StateViewModel::from_game_state_for(
            &state,
            &player_names,
            mtg_view_model::Viewer::Seat(p1),
        ));

        let action = LegalAction::AnswerEffectChoice {
            choice_id: 7,
            source: host_id,
            question: EffectChoiceQuestion::MutateOnTop { host: host_id },
            answer: EffectChoiceAnswer::MutateOnTop { on_top: true },
        };
        let view = blocking_decision_view(&action, p1, &state, &names, &player_names)
            .expect("the MutateOnTop arm must produce a decision view");
        let json = serde_json::to_value(&view).expect("BlockingDecisionView serializes");

        assert_eq!(json["question"], "MutateOnTop");
        assert_eq!(
            json["answer_field"], "effect_choice_answer",
            "`api::validate_decision_params` reads the answer out of this \
             `ActionParamsDto` field; a mismatch is a guaranteed 400 (PB-DX45's own \
             defect on this exact surface): {json}"
        );

        let answer = &json["answer"];
        assert_eq!(
            answer["shape"], "BinaryChoice",
            "`ActionBar.svelte` dispatches on this tag; anything else renders NO picker \
             and the decision is unanswerable from the browser: {answer}"
        );
        // The five keys `ActionBar.svelte` passes into the picker, by name.
        for key in [
            "true_label",
            "false_label",
            "template",
            "choice_key",
            "default",
        ] {
            assert!(
                answer.get(key).is_some(),
                "`BinaryChoicePicker` reads `currentShape.{key}`; a rename here is a \
                 silently dead control, not a compile error: {answer}"
            );
        }
        assert_eq!(answer["choice_key"], "on_top");
        assert_eq!(
            answer["default"], true,
            "CR 702.140c: the engine's default is the exact recovery of the pre-PB-DX50 \
             hard-coded `on_top: true`, so bots and replays are unmoved"
        );
        let true_label = answer["true_label"]
            .as_str()
            .expect("true_label is a string");
        let false_label = answer["false_label"]
            .as_str()
            .expect("false_label is a string");
        assert!(
            !true_label.is_empty() && !false_label.is_empty() && true_label != false_label,
            "the two buttons must be distinguishable and non-empty: \
             {true_label:?} / {false_label:?}"
        );

        // The template carries its variant KEY, asserted by presence rather than
        // against a literal, and `choice_key` must name a key INSIDE it -- which is
        // the invariant `BinaryChoicePicker` relies on when it clones and mutates.
        let template = answer["template"]
            .as_object()
            .expect("template is a JSON object");
        assert_eq!(
            template.len(),
            1,
            "an externally-tagged Rust enum serializes to exactly one key: {template:?}"
        );
        assert!(
            template.contains_key("MutateOnTop"),
            "template must carry the MutateOnTop variant key: {template:?}"
        );
        assert_eq!(
            answer["template"]["MutateOnTop"]["on_top"], true,
            "`choice_key` is \"on_top\" and the picker writes the boolean THERE; if this \
             key does not exist in the template the picker mutates a clone that the \
             server then rejects: {template:?}"
        );

        // The prompt is a user-visible sentence: no run of collapsed whitespace, which
        // is how the first draft of this arm rendered (three 30-space gaps from a
        // single-physical-line `format!`).
        let prompt = json["prompt"].as_str().expect("prompt is a string");
        assert!(
            !prompt.contains("  "),
            "the prompt must read as one sentence, not a `format!` literal's leading \
             indentation: {prompt:?}"
        );
    }
}

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

use mtg_engine::{AdditionalCost, AttackTarget, GameState, ObjectId, PlayerId, Target};
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
    /// The **base** seed — `--seed`, or the `POST /api/game` override.
    ///
    /// S5 review LOW 7: this is *not* the seed the table in play was built from
    /// once a mulligan has been taken. `PlaySession::mulligan` goes through
    /// `setup::redeal`, which builds from `redeal_seed(seed, human_seat,
    /// mulligan_count)` and leaves `cfg.seed` untouched. The table is still
    /// exactly reproducible, but from **four** fields rather than one:
    /// [`GameSummary::seed`], [`GameSummary::players`], [`GameSummary::bot`] and
    /// [`GameSummary::mulligan_count`] — all four of which are right here, which
    /// is why the effective seed is documented rather than duplicated (the
    /// derivation is private to `mtg_simulator::setup` and recomputing it here
    /// would be a copy that could silently drift from it).
    pub seed: u64,
    pub turn: u32,
    /// Commands applied so far — 0 exactly while the game is still pregame.
    pub command_count: u32,
    /// CR 103.5: pregame redeals this seat has taken. Part of the table's
    /// reproduction key — see [`GameSummary::seed`].
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
    /// CR 601.2c: per-slot legal target candidates.
    ///
    /// **S7 populates this** from `crates/engine/src/rules/queries.rs`
    /// (`spell_target_requirements` + `legal_targets_per_slot`). This session
    /// ships the field and always leaves it empty, so the wire shape is settled
    /// before the frontend lands. Per the S4 handoff, those labels are a fifth
    /// rendering site and must be built from the seat-redacted view exactly as
    /// [`NameIndex`] does here.
    pub target_slots: Vec<Vec<TargetOptionView>>,
    /// CR 107.3 / 601.2b: this action needs an `x_value` announced.
    pub needs_x: bool,
    /// CR 700.2: selectable modes.
    ///
    /// **S7 populates this**, for the same reason as `target_slots`; empty this
    /// session.
    pub modes: Vec<ModeOptionView>,
}

/// One legal target for one slot.
#[derive(Debug, Serialize)]
pub struct TargetOptionView {
    /// `"object"` or `"player"`.
    pub kind: String,
    pub id: u64,
    /// Redacted display text.
    pub label: String,
}

/// One selectable mode of a modal spell or ability (CR 700.2).
#[derive(Debug, Serialize)]
pub struct ModeOptionView {
    pub index: usize,
    pub label: String,
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
    pub reason: Option<String>,
    /// Simulator invariant violations observed during the game, stringified.
    /// Always empty in a healthy game; surfaced because this is a play-testing
    /// surface and a violation is exactly what it is here to find.
    pub violations: Vec<String>,
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
#[derive(Debug, Default, Deserialize)]
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
    /// CR 103.5.
    pub cards_to_bottom: Vec<ObjectId>,
    pub additional_costs: Vec<AdditionalCost>,
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

impl From<ActionParamsDto> for ActionParams {
    fn from(dto: ActionParamsDto) -> Self {
        ActionParams {
            targets: dto.targets,
            x_value: dto.x_value,
            modes_chosen: dto.modes_chosen,
            attackers: dto.attackers,
            blockers: dto.blockers,
            cards_to_bottom: dto.cards_to_bottom,
            additional_costs: dto.additional_costs,
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
            names.insert(item.id, non_empty(&item.source_name));
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
/// directly — the standing layer-correctness gotcha. Only `CastSpell` is
/// answered today: an activated ability's `{X}` lives in its `ActivationCost`,
/// which `LegalAction::ActivateAbility` does not carry, so S7 will answer that
/// half alongside `modes`.
fn action_needs_x(action: &LegalAction, state: &GameState) -> bool {
    let LegalAction::CastSpell { card, .. } = action else {
        return false;
    };
    mtg_engine::calculate_characteristics(state, *card)
        .and_then(|chars| chars.mana_cost)
        .is_some_and(|cost| cost.x_count > 0)
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
) -> DecisionView {
    let actions = decision
        .actions
        .iter()
        .enumerate()
        .map(|(index, action)| ActionOptionView {
            index,
            kind: action_kind(action).to_string(),
            label: action_label(action, names),
            object_id: action_object(action).map(|id| id.0),
            // S7 populates this from crates/engine/src/rules/queries.rs.
            target_slots: Vec::new(),
            needs_x: action_needs_x(action, state),
            // S7 populates this too.
            modes: Vec::new(),
        })
        .collect();

    DecisionView {
        seq: wire_seq,
        kind: decision_kind_tag(decision.kind),
        player: decision.player.0,
        actions,
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
        reason: result.error.as_ref().map(|e| format!("{e:?}")),
        violations: result.violations.iter().map(|v| format!("{v:?}")).collect(),
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
        reason: Some(format!("{reason:?}")),
        violations: Vec::new(),
    }
}

/// Player display names are public information (they are shown to the whole
/// table), so this needs no entitlement check.
pub fn display_name(player: PlayerId, player_names: &HashMap<PlayerId, String>) -> String {
    player_names
        .get(&player)
        .cloned()
        .unwrap_or_else(|| format!("player_{}", player.0))
}

//! Pass-until as a server parameter (play surface v3 C3, `scutemob-268`;
//! `docs/play-surface-v3-brief.md` §5).
//!
//! `POST /api/game/action` may carry a `pass_until` object beside a
//! `PassPriority` answer. The server then keeps passing for this seat, inside the
//! one request and under the one session lock, until the rule says stop — instead
//! of the browser issuing one request per pass.
//!
//! # This is a port, not a redesign
//!
//! The rule is the one the v3 auto-pass panel already states and the client
//! already runs: `frontend/src/lib/v3/autopass.js` (`buildPredicate`) for *when*
//! to stop and `frontend/src/lib/stores.js::startPassUntil` for the loop around
//! it. Every check here names the function it mirrors. Where the two could have
//! been "improved" they were not — a run must stop at the same pass whichever
//! side drives it, or the panel's sentence stops being true.
//!
//! This module holds the wire types and the stop rule. The loop itself is
//! `api.rs::run_pass_until`, because it is the part that touches the session.
//!
//! # Everything is read from the seat-redacted view (Architecture Invariant 7)
//!
//! [`PassRule::check`] takes a [`StateViewModel`] and a slice of [`EventView`]s
//! and nothing else — no `GameState`, no `GameEvent`. The caller builds both for
//! `Viewer::Seat(human)`, exactly as it would for a response. So the server can
//! only ever stop a run on something the seat would have been shown had it made
//! that pass by hand: *where* a run stops is itself a signal, and a rule that
//! consulted the engine could leak through it (stopping on a spell the seat
//! cannot see, say). The type signature is what rules that out.
//!
//! # What a server-side run cannot do
//!
//! Be cancelled part-way. The client loop checks a cancel flag between
//! requests; here there is one request. The iteration cap
//! ([`MAX_AUTO_PASSES`]) and the "unless" stops are the only brakes.

use std::collections::BTreeSet;

use mtg_view_model::{EventView, StateViewModel};
use serde::{Deserialize, Serialize};

/// The most passes one run will make. `stores.js::MAX_AUTO_PASSES`, same number
/// and same reasoning: `session::config_for` allows 500 consecutive passes and
/// 200 turns, so this stops first and stops visibly.
pub const MAX_AUTO_PASSES: u32 = 400;

// ── Request ──────────────────────────────────────────────────────────────────

/// The `pass_until` object of a `POST /api/game/action` body.
///
/// ```json
/// { "until": { "kind": "step", "step": "PostCombatMain" },
///   "unless": { "spell": true, "ability": false, "attackers": true },
///   "anchor": { "turn": 7, "step": "Upkeep" } }
/// ```
///
/// `unless` and `anchor` are optional; an omitted `unless` stops on nothing.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassUntilRequest {
    /// The point the run is headed for.
    pub until: Until,
    /// Things that interrupt it on the way.
    #[serde(default)]
    pub unless: Unless,
    /// Where the run **originally** started, when this request resumes a run an
    /// "unless" stop interrupted. Omitted for a fresh run, which is measured
    /// from the state the request finds.
    ///
    /// `autopass.js::buildPredicate`'s `anchor` argument: "until end of turn"
    /// keeps meaning the turn the human chose it in, not the turn they pressed
    /// Resume in.
    #[serde(default)]
    pub anchor: Option<Anchor>,
}

/// What the run passes *until* — the panel's three modes (`autopass.js::MODES`,
/// less `off`, which is the absence of a `pass_until`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Until {
    /// `{"kind":"end_of_turn"}` — the turn number moves on.
    EndOfTurn,
    /// `{"kind":"my_next_turn"}` — this seat is the active player in a turn
    /// other than the one the run started in.
    MyNextTurn,
    /// `{"kind":"step","step":"<StopStep>"}` — any player's next occurrence of
    /// that step.
    Step { step: StopStep },
}

/// The steps a run can be told to stop at: `autopass.js::STOPPABLE_STEPS`.
///
/// `Untap` and `Cleanup` are not here, for that function's stated reasons — no
/// player receives priority in the untap step (CR 502.4), nor in cleanup unless
/// something happens there (CR 514.3), and a run only ever stops where this seat
/// is handed a `Priority` decision. Sending either is a 400 `invalid_body`.
///
/// The spellings are the engine's `Step` variant names, which is what
/// `TurnView::step` carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StopStep {
    Upkeep,
    Draw,
    PreCombatMain,
    BeginningOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndOfCombat,
    PostCombatMain,
    End,
}

impl StopStep {
    /// The name as `TurnView::step` spells it (`format!("{:?}", Step::..)`).
    pub fn wire_name(self) -> &'static str {
        match self {
            StopStep::Upkeep => "Upkeep",
            StopStep::Draw => "Draw",
            StopStep::PreCombatMain => "PreCombatMain",
            StopStep::BeginningOfCombat => "BeginningOfCombat",
            StopStep::DeclareAttackers => "DeclareAttackers",
            StopStep::DeclareBlockers => "DeclareBlockers",
            StopStep::FirstStrikeDamage => "FirstStrikeDamage",
            StopStep::CombatDamage => "CombatDamage",
            StopStep::EndOfCombat => "EndOfCombat",
            StopStep::PostCombatMain => "PostCombatMain",
            StopStep::End => "End",
        }
    }
}

/// The stop-conditions (`autopass.js::STOPS`). Every one defaults to off.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Unless {
    /// A spell is cast by another player (CR 601).
    pub spell: bool,
    /// An ability is **activated** by another player (CR 602). Not a triggered
    /// ability, and not a mana ability — see [`ACTIVATION_KINDS`].
    pub ability: bool,
    /// Attackers are declared at this seat, or at a planeswalker it controls
    /// (CR 508.1). The panel's `combat` checkbox.
    pub attackers: bool,
}

/// `{turn, step}` of the state a resumed run originally started from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    /// `summary.turn` / `state.turn.number` at the original start.
    pub turn: u32,
    /// `state.turn.step` at the original start. Any step name; it is only ever
    /// compared for equality.
    pub step: String,
}

// ── Response ─────────────────────────────────────────────────────────────────

/// The `pass_until` object of the response to a request that carried one.
///
/// ```json
/// { "passes": 4, "stop": "unless", "reason": "Bot-2 casts Spidersilk Net" }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PassUntilOutcome {
    /// How many times the server passed priority for this seat, the request's
    /// own pass included.
    pub passes: u32,
    /// Why the run ended, as a tag.
    pub stop: StopKind,
    /// Why the run ended, as a sentence for the panel.
    pub reason: String,
}

/// Why a run ended.
///
/// `until` is the only one that means the run is *finished*; `unless` means it
/// was *interrupted* and may be resumed (send the same `pass_until` again with
/// an `anchor`). Every other value is an unconditional stop of the loop — the
/// ones `stores.js::startPassUntil` applies before it ever consults the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    /// The point `until` names was reached.
    Until,
    /// An `unless` condition fired first.
    Unless,
    /// This seat has a decision that is not a `Priority` decision. The loop
    /// never answers one.
    Decision,
    /// Nothing is awaiting this seat's answer (and the game is not over).
    NoDecision,
    /// The game ended, or was halted.
    GameOver,
    /// The `Priority` decision does not offer `PassPriority`.
    NoPass,
    /// [`MAX_AUTO_PASSES`] passes were made.
    Limit,
    /// A pass after the first was refused by the server. The passes already
    /// made stand; `reason` carries the refusal.
    Rejected,
}

// ── The rule ─────────────────────────────────────────────────────────────────

/// The event kinds that are "an ability was activated":
/// `autopass.js::ACTIVATION_KINDS`, with its reasoning. `AbilityActivated` is
/// every non-mana activation (CR 602) including loyalty abilities and keyword
/// activations; `CraftActivated` is the one activation with an event of its own.
/// Mana abilities (`ManaAdded` / `PermanentTapped`) and triggered abilities
/// (`AbilityTriggered`) are deliberately not here.
pub const ACTIVATION_KINDS: [&str; 2] = ["AbilityActivated", "CraftActivated"];

/// `{ number, active, step }` off a seat view's turn — `autopass.js::turnFacts`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnFacts {
    number: u32,
    active: String,
    step: String,
}

impl TurnFacts {
    fn of(view: &StateViewModel) -> Self {
        TurnFacts {
            number: view.turn.number,
            active: view.turn.active_player.clone(),
            step: view.turn.step.clone(),
        }
    }
}

/// One run's stop rule: `autopass.js::buildPredicate`.
///
/// Built from the seat view the run starts from, then asked after every pass.
#[derive(Debug)]
pub struct PassRule {
    until: Until,
    unless: Unless,
    human_name: String,
    /// The turn and step the "until" checks compare against: the anchor when
    /// the request gave one, else where the run started.
    start_turn: u32,
    start_step: String,
    /// Attackers already known to be coming at this seat, by object id, so the
    /// same attack does not stop a run twice.
    seen_attackers: BTreeSet<u64>,
}

impl PassRule {
    /// `start` is this seat's view **before the first pass**.
    pub fn new(request: &PassUntilRequest, start: &StateViewModel, human_name: &str) -> Self {
        let facts = TurnFacts::of(start);
        let (start_turn, start_step) = match &request.anchor {
            Some(anchor) => (anchor.turn, anchor.step.clone()),
            None => (facts.number, facts.step),
        };
        PassRule {
            until: request.until,
            unless: request.unless,
            human_name: human_name.to_string(),
            start_turn,
            start_step,
            seen_attackers: attackers_at(start, human_name)
                .into_iter()
                .map(|(id, _)| id)
                .collect(),
        }
    }

    /// Should the run stop, given this seat's view after a pass and the event
    /// lines that pass produced?
    ///
    /// "Until" is checked first: when the run has arrived it is over, and an
    /// "unless" that fired on the same pass is appended to the reason rather
    /// than turning an arrival into a pause. The attack bookkeeping runs whether
    /// or not `unless.attackers` is on — in the client so that switching the
    /// box on mid-run does not fire for an old attack; here it is kept only so
    /// the two stay step-for-step identical.
    pub fn check(
        &mut self,
        view: &StateViewModel,
        events: &[EventView],
    ) -> Option<(StopKind, String)> {
        let attack = self.new_attackers(view);
        let unless = self
            .unless
            .spell
            .then(|| spell_cast_by_another(events, &self.human_name))
            .flatten()
            .or_else(|| {
                self.unless
                    .ability
                    .then(|| ability_activated_by_another(events, &self.human_name))
                    .flatten()
            })
            .or_else(|| self.unless.attackers.then_some(attack).flatten());

        match (self.until_reached(view), unless) {
            (Some(until), Some(unless)) => Some((StopKind::Until, format!("{until}; {unless}"))),
            (Some(until), None) => Some((StopKind::Until, until)),
            (None, Some(unless)) => Some((StopKind::Unless, unless)),
            (None, None) => None,
        }
    }

    /// `autopass.js::untilReached`. All three compare against where the run
    /// **started**, never against the previous look.
    fn until_reached(&self, view: &StateViewModel) -> Option<String> {
        let now = TurnFacts::of(view);
        let whose = if now.active == self.human_name {
            "your".to_string()
        } else {
            format!("{}'s", now.active)
        };
        match self.until {
            Until::EndOfTurn => (now.number != self.start_turn)
                .then(|| format!("the turn ended (turn {} is {whose})", now.number)),
            Until::MyNextTurn => (now.active == self.human_name && now.number != self.start_turn)
                .then(|| format!("your turn {} began", now.number)),
            Until::Step { step } => {
                let at = now.step == step.wire_name();
                let same = now.number == self.start_turn && now.step == self.start_step;
                (at && !same).then(|| {
                    format!(
                        "reached {whose} {} (turn {})",
                        step_label(&now.step),
                        now.number
                    )
                })
            }
        }
    }

    /// `autopass.js::newAttackersAtMe`: attackers declared at this seat since
    /// the previous look. Replaces the seen set with the current one.
    fn new_attackers(&mut self, view: &StateViewModel) -> Option<String> {
        let now = attackers_at(view, &self.human_name);
        let fresh: Vec<&str> = now
            .iter()
            .filter(|(id, _)| !self.seen_attackers.contains(id))
            .map(|(_, name)| {
                if name.is_empty() {
                    "a creature"
                } else {
                    name.as_str()
                }
            })
            .collect();
        let reason = (!fresh.is_empty()).then(|| {
            let who = view
                .combat
                .as_ref()
                .map(|c| c.attacking_player.as_str())
                .filter(|who| !who.is_empty())
                .map(|who| format!("{who}: "))
                .unwrap_or_default();
            format!("attackers are declared at you ({who}{})", fresh.join(", "))
        });
        self.seen_attackers = now.iter().map(|(id, _)| *id).collect();
        reason
    }
}

/// `autopass.js::spellCastBy`: the first `SpellCast` line not attributed to
/// this seat, as its own sentence.
fn spell_cast_by_another(events: &[EventView], human_name: &str) -> Option<String> {
    events
        .iter()
        .find(|ev| ev.kind == "SpellCast" && ev.player.as_deref() != Some(human_name))
        .map(|ev| line_or(ev, "a spell was cast"))
}

/// `autopass.js::abilityActivatedBy`. A line with no `player` at all cannot be
/// attributed and stops the run: stopping is the safe direction.
fn ability_activated_by_another(events: &[EventView], human_name: &str) -> Option<String> {
    events
        .iter()
        .find(|ev| {
            ACTIVATION_KINDS.contains(&ev.kind.as_str()) && ev.player.as_deref() != Some(human_name)
        })
        .map(|ev| line_or(ev, "an ability was activated"))
}

fn line_or(ev: &EventView, fallback: &str) -> String {
    if ev.text.is_empty() {
        fallback.to_string()
    } else {
        ev.text.clone()
    }
}

/// `autopass.js::attackersAtMe`: `(object_id, name)` of every attacker whose
/// target is `player:<human>`, or `planeswalker:<name>` naming a planeswalker on
/// this seat's battlefield. The view spells a planeswalker target by name only
/// (`AttackerView::target`'s doc), so a same-named planeswalker elsewhere also
/// matches — the cautious reading, and the client's.
fn attackers_at(view: &StateViewModel, human_name: &str) -> Vec<(u64, String)> {
    let Some(combat) = view.combat.as_ref() else {
        return Vec::new();
    };
    let walkers: BTreeSet<&str> = view
        .zones
        .battlefield
        .get(human_name)
        .into_iter()
        .flatten()
        .filter(|p| p.card_types.iter().any(|t| t == "Planeswalker"))
        .map(|p| p.name.as_str())
        .collect();
    let at_player = format!("player:{human_name}");
    combat
        .attackers
        .iter()
        .filter(|a| {
            a.target == at_player
                || a.target
                    .strip_prefix("planeswalker:")
                    .is_some_and(|name| walkers.contains(name))
        })
        .map(|a| (a.object_id, a.name.clone()))
        .collect()
}

/// `names.js::stepName`, for the "reached … " sentence. An unlisted step falls
/// back to its wire spelling rather than to nothing.
fn step_label(step: &str) -> &str {
    match step {
        "Upkeep" => "Upkeep",
        "Draw" => "Draw",
        "PreCombatMain" => "Main 1",
        "BeginningOfCombat" => "Beginning of combat",
        "DeclareAttackers" => "Declare attackers",
        "DeclareBlockers" => "Declare blockers",
        "FirstStrikeDamage" => "First-strike damage",
        "CombatDamage" => "Combat damage",
        "EndOfCombat" => "End of combat",
        "PostCombatMain" => "Main 2",
        "End" => "End step",
        other => other,
    }
}

/// `names.js::decisionKindName`, for the "you have a … decision to make"
/// sentence. Takes the wire tag (`view::decision_kind_tag`).
pub fn decision_label(kind_tag: &str) -> &str {
    match kind_tag {
        "Priority" => "Priority",
        "Mulligan" => "Mulligan",
        "CommanderZoneChoice" => "Commander zone",
        "DeclareAttackers" => "Declare attackers",
        "DeclareBlockers" => "Declare blockers",
        "CleanupDiscard" => "Cleanup discard",
        "TriggerTargets" => "Trigger targets",
        "EffectChoice" => "Effect choice",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use mtg_engine::{
        AttackTarget, CombatState, GameState, GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
    };
    use mtg_view_model::{EventTier, Viewer};

    const ME: &str = "Human-1";

    fn names() -> HashMap<PlayerId, String> {
        [
            (PlayerId(1), ME.to_string()),
            (PlayerId(2), "Bot-2".to_string()),
        ]
        .into_iter()
        .collect()
    }

    /// A two-seat table at `(turn, active, step)` with one creature each and a
    /// planeswalker on seat 1's side.
    fn table(turn: u32, active: u64, step: Step) -> GameState {
        GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .object(ObjectSpec::creature(PlayerId(2), "Hill Giant", 3, 3))
            .object(ObjectSpec::creature(PlayerId(2), "Gray Ogre", 2, 2))
            .object(ObjectSpec::planeswalker(PlayerId(1), "Jace Beleren", 3))
            .turn_number(turn)
            .active_player(PlayerId(active))
            .at_step(step)
            .build()
            .expect("the table builds")
    }

    /// This seat's view — the only thing the rule is ever shown.
    fn seat(state: &GameState) -> StateViewModel {
        StateViewModel::from_game_state_for(state, &names(), Viewer::Seat(PlayerId(1)))
    }

    fn id_of(state: &GameState, name: &str) -> mtg_engine::ObjectId {
        state
            .objects()
            .values()
            .find(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
            .expect("the fixture has it")
            .id
    }

    fn attack(state: &mut GameState, attacker: &str, target: AttackTarget) {
        let id = id_of(state, attacker);
        let combat = state
            .combat_mut()
            .get_or_insert_with(|| CombatState::new(PlayerId(2)));
        combat.attackers.insert(id, target);
    }

    fn line(kind: &str, text: &str, player: Option<&str>) -> EventView {
        EventView {
            kind: kind.to_string(),
            text: text.to_string(),
            player: player.map(str::to_string),
            tier: EventTier::Game,
        }
    }

    fn request(json: serde_json::Value) -> PassUntilRequest {
        serde_json::from_value(json).expect("a valid pass_until")
    }

    /// The three `until` spellings, and the two things `deny_unknown_fields`
    /// and the `StopStep` enum are there to refuse.
    #[test]
    fn test_pass_until_request_shape() {
        let fresh = request(serde_json::json!({ "until": { "kind": "end_of_turn" } }));
        assert_eq!(fresh.until, Until::EndOfTurn);
        assert_eq!(
            fresh.unless,
            Unless::default(),
            "an omitted unless stops on nothing"
        );
        assert!(fresh.anchor.is_none());

        let full = request(serde_json::json!({
            "until": { "kind": "step", "step": "PostCombatMain" },
            "unless": { "spell": true, "attackers": true },
            "anchor": { "turn": 7, "step": "Upkeep" },
        }));
        assert_eq!(
            full.until,
            Until::Step {
                step: StopStep::PostCombatMain
            }
        );
        assert_eq!(
            full.unless,
            Unless {
                spell: true,
                ability: false,
                attackers: true
            }
        );
        assert_eq!(
            full.anchor,
            Some(Anchor {
                turn: 7,
                step: "Upkeep".to_string()
            })
        );
        assert_eq!(
            request(serde_json::json!({ "until": { "kind": "my_next_turn" } })).until,
            Until::MyNextTurn
        );

        for bad in [
            // CR 502.4 / CR 514.3: nobody gets priority there.
            serde_json::json!({ "until": { "kind": "step", "step": "Untap" } }),
            serde_json::json!({ "until": { "kind": "step", "step": "Cleanup" } }),
            serde_json::json!({ "until": { "kind": "step" } }),
            serde_json::json!({ "until": { "kind": "forever" } }),
            serde_json::json!({ "until": { "kind": "end_of_turn" }, "unless": { "combat": true } }),
            serde_json::json!({ "until": { "kind": "end_of_turn" }, "stop": "now" }),
            serde_json::json!({}),
        ] {
            assert!(
                serde_json::from_value::<PassUntilRequest>(bad.clone()).is_err(),
                "{bad} must be refused"
            );
        }

        // Every stoppable step's wire name is the engine's own spelling.
        for (stop, step) in [
            (StopStep::Upkeep, Step::Upkeep),
            (StopStep::Draw, Step::Draw),
            (StopStep::PreCombatMain, Step::PreCombatMain),
            (StopStep::BeginningOfCombat, Step::BeginningOfCombat),
            (StopStep::DeclareAttackers, Step::DeclareAttackers),
            (StopStep::DeclareBlockers, Step::DeclareBlockers),
            (StopStep::FirstStrikeDamage, Step::FirstStrikeDamage),
            (StopStep::CombatDamage, Step::CombatDamage),
            (StopStep::EndOfCombat, Step::EndOfCombat),
            (StopStep::PostCombatMain, Step::PostCombatMain),
            (StopStep::End, Step::End),
        ] {
            assert_eq!(stop.wire_name(), format!("{step:?}"));
        }
    }

    #[test]
    fn test_outcome_serializes_as_documented() {
        let outcome = PassUntilOutcome {
            passes: 4,
            stop: StopKind::NoDecision,
            reason: "there is nothing to answer".to_string(),
        };
        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            serde_json::json!({
                "passes": 4,
                "stop": "no_decision",
                "reason": "there is nothing to answer",
            })
        );
    }

    /// `untilReached`: each mode compares against where the run STARTED.
    #[test]
    fn test_until_modes_compare_against_the_start() {
        let start = seat(&table(5, 1, Step::PreCombatMain));

        // End of turn: same turn -> go on; next turn -> stop, saying whose.
        let mut eot = PassRule::new(
            &request(serde_json::json!({ "until": { "kind": "end_of_turn" } })),
            &start,
            ME,
        );
        assert_eq!(eot.check(&seat(&table(5, 1, Step::End)), &[]), None);
        assert_eq!(
            eot.check(&seat(&table(6, 2, Step::Upkeep)), &[]),
            Some((
                StopKind::Until,
                "the turn ended (turn 6 is Bot-2's)".to_string()
            ))
        );

        // My next turn, started ON my turn: another seat's turn is not it, and
        // neither is the rest of this one.
        let mut mine = PassRule::new(
            &request(serde_json::json!({ "until": { "kind": "my_next_turn" } })),
            &start,
            ME,
        );
        assert_eq!(mine.check(&seat(&table(5, 1, Step::End)), &[]), None);
        assert_eq!(mine.check(&seat(&table(6, 2, Step::Upkeep)), &[]), None);
        assert_eq!(
            mine.check(&seat(&table(7, 1, Step::Upkeep)), &[]),
            Some((StopKind::Until, "your turn 7 began".to_string()))
        );

        // A step: not the very (turn, step) the run started in, but any
        // player's next occurrence of it.
        let mut step = PassRule::new(
            &request(serde_json::json!({
                "until": { "kind": "step", "step": "PreCombatMain" }
            })),
            &start,
            ME,
        );
        assert_eq!(
            step.check(&seat(&table(5, 1, Step::PreCombatMain)), &[]),
            None
        );
        assert_eq!(step.check(&seat(&table(5, 1, Step::End)), &[]), None);
        assert_eq!(
            step.check(&seat(&table(6, 2, Step::PreCombatMain)), &[]),
            Some((
                StopKind::Until,
                "reached Bot-2's Main 1 (turn 6)".to_string()
            ))
        );
    }

    /// A resumed run measures from the ANCHOR, not from where Resume was pressed.
    #[test]
    fn test_anchor_replaces_the_start_turn() {
        // Resumed on turn 6; the run was started on turn 5.
        let resumed_at = seat(&table(6, 2, Step::PreCombatMain));
        let still_turn_six = seat(&table(6, 2, Step::End));

        let mut fresh = PassRule::new(
            &request(serde_json::json!({ "until": { "kind": "end_of_turn" } })),
            &resumed_at,
            ME,
        );
        assert_eq!(fresh.check(&still_turn_six, &[]), None);

        let mut anchored = PassRule::new(
            &request(serde_json::json!({
                "until": { "kind": "end_of_turn" },
                "anchor": { "turn": 5, "step": "PreCombatMain" },
            })),
            &resumed_at,
            ME,
        );
        assert_eq!(
            anchored.check(&still_turn_six, &[]),
            Some((
                StopKind::Until,
                "the turn ended (turn 6 is Bot-2's)".to_string()
            ))
        );
    }

    /// `spellCastBy` / `abilityActivatedBy`: another player's, only when the box
    /// is on, and never a mana ability or a trigger.
    #[test]
    fn test_unless_spell_and_ability_read_the_event_lines() {
        let start = seat(&table(5, 2, Step::PreCombatMain));
        let later = seat(&table(5, 2, Step::End));
        let cast = line("SpellCast", "Bot-2 casts Shock", Some("Bot-2"));
        let my_cast = line("SpellCast", "Human-1 casts Shock", Some(ME));
        let activated = line(
            "AbilityActivated",
            "Bot-2 activates Prodigal Pyromancer",
            Some("Bot-2"),
        );
        let craft = line("CraftActivated", "CraftActivated", None);
        let noise = [
            line("ManaAdded", "Bot-2 adds {R}", Some("Bot-2")),
            line("AbilityTriggered", "Soul Warden triggers", Some("Bot-2")),
            line("PermanentTapped", "Mountain tapped", None),
        ];
        let rule = |unless: serde_json::Value| {
            PassRule::new(
                &request(serde_json::json!({
                    "until": { "kind": "my_next_turn" },
                    "unless": unless,
                })),
                &start,
                ME,
            )
        };

        // Off by default: nothing stops it.
        let everything = [cast.clone(), activated.clone(), craft.clone()];
        assert_eq!(rule(serde_json::json!({})).check(&later, &everything), None);

        let mut spell = rule(serde_json::json!({ "spell": true }));
        assert_eq!(spell.check(&later, &noise), None);
        assert_eq!(spell.check(&later, std::slice::from_ref(&my_cast)), None);
        assert_eq!(spell.check(&later, std::slice::from_ref(&activated)), None);
        assert_eq!(
            spell.check(&later, std::slice::from_ref(&cast)),
            Some((StopKind::Unless, "Bot-2 casts Shock".to_string()))
        );

        let mut ability = rule(serde_json::json!({ "ability": true }));
        assert_eq!(ability.check(&later, &noise), None);
        assert_eq!(ability.check(&later, std::slice::from_ref(&cast)), None);
        assert_eq!(
            ability.check(&later, std::slice::from_ref(&activated)),
            Some((
                StopKind::Unless,
                "Bot-2 activates Prodigal Pyromancer".to_string()
            ))
        );
        // A line nobody can vouch for stops the run.
        assert_eq!(
            ability.check(&later, std::slice::from_ref(&craft)),
            Some((StopKind::Unless, "CraftActivated".to_string()))
        );

        // Arrival wins, and the interruption is appended to the reason.
        let mut both = rule(serde_json::json!({ "spell": true }));
        assert_eq!(
            both.check(
                &seat(&table(7, 1, Step::Upkeep)),
                std::slice::from_ref(&cast)
            ),
            Some((
                StopKind::Until,
                "your turn 7 began; Bot-2 casts Shock".to_string()
            ))
        );
    }

    /// `newAttackersAtMe`: an attack this seat has already seen does not stop a
    /// run, a new one does once, and an attack at somebody else never does.
    #[test]
    fn test_unless_attackers_fires_once_per_new_attacker_at_this_seat() {
        let mut state = table(6, 2, Step::DeclareAttackers);
        attack(&mut state, "Hill Giant", AttackTarget::Player(PlayerId(1)));
        let giant_only = seat(&state);
        let walker = id_of(&state, "Jace Beleren");
        attack(&mut state, "Gray Ogre", AttackTarget::Planeswalker(walker));
        let giant_and_ogre = seat(&state);

        let mut elsewhere = table(6, 2, Step::DeclareAttackers);
        attack(
            &mut elsewhere,
            "Hill Giant",
            AttackTarget::Player(PlayerId(2)),
        );
        let at_someone_else = seat(&elsewhere);

        let spec = request(serde_json::json!({
            "until": { "kind": "my_next_turn" },
            "unless": { "attackers": true },
        }));

        // Started before any attack: the first attacker stops it, by name.
        let quiet = seat(&table(6, 2, Step::BeginningOfCombat));
        let mut rule = PassRule::new(&spec, &quiet, ME);
        assert_eq!(rule.check(&at_someone_else, &[]), None);
        assert_eq!(
            rule.check(&giant_only, &[]),
            Some((
                StopKind::Unless,
                "attackers are declared at you (Bot-2: Hill Giant)".to_string()
            ))
        );
        // The same attack, looked at again, is not news.
        assert_eq!(rule.check(&giant_only, &[]), None);
        // A second attacker, at this seat's planeswalker, is.
        assert_eq!(
            rule.check(&giant_and_ogre, &[]),
            Some((
                StopKind::Unless,
                "attackers are declared at you (Bot-2: Gray Ogre)".to_string()
            ))
        );

        // Started (or resumed) with the giant already attacking: not news either.
        let mut resumed = PassRule::new(&spec, &giant_only, ME);
        assert_eq!(resumed.check(&giant_only, &[]), None);

        // The box off: tracked, never reported.
        let off = request(serde_json::json!({ "until": { "kind": "my_next_turn" } }));
        let mut silent = PassRule::new(&off, &quiet, ME);
        assert_eq!(silent.check(&giant_and_ogre, &[]), None);
    }
}

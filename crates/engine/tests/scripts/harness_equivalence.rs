//! SR-9b — harness-vs-direct-dispatch equivalence.
//!
//! The engine has two ways to be driven, and until this file they never met.
//!
//! * The **script regime**: a JSON [`GameScript`] is deserialized, its
//!   `initial_state` block is turned into a `GameState` by
//!   [`build_initial_state`], and each action string is turned into a `Command`
//!   by [`translate_player_action`]. ~271 golden scripts run this way.
//! * The **direct regime**: a hand-written integration test assembles a
//!   `GameState` with [`GameStateBuilder`] and feeds `Command` literals to
//!   [`process_command`]. ~3000 tests run this way.
//!
//! `replay_harness.rs` is ~3,700 lines of translation sitting between the first
//! regime and the engine. `translate_player_action` alone builds 60+ distinct
//! `Command` shapes, and `build_initial_state` reconstructs a board from a JSON
//! description. Nothing checked that the two regimes describe the same game.
//! A `Command` field that `translate_player_action` forgets to populate makes
//! every script that exercises it green while testing something other than what
//! it says it tests.
//!
//! This file drives the *same scenario* both ways and requires the same state
//! after every single step.
//!
//! # The rule when they disagree
//!
//! Per `docs/sr-remediation-plan.md` gotcha SR-9(b): **the harness is wrong
//! until proven otherwise, not the engine.** The harness shadow-implements
//! object construction; the engine is what 3,000 direct tests already pin.
//! Every divergence this file found was in fact a harness bug — see the
//! "Divergences found" ledger below.
//!
//! # What "same state" means here
//!
//! [`GameState::public_state_hash`] deliberately omits hand and library
//! *contents* (hidden information). A harness bug that shuffled a library or
//! swapped two cards in hand would be invisible to it. So the comparison here
//! is a [`Fingerprint`]: the public hash **plus** every player's
//! `private_state_hash`. That is a strict superset of the acceptance
//! criterion's "final state hash", and it is taken after *every* step, not only
//! the last one, so a divergence is reported at the step that caused it rather
//! than at the end.
//!
//! # Divergences found (all fixed on the harness side)
//!
//! 1. **`build_initial_state` was not deterministic.** `InitialState`'s zone and
//!    player maps are `std::collections::HashMap`, whose iteration order is
//!    seeded per instance. Objects are assigned `ObjectId`s in insertion order,
//!    so two builds of the *same* script in the *same* process handed out
//!    different `ObjectId`s and produced different hashes. Measured: 40 builds
//!    of one two-player script yielded 2 distinct hashes. Fixed by
//!    `sorted_zone_entries` in `replay_harness.rs`; pinned by
//!    [`build_initial_state_is_deterministic`].
//! 2. **`initial_state.turn_number` was declared and never read.** Every script
//!    ran on turn 1 no matter what it claimed. `turn.turn_number` is hashed, and
//!    `entered_turn` plus every "this turn" comparison reads it, so a script
//!    that set up a turn-5 board was playing a different game than it described.
//!    Fixed by threading it into `GameStateBuilder::turn_number`.
//! 3. **A script may name a card that has no `CardDefinition`.**
//!    `enrich_spec_from_def` returns the bare `ObjectSpec` unchanged when the
//!    name is unknown, so the object enters the game with no card types, no
//!    mana cost, no power/toughness and no abilities — and nothing says a word.
//!    Architecture invariant #9 says exactly this must not happen. Found by
//!    `scenarios_are_not_vacuous`: the `equip` scenario had been quietly
//!    "passing" because both regimes rejected the equip identically, Grizzly
//!    Bears having no definition and therefore not being a creature. Authoring
//!    the card is card-authoring work, so the hole is pinned as a shrinking
//!    allowlist by [`scripts_only_name_cards_that_have_definitions`] and handed
//!    to SR-9c. One approved script is affected today.
//!
//! # Still-unread `initial_state` fields (documented, not fixed)
//!
//! These are declared by `script_schema.rs` and ignored by `build_initial_state`.
//! Each one is a way for a script to describe a board the harness will not build.
//! They are *not* reachable from this file's equivalence check, because the
//! direct regime cannot express them either — closing them is script-corpus
//! triage, which is SR-9c (`scutemob-71`):
//!
//! | Field | Consequence |
//! |---|---|
//! | `initial_state.priority` | priority always starts with the active player |
//! | `initial_state.step` | only `phase` is parsed; `step` is dropped |
//! | `initial_state.continuous_effects` | no continuous effect is ever installed |
//! | `zones.command_zone` | never populated — `find_in_command_zone` can never hit |
//! | `PermanentInitState.summoning_sick` | battlefield creatures are never sick |
//! | `PermanentInitState.attached` | auras/equipment start unattached |
//! | `PlayerInitState.commander_damage_received` | always zero |
//!
//! Also: `parse_step` has no arm for `"combat"`, which several scripts use as
//! their `phase`; it falls through to the default. And `replay_script` silently
//! skips any `ScriptAction` that `translate_player_action` cannot translate.
//!
//! # Demonstrated, not asserted
//!
//! Six perturbations were applied to `replay_harness.rs` and to the corpus, each
//! verified to have actually changed the file before the suite was run (SR-9a's
//! lesson: an attack that changes nothing "passes" every gate). Which tests fire:
//!
//! | Attack | Caught by |
//! |---|---|
//! | revert `sorted_zone_entries` on the battlefield loop | `build_initial_state_is_deterministic`, all three `equivalence_*` |
//! | drop the `turn_number()` call | `initial_state_turn_number_is_honored`, all three `equivalence_*` |
//! | `tap_for_mana` translates to `ability_index: 1` | `equivalence_bolt`, `scenarios_are_not_vacuous` |
//! | `pass_priority` always passes for `PlayerId(1)` | all three `equivalence_*`, `scenarios_are_not_vacuous` |
//! | `play_land` falls back to the battlefield | **only** the property test |
//! | an approved script names an undefined card | `scripts_only_name_cards_that_have_definitions` |
//!
//! Two of those rows carry the file's real argument. The `play_land` fallback is
//! invisible to every fixed scenario — it needs the *sequence* `[PlayLand Forest,
//! PlayLand Forest]` before the harness and a direct test disagree, and that is
//! what the property test generates. And `equivalence_equip` survives the
//! `sorted_zone_entries` revert, because only one player has permanents in it, so
//! map order cannot matter: a scenario proves nothing about the bug it cannot
//! express.

use std::collections::HashMap;

use mtg_engine::state::combat::AttackTarget;
use mtg_engine::testing::replay_harness::{build_initial_state, enrich_spec_from_def};
use mtg_engine::testing::script_schema::{ActionTarget, AttackerDeclaration, InitialState};
use mtg_engine::{
    all_cards, card_name_to_id, process_command, CardDefinition, CardRegistry, Command, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, Target, ZoneId,
};

// ── Fingerprint ───────────────────────────────────────────────────────────────

/// The public state hash plus every player's private state hash.
///
/// Stronger than `public_state_hash` alone, which omits hand and library
/// contents — a harness that put the right number of cards in the wrong order
/// would pass a public-hash-only comparison.
#[derive(PartialEq, Eq)]
struct Fingerprint {
    public: [u8; 32],
    private: Vec<(PlayerId, [u8; 32])>,
}

impl std::fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pub:{}", short(&self.public))?;
        for (pid, h) in &self.private {
            write!(f, " p{}:{}", pid.0, short(h))?;
        }
        Ok(())
    }
}

fn short(hash: &[u8; 32]) -> String {
    hash[..4].iter().map(|b| format!("{b:02x}")).collect()
}

fn fingerprint(state: &GameState) -> Fingerprint {
    let mut private: Vec<(PlayerId, [u8; 32])> = state
        .players()
        .keys()
        .map(|&pid| (pid, state.private_state_hash(pid)))
        .collect();
    private.sort_by_key(|(pid, _)| pid.0);
    Fingerprint {
        public: state.public_state_hash(),
        private,
    }
}

// ── Scenario description ──────────────────────────────────────────────────────

/// One scenario, expressed twice.
///
/// `script_json` is what the script regime consumes. `direct` is what a
/// hand-written integration test would have typed to set up the same board.
/// The two must agree before a single command is dispatched — that is the
/// `initial_state` half of the equivalence.
struct Scenario {
    name: &'static str,
    script_json: &'static str,
    direct: fn(&HashMap<String, CardDefinition>) -> GameState,
}

/// A player action, expressed once and rendered into both regimes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Move {
    Pass(&'static str),
    PlayLand {
        player: &'static str,
        card: &'static str,
    },
    TapForMana {
        player: &'static str,
        land: &'static str,
    },
    CastSpell {
        player: &'static str,
        card: &'static str,
        targets: &'static [Tgt],
    },
    ActivateAbility {
        player: &'static str,
        source: &'static str,
        index: usize,
        targets: &'static [Tgt],
    },
    DeclareAttackers {
        player: &'static str,
        /// (attacking creature name, defending player name)
        attackers: &'static [(&'static str, &'static str)],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tgt {
    Player(&'static str),
    Permanent(&'static str),
}

// ── Rendering a Move into the script regime ───────────────────────────────────

impl Move {
    /// The `action` string a JSON script would carry for this move.
    fn action_str(&self) -> &'static str {
        match self {
            Move::Pass(_) => "pass_priority",
            Move::PlayLand { .. } => "play_land",
            Move::TapForMana { .. } => "tap_for_mana",
            Move::CastSpell { .. } => "cast_spell",
            Move::ActivateAbility { .. } => "activate_ability",
            Move::DeclareAttackers { .. } => "declare_attackers",
        }
    }

    fn actor(&self) -> &'static str {
        match self {
            Move::Pass(p)
            | Move::PlayLand { player: p, .. }
            | Move::TapForMana { player: p, .. }
            | Move::CastSpell { player: p, .. }
            | Move::ActivateAbility { player: p, .. }
            | Move::DeclareAttackers { player: p, .. } => p,
        }
    }

    /// Build the `Command` the way a *script* does: through
    /// `translate_player_action`, the 3,700-line translation layer under test.
    fn harness_command(
        &self,
        state: &GameState,
        players: &HashMap<String, PlayerId>,
    ) -> Option<Command> {
        let card: Option<&str> = match self {
            Move::PlayLand { card, .. } | Move::CastSpell { card, .. } => Some(card),
            Move::TapForMana { land, .. } => Some(land),
            Move::ActivateAbility { source, .. } => Some(source),
            Move::Pass(_) | Move::DeclareAttackers { .. } => None,
        };
        let ability_index = match self {
            Move::ActivateAbility { index, .. } => *index,
            _ => 0,
        };
        let targets: Vec<ActionTarget> = match self {
            Move::CastSpell { targets, .. } | Move::ActivateAbility { targets, .. } => {
                targets.iter().map(to_action_target).collect()
            }
            _ => vec![],
        };
        let attackers: Vec<AttackerDeclaration> = match self {
            Move::DeclareAttackers { attackers, .. } => attackers
                .iter()
                .map(|(creature, defender)| AttackerDeclaration {
                    card: (*creature).to_string(),
                    target_player: Some((*defender).to_string()),
                    target_planeswalker: None,
                })
                .collect(),
            _ => vec![],
        };
        translate(
            self.action_str(),
            players[self.actor()],
            card,
            ability_index,
            &targets,
            &attackers,
            state,
            players,
        )
    }

    /// Build the `Command` the way a *hand-written test* does: a literal, with
    /// ObjectIds resolved by the obvious lookup. This is the direct regime's
    /// half of the comparison and must not call into `replay_harness`.
    fn direct_command(
        &self,
        state: &GameState,
        players: &HashMap<String, PlayerId>,
    ) -> Option<Command> {
        let player = players[self.actor()];
        match self {
            Move::Pass(_) => Some(Command::PassPriority { player }),
            Move::PlayLand { card, .. } => Some(Command::PlayLand {
                player,
                card: in_hand(state, player, card)?,
            }),
            Move::TapForMana { land, .. } => Some(Command::TapForMana {
                player,
                source: on_battlefield(state, player, land)?,
                ability_index: 0,
            }),
            Move::CastSpell { card, targets, .. } => Some(Command::CastSpell {
                player,
                card: in_hand(state, player, card)?,
                targets: resolve(targets, state, players)?,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
                face_down_kind: None,
                additional_costs: vec![],
            }),
            Move::ActivateAbility {
                source,
                index,
                targets,
                ..
            } => Some(Command::ActivateAbility {
                player,
                source: on_battlefield(state, player, source)?,
                ability_index: *index,
                targets: resolve(targets, state, players)?,
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            }),
            Move::DeclareAttackers { attackers, .. } => {
                let mut pairs = Vec::new();
                for (creature, defender) in *attackers {
                    pairs.push((
                        on_battlefield(state, player, creature)?,
                        AttackTarget::Player(*players.get(*defender)?),
                    ));
                }
                Some(Command::DeclareAttackers {
                    player,
                    attackers: pairs,
                    enlist_choices: vec![],
                    exert_choices: vec![],
                })
            }
        }
    }
}

fn to_action_target(t: &Tgt) -> ActionTarget {
    match t {
        Tgt::Player(name) => ActionTarget {
            target_type: "player".to_string(),
            card: None,
            controller: None,
            player: Some((*name).to_string()),
        },
        Tgt::Permanent(name) => ActionTarget {
            target_type: "permanent".to_string(),
            card: Some((*name).to_string()),
            controller: None,
            player: None,
        },
    }
}

/// `translate_player_action` with the 28 arguments this file never uses filled
/// in. Kept as one call site so a signature change fails here loudly rather
/// than silently shifting a positional argument.
#[allow(clippy::too_many_arguments)]
fn translate(
    action: &str,
    player: PlayerId,
    card: Option<&str>,
    ability_index: usize,
    targets: &[ActionTarget],
    attackers: &[AttackerDeclaration],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Command> {
    mtg_engine::testing::replay_harness::translate_player_action(
        action,
        player,
        card,
        ability_index,
        targets,
        attackers,
        &[], // blockers
        &[], // convoke
        &[], // improvise
        &[], // delve
        &[], // escape
        false,
        false,
        &[],  // enlist
        None, // attacker_name
        None, // discard_land
        None, // discard_card
        None, // bargain_sacrifice
        None, // emerge_sacrifice
        None, // casualty_sacrifice
        None, // assist_player
        0,    // assist_amount
        0,    // replicate_count
        &[],  // splice
        0,    // escalate_modes
        vec![],
        None,  // target_creature
        0,     // x_value
        &[],   // collect_evidence
        0,     // squad_count
        false, // mutate_on_top
        None,  // gift_opponent
        None,  // sacrifice_card
        &[],   // exert
        None,  // pitch_exile_card
        state,
        players,
    )
}

// ── Direct-regime lookups (independent of replay_harness's private finders) ───

fn in_hand(state: &GameState, player: PlayerId, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(player))
        .map(|(&id, _)| id)
}

fn on_battlefield(state: &GameState, controller: PlayerId, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == name
                && obj.zone == ZoneId::Battlefield
                && obj.controller == controller
        })
        .map(|(&id, _)| id)
}

fn any_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(&id, _)| id)
}

/// `None` if *any* target fails to resolve — mirrors `translate_player_action`'s
/// behaviour of producing a `Command` only when every name maps to something.
///
/// Note the asymmetry this test pins: `resolve_targets` in the harness uses
/// `filter_map`, so an unresolvable target is silently *dropped* rather than
/// failing the action. Here an unresolvable target aborts the move. The
/// scenarios below never exercise that difference; if a future scenario does,
/// the equivalence check will fail and the harness's `filter_map` is the bug.
fn resolve(
    targets: &[Tgt],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Vec<Target>> {
    targets
        .iter()
        .map(|t| match t {
            Tgt::Player(name) => players.get(*name).copied().map(Target::Player),
            Tgt::Permanent(name) => any_battlefield(state, name).map(Target::Object),
        })
        .collect()
}

// ── Driving one scenario through both regimes ─────────────────────────────────

/// What happened at one step, in one regime.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The move produced no `Command` at all (a name did not resolve).
    Untranslatable,
    /// The engine accepted the command; here is the resulting state.
    Accepted(Fingerprint),
    /// The engine rejected the command. The `Debug` string of the error.
    Rejected(String),
}

/// One regime's whole trace: the state before any move, then one entry per move.
struct Trace {
    initial: Fingerprint,
    steps: Vec<(Option<Command>, Outcome)>,
}

fn drive(
    mut state: GameState,
    players: &HashMap<String, PlayerId>,
    moves: &[Move],
    build: impl Fn(&Move, &GameState, &HashMap<String, PlayerId>) -> Option<Command>,
) -> Trace {
    let initial = fingerprint(&state);
    let mut steps = Vec::new();
    for m in moves {
        let Some(cmd) = build(m, &state, players) else {
            steps.push((None, Outcome::Untranslatable));
            continue;
        };
        match process_command(state.clone(), cmd.clone()) {
            Ok((next, _events)) => {
                state = next;
                steps.push((Some(cmd), Outcome::Accepted(fingerprint(&state))));
            }
            Err(e) => {
                // State is unchanged (we cloned), so the trace can continue —
                // both regimes see the same rejection and carry on from the
                // same state, which keeps the comparison meaningful past the
                // first illegal move.
                steps.push((Some(cmd), Outcome::Rejected(format!("{e:?}"))));
            }
        }
    }
    Trace { initial, steps }
}

/// The whole point of the file: run `moves` through both regimes and require
/// that nothing about the game differs at any step.
fn assert_equivalent(scenario: &Scenario, moves: &[Move]) {
    let defs = defs();

    let init: InitialState = serde_json::from_str(scenario.script_json)
        .unwrap_or_else(|e| panic!("scenario `{}` has invalid JSON: {e}", scenario.name));
    let (harness_state, players) = build_initial_state(&init);
    let direct_state = (scenario.direct)(&defs);

    let harness = drive(harness_state, &players, moves, |m, s, p| {
        m.harness_command(s, p)
    });
    let direct = drive(direct_state, &players, moves, |m, s, p| {
        m.direct_command(s, p)
    });

    assert_eq!(
        harness.initial, direct.initial,
        "scenario `{}`: build_initial_state and the hand-written GameStateBuilder \
         produced different initial states.\n  harness: {:?}\n  direct:  {:?}\n\
         Per SR-9(b): the harness is wrong until proven otherwise.",
        scenario.name, harness.initial, direct.initial
    );

    assert_eq!(
        harness.steps.len(),
        direct.steps.len(),
        "scenario `{}`: trace lengths differ",
        scenario.name
    );

    for (i, ((h_cmd, h_out), (d_cmd, d_out))) in
        harness.steps.iter().zip(direct.steps.iter()).enumerate()
    {
        assert_eq!(
            h_cmd, d_cmd,
            "scenario `{}` step {i} ({:?}): translate_player_action built a different \
             Command than a hand-written test would.\n  harness: {h_cmd:#?}\n  direct:  {d_cmd:#?}",
            scenario.name, moves[i]
        );
        assert_eq!(
            h_out, d_out,
            "scenario `{}` step {i} ({:?}): same Command, different outcome.\n  \
             harness: {h_out:?}\n  direct:  {d_out:?}",
            scenario.name, moves[i]
        );
    }

    // The acceptance criterion, stated in its own terms: same scenario, same
    // final state hash. Implied by the per-step assertions above, kept explicit
    // so deleting the loop cannot quietly delete the criterion.
    fn last(t: &Trace) -> Option<&Fingerprint> {
        match t.steps.last() {
            Some((_, Outcome::Accepted(f))) => Some(f),
            Some(_) => None,
            None => Some(&t.initial),
        }
    }
    assert_eq!(
        last(&harness).map(|f| f.public),
        last(&direct).map(|f| f.public),
        "scenario `{}`: final public state hash differs",
        scenario.name
    );
}

fn defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

/// A battlefield/hand card spec built the way a direct test builds one.
fn spec(
    defs: &HashMap<String, CardDefinition>,
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

// ── Scenarios ─────────────────────────────────────────────────────────────────

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Play a land, tap a Mountain, Bolt an opponent, let it resolve.
///
/// Exercises `play_land`, `tap_for_mana`, `cast_spell` with a `player` target,
/// and `pass_priority`.
const BOLT_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 1 },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "battlefield": {
      "p1": [{ "card": "Mountain" }],
      "p2": [{ "card": "Llanowar Elves" }]
    },
    "hand": {
      "p1": [{ "card": "Lightning Bolt" }, { "card": "Forest" }]
    }
  }
}"#;

fn bolt_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(1))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Mountain", ZoneId::Battlefield))
        .object(spec(defs, P2, "Llanowar Elves", ZoneId::Battlefield))
        .object(spec(defs, P1, "Lightning Bolt", ZoneId::Hand(P1)))
        .object(spec(defs, P1, "Forest", ZoneId::Hand(P1)))
        .build()
        .expect("bolt scenario builds")
}

const BOLT_MOVES: &[Move] = &[
    Move::PlayLand {
        player: "p1",
        card: "Forest",
    },
    Move::TapForMana {
        player: "p1",
        land: "Mountain",
    },
    Move::CastSpell {
        player: "p1",
        card: "Lightning Bolt",
        targets: &[Tgt::Player("p2")],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Equip Lightning Greaves onto a Llanowar Elves.
///
/// Exercises `activate_ability` with a `permanent` target — the other branch of
/// the harness's `resolve_targets`.
const EQUIP_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 4,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0 },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "battlefield": {
      "p1": [{ "card": "Llanowar Elves" }, { "card": "Lightning Greaves" }]
    }
  }
}"#;

fn equip_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(4)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Llanowar Elves", ZoneId::Battlefield))
        .object(spec(defs, P1, "Lightning Greaves", ZoneId::Battlefield))
        .build()
        .expect("equip scenario builds")
}

const EQUIP_MOVES: &[Move] = &[
    Move::ActivateAbility {
        player: "p1",
        source: "Lightning Greaves",
        index: 0,
        targets: &[Tgt::Permanent("Llanowar Elves")],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Swing with a mana elf.
///
/// Exercises `declare_attackers`, whose harness translation resolves both the
/// creature name and the defending player name.
const COMBAT_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 5,
  "active_player": "p1",
  "phase": "declare_attackers",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0 },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "battlefield": {
      "p1": [{ "card": "Llanowar Elves" }],
      "p2": [{ "card": "Mountain" }]
    }
  }
}"#;

fn combat_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::DeclareAttackers)
        .active_player(P1)
        .turn_number(5)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Llanowar Elves", ZoneId::Battlefield))
        .object(spec(defs, P2, "Mountain", ZoneId::Battlefield))
        .build()
        .expect("combat scenario builds")
}

const COMBAT_MOVES: &[Move] = &[
    Move::DeclareAttackers {
        player: "p1",
        attackers: &[("Llanowar Elves", "p2")],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "bolt",
        script_json: BOLT_JSON,
        direct: bolt_direct,
    },
    Scenario {
        name: "equip",
        script_json: EQUIP_JSON,
        direct: equip_direct,
    },
    Scenario {
        name: "combat",
        script_json: COMBAT_JSON,
        direct: combat_direct,
    },
];

fn scenario(name: &str) -> &'static Scenario {
    SCENARIOS
        .iter()
        .find(|s| s.name == name)
        .expect("unknown scenario")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Divergence #1's regression gate.
///
/// Deserialize the same JSON 32 times — each parse allocates fresh `HashMap`s
/// with fresh `RandomState` seeds — and require one distinct fingerprint. Before
/// `sorted_zone_entries`, this produced 2+ and the whole rest of this file was
/// untestable, since nothing can be compared to a hash that changes on its own.
#[test]
fn build_initial_state_is_deterministic() {
    for s in SCENARIOS {
        let mut seen: Vec<Fingerprint> = Vec::new();
        for _ in 0..32 {
            let init: InitialState = serde_json::from_str(s.script_json).unwrap();
            let (state, _) = build_initial_state(&init);
            let fp = fingerprint(&state);
            if !seen.contains(&fp) {
                seen.push(fp);
            }
        }
        assert_eq!(
            seen.len(),
            1,
            "scenario `{}`: build_initial_state produced {} distinct states from one \
             script — a script-supplied HashMap is being iterated unsorted, so ObjectIds \
             are assigned in random order. Fingerprints: {:?}",
            s.name,
            seen.len(),
            seen
        );
    }
}

/// Divergence #2's regression gate: `initial_state.turn_number` reaches the state.
#[test]
fn initial_state_turn_number_is_honored() {
    let init: InitialState = serde_json::from_str(COMBAT_JSON).unwrap();
    let (state, _) = build_initial_state(&init);
    assert_eq!(
        state.turn().turn_number,
        5,
        "`initial_state.turn_number` is declared by the schema and must reach the GameState; \
         if it does not, every script silently runs on turn 1"
    );
}

#[test]
fn equivalence_bolt() {
    assert_equivalent(scenario("bolt"), BOLT_MOVES);
}

#[test]
fn equivalence_equip() {
    assert_equivalent(scenario("equip"), EQUIP_MOVES);
}

#[test]
fn equivalence_combat() {
    assert_equivalent(scenario("combat"), COMBAT_MOVES);
}

/// The scenarios above must actually *do* something, or they prove equivalence
/// of two no-ops. Require that every move in every scenario is translatable and
/// accepted by the engine, and that the board actually moved.
#[test]
fn scenarios_are_not_vacuous() {
    for (s, moves) in [
        (scenario("bolt"), BOLT_MOVES),
        (scenario("equip"), EQUIP_MOVES),
        (scenario("combat"), COMBAT_MOVES),
    ] {
        let init: InitialState = serde_json::from_str(s.script_json).unwrap();
        let (state, players) = build_initial_state(&init);
        let trace = drive(state, &players, moves, |m, st, p| m.harness_command(st, p));
        for (i, (cmd, out)) in trace.steps.iter().enumerate() {
            assert!(
                cmd.is_some(),
                "scenario `{}` move {i} ({:?}) does not translate to a Command — \
                 the equivalence test would be comparing two skipped actions",
                s.name,
                moves[i]
            );
            assert!(
                matches!(out, Outcome::Accepted(_)),
                "scenario `{}` move {i} ({:?}) was not accepted by the engine: {out:?}",
                s.name,
                moves[i]
            );
        }
        let final_fp = match &trace.steps.last().expect("moves is non-empty").1 {
            Outcome::Accepted(f) => f,
            other => panic!("scenario `{}` ended in {other:?}", s.name),
        };
        assert_ne!(
            *final_fp, trace.initial,
            "scenario `{}` ends where it started — it exercises nothing",
            s.name
        );
    }
}

// ── Divergence #3: cards a script names but the engine has never heard of ─────

/// Approved scripts that name a card with no `CardDefinition`. Objects for these
/// enter the game typeless, costless and abilityless, because
/// `enrich_spec_from_def` returns the bare spec when the name is unknown.
///
/// This list must only ever shrink. Adding to it means authoring a script that
/// describes a board the engine cannot build — architecture invariant #9.
/// Emptying it is SR-9c (`scutemob-71`) / card-authoring work.
const UNDEFINED_CARDS_IN_APPROVED_SCRIPTS: &[&str] = &["Grizzly Bears"];

/// Every card name in an approved script must have a `CardDefinition`, modulo
/// the allowlist above.
///
/// This is the equivalence check's precondition, not a side quest: a card with
/// no definition is a different object in the script regime than any direct test
/// would ever construct, and no hash comparison over it means anything.
#[test]
fn scripts_only_name_cards_that_have_definitions() {
    use mtg_engine::testing::script_schema::{GameScript, ReviewStatus};

    let known: std::collections::HashSet<String> =
        all_cards().iter().map(|d| d.name.clone()).collect();
    let allow: std::collections::HashSet<&str> = UNDEFINED_CARDS_IN_APPROVED_SCRIPTS
        .iter()
        .copied()
        .collect();

    let root = std::path::Path::new("../../test-data/generated-scripts");
    assert!(
        root.is_dir(),
        "script corpus not found at {root:?} — this test is cwd-relative, like run_all_scripts"
    );

    let mut offenders: Vec<(String, String)> = Vec::new();
    let mut allowlist_hits: std::collections::HashSet<String> = Default::default();
    let mut approved_scripts = 0usize;

    for group in std::fs::read_dir(root).expect("read corpus root").flatten() {
        if !group.path().is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(group.path())
            .expect("read group")
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read script");
            let Ok(script) = serde_json::from_str::<GameScript>(&text) else {
                continue; // malformed scripts are run_all_scripts' problem, not this test's
            };
            if script.metadata.review_status != ReviewStatus::Approved {
                continue;
            }
            approved_scripts += 1;
            for name in card_names(&script.initial_state) {
                if known.contains(&name) {
                    continue;
                }
                if allow.contains(name.as_str()) {
                    allowlist_hits.insert(name);
                } else {
                    offenders.push((path.display().to_string(), name));
                }
            }
        }
    }

    assert!(
        approved_scripts > 0,
        "found no approved scripts — the walk is broken and this test proves nothing"
    );
    assert!(
        offenders.is_empty(),
        "{} approved script(s) name cards with no CardDefinition (invariant #9). \
         The harness builds these as typeless objects and says nothing. \
         Author the card, or fix the script — do not extend the allowlist:\n{}",
        offenders.len(),
        offenders
            .iter()
            .map(|(f, c)| format!("  {f}: {c}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Denominator guard: an allowlist nobody hits is an allowlist that has
    // stopped describing reality. Shrink it when a card gets authored.
    let stale: Vec<&&str> = UNDEFINED_CARDS_IN_APPROVED_SCRIPTS
        .iter()
        .filter(|c| !allowlist_hits.contains(**c))
        .collect();
    assert!(
        stale.is_empty(),
        "these allowlist entries are no longer referenced by any approved script \
         (or have since been authored) — remove them: {stale:?}"
    );
}

/// Every card name an `InitialState` places in any zone.
fn card_names(init: &InitialState) -> Vec<String> {
    let mut out = Vec::new();
    for perms in init.zones.battlefield.values() {
        out.extend(perms.iter().map(|p| p.card.clone()));
    }
    for zone in [
        &init.zones.hand,
        &init.zones.graveyard,
        &init.zones.library,
        &init.zones.command_zone,
    ] {
        for cards in zone.values() {
            out.extend(cards.iter().map(|c| c.card.clone()));
        }
    }
    out.extend(init.zones.exile.iter().map(|c| c.card.clone()));
    out
}

// ── Property test ─────────────────────────────────────────────────────────────

use proptest::prelude::*;

/// The move pool for the property test: every move any scenario uses, plus
/// deliberately out-of-order and illegal ones (tapping a land twice, casting a
/// sorcery-speed land play out of the main phase, attacking outside combat).
/// The engine will reject most sequences — the property is that *both regimes
/// reject identically*, and where the engine accepts, both reach the same state.
const MOVE_POOL: &[Move] = &[
    Move::Pass("p1"),
    Move::Pass("p2"),
    Move::PlayLand {
        player: "p1",
        card: "Forest",
    },
    Move::PlayLand {
        player: "p1",
        card: "Lightning Bolt",
    }, // illegal: not a land
    Move::TapForMana {
        player: "p1",
        land: "Mountain",
    },
    Move::TapForMana {
        player: "p1",
        land: "Forest",
    }, // untranslatable until the Forest is on the battlefield
    Move::CastSpell {
        player: "p1",
        card: "Lightning Bolt",
        targets: &[Tgt::Player("p2")],
    },
    Move::CastSpell {
        player: "p1",
        card: "Lightning Bolt",
        targets: &[Tgt::Permanent("Llanowar Elves")],
    },
    Move::DeclareAttackers {
        player: "p1",
        attackers: &[("Llanowar Elves", "p2")],
    }, // illegal: not p1's creature, and wrong step
    Move::ActivateAbility {
        player: "p1",
        source: "Mountain",
        index: 0,
        targets: &[],
    }, // illegal: Mountain has no non-mana activated ability
];

proptest! {
    // Each case builds two full GameStates and dispatches up to 8 commands
    // through the engine twice; 96 cases keeps the whole file under a second.
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// The property: for *any* sequence of moves — legal, illegal, or
    /// unresolvable — the script regime and the direct regime issue the same
    /// `Command`s, get the same accept/reject answers, and arrive at the same
    /// state after every step.
    #[test]
    fn harness_and_direct_dispatch_agree_on_any_move_sequence(
        moves in prop::collection::vec(
            prop::sample::select(MOVE_POOL),
            0..=8,
        )
    ) {
        assert_equivalent(scenario("bolt"), &moves);
    }
}

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
//! # What this proves, and what it does not
//!
//! **Both regimes call `enrich_spec_from_def`** — the direct regime already
//! imports it (`cost_primitives.rs`, `combat_harness.rs`, `golgari_grave_troll.rs`
//! all do), because `ObjectSpec::card` alone makes a naked object. So the
//! initial-state half of the comparison does **not** independently pin enrich's
//! *inference*: a bug inside it is applied to both sides and cancels. There is no
//! second source of truth for that short of re-implementing the function, which
//! would be a re-implementation gate and would rot.
//!
//! What the initial-state comparison does pin is everything wrapped around
//! enrich: `ObjectId` assignment order, `PlayerId` assignment from sorted names,
//! the turn number, the `phase`→`Step` parse, and the post-build patching of
//! life / mana pool / land plays. What the *per-step* comparison pins — the sharp
//! end — is that `translate_player_action` builds the same `Command` a hand-written
//! test would. `Command` derives `PartialEq`, so command inequality is asserted
//! directly and is the diagnostic you actually want; the hash is the backstop.
//!
//! **Coverage is a tracked subset, ratcheted (SR-31).** Eleven of
//! `translate_player_action`'s ~79 dispatch arms (more once the alt-cost
//! dimensions of `cast_spell` are counted separately) are driven here. The base
//! six from SR-9b — `pass_priority`, `play_land`, `tap_for_mana`, `cast_spell`
//! (single target), `activate_ability`, `declare_attackers` — plus five
//! alternative-cost shapes added by SR-31: **convoke, delve, kicker, escape,
//! modal**, chosen by what the golden-script corpus and the card base actually
//! use. `CROSS_VALIDATED_SHAPES` records the covered set and
//! `cross_validated_shape_coverage_is_ratcheted` enforces it exactly, so coverage
//! can only grow, never regress silently — which is how it sat at six,
//! undocumented, until SR-31. Still uncovered: improvise, bargain, emerge,
//! casualty, assist, replicate, splice, escalate, squad, mutate, ninjutsu — a
//! mis-populated field in any of those is invisible to this file. Adding a
//! scenario is cheap: a JSON blob, a `direct` fn, a `CastAlt` move, one label.
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
//! 4. **`resolve_targets` silently dropped targets it could not resolve.** It
//!    used `filter_map`, so a `cast_spell` naming one permanent that is not on
//!    the battlefield produced a `CastSpell` with an *empty* `targets` vec — a
//!    targeted spell cast with no target (CR 601.2c) — and the script passed.
//!    Now returns `None` if any target fails, so the action does not translate.
//!    All 95 approved scripts stayed green, so nothing was leaning on it. Pinned
//!    by [`equivalence_unresolvable_target`].
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
//! | `play_land` falls back to the battlefield | the property test, and `equivalence_repeated_play_land` |
//! | an approved script names an undefined card | `scripts_only_name_cards_that_have_definitions` |
//! | `resolve_targets` drops an unresolvable target | `equivalence_unresolvable_target` |
//! | revert `sorted_zone_entries` on hand/graveyard/library | `build_initial_state_is_deterministic` |
//! | an approved script names an undefined *commander* | `scripts_only_name_cards_that_have_definitions` |
//!
//! Two rows carry the file's real argument. The `play_land` fallback was
//! invisible to every fixed scenario — it needs the *sequence* `[PlayLand Forest,
//! PlayLand Forest]` before the regimes disagree, and only the property test
//! generated one. And `equivalence_equip` survives the `sorted_zone_entries`
//! revert, because only one player has permanents in it, so map order cannot
//! matter: a scenario proves nothing about a bug it cannot express.
//!
//! The last three rows are review findings — each was a perturbation that
//! *survived* the gate as first written. The determinism gate was pointed only at
//! the battlefield map (the scenarios have one-owner hands and no graveyards or
//! libraries at all), and `card_names` never read the commander block, which is
//! the canonical place a commander is named. Both holes are the same shape as the
//! seven SR tasks before this one: the author checks that the gate fires on the
//! thing he was thinking about, and never enumerates what it is not pointed at.

use mtg_engine::rules::command::CastSpellData;
use std::collections::HashMap;

use mtg_engine::state::combat::AttackTarget;
use mtg_engine::testing::replay_harness::{build_initial_state, enrich_spec_from_def};
use mtg_engine::testing::script_schema::{ActionTarget, AttackerDeclaration, InitialState};
use mtg_engine::{
    all_cards, card_name_to_id, process_command, AdditionalCost, AltCostKind, CardDefinition,
    CardRegistry, Command, GameState, GameStateBuilder, ManaPool, ObjectId, ObjectSpec, PlayerId,
    Step, Target, ZoneId,
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
    /// A cast that exercises one alternative/additional cost dimension (SR-31).
    ///
    /// This one variant renders every alt-cost shape the file cross-validates —
    /// convoke, delve, kicker, escape, modal — by populating exactly one of its
    /// cost fields and naming the matching `action` string. It is the "harness
    /// builds the same `Command` a hand-written test would" thesis applied to the
    /// 40+ alt-cost parameters of `translate_player_action`, which nothing
    /// exercised before this task.
    CastAlt {
        player: &'static str,
        card: &'static str,
        /// Which zone the card is cast from. Escape casts from the graveyard.
        from: CastZone,
        /// The `action` string the script would carry (`cast_spell`,
        /// `cast_spell_escape`, `cast_spell_modal`).
        action: &'static str,
        targets: &'static [Tgt],
        /// Creature names tapped for convoke (`convoke_creatures`).
        convoke: &'static [&'static str],
        /// Graveyard card names exiled for delve (`delve_cards`).
        delve: &'static [&'static str],
        /// Graveyard card names exiled for escape (`AdditionalCost::EscapeExile`).
        escape: &'static [&'static str],
        /// Whether the spell is kicked (`kicker_times = 1`).
        kicked: bool,
        /// Chosen mode indices for a modal spell (`modes_chosen`).
        modes: &'static [usize],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CastZone {
    Hand,
    Graveyard,
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
            Move::CastAlt { action, .. } => action,
        }
    }

    fn actor(&self) -> &'static str {
        match self {
            Move::Pass(p)
            | Move::PlayLand { player: p, .. }
            | Move::TapForMana { player: p, .. }
            | Move::CastSpell { player: p, .. }
            | Move::ActivateAbility { player: p, .. }
            | Move::DeclareAttackers { player: p, .. }
            | Move::CastAlt { player: p, .. } => p,
        }
    }

    /// The canonical shape label for the coverage ratchet (SR-31). Distinct
    /// alt-cost dimensions of `cast_spell` get distinct labels so the ratchet
    /// records *which* were cross-validated, not merely that `cast_spell` was.
    fn shape(&self) -> &'static str {
        match self {
            Move::Pass(_) => "pass_priority",
            Move::PlayLand { .. } => "play_land",
            Move::TapForMana { .. } => "tap_for_mana",
            Move::CastSpell { .. } => "cast_spell",
            Move::ActivateAbility { .. } => "activate_ability",
            Move::DeclareAttackers { .. } => "declare_attackers",
            Move::CastAlt {
                action,
                convoke,
                delve,
                kicked,
                ..
            } => {
                if !convoke.is_empty() {
                    "cast_spell:convoke"
                } else if !delve.is_empty() {
                    "cast_spell:delve"
                } else if *kicked {
                    "cast_spell:kicker"
                } else if *action == "cast_spell_escape" {
                    "cast_spell_escape"
                } else if *action == "cast_spell_modal" {
                    "cast_spell_modal"
                } else {
                    "cast_spell"
                }
            }
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
            Move::PlayLand { card, .. }
            | Move::CastSpell { card, .. }
            | Move::CastAlt { card, .. } => Some(card),
            Move::TapForMana { land, .. } => Some(land),
            Move::ActivateAbility { source, .. } => Some(source),
            Move::Pass(_) | Move::DeclareAttackers { .. } => None,
        };
        let ability_index = match self {
            Move::ActivateAbility { index, .. } => *index,
            _ => 0,
        };
        let targets: Vec<ActionTarget> = match self {
            Move::CastSpell { targets, .. }
            | Move::ActivateAbility { targets, .. }
            | Move::CastAlt { targets, .. } => targets.iter().map(to_action_target).collect(),
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
        // Alt-cost inputs, empty for every move but `CastAlt`.
        let to_strings = |names: &[&'static str]| -> Vec<String> {
            names.iter().map(|s| (*s).to_string()).collect()
        };
        let (convoke, delve, escape, kicked, modes) = match self {
            Move::CastAlt {
                convoke,
                delve,
                escape,
                kicked,
                modes,
                ..
            } => (
                to_strings(convoke),
                to_strings(delve),
                to_strings(escape),
                *kicked,
                modes.to_vec(),
            ),
            _ => (vec![], vec![], vec![], false, vec![]),
        };
        translate(
            self.action_str(),
            players[self.actor()],
            card,
            ability_index,
            &targets,
            &attackers,
            &convoke,
            &delve,
            &escape,
            kicked,
            modes,
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

                chosen_color: None,
            }),
            Move::CastSpell { card, targets, .. } => {
                Some(Command::CastSpell(Box::new(CastSpellData {
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
                })))
            }
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
                modes_chosen: vec![],
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
            Move::CastAlt {
                card,
                from,
                action,
                targets,
                convoke,
                delve,
                escape,
                kicked,
                modes,
                ..
            } => {
                // The card's own zone. Escape casts from the graveyard.
                let card_id = match from {
                    CastZone::Hand => in_hand(state, player, card)?,
                    CastZone::Graveyard => in_graveyard(state, player, card)?,
                };
                // Mirror `translate_player_action`'s `filter_map`: an unresolvable
                // convoke/delve/escape name is dropped, not fatal. Scenarios name
                // only real cards, so in practice every name resolves.
                let convoke_creatures: Vec<ObjectId> = convoke
                    .iter()
                    .filter_map(|n| on_battlefield(state, player, n))
                    .collect();
                let delve_cards: Vec<ObjectId> = delve
                    .iter()
                    .filter_map(|n| in_graveyard(state, player, n))
                    .collect();
                let escape_exile: Vec<ObjectId> = escape
                    .iter()
                    .filter_map(|n| in_graveyard(state, player, n))
                    .collect();
                let mut data = CastSpellData {
                    player,
                    card: card_id,
                    targets: resolve(targets, state, players)?,
                    convoke_creatures,
                    improvise_artifacts: vec![],
                    delve_cards,
                    kicker_times: if *kicked { 1 } else { 0 },
                    alt_cost: None,
                    prototype: false,
                    modes_chosen: modes.to_vec(),
                    x_value: 0,
                    hybrid_choices: vec![],
                    phyrexian_life_payments: vec![],
                    face_down_kind: None,
                    additional_costs: vec![],
                };
                if *action == "cast_spell_escape" {
                    data.alt_cost = Some(AltCostKind::Escape);
                    data.additional_costs = vec![AdditionalCost::EscapeExile {
                        cards: escape_exile,
                    }];
                }
                Some(Command::CastSpell(Box::new(data)))
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
    convoke: &[String],
    delve: &[String],
    escape: &[String],
    kicked: bool,
    modes_chosen: Vec<usize>,
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
        &[],     // blockers
        convoke, // convoke
        &[],     // improvise
        delve,   // delve
        escape,  // escape
        kicked,
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
        modes_chosen,
        None,  // target_creature
        0,     // x_value
        &[],   // collect_evidence
        0,     // squad_count
        false, // mutate_on_top
        None,  // gift_opponent
        None,  // sacrifice_card
        &[],   // exert
        None,  // pitch_exile_card
        None,  // chosen_color
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

fn in_graveyard(state: &GameState, player: PlayerId, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player))
        .map(|(&id, _)| id)
}

fn any_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(&id, _)| id)
}

/// `None` if *any* target fails to resolve — what a hand-written test does, since
/// it has no way to name a permanent that isn't there.
///
/// The harness's `resolve_targets` used to `filter_map` here, silently dropping
/// an unresolvable target and casting the spell with a shorter list. That
/// asymmetry is divergence #4; [`equivalence_unresolvable_target`] exercises it.
/// If this ever diverges again, the harness's target resolution is the bug.
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

// ── Alt-cost scenarios (SR-31) ────────────────────────────────────────────────
//
// Each drives one alternative/additional cost through both regimes. The five
// shapes were chosen by what the golden-script corpus and the card base actually
// use (checked with grep at authoring time): convoke leads corpus usage (5 refs)
// and has 7 defs; modal/modes is the next-heaviest corpus signal (7 combined);
// delve (2 corpus / 5 defs), escape (2 corpus / 4 defs) and kicker (10 defs, the
// most-implemented castable cost) round out the top of both lists. Mutate and
// ninjutsu — the remaining menu entries — are deferred: both are special actions
// rather than a plain cast-with-cost, so they need their own `Move` shapes.
//
// Every direct builder adds objects in `build_initial_state`'s insertion order
// (battlefield → hand → graveyard → exile → library, players sorted) so the
// initial-state fingerprints match; mana pools are generous so the cast is
// accepted (the point is that the *Command* matches, but a rejected-both-ways
// pair would be vacuous — `alt_cost_scenarios_are_not_vacuous` forbids it).

fn pool(g: u32, u: u32, b: u32, r: u32, c: u32) -> ManaPool {
    ManaPool {
        green: g,
        blue: u,
        black: b,
        red: r,
        colorless: c,
        ..Default::default()
    }
}

/// Convoke: cast Siege Wurm ({5}{G}{G}) tapping a Llanowar Elves to help pay.
const CONVOKE_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0, "mana_pool": { "green": 5, "colorless": 10 } },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "battlefield": { "p1": [{ "card": "Llanowar Elves" }] },
    "hand": { "p1": [{ "card": "Siege Wurm" }] }
  }
}"#;

fn convoke_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0).mana(pool(5, 0, 0, 0, 10)))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Llanowar Elves", ZoneId::Battlefield))
        .object(spec(defs, P1, "Siege Wurm", ZoneId::Hand(P1)))
        .build()
        .expect("convoke scenario builds")
}

const CONVOKE_MOVES: &[Move] = &[
    Move::CastAlt {
        player: "p1",
        card: "Siege Wurm",
        from: CastZone::Hand,
        action: "cast_spell",
        targets: &[],
        convoke: &["Llanowar Elves"],
        delve: &[],
        escape: &[],
        kicked: false,
        modes: &[],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Delve: cast Treasure Cruise ({7}{U}, Draw 3) exiling three graveyard cards.
const DELVE_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0, "mana_pool": { "blue": 3, "colorless": 10 } },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "hand": { "p1": [{ "card": "Treasure Cruise" }] },
    "graveyard": { "p1": [{ "card": "Island" }, { "card": "Swamp" }, { "card": "Mountain" }] },
    "library": { "p1": [{ "card": "Forest" }, { "card": "Plains" }, { "card": "Sol Ring" }] }
  }
}"#;

fn delve_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0).mana(pool(0, 3, 0, 0, 10)))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Treasure Cruise", ZoneId::Hand(P1)))
        .object(spec(defs, P1, "Island", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Swamp", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Mountain", ZoneId::Graveyard(P1)))
        // PB-RS1: library entries are pushed bottom-to-top to match the (now
        // fixed) harness, which reverses the script's top-to-bottom
        // declaration ("Forest", "Plains", "Sol Ring") so the first-declared
        // card (Forest) ends up as the true top (CR 121.1).
        .object(spec(defs, P1, "Sol Ring", ZoneId::Library(P1)))
        .object(spec(defs, P1, "Plains", ZoneId::Library(P1)))
        .object(spec(defs, P1, "Forest", ZoneId::Library(P1)))
        .build()
        .expect("delve scenario builds")
}

const DELVE_MOVES: &[Move] = &[
    Move::CastAlt {
        player: "p1",
        card: "Treasure Cruise",
        from: CastZone::Hand,
        action: "cast_spell",
        targets: &[],
        convoke: &[],
        delve: &["Island", "Swamp", "Mountain"],
        escape: &[],
        kicked: false,
        modes: &[],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Kicker: cast Burst Lightning ({R}, Kicker {4}) kicked at p2 for 4 damage.
const KICKER_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0, "mana_pool": { "red": 3, "colorless": 10 } },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "hand": { "p1": [{ "card": "Burst Lightning" }] }
  }
}"#;

fn kicker_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0).mana(pool(0, 0, 0, 3, 10)))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Burst Lightning", ZoneId::Hand(P1)))
        .build()
        .expect("kicker scenario builds")
}

const KICKER_MOVES: &[Move] = &[
    Move::CastAlt {
        player: "p1",
        card: "Burst Lightning",
        from: CastZone::Hand,
        action: "cast_spell",
        targets: &[Tgt::Player("p2")],
        convoke: &[],
        delve: &[],
        escape: &[],
        kicked: true,
        modes: &[],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Escape: cast Woe Strider (Escape—{3}{B}{B}, exile four other cards) from the
/// graveyard, exiling four distinct fodder cards.
const ESCAPE_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0, "mana_pool": { "black": 5, "colorless": 10 } },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "graveyard": {
      "p1": [
        { "card": "Woe Strider" },
        { "card": "Swamp" },
        { "card": "Island" },
        { "card": "Forest" },
        { "card": "Plains" }
      ]
    }
  }
}"#;

fn escape_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0).mana(pool(0, 0, 5, 0, 10)))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Woe Strider", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Swamp", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Island", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Forest", ZoneId::Graveyard(P1)))
        .object(spec(defs, P1, "Plains", ZoneId::Graveyard(P1)))
        .build()
        .expect("escape scenario builds")
}

const ESCAPE_MOVES: &[Move] = &[
    Move::CastAlt {
        player: "p1",
        card: "Woe Strider",
        from: CastZone::Graveyard,
        action: "cast_spell_escape",
        targets: &[],
        convoke: &[],
        delve: &[],
        escape: &["Swamp", "Island", "Forest", "Plains"],
        kicked: false,
        modes: &[],
    },
    Move::Pass("p1"),
    Move::Pass("p2"),
];

/// Modal: cast Archmage's Charm ({U}{U}{U}) choosing mode 1 ("Target player draws
/// two cards"), targeting p1.
const MODAL_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 3,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 0, "mana_pool": { "blue": 3 } },
    "p2": { "life": 40, "land_plays_remaining": 0 }
  },
  "zones": {
    "hand": { "p1": [{ "card": "Archmage's Charm" }] },
    "library": { "p1": [{ "card": "Forest" }, { "card": "Plains" }] }
  }
}"#;

fn modal_direct(defs: &HashMap<String, CardDefinition>) -> GameState {
    GameStateBuilder::new()
        .at_step(Step::PreCombatMain)
        .active_player(P1)
        .turn_number(3)
        .with_registry(CardRegistry::new(all_cards()))
        .add_player_with(P1, |p| p.life(40).land_plays(0).mana(pool(0, 3, 0, 0, 0)))
        .add_player_with(P2, |p| p.life(40).land_plays(0))
        .object(spec(defs, P1, "Archmage's Charm", ZoneId::Hand(P1)))
        // PB-RS1: library entries pushed bottom-to-top (Forest ends up the true
        // top, matching the script's top-to-bottom declaration -- CR 121.1).
        .object(spec(defs, P1, "Plains", ZoneId::Library(P1)))
        .object(spec(defs, P1, "Forest", ZoneId::Library(P1)))
        .build()
        .expect("modal scenario builds")
}

const MODAL_MOVES: &[Move] = &[
    Move::CastAlt {
        player: "p1",
        card: "Archmage's Charm",
        from: CastZone::Hand,
        action: "cast_spell_modal",
        targets: &[Tgt::Player("p1")],
        convoke: &[],
        delve: &[],
        escape: &[],
        kicked: false,
        modes: &[1],
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
    Scenario {
        name: "convoke",
        script_json: CONVOKE_JSON,
        direct: convoke_direct,
    },
    Scenario {
        name: "delve",
        script_json: DELVE_JSON,
        direct: delve_direct,
    },
    Scenario {
        name: "kicker",
        script_json: KICKER_JSON,
        direct: kicker_direct,
    },
    Scenario {
        name: "escape",
        script_json: ESCAPE_JSON,
        direct: escape_direct,
    },
    Scenario {
        name: "modal",
        script_json: MODAL_JSON,
        direct: modal_direct,
    },
];

/// Every alt-cost scenario paired with its move list, for the ratchet and the
/// non-vacuity gate.
const ALT_COST_SCENARIOS: &[(&str, &[Move])] = &[
    ("convoke", CONVOKE_MOVES),
    ("delve", DELVE_MOVES),
    ("kicker", KICKER_MOVES),
    ("escape", ESCAPE_MOVES),
    ("modal", MODAL_MOVES),
];

fn scenario(name: &str) -> &'static Scenario {
    SCENARIOS
        .iter()
        .find(|s| s.name == name)
        .expect("unknown scenario")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A script whose every zone map has **two owners**, so that dropping the sort
/// on *any* of the four zone loops changes `ObjectId` assignment.
///
/// The three `SCENARIOS` cannot do this job: their hands have a single owner and
/// none of them populates a graveyard or library at all. Reverting
/// `sorted_zone_entries` on the hand, graveyard and library loops therefore left
/// the whole scripts suite green — the determinism gate was pointed only at the
/// battlefield. (Found in review; the perturbation is now caught.)
///
/// Every card here must have a `CardDefinition`, or the objects come out
/// typeless and the fixture stops distinguishing anything.
const MULTI_OWNER_ZONES_JSON: &str = r#"{
  "format": "commander",
  "turn_number": 2,
  "active_player": "p1",
  "phase": "precombat_main",
  "priority": "p1",
  "players": {
    "p1": { "life": 40, "land_plays_remaining": 1 },
    "p2": { "life": 40, "land_plays_remaining": 1 }
  },
  "zones": {
    "battlefield": {
      "p1": [{ "card": "Mountain" }],
      "p2": [{ "card": "Llanowar Elves" }]
    },
    "hand": {
      "p1": [{ "card": "Lightning Bolt" }],
      "p2": [{ "card": "Forest" }]
    },
    "graveyard": {
      "p1": [{ "card": "Island" }],
      "p2": [{ "card": "Swamp" }]
    },
    "library": {
      "p1": [{ "card": "Plains" }, { "card": "Sol Ring" }],
      "p2": [{ "card": "Lightning Greaves" }]
    }
  }
}"#;

/// Divergence #1's regression gate.
///
/// Deserialize the same JSON 32 times — each parse allocates fresh `HashMap`s
/// with fresh `RandomState` seeds — and require one distinct fingerprint. Before
/// `sorted_zone_entries`, this produced 2+ and the whole rest of this file was
/// untestable, since nothing can be compared to a hash that changes on its own.
///
/// Uses `Fingerprint`, not `public_state_hash`: library order and hand contents
/// are exactly what an unsorted library/hand loop scrambles, and the public hash
/// records only their *sizes*.
#[test]
fn build_initial_state_is_deterministic() {
    let fixtures = SCENARIOS
        .iter()
        .map(|s| (s.name, s.script_json))
        .chain(std::iter::once((
            "multi_owner_zones",
            MULTI_OWNER_ZONES_JSON,
        )));

    for (name, json) in fixtures {
        let mut seen: Vec<Fingerprint> = Vec::new();
        for _ in 0..32 {
            let init: InitialState = serde_json::from_str(json).unwrap();
            let (state, _) = build_initial_state(&init);
            let fp = fingerprint(&state);
            if !seen.contains(&fp) {
                seen.push(fp);
            }
        }
        assert_eq!(
            seen.len(),
            1,
            "scenario `{name}`: build_initial_state produced {} distinct states from one \
             script — a script-supplied HashMap is being iterated unsorted, so ObjectIds \
             are assigned in random order. Fingerprints: {:?}",
            seen.len(),
            seen
        );
    }
}

/// The determinism gate is only as good as its fixture. If `MULTI_OWNER_ZONES_JSON`
/// ever stops having two owners in every zone map, dropping a sort on the
/// one-owner zone becomes undetectable — silently, because a one-owner `HashMap`
/// has exactly one iteration order.
#[test]
fn determinism_fixture_has_two_owners_in_every_zone_map() {
    let init: InitialState = serde_json::from_str(MULTI_OWNER_ZONES_JSON).unwrap();
    for (zone, len) in [
        ("battlefield", init.zones.battlefield.len()),
        ("hand", init.zones.hand.len()),
        ("graveyard", init.zones.graveyard.len()),
        ("library", init.zones.library.len()),
    ] {
        assert!(
            len >= 2,
            "MULTI_OWNER_ZONES_JSON's `{zone}` map has {len} owner(s); with fewer than 2 \
             the sort on that loop cannot be observed and its regression gate is a no-op"
        );
    }
    assert!(
        init.players.len() >= 2,
        "the players map needs two keys for the same reason"
    );
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

// ── Alt-cost equivalence (SR-31) ──────────────────────────────────────────────

#[test]
fn equivalence_convoke() {
    assert_equivalent(scenario("convoke"), CONVOKE_MOVES);
}

#[test]
fn equivalence_delve() {
    assert_equivalent(scenario("delve"), DELVE_MOVES);
}

#[test]
fn equivalence_kicker() {
    assert_equivalent(scenario("kicker"), KICKER_MOVES);
}

#[test]
fn equivalence_escape() {
    assert_equivalent(scenario("escape"), ESCAPE_MOVES);
}

#[test]
fn equivalence_modal() {
    assert_equivalent(scenario("modal"), MODAL_MOVES);
}

/// The alt-cost scenarios must be non-vacuous for the same reason the base ones
/// are: an equivalence that both regimes *reject* identically proves nothing
/// (the pre-SR-9b `equip` scenario was green precisely because both sides
/// rejected it). Every move must translate, be accepted, and move the board — and
/// the cast step's Command must actually carry the alt-cost payload, or the
/// scenario would be validating a plain cast wearing an alt-cost label.
#[test]
fn alt_cost_scenarios_are_not_vacuous() {
    for (name, moves) in ALT_COST_SCENARIOS {
        let s = scenario(name);
        let init: InitialState = serde_json::from_str(s.script_json).unwrap();
        let (state, players) = build_initial_state(&init);
        let trace = drive(state, &players, moves, |m, st, p| m.harness_command(st, p));
        for (i, (cmd, out)) in trace.steps.iter().enumerate() {
            assert!(
                cmd.is_some(),
                "alt-cost scenario `{name}` move {i} ({:?}) does not translate",
                moves[i]
            );
            assert!(
                matches!(out, Outcome::Accepted(_)),
                "alt-cost scenario `{name}` move {i} ({:?}) was not accepted: {out:?}",
                moves[i]
            );
        }
        // The cast step (move 0) must carry a non-empty alt-cost payload.
        let cast_cmd = trace.steps[0].0.as_ref().expect("cast translated");
        let carries_payload = match cast_cmd {
            Command::CastSpell(data) => {
                !data.convoke_creatures.is_empty()
                    || !data.delve_cards.is_empty()
                    || data.kicker_times > 0
                    || data.alt_cost.is_some()
                    || !data.modes_chosen.is_empty()
                    || !data.additional_costs.is_empty()
            }
            _ => false,
        };
        assert!(
            carries_payload,
            "alt-cost scenario `{name}` cast carries no alt-cost payload — it is a \
             plain cast mislabeled, and would not exercise the translation it claims"
        );
        let final_fp = match &trace.steps.last().expect("moves non-empty").1 {
            Outcome::Accepted(f) => f,
            other => panic!("alt-cost scenario `{name}` ended in {other:?}"),
        };
        assert_ne!(
            *final_fp, trace.initial,
            "alt-cost scenario `{name}` ends where it started"
        );
    }
}

/// `Move::shape()` reports a single label per `CastAlt` by checking the cost
/// dimensions in a fixed order (convoke → delve → kicker → escape → modal). That
/// is unambiguous only while every `CastAlt` sets exactly one dimension. A future
/// combined scenario (e.g. convoke + kicker) would be silently mislabeled — and
/// its second dimension would go uncounted by the ratchet. Guard the precondition.
#[test]
fn each_alt_cost_move_sets_exactly_one_dimension() {
    for (name, moves) in ALT_COST_SCENARIOS {
        for m in *moves {
            if let Move::CastAlt {
                action,
                convoke,
                delve,
                escape,
                kicked,
                modes,
                ..
            } = m
            {
                let dimensions = [
                    !convoke.is_empty(),
                    !delve.is_empty(),
                    !escape.is_empty() || *action == "cast_spell_escape",
                    *kicked,
                    !modes.is_empty() || *action == "cast_spell_modal",
                ];
                let set = dimensions.iter().filter(|&&b| b).count();
                assert_eq!(
                    set, 1,
                    "alt-cost scenario `{name}`: a CastAlt sets {set} cost dimensions; \
                     `shape()` labels by the first, so anything but exactly one is \
                     mislabeled and under-counts the ratchet"
                );
            }
        }
    }
}

// ── Coverage ratchet (SR-31) ──────────────────────────────────────────────────

/// The shapes this file cross-validates, one label per distinct
/// `translate_player_action` Command shape (alt-cost dimensions of `cast_spell`
/// get their own label). **This list may only grow.** Adding a scenario that
/// exercises a new shape means adding its label here; deleting a scenario that
/// was the sole cover for a shape fails the ratchet below rather than silently
/// thinning coverage — which is exactly how this file's coverage sat at 6 shapes,
/// undocumented, until SR-31.
const CROSS_VALIDATED_SHAPES: &[&str] = &[
    // Base shapes (SR-9b).
    "pass_priority",
    "play_land",
    "tap_for_mana",
    "cast_spell",
    "activate_ability",
    "declare_attackers",
    // Alt-cost shapes (SR-31).
    "cast_spell:convoke",
    "cast_spell:delve",
    "cast_spell:kicker",
    "cast_spell_escape",
    "cast_spell_modal",
];

/// A lower bound on the size of `translate_player_action`'s dispatch surface —
/// the count of distinct `"action" =>` arms. Recompute with:
///
/// ```text
/// grep -coE '^\s*"[a-z_]+" =>' crates/engine/src/testing/replay_harness.rs
/// ```
///
/// This is the *denominator*: it makes the ratchet honest about tracking a
/// minority of the surface (and the alt-cost dimensions of `cast_spell` push the
/// true shape count still higher). It exists so nobody can quietly redefine the
/// ratchet's baseline as "everything" and delete it.
const TRANSLATE_PLAYER_ACTION_ARMS: usize = 79;

/// Every move list fed to `assert_equivalent` anywhere in this file. The ratchet
/// derives the *actually-covered* shape set from these, so a shape can only count
/// as covered if a real scenario drives it.
const ALL_VALIDATED_MOVE_SETS: &[&[Move]] = &[
    BOLT_MOVES,
    EQUIP_MOVES,
    COMBAT_MOVES,
    CONVOKE_MOVES,
    DELVE_MOVES,
    KICKER_MOVES,
    ESCAPE_MOVES,
    MODAL_MOVES,
];

/// SR-31 ratchet: the set of shapes actually driven through both regimes must
/// equal `CROSS_VALIDATED_SHAPES` exactly.
///
/// - **⊇** (no fiction): every recorded shape is exercised by a live scenario, so
///   the baseline cannot list a shape no test covers.
/// - **⊆** (ratchet): adding a scenario that covers a new shape *forces* an update
///   to `CROSS_VALIDATED_SHAPES`, and removing the last cover for a shape fails —
///   coverage cannot regress silently.
#[test]
fn cross_validated_shape_coverage_is_ratcheted() {
    use std::collections::BTreeSet;

    let covered: BTreeSet<&str> = ALL_VALIDATED_MOVE_SETS
        .iter()
        .flat_map(|set| set.iter())
        .map(|m| m.shape())
        .collect();
    let recorded: BTreeSet<&str> = CROSS_VALIDATED_SHAPES.iter().copied().collect();

    let unrecorded: Vec<&&str> = covered.difference(&recorded).collect();
    assert!(
        unrecorded.is_empty(),
        "these shapes are cross-validated by a scenario but not recorded in \
         CROSS_VALIDATED_SHAPES — add them (the list may only grow): {unrecorded:?}"
    );
    let uncovered: Vec<&&str> = recorded.difference(&covered).collect();
    assert!(
        uncovered.is_empty(),
        "CROSS_VALIDATED_SHAPES records these shapes, but no live scenario drives \
         them — the ratchet baseline has gone stale (a scenario was deleted?): {uncovered:?}"
    );

    // Denominator guard: the ratchet must track a strict subset of the translator
    // surface, or it has stopped being a coverage *floor* and become a claim of
    // completeness it cannot back.
    assert!(
        recorded.len() < TRANSLATE_PLAYER_ACTION_ARMS,
        "the ratchet now records {} shapes against a {}-arm translator surface — if \
         coverage really is near-total, replace this ratchet with an exhaustive \
         check; until then it must track a subset",
        recorded.len(),
        TRANSLATE_PLAYER_ACTION_ARMS
    );
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
    use mtg_engine::testing::script_schema::ReviewStatus;

    let known: std::collections::HashSet<String> =
        all_cards().iter().map(|d| d.name.clone()).collect();
    let allow: std::collections::HashSet<&str> = UNDEFINED_CARDS_IN_APPROVED_SCRIPTS
        .iter()
        .copied()
        .collect();

    let mut offenders: Vec<(String, String)> = Vec::new();
    let mut allowlist_hits: std::collections::HashSet<String> = Default::default();
    let mut approved_scripts = 0usize;

    // SR-22b: use `run_all_scripts`'s single, fully-recursive discovery function
    // rather than the hand-rolled two-level walk this test used to run. That walk
    // stopped at `group/file.json`, so a script nested any deeper (e.g.
    // `group/sub/file.json`) was silently outside the gate and could name an
    // undefined card without failing. `discover_scripts` recurses to arbitrary
    // depth; the two gates now see the identical file set.
    let corpus = crate::run_all_scripts::discover_scripts(std::path::Path::new(
        crate::run_all_scripts::SCRIPTS_DIR,
    ));
    for (label, parsed) in &corpus {
        let Ok(script) = parsed else {
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
                offenders.push((label.clone(), name));
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

/// Every card name an `InitialState` mentions anywhere.
///
/// Not just the zone maps: `players.<p>.commander` and `.partner_commander` name
/// a card outside every zone, and are the *canonical* place a commander is named
/// — `build_initial_state` reads them to populate `commander_ids`. Omitting them
/// left the gate blind to exactly the card most likely to be missing a
/// definition. (Found in review; no approved script exploits it today, because
/// all nine commander-bearing scripts also list the commander in a zone.)
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
    for pstate in init.players.values() {
        for cmdr in [&pstate.commander, &pstate.partner_commander]
            .into_iter()
            .flatten()
        {
            out.push(cmdr.card.clone());
        }
    }
    out
}

// ── Directed sequences ────────────────────────────────────────────────────────

/// Sequences that no single-shot scenario expresses, pinned as fixed cases.
///
/// The property test below reaches these too, but only by sampling: measured at
/// 96 cases, 6 draws contained two accepted `PlayLand Forest` moves, and the
/// `play_land`-falls-back-to-the-battlefield perturbation was caught 12/12 runs.
/// "Empirically always" is not "always" — lowering `PROPTEST_CASES` would quietly
/// stop catching it. So the sequence that carries the argument is also a fixed
/// test. (Found in review.)
#[test]
fn equivalence_repeated_play_land() {
    // The second PlayLand must not resolve: the Forest has left the hand. A
    // harness that fell back to `find_on_battlefield` would translate it, the
    // engine would reject it, and a direct test would never have built it at all.
    const MOVES: &[Move] = &[
        Move::PlayLand {
            player: "p1",
            card: "Forest",
        },
        Move::PlayLand {
            player: "p1",
            card: "Forest",
        },
    ];
    assert_equivalent(scenario("bolt"), MOVES);
}

/// A cast whose target list is partly unresolvable.
///
/// The harness's `resolve_targets` uses `filter_map`, so it *drops* a target it
/// cannot resolve and casts the spell with the targets that survived. The direct
/// regime aborts the move. Pinned so that the asymmetry — documented on
/// [`resolve`] — is exercised rather than merely described. If this test ever
/// fails, the harness's `filter_map` is the bug.
#[test]
fn equivalence_unresolvable_target() {
    const MOVES: &[Move] = &[
        Move::TapForMana {
            player: "p1",
            land: "Mountain",
        },
        Move::CastSpell {
            player: "p1",
            card: "Lightning Bolt",
            // No such permanent is on the battlefield in the `bolt` scenario.
            targets: &[Tgt::Permanent("Nonexistent Permanent")],
        },
    ];
    assert_equivalent(scenario("bolt"), MOVES);
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

use proptest::test_runner::{Config, TestRunner};
use std::sync::atomic::{AtomicU32, Ordering};

/// The property: for *any* sequence of moves — legal, illegal, or unresolvable —
/// the script regime and the direct regime issue the same `Command`s, get the
/// same accept/reject answers, and arrive at the same state after every step.
///
/// Driven by a **hand-rolled** [`TestRunner`] rather than the `proptest!` macro
/// (SR-22d). The macro re-applies `contextualize_config` at runtime (proptest's
/// `sugar.rs`), so the `PROPTEST_CASES` env var silently overrides the configured
/// case count — measured: `PROPTEST_CASES=1` ran this property exactly once while
/// the file still reported "ok", quietly defeating the coverage the fixed count
/// was chosen to give. `Config::with_cases` sets `cases` *after* the env-derived
/// default and `TestRunner::run` does not re-read the env, so the count sticks;
/// counting the closure invocations then makes ">= 96 cases actually ran" an
/// asserted runtime fact. Shrinking is preserved — `runner.run` still catches the
/// panic from `assert_equivalent` and minimizes the failing move sequence.
#[test]
fn harness_and_direct_dispatch_agree_on_any_move_sequence() {
    // Each case builds two full GameStates and dispatches up to 8 commands
    // through the engine twice; 96 cases keeps the whole file under a second.
    const CASES: u32 = 96;

    let ran = AtomicU32::new(0);
    let mut config = Config::with_cases(CASES);
    // A hand-rolled runner has no `source_file`, so proptest's default
    // `SourceParallel` persistence would print a spurious "no source file known"
    // warning and could never write a regression file anyway. Disable it — a
    // failure here surfaces via the panic, already shrunk.
    config.failure_persistence = None;
    let mut runner = TestRunner::new(config);
    let strategy = prop::collection::vec(prop::sample::select(MOVE_POOL), 0..=8);

    let outcome = runner.run(&strategy, |moves| {
        ran.fetch_add(1, Ordering::Relaxed);
        assert_equivalent(scenario("bolt"), &moves);
        Ok(())
    });
    // A failure here has already been shrunk by the runner; surface it.
    outcome.expect("harness/direct equivalence property failed");

    let ran = ran.load(Ordering::Relaxed);
    assert!(
        ran >= CASES,
        "equivalence proptest executed {ran} case(s), expected at least {CASES}. \
         The `proptest!` macro honours PROPTEST_CASES and this guard exists so a \
         reduced env override cannot silently thin the run (SR-22d)."
    );
}

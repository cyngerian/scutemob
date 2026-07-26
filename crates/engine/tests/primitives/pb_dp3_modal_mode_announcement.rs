//! PB-DP3 (DP-4): mode announcement is mandatory (CR 601.2b / 602.2b / 700.2a).
//!
//! Before this PB, `Command::CastSpell { modes_chosen: vec![], .. }` on a modal spell
//! silently auto-selected mode 0 and paid full cost — a rules violation, not a
//! convenience (CR 700.2a: "the controller of a modal spell or activated ability
//! chooses the mode(s) as part of casting/activating"; CR 601.2b puts that
//! announcement before costs are determined or paid). The same bypass existed for
//! modal activated abilities (`abilities.rs`). This file proves the fix on three real
//! `Complete` cards (Cryptic Command, Austere Command, Incendiary Command — all three
//! `min_modes: 2`), on the broader `min_modes: 1` case (37 of the 41 modal defs), on
//! the `min_modes: 0` edge cases (Spell fail-safe reject vs. Activated legal-no-op),
//! and pins the entwine/escalate backward-compat exemptions against regression.
//!
//! CR rules covered: 601.2b, 602.2b, 700.2, 700.2a, 700.2d, 702.42b (entwine),
//! 702.120a (escalate).

use mtg_engine::cards::card_definition::EffectTarget;
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, ActivatedAbility,
    ActivationCost, AdditionalCost, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, GameStateError, KeywordAbility,
    ManaColor, ManaCost, ModeSelection, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, Target,
    TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_hand(state: &GameState, player: PlayerId, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

/// Pass priority for all listed players once (resolves the top of the stack).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Cast a (possibly modal) card by name from `player`'s hand, with explicit
/// `additional_costs` (entwine/escalate).
fn cast_with(
    state: GameState,
    player: PlayerId,
    name: &str,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
    additional_costs: Vec<AdditionalCost>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let card_id = find_in_hand(&state, player, name);
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card: card_id,
            targets,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen,
            x_value: 0,
            face_down_kind: None,
            additional_costs,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
}

fn cast_modal(
    state: GameState,
    player: PlayerId,
    name: &str,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    cast_with(state, player, name, targets, modes_chosen, vec![])
}

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// Build a 2-player state with a REAL modal card (looked up via `all_cards()`) in p1's
/// hand, `mana` pre-loaded into p1's pool, plus any `extra` battlefield/hand/library
/// permanents.
fn build_command_state(
    card_name: &str,
    card_id: &str,
    mana: Vec<(ManaColor, u32)>,
    extra: Vec<ObjectSpec>,
) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let spell = enrich_spec_from_def(
        ObjectSpec::card(p1, card_name)
            .with_card_id(CardId(card_id.to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(spell);
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    for (color, amount) in mana {
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(color, amount);
    }
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Build a 2-player state with the real Goblin Cratermaker on the battlefield under
/// p1's control, `mana` colorless mana pre-loaded, plus an opposing 2/4 creature
/// (`"Goblin Cratermaker Retry Target"`, toughness 4 so 2 damage is NOT lethal and
/// survives observably) so a rejected activation probe can retry mode 0 ("deals 2
/// damage to target creature") legally on the same, untouched `state` (review
/// Finding 8 — proving the no-cost-paid guarantee with a real cast rather than a
/// vacuous self-comparison).
fn build_goblin_state(mana: u32) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::goblin_cratermaker::card();
    let defs_map: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Goblin Cratermaker")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ))
        .object(ObjectSpec::creature(
            p2,
            "Goblin Cratermaker Retry Target",
            2,
            4,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    if mana > 0 {
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, mana);
    }
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

// ── Synthetic card defs (for cases no real card exercises) ────────────────────

/// Sorcery {1}{U}, Choose one — Gain 3 life / draw 2 cards / deal 2 damage to
/// yourself. `min_modes: 1, max_modes: 1`. Covers the broad `min_modes: 1` case (37 of
/// the 41 modal defs) plus range/duplicate/max_modes validation, none of which need a
/// real card.
fn dp3_three_mode_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-three-mode-spell".to_string()),
        name: "DP3 Three Mode Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one — Gain 3 life; or draw 2 cards; or deal 2 damage to yourself."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::DealDamage {
                        source: None,
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ],
                mode_targets: None,
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn build_three_mode_state() -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp3_three_mode_spell_def()]);
    let spell = ObjectSpec::card(p1, "DP3 Three Mode Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dp3-three-mode-spell".to_string()))
        .with_types(vec![CardType::Sorcery]);
    let lib_cards: Vec<_> = (0..4)
        .map(|i| {
            ObjectSpec::card(p1, &format!("DP3 Lib {i}"))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib in lib_cards {
        builder = builder.object(lib);
    }
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Sorcery {1}, Choose up to one — Gain 3 life. `min_modes: 0, max_modes: 1`. No such
/// card exists in the corpus (the only `min_modes: 0` object is the triggered
/// `hullbreaker_horror`) — this is the OOS-DP3-2 fail-safe scenario.
fn dp3_zero_min_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-zero-min-spell".to_string()),
        name: "DP3 Zero Min Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose up to one — Gain 3 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 0,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                }],
                mode_targets: None,
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn build_zero_min_state() -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp3_zero_min_spell_def()]);
    let spell = ObjectSpec::card(p1, "DP3 Zero Min Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dp3-zero-min-spell".to_string()))
        .with_types(vec![CardType::Sorcery]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// A synthetic modal ACTIVATED ability, `min_modes: 0, max_modes: 1`, with two modes
/// that each have a distinct measurable board consequence (GainLife / DrawCards). No
/// sacrifice cost, so the source survives regardless of outcome.
fn build_synthetic_zero_mode_activated_state() -> (GameState, PlayerId, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let modes = ModeSelection {
        min_modes: 0,
        max_modes: 1,
        allow_duplicate_modes: false,
        mode_costs: None,
        modes: vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            },
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
        ],
        mode_targets: None,
    };
    let ability = ActivatedAbility {
        cost: ActivationCost {
            mana_cost: Some(ManaCost {
                generic: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
        description: "DP3 synthetic min_modes:0 activated ability".to_string(),
        effect: Some(Effect::Nothing),
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: Some(modes),
    };
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::artifact(p1, "DP3 Synthetic Zero-Mode Artifact")
                .with_activated_ability(ability),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);
    let source_id = find_object(&state, "DP3 Synthetic Zero-Mode Artifact");
    (state, p1, p2, source_id)
}

/// Sorcery {1}{U}, Choose one — Gain 3 life; or draw 2 cards. Entwine {2}. Mirrors
/// `mechanics_e_l/entwine.rs`'s synthetic card, self-contained here so this file has no
/// cross-file dependency.
fn dp3_entwine_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-entwine-spell".to_string()),
        name: "DP3 Entwine Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one — Gain 3 life; or draw 2 cards. Entwine {2}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            AbilityDefinition::Entwine {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

fn build_entwine_state() -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp3_entwine_spell_def()]);
    let spell = ObjectSpec::card(p1, "DP3 Entwine Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dp3-entwine-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);
    let lib_cards: Vec<_> = (0..3)
        .map(|i| {
            ObjectSpec::card(p1, &format!("DP3 Entwine Lib {i}"))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib in lib_cards {
        builder = builder.object(lib);
    }
    let mut state = builder.build().unwrap();
    // {1}{U} base + {2} entwine = {3}{U} = 4 mana; blue covers both generic and pips.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Sorcery {1}{R}, Choose one or more — Gain 3 life; or draw 2 cards; or deal 2 damage
/// to yourself. Escalate {1}. Mirrors `mechanics_e_l/escalate.rs`'s synthetic card.
fn dp3_escalate_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-escalate-spell".to_string()),
        name: "DP3 Escalate Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one or more — Gain 3 life; or draw 2 cards; or deal 2 damage to \
                      yourself. Escalate {1}."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::DealDamage {
                            source: None,
                            target: EffectTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

fn build_escalate_state(library_count: usize) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp3_escalate_spell_def()]);
    let spell = ObjectSpec::card(p1, "DP3 Escalate Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dp3-escalate-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Escalate);
    let lib_cards: Vec<_> = (0..library_count)
        .map(|i| {
            ObjectSpec::card(p1, &format!("DP3 Escalate Lib {i}"))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib in lib_cards {
        builder = builder.object(lib);
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Generic escalate-state builder parameterized by `def`, for the synthetic
/// derived-count-bounds cards below (review Finding 2) that don't share
/// `dp3_escalate_spell_def`'s fixed `ModeSelection`. Mirrors `build_escalate_state`.
fn build_escalate_state_for(
    def: CardDefinition,
    library_count: usize,
) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let name = def.name.clone();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);
    let spell = ObjectSpec::card(p1, &name)
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(card_id)
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Escalate);
    let lib_cards: Vec<_> = (0..library_count)
        .map(|i| {
            ObjectSpec::card(p1, &format!("DP3 Escalate Lib {i}"))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib in lib_cards {
        builder = builder.object(lib);
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

/// Sorcery {1}{R}, Choose one or two — Gain 3 life; or draw 2 cards; or deal 2 damage
/// to yourself. Escalate {1}. Same shape as `dp3_escalate_spell_def` but `max_modes: 2`
/// (one fewer than `modes.len()`), so paying escalate twice derives a count ABOVE
/// `max_modes` — drives the `derived > max_modes` rejection branch
/// (`casting.rs:3545-3550`, review Finding 2). Synthetic-only: no corpus escalate card
/// prints `max_modes < modes.len()`.
fn dp3_escalate_spell_capped_max_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-escalate-spell-capped-max".to_string()),
        name: "DP3 Escalate Spell Capped Max".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one or two — Gain 3 life; or draw 2 cards; or deal 2 damage to \
                      yourself. Escalate {1}."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 2,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::DealDamage {
                            source: None,
                            target: EffectTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Sorcery {1}{R}, four modes, `min_modes: 3, max_modes: 4`. Escalate {1}. Exists ONLY
/// to drive the `derived < min_modes` rejection branch (`casting.rs:3539-3544`, review
/// Finding 2): it requires `escalate_modes >= 1` (so `derived >= 2`) AND
/// `min_modes > 2`, and no corpus escalate card prints `min_modes > 2` (the corpus's
/// only escalate cards, `blessed_alliance` and `collective_resistance`, are both
/// `min_modes: 1`) — so this def is synthetic-only and intentionally not
/// representative of a real printed card.
fn dp3_escalate_spell_min3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dp3-escalate-spell-min3".to_string()),
        name: "DP3 Escalate Spell Min3".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose three or four — Gain 3 life; or draw 2 cards; or deal 2 damage to \
                      yourself; or gain 1 life. Escalate {1}."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 3,
                    max_modes: 4,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::DealDamage {
                            source: None,
                            target: EffectTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(1),
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// §7.1 — Fail-before / pass-after probes (the real DP-4 evidence)
// ══════════════════════════════════════════════════════════════════════════════

/// CR 601.2b/700.2a — Cryptic Command (`min_modes: 2`, `Complete`) cast with no
/// announced modes is rejected before any mana is paid.
#[test]
fn test_601_2b_cryptic_command_empty_modes_rejected() {
    let (state, p1, _p2) = build_command_state(
        "Cryptic Command",
        "cryptic-command",
        vec![(ManaColor::Blue, 4)],
        vec![],
    );

    let result = cast_modal(state.clone(), p1, "Cryptic Command", vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 601.2b/700.2a: empty modes_chosen on Cryptic Command (min_modes: 2) must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 2 mode"),
        "expected 'at least 2 mode', got: {err}"
    );
    assert!(
        err.contains("601.2b"),
        "expected a CR 601.2b citation, got: {err}"
    );

    // CR 601.2b/601.2h: the rejection must leave the original state fully intact --
    // prove it by casting the SAME `state` again (not a clone) with explicit legal
    // modes and asserting it succeeds, spending the full mana pool. (A comparison
    // against `state` after only the CLONE was passed into the rejected call above is
    // vacuous -- `cast_modal` takes `GameState` by value and drops it on `Err`, so
    // `state` itself was never at risk; review Finding 8.)
    let (state, _) =
        cast_modal(state, p1, "Cryptic Command", vec![], vec![2, 3]).unwrap_or_else(|e| {
            panic!(
                "a legal retry (modes [2,3]) on the untouched state must succeed -- \
                 the rejected cast must not have mutated it: {:?}",
                e
            )
        });
    assert_eq!(
        state.players()[&p1].mana_pool.total(),
        0,
        "the retry spent the full {{1}}{{U}}{{U}}{{U}} cost out of the untouched pool"
    );
}

/// CR 601.2b/700.2a — Austere Command (`min_modes: 2`, `Complete`) cast with no
/// announced modes is rejected before any mana is paid.
#[test]
fn test_601_2b_austere_command_empty_modes_rejected() {
    let (state, p1, _p2) = build_command_state(
        "Austere Command",
        "austere-command",
        vec![(ManaColor::Colorless, 4), (ManaColor::White, 2)],
        vec![],
    );

    let result = cast_modal(state.clone(), p1, "Austere Command", vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 601.2b/700.2a: empty modes_chosen on Austere Command (min_modes: 2) must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 2 mode"),
        "expected 'at least 2 mode', got: {err}"
    );
    assert!(
        err.contains("601.2b"),
        "expected a CR 601.2b citation, got: {err}"
    );

    // CR 601.2b/601.2h: the rejection must leave the original state fully intact --
    // prove it by casting the SAME `state` again (not a clone) with explicit legal
    // modes and asserting it succeeds, spending the full mana pool. (Review Finding 8
    // -- see the identical rationale on the Cryptic Command probe above.)
    let (state, _) =
        cast_modal(state, p1, "Austere Command", vec![], vec![0, 1]).unwrap_or_else(|e| {
            panic!(
                "a legal retry (modes [0,1]) on the untouched state must succeed -- \
                 the rejected cast must not have mutated it: {:?}",
                e
            )
        });
    assert_eq!(
        state.players()[&p1].mana_pool.total(),
        0,
        "the retry spent the full {{4}}{{W}}{{W}} cost out of the untouched pool"
    );
}

/// CR 601.2b/700.2a — Incendiary Command (`min_modes: 2`, `Complete`) cast with no
/// announced modes is rejected before any mana is paid.
#[test]
fn test_601_2b_incendiary_command_empty_modes_rejected() {
    let (state, p1, p2) = build_command_state(
        "Incendiary Command",
        "incendiary-command",
        vec![(ManaColor::Colorless, 3), (ManaColor::Red, 2)],
        vec![],
    );

    let result = cast_modal(state.clone(), p1, "Incendiary Command", vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 601.2b/700.2a: empty modes_chosen on Incendiary Command (min_modes: 2) must be \
         rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 2 mode"),
        "expected 'at least 2 mode', got: {err}"
    );
    assert!(
        err.contains("601.2b"),
        "expected a CR 601.2b citation, got: {err}"
    );

    // CR 601.2b/601.2h: the rejection must leave the original state fully intact --
    // prove it by casting the SAME `state` again (not a clone) with explicit legal
    // modes and asserting it succeeds, spending the full mana pool. Mode 3 (wheel)
    // needs both players' hands to be resolvable, so this retry runs `pass_all` before
    // checking the pool (the mana is spent at cast time either way; the pass just lets
    // the resolution complete without leaving a dangling stack object). (Review
    // Finding 8 -- see the identical rationale on the Cryptic Command probe above.)
    let (state, _) = cast_modal(state, p1, "Incendiary Command", vec![], vec![1, 3])
        .unwrap_or_else(|e| {
            panic!(
                "a legal retry (modes [1,3]) on the untouched state must succeed -- \
                 the rejected cast must not have mutated it: {:?}",
                e
            )
        });
    assert_eq!(
        state.players()[&p1].mana_pool.total(),
        0,
        "the retry spent the full {{3}}{{R}}{{R}} cost out of the untouched pool"
    );
    let (_state, _) = pass_all(state, &[p1, p2]);
}

/// CR 601.2b/700.2a — the broad half of the fix: a `min_modes: 1` modal spell (37 of
/// the 41 modal defs) cast with no announced modes is rejected.
#[test]
fn test_601_2b_min_modes_one_modal_spell_empty_modes_rejected() {
    let (state, p1, _p2) = build_three_mode_state();
    let result = cast_modal(state, p1, "DP3 Three Mode Spell", vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 601.2b/700.2a: empty modes_chosen on a min_modes: 1 modal spell must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 1 mode"),
        "expected 'at least 1 mode', got: {err}"
    );
    assert!(
        err.contains("601.2b"),
        "expected a CR 601.2b citation, got: {err}"
    );
}

/// CR 700.2a — a `min_modes: 0` modal SPELL cast with no announced modes is
/// unrepresentable and hard-rejected as a fail-safe (OOS-DP3-2), even though CR 700.2a
/// itself permits announcing zero modes for "choose up to N". No shipped card has this
/// shape.
#[test]
fn test_601_2b_min_modes_zero_modal_spell_empty_modes_rejected_failsafe() {
    let (state, p1, _p2) = build_zero_min_state();
    let result = cast_modal(state, p1, "DP3 Zero Min Spell", vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 700.2a permits announcing zero modes, but this engine cannot represent that on a \
         Spell stack object -- the OOS-DP3-2 fail-safe rejects instead of silently resolving \
         mode 0"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("OOS-DP3-2"),
        "expected the OOS-DP3-2 fail-safe citation, got: {err}"
    );
}

/// CR 602.2b/700.2a — Goblin Cratermaker's modal activated ability (`min_modes: 1`)
/// activated with no announced modes is rejected before its `{1}, Sacrifice this
/// creature` cost is paid.
#[test]
fn test_602_2b_modal_activated_ability_empty_modes_rejected() {
    let (state, p1, p2) = build_goblin_state(1);
    let source_id = find_object(&state, "Goblin Cratermaker");

    let result = activate(state.clone(), p1, source_id, 0, vec![], vec![]);
    assert!(
        result.is_err(),
        "CR 602.2b/700.2a: empty modes_chosen on a modal activated ability must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 1 mode"),
        "expected 'at least 1 mode', got: {err}"
    );
    assert!(
        err.contains("602.2b"),
        "expected a CR 602.2b citation, got: {err}"
    );

    // CR 602.2/602.2b/601.2h: an illegal activation must leave the original state
    // fully intact -- prove it by activating the SAME `state` again (not a clone) with
    // an explicit legal mode and asserting it succeeds, spending the {1} mana and
    // sacrificing the source for real this time. (A comparison against `state` after
    // only the CLONE was passed into the rejected call above is vacuous -- `activate`
    // takes `GameState` by value and drops it on `Err`, so `state` itself was never at
    // risk; review Finding 8.)
    let target_id = find_object(&state, "Goblin Cratermaker Retry Target");
    let (state, _) = activate(
        state,
        p1,
        source_id,
        0,
        vec![Target::Object(target_id)],
        vec![0],
    )
    .unwrap_or_else(|e| {
        panic!(
            "a legal retry (mode 0, explicit target) on the untouched state must \
             succeed -- the rejected activation must not have mutated it: {:?}",
            e
        )
    });
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players()[&p1].mana_pool.total(),
        0,
        "the retry spent the {{1}} mana out of the full, untouched pool"
    );
    assert!(
        !state.objects().values().any(
            |o| o.characteristics.name == "Goblin Cratermaker" && o.zone == ZoneId::Battlefield
        ),
        "the retry's sacrifice cost was paid for real this time"
    );
    let target = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Goblin Cratermaker Retry Target")
        .unwrap_or_else(|| panic!("retry target not found"));
    assert_eq!(
        target.damage_marked, 2,
        "mode 0 (2 damage to target creature) fired on the retry"
    );
}

/// CR 700.2a — a `min_modes: 0` modal ACTIVATED ability activated with no announced
/// modes is a LEGAL no-op (unlike the Spell case above, this IS representable: an
/// empty `validated_modes_chosen` leaves `embedded_effect` as the ability's own base
/// effect). Behaviour flip vs. pre-PB-DP3: before, mode 0 fired regardless.
#[test]
fn test_700_2a_modal_activated_min_modes_zero_empty_accepted_resolves_no_mode() {
    let (state, p1, p2, source_id) = build_synthetic_zero_mode_activated_state();
    let initial_life = state.players()[&p1].life_total;
    let initial_hand = hand_count(&state, p1);

    let (state, _events) = activate(state, p1, source_id, 0, vec![], vec![]).unwrap_or_else(|e| {
        panic!(
            "min_modes: 0 activation with empty modes must succeed: {:?}",
            e
        )
    });
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.players()[&p1].life_total,
        initial_life,
        "CR 700.2a: mode 0 (GainLife) must NOT have fired -- no mode was chosen"
    );
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "CR 700.2a: mode 1 (DrawCards) must NOT have fired -- no mode was chosen"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// §7.2 — Positive regression guards (pass before AND after)
// ══════════════════════════════════════════════════════════════════════════════

/// CR 700.2a — Cryptic Command cast choosing modes [2, 3] (tap all opponents'
/// creatures; draw a card) resolves BOTH modes with real board consequences.
#[test]
fn test_700_2a_cryptic_command_modes_2_and_3_both_resolve() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, p1, p2) = build_command_state(
        "Cryptic Command",
        "cryptic-command",
        vec![(ManaColor::Blue, 4)],
        vec![
            ObjectSpec::creature(p2, "Cryptic Opponent Creature", 2, 2),
            ObjectSpec::card(p1, "Cryptic Draw Fodder")
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery]),
        ],
    );
    let initial_hand = hand_count(&state, p1); // 1: the spell itself.

    let (state, _) = cast_modal(state, p1, "Cryptic Command", vec![], vec![2, 3])
        .unwrap_or_else(|e| panic!("modes [2,3] cast should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    let opp_creature = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Cryptic Opponent Creature")
        .unwrap_or_else(|| panic!("opponent creature not found"));
    assert!(
        opp_creature.status.tapped,
        "mode 2 (tap all creatures opponents control) should have tapped the opponent's \
         creature"
    );
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "mode 3 (draw a card) should have replaced the spell that left the hand -- hand \
         count returns to its pre-cast size"
    );
}

/// CR 700.2a — Austere Command cast choosing modes [0, 1] (destroy all artifacts;
/// destroy all enchantments) resolves BOTH modes, leaving a creature (modes 2/3's
/// domain) untouched.
#[test]
fn test_700_2a_austere_command_modes_0_and_1_both_resolve() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, p1, p2) = build_command_state(
        "Austere Command",
        "austere-command",
        vec![(ManaColor::Colorless, 4), (ManaColor::White, 2)],
        vec![
            ObjectSpec::artifact(p1, "Austere Artifact Victim"),
            ObjectSpec::enchantment(p2, "Austere Enchantment Victim"),
            ObjectSpec::creature(p1, "Austere Surviving Creature", 2, 2),
        ],
    );

    let (state, _) = cast_modal(state, p1, "Austere Command", vec![], vec![0, 1])
        .unwrap_or_else(|e| panic!("modes [0,1] cast should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Austere Artifact Victim"
                && o.zone == ZoneId::Graveyard(p1)),
        "mode 0 (destroy all artifacts) should have put the artifact in its owner's \
         graveyard"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Austere Enchantment Victim"
                && o.zone == ZoneId::Graveyard(p2)),
        "mode 1 (destroy all enchantments) should have put the enchantment in its \
         owner's graveyard"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Austere Surviving Creature"
                && o.zone == ZoneId::Battlefield),
        "modes 2/3 (destroy creatures) must NOT have fired -- the creature survives"
    );
}

/// CR 700.2a — Incendiary Command cast choosing modes [1, 3] (2 damage to each
/// creature; wheel each player's hand) resolves BOTH modes.
#[test]
fn test_700_2a_incendiary_command_modes_1_and_3_both_resolve() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, p1, p2) = build_command_state(
        "Incendiary Command",
        "incendiary-command",
        vec![(ManaColor::Colorless, 3), (ManaColor::Red, 2)],
        vec![
            ObjectSpec::creature(p2, "Incendiary Target Creature", 2, 2),
            ObjectSpec::card(p1, "IC P1 Hand Card 0")
                .in_zone(ZoneId::Hand(p1))
                .with_types(vec![CardType::Sorcery]),
            ObjectSpec::card(p1, "IC P1 Hand Card 1")
                .in_zone(ZoneId::Hand(p1))
                .with_types(vec![CardType::Sorcery]),
            ObjectSpec::card(p2, "IC P2 Hand Card 0")
                .in_zone(ZoneId::Hand(p2))
                .with_types(vec![CardType::Sorcery]),
            ObjectSpec::card(p2, "IC P2 Hand Card 1")
                .in_zone(ZoneId::Hand(p2))
                .with_types(vec![CardType::Sorcery]),
        ]
        .into_iter()
        .chain((0..5).map(|i| {
            ObjectSpec::card(p1, &format!("IC P1 Lib {i}"))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        }))
        .chain((0..5).map(|i| {
            ObjectSpec::card(p2, &format!("IC P2 Lib {i}"))
                .in_zone(ZoneId::Library(p2))
                .with_types(vec![CardType::Sorcery])
        }))
        .collect(),
    );

    let (state, _) = cast_modal(state, p1, "Incendiary Command", vec![], vec![1, 3])
        .unwrap_or_else(|e| panic!("modes [1,3] cast should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Incendiary Target Creature"
                && o.zone == ZoneId::Graveyard(p2)),
        "mode 1 (2 damage to each creature) should have killed the 2/2"
    );
    assert_eq!(
        hand_count(&state, p1),
        2,
        "mode 3 (wheel) should have discarded p1's 2 remaining hand cards then drawn 2, \
         leaving hand count at 2"
    );
    assert_eq!(
        hand_count(&state, p2),
        2,
        "mode 3 (wheel) should have discarded p2's 2 hand cards then drawn 2, leaving \
         hand count at 2"
    );
}

/// CR 700.2a — an out-of-range mode index is rejected even as the sole chosen mode.
#[test]
fn test_700_2a_out_of_range_index_rejected_as_sole_mode() {
    let (state, p1, _p2) = build_three_mode_state();
    let result = cast_modal(state, p1, "DP3 Three Mode Spell", vec![], vec![7]);
    assert!(
        result.is_err(),
        "mode index 7 (only 3 modes exist) must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("out of range"),
        "expected 'out of range', got: {err}"
    );
}

/// CR 700.2d — a duplicate mode index is rejected even as the sole chosen pair.
#[test]
fn test_700_2d_duplicate_mode_rejected_as_sole_pair() {
    let (state, p1, _p2) = build_three_mode_state();
    let result = cast_modal(state, p1, "DP3 Three Mode Spell", vec![], vec![0, 0]);
    assert!(
        result.is_err(),
        "mode 0 chosen twice on a card without allow_duplicate_modes must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("chosen more than once"),
        "expected a duplicate-mode message, got: {err}"
    );
}

/// CR 700.2a — choosing more modes than `max_modes` permits is rejected.
#[test]
fn test_700_2a_max_modes_exceeded_rejected() {
    let (state, p1, _p2) = build_three_mode_state();
    let result = cast_modal(state, p1, "DP3 Three Mode Spell", vec![], vec![0, 1]);
    assert!(
        result.is_err(),
        "min_modes: 1, max_modes: 1 -- choosing 2 distinct modes must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at most 1 mode"),
        "expected an 'at most 1 mode' message, got: {err}"
    );
}

/// CR 702.42b — entwine's backward-compat path is unregressed: casting with the
/// entwine additional cost paid and an EMPTY `modes_chosen` still executes every mode.
#[test]
fn test_702_42b_entwine_with_empty_modes_still_resolves_all() {
    let (state, p1, p2) = build_entwine_state();
    let initial_life = state.players()[&p1].life_total;

    let (state, _) = cast_with(
        state,
        p1,
        "DP3 Entwine Spell",
        vec![],
        vec![],
        vec![AdditionalCost::Entwine],
    )
    .unwrap_or_else(|e| {
        panic!(
            "entwine cast with empty modes_chosen must still succeed: {:?}",
            e
        )
    });

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.players()[&p1].life_total,
        initial_life + 3,
        "mode 0 (GainLife 3) must have fired under entwine"
    );
    // Hand was emptied by the cast (the spell itself left for the stack), so a hand
    // count of 2 proves DrawCards(2) (mode 1) ALSO fired.
    assert_eq!(
        hand_count(&state, p1),
        2,
        "mode 1 (DrawCards 2) must ALSO have fired under entwine"
    );
}

/// CR 702.120a — escalate's backward-compat path is unregressed: casting with
/// `EscalateModes { count: 1 }` and an EMPTY `modes_chosen` executes modes 0 AND 1
/// (the §3.4 exemption).
#[test]
fn test_702_120a_escalate_with_empty_modes_unregressed() {
    let (mut state, p1, p2) = build_escalate_state(4);
    // {1}{R} base + escalate {1} once = {2}{R} = 3 mana.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    let initial_life = state.players()[&p1].life_total;

    let (state, _) = cast_with(
        state,
        p1,
        "DP3 Escalate Spell",
        vec![],
        vec![],
        vec![AdditionalCost::EscalateModes { count: 1 }],
    )
    .unwrap_or_else(|e| {
        panic!(
            "escalate cast (count=1) with empty modes_chosen must still succeed: {:?}",
            e
        )
    });

    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.players()[&p1].life_total,
        initial_life + 3,
        "mode 0 (GainLife 3) must have fired"
    );
    assert_eq!(
        hand_count(&state, p1),
        2,
        "mode 1 (DrawCards 2) must ALSO have fired -- modes 0..=1 under the escalate \
         exemption"
    );
}

/// CR 702.120a/601.2b — the exemption's boundary: `escalate_modes == 0` is NOT exempt.
/// An empty `modes_chosen` with no escalate payment is still rejected -- this is the
/// case that forced the one-line `escalate.rs:244` edit.
#[test]
fn test_702_120a_escalate_count_zero_requires_explicit_mode() {
    let (mut state, p1, _p2) = build_escalate_state(0);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let result = cast_with(
        state,
        p1,
        "DP3 Escalate Spell",
        vec![],
        vec![],
        vec![AdditionalCost::EscalateModes { count: 0 }],
    );
    assert!(
        result.is_err(),
        "escalate_modes == 0 is NOT exempt -- an explicit mode is required (CR 601.2b)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least 1 mode"),
        "expected 'at least 1 mode', got: {err}"
    );
    assert!(
        err.contains("601.2b"),
        "expected a CR 601.2b citation, got: {err}"
    );
}

/// CR 702.120a/700.2a — the escalate derived-count bound is not theatre. On a card
/// whose printed `max_modes` is BELOW `modes.len()`, paying escalate enough times to
/// derive a count above `max_modes` (`casting.rs:3545-3550`) is rejected outright,
/// where pre-PB-DP3 it silently resolved every mode. Synthetic-only: no corpus
/// escalate card sets `max_modes < modes.len()` (review Finding 2).
#[test]
fn test_702_120a_escalate_derived_count_over_max_modes_rejected() {
    let (mut state, p1, _p2) = build_escalate_state_for(dp3_escalate_spell_capped_max_def(), 0);
    // {1}{R} base + escalate {1} x2 = {3}{R} = 4 mana -- funded even though the
    // rejection fires before payment, so a mana-insufficiency error can't be mistaken
    // for the intended one.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);

    let result = cast_with(
        state,
        p1,
        "DP3 Escalate Spell Capped Max",
        vec![],
        vec![],
        vec![AdditionalCost::EscalateModes { count: 2 }],
    );
    assert!(
        result.is_err(),
        "escalate count=2 on a 3-mode spell with max_modes: 2 derives 3 modes -- must be \
         rejected, not silently clamped to max_modes (CR 702.120a/700.2a)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at most"),
        "expected 'at most N mode(s) allowed', got: {err}"
    );
    assert!(
        err.contains("702.120a"),
        "expected a CR 702.120a citation, got: {err}"
    );
}

/// CR 702.120a/700.2a — the `derived < min_modes` branch (`casting.rs:3539-3544`) is
/// reachable only via a synthetic card in this corpus: it requires `escalate_modes >=
/// 1` (so `derived >= 2`) AND `min_modes > 2`, and no printed escalate card sets
/// `min_modes > 2` (`blessed_alliance` and `collective_resistance` are both
/// `min_modes: 1`). It is reachable in principle, so this test drives it directly with
/// a synthetic 4-mode card (`min_modes: 3, max_modes: 4`) — paying escalate once
/// derives `(1+1).min(4) == 2 < 3` (review Finding 2).
#[test]
fn test_702_120a_escalate_derived_count_under_min_modes_rejected() {
    let (mut state, p1, _p2) = build_escalate_state_for(dp3_escalate_spell_min3_def(), 0);
    // {1}{R} base + escalate {1} x1 = {2}{R} = 3 mana.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let result = cast_with(
        state,
        p1,
        "DP3 Escalate Spell Min3",
        vec![],
        vec![],
        vec![AdditionalCost::EscalateModes { count: 1 }],
    );
    assert!(
        result.is_err(),
        "escalate count=1 on a 4-mode spell with min_modes: 3 derives 2 modes -- must be \
         rejected (CR 702.120a/700.2a)"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("at least"),
        "expected 'at least N mode(s) required', got: {err}"
    );
    assert!(
        err.contains("702.120a"),
        "expected a CR 702.120a citation, got: {err}"
    );
}

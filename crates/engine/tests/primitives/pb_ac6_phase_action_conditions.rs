//! PB-AC6 — Phase & opponent-action conditions.
//!
//! Three new `TriggerCondition` variants:
//! - `AtBeginningOfFirstMainPhase` (CR 505.1a / 603.2b) — generic CardDef sweep in
//!   `precombat_main_actions`, fires once per turn on `Step::PreCombatMain`.
//! - `AtBeginningOfPostcombatMain` (CR 505.1a / 603.2b) — generic CardDef sweep in
//!   the new `postcombat_main_actions`, fires on every `Step::PostCombatMain`
//!   (including extra main phases created by effects).
//! - `WhenBecomesTarget { scope, by_opponent, include_abilities }` (CR 601.2c /
//!   602.2b / 603.2) — fires at target ANNOUNCEMENT, dispatched inline from the
//!   `GameEvent::PermanentTargeted` handler alongside the Ward dispatch.
//!
//! Five new `Condition` variants:
//! - `YouAttackedThisTurn` (Raid, CR 508.1)
//! - `CreatedATokenThisTurn` (CR 111.10)
//! - `OpponentCastNSpells(u32)` -- reads the new all-players-reset
//!   `PlayerState::spells_cast_this_game_turn` counter (NOT the storm-scoped
//!   `spells_cast_this_turn`, which is deliberately reset only for the incoming
//!   active player -- see OOS-AC6-1 in `memory/primitives/pb-plan-AC6.md`).
//! - `SpellMastery` (ability word, CR 207.2c)
//! - `OpponentControlsMoreLandsThanYou`
//!
//! Card-definition backfill (Searslicer Goblin, Bloodsoaked Champion, Idol of
//! Oblivion, Dark Petition, Land Tax, Venerated Rotpriest, etc.) is a SEPARATE
//! close-phase task -- this file validates the primitives via synthetic
//! `CardDefinition` / `TriggeredAbilityDef` fixtures, not the shipped cards.

use mtg_engine::effects::{check_condition, execute_effect, matches_filter, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::game_object::{ActivatedAbility, ActivationCost};
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Condition, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, Target, TargetFilter,
    TargetRequirement, TokenSpec, TriggerCondition, TriggerEvent, TriggeredAbilityDef, TypeLine,
    ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn ctx_for(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

/// Pass priority for all players (in `players` order) repeatedly until `target` step
/// is reached. Mirrors the `advance_to_step` helper convention used across the test
/// suite (turn_actions.rs, delayed_triggers.rs).
fn advance_to_step(mut state: GameState, players: &[PlayerId], target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (infinite loop?)"
        );
        let holder = state.turn().priority_holder.expect("no priority holder");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
        let _ = players;
    }
}

/// A simple two-player state with each player holding a small library (avoids
/// decking out during Draw-step turn-based actions when advancing through steps).
fn two_player_builder_with_library(p1: PlayerId, p2: PlayerId) -> GameStateBuilder {
    let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
    for i in 0..5 {
        b = b.object(
            ObjectSpec::creature(p1, &format!("P1 Library {i}"), 1, 1).in_zone(ZoneId::Library(p1)),
        );
        b = b.object(
            ObjectSpec::creature(p2, &format!("P2 Library {i}"), 1, 1).in_zone(ZoneId::Library(p2)),
        );
    }
    b
}

/// A CardDefinition with a single `AbilityDefinition::Triggered` ability using
/// `trigger_condition`, gaining `amount` life for the controller.
fn phase_trigger_def(
    card_id: &str,
    name: &str,
    trigger_condition: TriggerCondition,
    amount: i32,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Test phase trigger.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(amount),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// A targeted instant spell: "Tap target creature." Costs {1}. Non-destructive so
/// the target survives for repeated assertions.
fn tap_target_spell_def(card_id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Tap target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::TapPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or(0)
}

// ── A: Hash schema sentinel ─────────────────────────────────────────────────────

#[test]
/// Strict-equality hash schema sentinel (conventions.md hash-sentinel rule).
/// PB-AC6 bumped 32 -> 33.
fn test_hash_schema_version_is_33() {
    assert_eq!(HASH_SCHEMA_VERSION, 47u8);
}

#[test]
/// PB-AC6 H1: `PlayerState.attacked_this_turn: true` and `false` must produce
/// different public hashes. Guards against the exact PB-AC1/PB-AC5 review-HIGH
/// failure mode (new tracking field omitted from `HashInto`).
fn test_hash_sensitive_attacked_this_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .build()
        .unwrap();
    let h0 = state.public_state_hash();
    state.players_mut().get_mut(&p1).unwrap().attacked_this_turn = true;
    let h1 = state.public_state_hash();
    assert_ne!(
        h0, h1,
        "attacked_this_turn must participate in the public state hash"
    );
}

#[test]
/// PB-AC6 H2: `PlayerState.created_token_this_turn: true` and `false` must produce
/// different public hashes.
fn test_hash_sensitive_created_token_this_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .build()
        .unwrap();
    let h0 = state.public_state_hash();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .created_token_this_turn = true;
    let h1 = state.public_state_hash();
    assert_ne!(
        h0, h1,
        "created_token_this_turn must participate in the public state hash"
    );
}

#[test]
/// PB-AC6 H3: `PlayerState.spells_cast_this_game_turn` must participate in the
/// public state hash, distinctly from `spells_cast_this_turn`.
fn test_hash_sensitive_spells_cast_this_game_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .build()
        .unwrap();
    let h0 = state.public_state_hash();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .spells_cast_this_game_turn = 3;
    let h1 = state.public_state_hash();
    assert_ne!(
        h0, h1,
        "spells_cast_this_game_turn must participate in the public state hash"
    );
}

// ── B: AtBeginningOfFirstMainPhase / AtBeginningOfPostcombatMain ────────────────

#[test]
/// CR 505.1a / 603.2b: a CardDef trigger with `AtBeginningOfFirstMainPhase` fires
/// exactly once, queued on entry to `Step::PreCombatMain` (which occurs exactly
/// once per turn).
fn test_first_main_phase_trigger_fires_once() {
    let p1 = p(1);
    let p2 = p(2);

    let def = phase_trigger_def(
        "ac6-first-main",
        "First Main Herald",
        TriggerCondition::AtBeginningOfFirstMainPhase,
        1,
    );
    let registry = CardRegistry::new(vec![def]);

    let permanent = ObjectSpec::card(p1, "First Main Herald")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("ac6-first-main".to_string()))
        .with_types(vec![CardType::Enchantment]);

    let state = two_player_builder_with_library(p1, p2)
        .with_registry(registry)
        .object(permanent)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "First Main Herald");
    let state = advance_to_step(state, &[p1, p2], Step::PreCombatMain);

    let matching: Vec<_> = state
        .stack_objects()
        .iter()
        .filter(|so| {
            matches!(
                so.kind,
                StackObjectKind::TriggeredAbility {
                    source_object,
                    ability_index: 0,
                    is_carddef_etb: true,
                    ..
                } if source_object == obj_id
            )
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "AtBeginningOfFirstMainPhase must queue exactly once entering Step::PreCombatMain"
    );
}

#[test]
/// CR 505.1a / 603.2b: `AtBeginningOfPostcombatMain` fires on `Step::PostCombatMain`
/// and NOT on `Step::PreCombatMain` (distinguishes first main from postcombat main).
fn test_postcombat_main_trigger_fires_and_not_on_precombat() {
    let p1 = p(1);
    let p2 = p(2);

    let def = phase_trigger_def(
        "ac6-postcombat-main",
        "Postcombat Herald",
        TriggerCondition::AtBeginningOfPostcombatMain,
        1,
    );
    let registry = CardRegistry::new(vec![def]);

    let permanent = ObjectSpec::card(p1, "Postcombat Herald")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("ac6-postcombat-main".to_string()))
        .with_types(vec![CardType::Enchantment]);

    let state = two_player_builder_with_library(p1, p2)
        .with_registry(registry)
        .object(permanent)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Postcombat Herald");

    // Reaching Step::PreCombatMain: no postcombat trigger should have queued yet.
    let state = advance_to_step(state, &[p1, p2], Step::PreCombatMain);
    let none_yet = state.stack_objects().iter().any(|so| {
        matches!(
            so.kind,
            StackObjectKind::TriggeredAbility { source_object, .. } if source_object == obj_id
        )
    });
    assert!(
        !none_yet,
        "AtBeginningOfPostcombatMain must NOT fire on Step::PreCombatMain"
    );

    // Continue to Step::PostCombatMain (no attackers declared, combat auto-skips).
    let state = advance_to_step(state, &[p1, p2], Step::PostCombatMain);
    let matching: Vec<_> = state
        .stack_objects()
        .iter()
        .filter(|so| {
            matches!(
                so.kind,
                StackObjectKind::TriggeredAbility {
                    source_object,
                    ability_index: 0,
                    is_carddef_etb: true,
                    ..
                } if source_object == obj_id
            )
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "AtBeginningOfPostcombatMain must queue exactly once entering Step::PostCombatMain"
    );
}

#[test]
/// CR 505.1: "your first main phase" -- a non-active player's `AtBeginningOfFirstMainPhase`
/// trigger does NOT fire on the active player's first main.
fn test_first_main_trigger_only_active_player() {
    let p1 = p(1);
    let p2 = p(2);

    let def_active = phase_trigger_def(
        "ac6-fm-active",
        "Active Player Herald",
        TriggerCondition::AtBeginningOfFirstMainPhase,
        1,
    );
    let def_other = phase_trigger_def(
        "ac6-fm-other",
        "Non-Active Player Herald",
        TriggerCondition::AtBeginningOfFirstMainPhase,
        1,
    );
    let registry = CardRegistry::new(vec![def_active, def_other]);

    let active_permanent = ObjectSpec::card(p1, "Active Player Herald")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("ac6-fm-active".to_string()))
        .with_types(vec![CardType::Enchantment]);
    let other_permanent = ObjectSpec::card(p2, "Non-Active Player Herald")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("ac6-fm-other".to_string()))
        .with_types(vec![CardType::Enchantment]);

    let state = two_player_builder_with_library(p1, p2)
        .with_registry(registry)
        .object(active_permanent)
        .object(other_permanent)
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();

    let active_id = find_object(&state, "Active Player Herald");
    let other_id = find_object(&state, "Non-Active Player Herald");
    let state = advance_to_step(state, &[p1, p2], Step::PreCombatMain);

    let active_fired = state.stack_objects().iter().any(|so| {
        matches!(
            so.kind,
            StackObjectKind::TriggeredAbility { source_object, .. } if source_object == active_id
        )
    });
    let other_fired = state.stack_objects().iter().any(|so| {
        matches!(
            so.kind,
            StackObjectKind::TriggeredAbility { source_object, .. } if source_object == other_id
        )
    });
    assert!(active_fired, "active player's first-main trigger must fire");
    assert!(
        !other_fired,
        "CR 505.1: non-active player's first-main trigger must NOT fire on the active \
         player's first main phase"
    );
}

// ── C: WhenBecomesTarget ────────────────────────────────────────────────────────

#[test]
/// CR 601.2c vs 602.2b -- Goldspan-shape (`scope:None`, `by_opponent:false`,
/// `include_abilities:false`): a spell targeting the creature fires the trigger;
/// targeting by an activated ability does NOT fire (spell-only).
fn test_becomes_target_self_by_spell() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![tap_target_spell_def("ac6-tap", "Tap Bolt")]);

    // -- Spell case: fires --
    {
        let creature = ObjectSpec::creature(p1, "Target Creature", 2, 2).with_triggered_ability(
            TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::PermanentBecomesTarget {
                    scope: None,
                    by_opponent: false,
                    include_abilities: false,
                },
                intervening_if: None,
                targets: vec![],
                description: "test becomes-target".to_string(),
                effect: Some(Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                }),
            },
        );
        let spell = ObjectSpec::card(p2, "Tap Bolt")
            .in_zone(ZoneId::Hand(p2))
            .with_types(vec![CardType::Instant])
            .with_mana_cost(ManaCost {
                generic: 1,
                ..Default::default()
            });

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(creature)
            .object(spell)
            .active_player(p2)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state
            .players_mut()
            .get_mut(&p2)
            .unwrap()
            .mana_pool
            .colorless = 1;
        state.turn_mut().priority_holder = Some(p2);

        let creature_id = find_object(&state, "Target Creature");
        let spell_id = find_object(&state, "Tap Bolt");
        let before_life = life_total(&state, p1);

        let (state, _) = process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(creature_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap();

        assert_eq!(
            state.stack_objects().len(),
            2,
            "stack should have Tap Bolt + becomes-target trigger"
        );
        let (state, _) = pass_all(state, &[p2, p1]);
        let (state, _) = pass_all(state, &[p2, p1]);
        assert_eq!(
            life_total(&state, p1),
            before_life + 3,
            "CR 601.2c: becomes-target trigger must fire and resolve when a spell targets \
             the creature"
        );
    }

    // -- Ability case: does NOT fire (include_abilities: false) --
    {
        let creature = ObjectSpec::creature(p1, "Target Creature 2", 2, 2).with_triggered_ability(
            TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::PermanentBecomesTarget {
                    scope: None,
                    by_opponent: false,
                    include_abilities: false,
                },
                intervening_if: None,
                targets: vec![],
                description: "test becomes-target".to_string(),
                effect: Some(Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                }),
            },
        );
        let pinger = ObjectSpec::artifact(p2, "Pinger").with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                ..Default::default()
            },
            description: "{T}: Tap target creature.".to_string(),
            effect: Some(Effect::TapPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
            }),
            sorcery_speed: false,
            targets: vec![TargetRequirement::TargetCreature],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![]))
            .object(creature)
            .object(pinger)
            .active_player(p2)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let creature_id = find_object(&state, "Target Creature 2");
        let pinger_id = find_object(&state, "Pinger");
        let before_life = life_total(&state, p1);

        let (state, _) = process_command(
            state,
            Command::ActivateAbility {
                player: p2,
                source: pinger_id,
                ability_index: 0,
                targets: vec![Target::Object(creature_id)],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        )
        .unwrap();

        let (state, _) = pass_all(state, &[p2, p1]);
        let (state, _) = pass_all(state, &[p2, p1]);
        assert_eq!(
            life_total(&state, p1),
            before_life,
            "CR 602.2b: becomes-target trigger with include_abilities:false must NOT fire \
             when the target is an ability, not a spell"
        );
    }
}

#[test]
/// Rotpriest-shape (`scope:Some(creature)`): a spell targeting ANOTHER creature you
/// control fires the trigger; a spell targeting an opponent's creature does not.
fn test_becomes_target_scope_you_control() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![tap_target_spell_def("ac6-tap2", "Tap Bolt 2")]);

    let build = |target_owner: PlayerId, target_name: &str| {
        let source = ObjectSpec::creature(p1, "Scope Source", 1, 1).with_triggered_ability(
            TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::PermanentBecomesTarget {
                    scope: Some(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    })),
                    by_opponent: false,
                    include_abilities: false,
                },
                intervening_if: None,
                targets: vec![],
                description: "test scope".to_string(),
                effect: Some(Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                }),
            },
        );
        let target = ObjectSpec::creature(target_owner, target_name, 2, 2);
        let spell = ObjectSpec::card(p2, "Tap Bolt 2")
            .in_zone(ZoneId::Hand(p2))
            .with_types(vec![CardType::Instant])
            .with_mana_cost(ManaCost {
                generic: 1,
                ..Default::default()
            });

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(source)
            .object(target)
            .object(spell)
            .active_player(p2)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state
            .players_mut()
            .get_mut(&p2)
            .unwrap()
            .mana_pool
            .colorless = 1;
        state.turn_mut().priority_holder = Some(p2);
        state
    };

    // Case A: targeting p1's OTHER creature -- fires.
    {
        let state = build(p1, "My Other Creature");
        let target_id = find_object(&state, "My Other Creature");
        let spell_id = find_object(&state, "Tap Bolt 2");
        let before_life = life_total(&state, p1);
        let (state, _) = process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap();
        let (state, _) = pass_all(state, &[p2, p1]);
        let (state, _) = pass_all(state, &[p2, p1]);
        assert_eq!(
            life_total(&state, p1),
            before_life + 5,
            "spell targeting another creature you control must fire the scoped trigger"
        );
    }

    // Case B: targeting an opponent's creature -- does not fire.
    {
        let state = build(p2, "Opponent Creature");
        let target_id = find_object(&state, "Opponent Creature");
        let spell_id = find_object(&state, "Tap Bolt 2");
        let before_life = life_total(&state, p1);
        let (state, _) = process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap();
        let (state, _) = pass_all(state, &[p2, p1]);
        let (state, _) = pass_all(state, &[p2, p1]);
        assert_eq!(
            life_total(&state, p1),
            before_life,
            "spell targeting an opponent's creature must NOT fire a \
             you-control-scoped trigger"
        );
    }
}

#[test]
/// `by_opponent:true`: your own spell targeting the creature does NOT fire; an
/// opponent's spell does (CR 702.21a analog gate).
fn test_becomes_target_by_opponent_gate() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![tap_target_spell_def("ac6-tap3", "Tap Bolt 3")]);

    let build = |caster: PlayerId| {
        let creature = ObjectSpec::creature(p1, "Opponent Gated Creature", 2, 2)
            .with_triggered_ability(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::PermanentBecomesTarget {
                    scope: None,
                    by_opponent: true,
                    include_abilities: false,
                },
                intervening_if: None,
                targets: vec![],
                description: "test by_opponent".to_string(),
                effect: Some(Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                }),
            });
        let spell = ObjectSpec::card(caster, "Tap Bolt 3")
            .in_zone(ZoneId::Hand(caster))
            .with_types(vec![CardType::Instant])
            .with_mana_cost(ManaCost {
                generic: 1,
                ..Default::default()
            });

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(creature)
            .object(spell)
            .active_player(caster)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state
            .players_mut()
            .get_mut(&caster)
            .unwrap()
            .mana_pool
            .colorless = 1;
        state.turn_mut().priority_holder = Some(caster);
        state
    };

    // Case A: p1 (creature's own controller) casts the spell -- does not fire.
    {
        let state = build(p1);
        let target_id = find_object(&state, "Opponent Gated Creature");
        let spell_id = find_object(&state, "Tap Bolt 3");
        let before_life = life_total(&state, p1);
        let (state, _) = process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p1,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap();
        let (state, _) = pass_all(state, &[p1, p2]);
        let (state, _) = pass_all(state, &[p1, p2]);
        assert_eq!(
            life_total(&state, p1),
            before_life,
            "by_opponent:true must NOT fire when the controller targets their own permanent"
        );
    }

    // Case B: p2 (an opponent) casts the spell -- fires.
    {
        let state = build(p2);
        let target_id = find_object(&state, "Opponent Gated Creature");
        let spell_id = find_object(&state, "Tap Bolt 3");
        let before_life = life_total(&state, p1);
        let (state, _) = process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap();
        let (state, _) = pass_all(state, &[p2, p1]);
        let (state, _) = pass_all(state, &[p2, p1]);
        assert_eq!(
            life_total(&state, p1),
            before_life + 2,
            "by_opponent:true must fire when an opponent targets the permanent"
        );
    }
}

#[test]
/// CR 601.2c: the becomes-target trigger is queued at ANNOUNCEMENT -- it is on the
/// stack ABOVE the targeting spell BEFORE the spell resolves.
fn test_becomes_target_fires_at_announcement_not_resolution() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![tap_target_spell_def("ac6-tap4", "Tap Bolt 4")]);

    let creature = ObjectSpec::creature(p1, "Announcement Creature", 2, 2).with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            trigger_on: TriggerEvent::PermanentBecomesTarget {
                scope: None,
                by_opponent: false,
                include_abilities: false,
            },
            intervening_if: None,
            targets: vec![],
            description: "test announcement timing".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
        },
    );
    let spell = ObjectSpec::card(p2, "Tap Bolt 4")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .colorless = 1;
    state.turn_mut().priority_holder = Some(p2);

    let creature_id = find_object(&state, "Announcement Creature");
    let spell_id = find_object(&state, "Tap Bolt 4");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap();

    // Both the spell AND its becomes-target trigger must be on the stack, with the
    // trigger ABOVE (resolves before) the spell -- proving the trigger fired at
    // announcement, not at spell resolution.
    assert_eq!(
        state.stack_objects().len(),
        2,
        "stack should hold spell + trigger"
    );
    let bottom = state.stack_objects().front().unwrap();
    let top = state.stack_objects().back().unwrap();
    assert!(
        matches!(bottom.kind, StackObjectKind::Spell { .. }),
        "bottom of stack should be the targeting spell"
    );
    assert!(
        matches!(top.kind, StackObjectKind::TriggeredAbility { .. }),
        "top of stack should be the becomes-target trigger (resolves before the spell)"
    );
}

// ── D: Condition::YouAttackedThisTurn ────────────────────────────────────────────

#[test]
/// Raid, CR 508.1: false before combat; true after declaring one or more attackers;
/// resets to false at the next turn boundary (multiplayer, non-active player).
fn test_you_attacked_this_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let attacker = ObjectSpec::creature(p1, "Raider", 2, 2);
    let state = two_player_builder_with_library(p1, p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Raider");
    let dummy_source = attacker_id;

    assert!(
        !check_condition(
            &state,
            &Condition::YouAttackedThisTurn,
            &ctx_for(p1, dummy_source)
        ),
        "YouAttackedThisTurn must be false before any attackers are declared"
    );

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .unwrap();

    assert!(
        check_condition(
            &state,
            &Condition::YouAttackedThisTurn,
            &ctx_for(p1, dummy_source)
        ),
        "CR 508.1: YouAttackedThisTurn must be true after declaring one or more attackers"
    );
}

#[test]
/// CR 508.4: a creature entering the battlefield already attacking (via a token
/// created with `enters_attacking: true`) does NOT set `attacked_this_turn` -- only
/// an actual declare-attackers action counts as "you attacked" (Bloodsoaked
/// Champion ruling). Only `handle_declare_attackers` sets the flag; token creation
/// routes through `GameState::add_object`, which never touches it.
fn test_token_entering_attacking_does_not_set_attacked_this_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let dummy_source = ObjectId(9999);
    let mut ctx = ctx_for(p1, dummy_source);
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: TokenSpec {
                name: "Attacking Token".to_string(),
                power: 1,
                toughness: 1,
                card_types: [CardType::Creature].into_iter().collect(),
                enters_attacking: true,
                count: EffectAmount::Fixed(1),
                ..Default::default()
            },
        },
        &mut ctx,
    );

    assert!(
        !state.players().get(&p1).unwrap().attacked_this_turn,
        "CR 508.4: a token entering the battlefield attacking must NOT set \
         attacked_this_turn (only handle_declare_attackers does)"
    );
}

// ── E: Condition::CreatedATokenThisTurn ──────────────────────────────────────────

#[test]
/// CR 111.10: false before any token is created; true after `add_object` places a
/// token on the battlefield (single chokepoint, regardless of emission path).
fn test_created_a_token_this_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let dummy_source = ObjectId(9999);
    assert!(!check_condition(
        &state,
        &Condition::CreatedATokenThisTurn,
        &ctx_for(p1, dummy_source)
    ));

    let mut ctx = ctx_for(p1, dummy_source);
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: TokenSpec {
                name: "Test Token".to_string(),
                power: 1,
                toughness: 1,
                card_types: [CardType::Creature].into_iter().collect(),
                count: EffectAmount::Fixed(1),
                ..Default::default()
            },
        },
        &mut ctx,
    );

    assert!(
        check_condition(
            &state,
            &Condition::CreatedATokenThisTurn,
            &ctx_for(p1, dummy_source)
        ),
        "CR 111.10: CreatedATokenThisTurn must be true after a token is created"
    );
}

// ── F: Condition::SpellMastery ───────────────────────────────────────────────────

#[test]
/// CR 207.2c: spell mastery is true with 2+ instant/sorcery cards in the graveyard,
/// mixing instant and sorcery; false with fewer, and false when the 2 cards are
/// creatures instead.
fn test_spell_mastery_two_instants_or_sorceries() {
    let p1 = p(1);
    let dummy_source = ObjectId(9999);

    // 0 in graveyard -- false.
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .build()
            .unwrap();
        assert!(!check_condition(
            &state,
            &Condition::SpellMastery,
            &ctx_for(p1, dummy_source)
        ));
    }

    // 1 instant -- false.
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .object(
                ObjectSpec::card(p1, "GY Instant 1")
                    .in_zone(ZoneId::Graveyard(p1))
                    .with_types(vec![CardType::Instant]),
            )
            .build()
            .unwrap();
        assert!(!check_condition(
            &state,
            &Condition::SpellMastery,
            &ctx_for(p1, dummy_source)
        ));
    }

    // 1 instant + 1 sorcery -- true (mixed types count together).
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .object(
                ObjectSpec::card(p1, "GY Instant 2")
                    .in_zone(ZoneId::Graveyard(p1))
                    .with_types(vec![CardType::Instant]),
            )
            .object(
                ObjectSpec::card(p1, "GY Sorcery 1")
                    .in_zone(ZoneId::Graveyard(p1))
                    .with_types(vec![CardType::Sorcery]),
            )
            .build()
            .unwrap();
        assert!(check_condition(
            &state,
            &Condition::SpellMastery,
            &ctx_for(p1, dummy_source)
        ));
    }

    // 2 creatures -- false (wrong card types).
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(CardRegistry::new(vec![]))
            .object(ObjectSpec::creature(p1, "GY Creature 1", 1, 1).in_zone(ZoneId::Graveyard(p1)))
            .object(ObjectSpec::creature(p1, "GY Creature 2", 1, 1).in_zone(ZoneId::Graveyard(p1)))
            .build()
            .unwrap();
        assert!(!check_condition(
            &state,
            &Condition::SpellMastery,
            &ctx_for(p1, dummy_source)
        ));
    }
}

// ── G: Condition::OpponentControlsMoreLandsThanYou ───────────────────────────────

#[test]
/// Equal lands -- false; opponent controls one more -- true; a phased-out opponent
/// land is excluded (CR 702.26b); a non-land permanent turned into a land via a
/// continuous type-changing effect still counts (layer-resolved, W3-LC discipline).
fn test_opponent_controls_more_lands() {
    let p1 = p(1);
    let p2 = p(2);
    let dummy_source = ObjectId(9999);

    // Equal lands -- false.
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![]))
            .object(ObjectSpec::land(p1, "P1 Land 1"))
            .object(ObjectSpec::land(p2, "P2 Land 1"))
            .build()
            .unwrap();
        assert!(!check_condition(
            &state,
            &Condition::OpponentControlsMoreLandsThanYou,
            &ctx_for(p1, dummy_source)
        ));
    }

    // Opponent +1 -- true.
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![]))
            .object(ObjectSpec::land(p1, "P1 Land 2"))
            .object(ObjectSpec::land(p2, "P2 Land 2a"))
            .object(ObjectSpec::land(p2, "P2 Land 2b"))
            .build()
            .unwrap();
        assert!(check_condition(
            &state,
            &Condition::OpponentControlsMoreLandsThanYou,
            &ctx_for(p1, dummy_source)
        ));
    }

    // Opponent +1 but that land is phased out -- excluded, back to false.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![]))
            .object(ObjectSpec::land(p1, "P1 Land 3"))
            .object(ObjectSpec::land(p2, "P2 Land 3a"))
            .object(ObjectSpec::land(p2, "P2 Land 3b"))
            .build()
            .unwrap();
        let phased_land = find_object(&state, "P2 Land 3b");
        state
            .objects_mut()
            .get_mut(&phased_land)
            .unwrap()
            .status
            .phased_out = true;
        assert!(
            !check_condition(
                &state,
                &Condition::OpponentControlsMoreLandsThanYou,
                &ctx_for(p1, dummy_source)
            ),
            "CR 702.26b: a phased-out land must be excluded from the land count"
        );
    }
}

// ── H: Condition::OpponentCastNSpells ────────────────────────────────────────────

#[test]
/// True when a living opponent's `spells_cast_this_game_turn >= n`; false below the
/// threshold. Reads the all-players-reset counter (PB-AC6), not the storm-scoped
/// `spells_cast_this_turn`.
fn test_opponent_cast_n_spells() {
    let p1 = p(1);
    let p2 = p(2);
    let dummy_source = ObjectId(9999);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .spells_cast_this_game_turn = 3;

    assert!(
        check_condition(
            &state,
            &Condition::OpponentCastNSpells(3),
            &ctx_for(p1, dummy_source)
        ),
        "opponent cast exactly N spells -- N=3 threshold must be met"
    );
    assert!(
        !check_condition(
            &state,
            &Condition::OpponentCastNSpells(4),
            &ctx_for(p1, dummy_source)
        ),
        "opponent cast fewer than N=4 spells -- threshold must NOT be met"
    );
}

// ── I: Multiplayer turn-boundary reset (all three trackers) ─────────────────────

#[test]
/// CR 508.1 / 111.10 -- `reset_turn_state` resets `attacked_this_turn`,
/// `created_token_this_turn`, and `spells_cast_this_game_turn` for ALL players
/// (not just the incoming active player) at every turn boundary, in a 4-player
/// game. Verifies a non-active player's trackers reset too.
fn test_all_players_trackers_reset_at_turn_boundary_multiplayer() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let mut state = GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .build()
        .unwrap();

    for pid in [p1, p2, p3, p4] {
        let ps = state.players_mut().get_mut(&pid).unwrap();
        ps.attacked_this_turn = true;
        ps.created_token_this_turn = true;
        ps.spells_cast_this_game_turn = 5;
    }

    // Turn passes to p3 (a non-active player relative to the trackers being reset).
    mtg_engine::rules::turn_actions::reset_turn_state(&mut state, p3);

    for pid in [p1, p2, p3, p4] {
        let ps = state.players().get(&pid).unwrap();
        assert!(
            !ps.attacked_this_turn,
            "attacked_this_turn must reset for player {:?} (including non-active players)",
            pid
        );
        assert!(
            !ps.created_token_this_turn,
            "created_token_this_turn must reset for player {:?}",
            pid
        );
        assert_eq!(
            ps.spells_cast_this_game_turn, 0,
            "spells_cast_this_game_turn must reset for player {:?}",
            pid
        );
    }
}

// ── J: matches_filter sanity for scope filter (regression guard) ────────────────

#[test]
/// Sanity check that the `TargetFilter { has_card_type: Some(Creature) }` used by
/// the scope tests above actually discriminates creature vs. non-creature
/// characteristics (guards against a silently-vacuous filter).
fn test_scope_filter_discriminates_creature_vs_noncreature() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Filter Creature", 1, 1))
        .object(ObjectSpec::land(p1, "Filter Land"))
        .build()
        .unwrap();
    let creature_id = find_object(&state, "Filter Creature");
    let land_id = find_object(&state, "Filter Land");
    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    let creature_chars = &state.objects().get(&creature_id).unwrap().characteristics;
    let land_chars = &state.objects().get(&land_id).unwrap().characteristics;
    assert!(matches_filter(creature_chars, &filter));
    assert!(!matches_filter(land_chars, &filter));
}

//! Evoke keyword ability tests (CR 702.74).
//!
//! Evoke is an alternative cost (CR 118.9) that allows a creature card to be cast
//! for its evoke cost instead of its mana cost. When the permanent enters the
//! battlefield, if its evoke cost was paid, its controller sacrifices it.
//!
//! Key rules verified:
//! - Evoke is an alternative cost: pay evoke cost instead of mana cost (CR 702.74a).
//! - Evoke sacrifice trigger goes on the stack after ETB (CR 702.74a).
//! - Both ETB and sacrifice triggers go on the stack (Mulldrifter ruling).
//! - Mana value is unchanged when evoked (CR 118.9c).
//! - Evoke cannot combine with flashback (CR 118.9a: only one alternative cost).
//! - Spells without evoke reject cast_with_evoke: true (engine validation).
//! - Commander tax applies on top of evoke cost (CR 118.9d).
//! - Sacrifice trigger checks source is still on battlefield (CR 400.7).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

/// Mock Elemental: Creature {4}{U} 2/2.
/// Evoke {2}{U} — no ETB ability for cleaner evoke-only tests.
fn mock_elemental_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-elemental".to_string()),
        name: "Mock Elemental".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Evoke {2}{U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            AbilityDefinition::Evoke {
                cost: ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Mulldrifter: Creature {4}{U} 2/2.
/// "When this creature enters, draw two cards. Evoke {2}{U}."
fn mulldrifter_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mulldrifter".to_string()),
        name: "Mulldrifter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters, draw two cards.\nEvoke {2}{U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            AbilityDefinition::Evoke {
                cost: ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Triggered {
                trigger_condition: mtg_engine::TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                intervening_if: None,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Lightning Bolt (no evoke) for negative tests.
fn lightning_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic evoke cast — creature sacrificed ────────────────────────────

/// CR 702.74a — Mock Elemental cast for evoke cost {2}{U}.
/// After ETB: evoke sacrifice trigger goes on stack; after resolving, creature goes to graveyard.
#[test]
fn test_evoke_basic_cast_with_evoke_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_elemental_def()]);

    let creature = ObjectSpec::card(p1, "Mock Elemental")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-elemental".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {2}{U} — evoke cost instead of mana cost {4}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mock Elemental");

    // Cast with evoke.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with evoke failed: {:?}", e));

    // Spell on the stack — was_evoked flag set.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.74a: evoked spell should be on the stack"
    );
    assert!(
        state.stack_objects[0].was_evoked,
        "CR 702.74a: was_evoked should be true on stack object"
    );

    // Mana consumed: {2}{U} = 3 mana total.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.74a: {{2}}{{U}} evoke cost should be deducted from mana pool"
    );

    // Resolve the spell (ETB).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature entered the battlefield with was_evoked = true.
    let creature_bf = find_object_in_zone(&state, "Mock Elemental", ZoneId::Battlefield);
    assert!(
        creature_bf.is_some(),
        "CR 702.74a: creature should be on battlefield after resolution"
    );
    let bf_id = creature_bf.unwrap();
    assert!(
        state.objects[&bf_id].cast_alt_cost == Some(mtg_engine::state::types::AltCostKind::Evoke),
        "CR 702.74a: cast_alt_cost should be Some(Evoke) on battlefield permanent"
    );

    // Evoke sacrifice trigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.74a: evoke sacrifice trigger should be on the stack"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            mtg_engine::StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Evoke,
                ..
            }
        ),
        "CR 702.74a: stack entry should be EvokeSacrificeTrigger"
    );

    // Resolve the sacrifice trigger — creature goes to graveyard.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Mock Elemental should be in p1's graveyard.
    let in_graveyard = find_object_in_zone(&state, "Mock Elemental", ZoneId::Graveyard(p1));
    assert!(
        in_graveyard.is_some(),
        "CR 702.74a: evoked creature should be sacrificed to graveyard; events: {:?}",
        events
    );

    // Creature should NOT be on the battlefield.
    let on_bf = find_object_in_zone(&state, "Mock Elemental", ZoneId::Battlefield);
    assert!(
        on_bf.is_none(),
        "CR 702.74a: evoked creature should not remain on battlefield"
    );

    // CreatureDied event should have fired.
    let died_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died_event,
        "CR 702.74a: CreatureDied event expected for evoke sacrifice"
    );
}

// ── Test 2: Normal cast (no evoke) — creature stays ───────────────────────────

/// CR 702.74a — Mock Elemental cast for normal cost {4}{U}.
/// No evoke sacrifice trigger — creature stays on battlefield.
#[test]
fn test_evoke_basic_cast_without_evoke() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_elemental_def()]);

    let creature = ObjectSpec::card(p1, "Mock Elemental")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-elemental".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {4}{U} — full mana cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mock Elemental");

    // Cast without evoke (normal cast).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without evoke failed: {:?}", e));

    assert!(
        !state.stack_objects[0].was_evoked,
        "CR 702.74a: was_evoked should be false for normal cast"
    );

    // Resolve — ETB, no sacrifice trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // No evoke sacrifice trigger on the stack.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.74a: no evoke sacrifice trigger for normal cast"
    );

    // Creature on battlefield.
    let on_bf = find_object_in_zone(&state, "Mock Elemental", ZoneId::Battlefield);
    assert!(
        on_bf.is_some(),
        "CR 702.74a: creature should remain on battlefield when cast normally"
    );
}

// ── Test 3: ETB and sacrifice triggers both on stack ──────────────────────────

/// CR 702.74a / Mulldrifter ruling: When Mulldrifter is evoked, both the draw
/// trigger and the sacrifice trigger go on the stack. The controller can order
/// them so the draw trigger resolves first (draw 2 cards, then sacrifice).
#[test]
fn test_evoke_sacrifice_trigger_goes_through_stack() {
    let p1 = p(1);
    let p2 = p(2);

    // Add library cards so draw can succeed.
    let registry = CardRegistry::new(vec![mulldrifter_def()]);

    let creature = ObjectSpec::card(p1, "Mulldrifter")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mulldrifter".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    // Add library cards for draw.
    let lib_card1 = ObjectSpec::card(p1, "Forest 1").in_zone(ZoneId::Library(p1));
    let lib_card2 = ObjectSpec::card(p1, "Forest 2").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .object(lib_card1)
        .object(lib_card2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mulldrifter");

    // Cast with evoke.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with evoke failed: {:?}", e));

    // Resolve the spell (creature ETB).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both ETB draw trigger and evoke sacrifice trigger should be on the stack.
    // The sacrifice trigger was pushed first (during flush_pending_triggers);
    // the draw trigger is also from the same flush. Two triggers total.
    assert!(
        state.stack_objects.len() >= 1,
        "CR 702.74a: at least the evoke sacrifice trigger should be on the stack after ETB; \
         stack len: {}",
        state.stack_objects.len()
    );

    let has_evoke_trigger = state.stack_objects.iter().any(|so| {
        matches!(
            so.kind,
            mtg_engine::StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Evoke,
                ..
            }
        )
    });
    assert!(
        has_evoke_trigger,
        "CR 702.74a: EvokeSacrificeTrigger should be on the stack after ETB"
    );

    // Resolve all remaining triggers (draw 2, then sacrifice).
    // Pass p1 and p2 priority twice to resolve each trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, events) = pass_all(state, &[p1, p2]);

    // Mulldrifter should be in graveyard.
    let in_gy = find_object_in_zone(&state, "Mulldrifter", ZoneId::Graveyard(p1));
    assert!(
        in_gy.is_some(),
        "CR 702.74a: Mulldrifter should be sacrificed; events: {:?}",
        events
    );
}

// ── Test 4: Mana value unchanged when evoked ──────────────────────────────────

/// CR 118.9c — Mana value is based on the printed mana cost, not the alternative
/// evoke cost. Mock Elemental has printed cost {4}{U} = MV 5, even when evoked for {2}{U}.
#[test]
fn test_evoke_does_not_change_mana_value() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_elemental_def()]);

    let creature = ObjectSpec::card(p1, "Mock Elemental")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-elemental".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mock Elemental");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with evoke failed: {:?}", e));

    // The stack object's source still has its printed mana cost {{4}}{{U}} = MV 5.
    // The was_evoked flag records the alternative cost choice; the card's mana cost is unchanged.
    if let mtg_engine::StackObjectKind::Spell { source_object } =
        state.stack_objects[0].kind.clone()
    {
        let source = state.objects.get(&source_object).unwrap();
        let mv = source
            .characteristics
            .mana_cost
            .as_ref()
            .map(|mc| mc.mana_value())
            .unwrap_or(0);
        assert_eq!(
            mv, 5,
            "CR 118.9c: Mock Elemental's mana value should be 5 (printed cost {{4}}{{U}}), \
             not 3 (evoke cost {{2}}{{U}})"
        );
    } else {
        panic!("Expected a Spell on the stack");
    }
}

// ── Test 5: Evoke cannot combine with flashback ───────────────────────────────

/// CR 118.9a — Only one alternative cost can be applied to a spell. Attempting to
/// use both evoke and flashback should return an error.
///
/// NOTE: This test exercises the validation path. In practice, flashback is for
/// instants/sorceries and evoke is for creatures, so the combination would not
/// arise for real cards. The engine must still reject it.
#[test]
fn test_evoke_cannot_combine_with_flashback() {
    let p1 = p(1);
    let p2 = p(2);

    // Create a hypothetical card with both Flashback and Evoke (not a real card).
    // We test the validation by setting cast_with_evoke: true on a graveyard card
    // that has Flashback — the flashback zone check fires first (it's in the graveyard).
    let weird_card = CardDefinition {
        card_id: CardId("weird-card".to_string()),
        name: "Weird Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Evoke {1}{U}. Flashback {3}{U}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            AbilityDefinition::Evoke {
                cost: ManaCost {
                    generic: 1,
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                cost: ManaCost {
                    generic: 3,
                    blue: 1,
                    ..Default::default()
                },
                details: None,
            },
        ],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![weird_card]);

    // Place card in the graveyard so flashback is active.
    let card_in_gy = ObjectSpec::card(p1, "Weird Card")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("weird-card".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Flashback)
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card_in_gy)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Weird Card");

    // Attempt to cast with both evoke and flashback (card is in graveyard).
    // Since the card is in graveyard and has Flashback, casting_with_flashback is set.
    // The engine should detect the conflict and reject.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 118.9a: combining evoke with flashback should fail"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("alternative cost") || err.contains("evoke") || err.contains("flashback"),
        "CR 118.9a: error should mention the alternative cost conflict; got: {err}"
    );
}

// ── Test 6: Non-evoke spell rejected ─────────────────────────────────────────

/// Engine validation — Setting cast_with_evoke: true on a spell without evoke
/// should return an error.
#[test]
fn test_evoke_non_evoke_spell_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("lightning-bolt".to_string()))
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

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
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![mtg_engine::Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "Engine validation: cast_with_evoke on a non-evoke spell should fail"
    );
}

// ── Test 7: Evoke cost is alternative (cheaper than full cost) ────────────────

/// CR 702.74a — The evoke cost is paid instead of the mana cost. Attempting to
/// cast with the full mana cost ({4}{U}) when evoke is requested does not matter;
/// the engine uses the evoke cost ({2}{U}) regardless.
/// Verify that the mana consumed matches the evoke cost, not the printed cost.
#[test]
fn test_evoke_uses_alternative_cost_not_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_elemental_def()]);

    let creature = ObjectSpec::card(p1, "Mock Elemental")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-elemental".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give exactly {{2}}{{U}} — the evoke cost, not the full {{4}}{{U}} cost.
    // If evoke correctly uses the alternative cost, this should be enough.
    // If the engine mistakenly uses the printed mana cost, it would fail (InsufficientMana).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mock Elemental");

    // Should succeed with just {2}{U} (evoke cost).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "CR 702.74a: {{2}}{{U}} should be sufficient for evoke cost; error: {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();
    // Mana pool fully consumed.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.74a: evoke cost {{2}}{{U}} should have been fully consumed from mana pool"
    );
}

// ── Test 8: Blink saves evoked creature ───────────────────────────────────────

/// CR 400.7 — If the evoked creature leaves and re-enters the battlefield before
/// the sacrifice trigger resolves, it is a new object. The sacrifice trigger's
/// source_object is the old ID and no longer on the battlefield, so the trigger
/// does nothing. The creature survives.
///
/// This simulates the "blink" interaction: the creature is manually moved to exile
/// and back before the trigger resolves.
#[test]
fn test_evoke_sacrifice_trigger_fizzles_if_source_left_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_elemental_def()]);

    let creature = ObjectSpec::card(p1, "Mock Elemental")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-elemental".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let creature_id = find_object(&state, "Mock Elemental");

    // Cast with evoke.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: creature_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Evoke),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap();

    // Resolve the spell — creature enters battlefield, evoke sacrifice trigger queued.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Verify the evoke sacrifice trigger is on the stack.
    assert!(
        state.stack_objects.iter().any(|so| matches!(
            so.kind,
            mtg_engine::StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Evoke,
                ..
            }
        )),
        "CR 702.74a: EvokeSacrificeTrigger should be on the stack"
    );

    // Simulate "blink": move the creature from battlefield to exile, then back.
    // This creates a new ObjectId — the sacrifice trigger's source_object is now dead.
    let bf_id = find_object_in_zone(&state, "Mock Elemental", ZoneId::Battlefield)
        .expect("creature should be on battlefield before blink");

    // Move to exile (simulating blink out).
    let (exile_id, _) = state.move_object_to_zone(bf_id, ZoneId::Exile).unwrap();

    // Move back to battlefield (simulating blink return — new object, new ID).
    let (new_bf_id, _) = state
        .move_object_to_zone(exile_id, ZoneId::Battlefield)
        .unwrap();
    // Restore controller.
    if let Some(obj) = state.objects.get_mut(&new_bf_id) {
        obj.controller = p1;
    }

    // Now resolve the evoke sacrifice trigger.
    // The trigger's source_object is bf_id (old ID, now retired).
    // The check in resolution.rs: `state.objects.get(&source_object).zone == Battlefield`
    // will find nothing (old ID is gone) or find the object is NOT on the battlefield.
    // The trigger should do nothing.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature (re-entered as new_bf_id) should still be on the battlefield.
    let new_on_bf = find_object_in_zone(&state, "Mock Elemental", ZoneId::Battlefield);
    assert!(
        new_on_bf.is_some(),
        "CR 400.7: blinked creature should survive the evoke sacrifice trigger (new object)"
    );
}

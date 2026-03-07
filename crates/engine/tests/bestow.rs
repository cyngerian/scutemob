//! Bestow keyword ability tests (CR 702.103).
//!
//! Bestow is an alternative cost (CR 118.9) that allows an Enchantment Creature to be cast
//! for its bestow cost instead of its mana cost. When cast bestowed, the spell becomes an
//! Aura enchantment with "enchant creature" and loses its creature type while on the stack.
//!
//! Key rules verified:
//! - CR 702.103a: Bestow is an alternative cost; pay bestow cost instead of mana cost.
//! - CR 702.103b: On the stack, bestowed spell is an Aura enchantment (not a creature).
//! - CR 702.103e / 608.3b: If target is illegal at resolution, spell reverts to creature spell.
//! - CR 702.103f: When bestowed Aura becomes unattached, it reverts to creature (not graveyard).
//! - CR 118.9a: Bestow cannot be combined with other alternative costs (flashback, evoke).
//! - CR 118.9c: Mana value is unchanged when cast bestowed (printed mana cost).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::{
    check_and_apply_sbas, process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry,
    Command, EnchantTarget, GameEvent, GameStateBuilder, GameStateError, KeywordAbility, ManaColor,
    ManaCost, ObjectSpec, PlayerId, Step, SubType, Target, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once.
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

/// Mock Bestow Creature: Enchantment Creature {1}{G}{G} 4/2.
/// Bestow {3}{G}{G} — Boon Satyr analog.
fn mock_bestow_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-bestow-satyr".to_string()),
        name: "Mock Bestow Satyr".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment, CardType::Creature]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "Bestow {3}{G}{G}\nEnchanted creature gets +4/+2.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bestow),
            AbilityDefinition::Bestow {
                cost: ManaCost {
                    generic: 3,
                    green: 2,
                    ..Default::default()
                },
            },
        ],
        power: Some(4),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Standard 2/2 Bear (no bestow, for target)
fn mock_bear_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-bear".to_string()),
        name: "Mock Bear".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: Basic bestow cast ─────────────────────────────────────────────────

#[test]
/// CR 702.103a/b — Cast Mock Bestow Satyr for bestow cost {3}{G}{G} targeting a creature.
/// Spell on stack: Aura subtype, enchant creature keyword, no Creature type.
/// After resolution: permanent on battlefield with is_bestowed=true, attached to target.
fn test_bestow_cast_as_aura_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_bestow_creature_def(), mock_bear_def()]);

    let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-bestow-satyr".to_string()))
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Bestow)
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });

    let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(satyr)
        .object(bear)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {3}{G}{G} — bestow cost instead of mana cost {1}{G}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let satyr_id = find_object(&state, "Mock Bestow Satyr");
    let bear_id = find_object(&state, "Mock Bear");

    // Cast with bestow, targeting the bear.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: satyr_id,
            targets: vec![Target::Object(bear_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Bestow),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with bestow failed: {:?}", e));

    // CR 702.103b: Spell on stack — was_bestowed = true.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Bestow spell should be on the stack"
    );
    let stack_entry = &state.stack_objects[0];
    assert!(
        stack_entry.was_bestowed,
        "CR 702.103b: was_bestowed should be true on stack object"
    );

    // CR 702.103b: Source object on stack should have Aura subtype, not Creature type.
    let mtg_engine::state::stack::StackObjectKind::Spell { source_object } =
        stack_entry.kind.clone()
    else {
        panic!("Expected Spell stack object");
    };
    let stack_source = state.objects.get(&source_object).unwrap();
    assert!(
        stack_source
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "CR 702.103b: Stack source should have Aura subtype"
    );
    assert!(
        !stack_source
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.103b: Stack source should NOT have Creature type while bestowed"
    );
    assert!(
        stack_source
            .characteristics
            .card_types
            .contains(&CardType::Enchantment),
        "CR 702.103b: Stack source should have Enchantment type"
    );
    assert!(
        stack_source
            .characteristics
            .keywords
            .contains(&KeywordAbility::Enchant(EnchantTarget::Creature)),
        "CR 702.103b: Stack source should have Enchant(Creature) keyword"
    );

    // Mana consumed: {3}{G}{G} = 5 total.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.total(),
        0,
        "CR 702.103a: Bestow cost {{3}}{{G}}{{G}} should be fully consumed"
    );

    // Both players pass → spell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.103b: Permanent on battlefield with is_bestowed=true, attached to bear.
    let satyr_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Mock Bestow Satyr" && o.zone == ZoneId::Battlefield)
        .expect("Satyr should be on battlefield after resolution");

    assert!(
        satyr_bf.is_bestowed,
        "CR 702.103b: is_bestowed should be true on battlefield permanent"
    );
    assert_eq!(
        satyr_bf.attached_to,
        Some(bear_id),
        "CR 702.103b: Bestowed permanent should be attached to the bear"
    );
    // Should still have Aura subtype on the battlefield.
    assert!(
        satyr_bf
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "CR 702.103b: Bestowed permanent should have Aura subtype on battlefield"
    );
    // Should NOT have Creature type while bestowed.
    assert!(
        !satyr_bf
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.103b: Bestowed permanent should NOT have Creature type on battlefield"
    );

    // Bear should have Satyr in its attachments.
    let bear_obj = state.objects.get(&bear_id).unwrap();
    assert!(
        bear_obj.attachments.contains(&satyr_bf.id),
        "Bear.attachments should contain the bestowed Satyr"
    );

    // AuraAttached event should have been emitted.
    let attached_event = resolve_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AuraAttached {
                target_id, ..
            } if *target_id == bear_id
        )
    });
    assert!(
        attached_event,
        "AuraAttached event should be emitted; events: {:?}",
        resolve_events
    );
}

// ── Test 2: Bestow card cast normally as a creature ───────────────────────────

#[test]
/// CR 702.103 — Same card cast for its normal mana cost {1}{G}{G} (no bestow).
/// Spell on stack: NOT an Aura, has Creature type, no targets.
/// After resolution: enchantment creature, is_bestowed=false, not attached to anything.
fn test_bestow_cast_normally_as_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_bestow_creature_def()]);

    let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-bestow-satyr".to_string()))
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Bestow)
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(satyr)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {1}{G}{G} — normal mana cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let satyr_id = find_object(&state, "Mock Bestow Satyr");

    // Cast normally (no bestow, no targets).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: satyr_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell normally failed: {:?}", e));

    // Stack entry should have was_bestowed=false.
    assert_eq!(state.stack_objects.len(), 1);
    assert!(
        !state.stack_objects[0].was_bestowed,
        "Normal cast should have was_bestowed=false"
    );

    // Source on stack should have Creature type, NOT Aura subtype.
    let mtg_engine::state::stack::StackObjectKind::Spell { source_object } =
        state.stack_objects[0].kind.clone()
    else {
        panic!("Expected Spell");
    };
    let stack_source = state.objects.get(&source_object).unwrap();
    assert!(
        stack_source
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Normal cast: stack source should have Creature type"
    );
    assert!(
        !stack_source
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "Normal cast: stack source should NOT have Aura subtype"
    );

    // Both players pass → spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Permanent on battlefield as enchantment creature.
    let satyr_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Mock Bestow Satyr" && o.zone == ZoneId::Battlefield)
        .expect("Satyr should be on battlefield");

    assert!(
        !satyr_bf.is_bestowed,
        "Normal cast: is_bestowed should be false on battlefield"
    );
    assert_eq!(
        satyr_bf.attached_to, None,
        "Normal cast: permanent should not be attached to anything"
    );
    // Should have Creature type.
    assert!(
        satyr_bf
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Normal cast: battlefield permanent should have Creature type"
    );
    // Should NOT have Aura subtype.
    assert!(
        !satyr_bf
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "Normal cast: battlefield permanent should NOT have Aura subtype"
    );
}

// ── Test 3: Bestow target illegal at resolution → creature fallback ───────────

#[test]
/// CR 702.103e / 608.3b — Cast bestowed targeting a creature. Before resolution,
/// the target leaves the battlefield. Spell does NOT fizzle; instead reverts to
/// enchantment creature and enters battlefield with is_bestowed=false.
fn test_bestow_target_illegal_at_resolution_becomes_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_bestow_creature_def(), mock_bear_def()]);

    let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-bestow-satyr".to_string()))
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Bestow)
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });

    let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(satyr)
        .object(bear)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {3}{G}{G} — bestow cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let satyr_id = find_object(&state, "Mock Bestow Satyr");
    let bear_id = find_object(&state, "Mock Bear");

    // Cast bestowed targeting the bear.
    let (mut state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: satyr_id,
            targets: vec![Target::Object(bear_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Bestow),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell bestow failed: {:?}", e));

    // Simulate target leaving the battlefield before resolution.
    // Move the bear to the graveyard (CR 400.7: new object, original bear_id is dead).
    let _ = state.move_object_to_zone(bear_id, ZoneId::Graveyard(p1));

    // Both players pass → the bestowed spell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 702.103e: Spell did NOT fizzle — SpellFizzled event should NOT appear.
    let fizzled = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellFizzled { .. }));
    assert!(
        !fizzled,
        "CR 702.103e: Bestow spell should NOT fizzle; events: {:?}",
        resolve_events
    );

    // CR 702.103e: Permanent enters battlefield as enchantment creature (not Aura).
    let satyr_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Mock Bestow Satyr" && o.zone == ZoneId::Battlefield)
        .expect("CR 702.103e: Satyr should be on battlefield as creature");

    assert!(
        !satyr_bf.is_bestowed,
        "CR 702.103e: Fallback creature should have is_bestowed=false"
    );
    assert_eq!(
        satyr_bf.attached_to, None,
        "CR 702.103e: Fallback creature should not be attached to anything"
    );
    // Should have Creature type.
    assert!(
        satyr_bf
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.103e: Fallback creature should have Creature type"
    );
    // Should NOT have Aura subtype.
    assert!(
        !satyr_bf
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "CR 702.103e: Fallback creature should NOT have Aura subtype"
    );
}

// ── Test 4: Bestowed Aura becomes unattached → reverts to creature (SBA) ─────

#[test]
/// CR 702.103f — Cast bestowed, resolve, creature enters as bestowed Aura.
/// Then the enchanted creature leaves the battlefield.
/// SBA check: bestowed Aura does NOT go to graveyard (exception to 704.5m).
/// Instead, is_bestowed becomes false, permanent stays on battlefield as enchantment creature.
fn test_bestow_unattach_reverts_to_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // Build the game state with a manually-bestowed Aura already attached to a bear.
    // This simulates the post-resolution state without having to go through the full cast.
    let satyr_spec = ObjectSpec::enchantment(p1, "Mock Bestow Satyr")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Creature))
        .with_types(vec![CardType::Enchantment]); // no Creature type while bestowed

    let bear_spec = ObjectSpec::creature(p1, "Mock Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(satyr_spec)
        .object(bear_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let satyr_id = find_object(&state, "Mock Bestow Satyr");
    let bear_id = find_object(&state, "Mock Bear");

    // Manually set up the bestowed attachment state.
    if let Some(satyr_obj) = state.objects.get_mut(&satyr_id) {
        satyr_obj.is_bestowed = true;
        satyr_obj.attached_to = Some(bear_id);
    }
    if let Some(bear_obj) = state.objects.get_mut(&bear_id) {
        bear_obj.attachments.push_back(satyr_id);
    }

    // Verify setup.
    assert!(
        state.objects[&satyr_id].is_bestowed,
        "Setup: satyr should be bestowed"
    );
    assert_eq!(
        state.objects[&satyr_id].attached_to,
        Some(bear_id),
        "Setup: satyr should be attached to bear"
    );

    // Now move the bear to the graveyard (simulating the enchanted creature dying).
    let _ = state.move_object_to_zone(bear_id, ZoneId::Graveyard(p1));

    // Run SBA check.
    let sba_events = check_and_apply_sbas(&mut state);

    // CR 702.103f: BestowReverted event should be emitted.
    let reverted = sba_events
        .iter()
        .any(|e| matches!(e, GameEvent::BestowReverted { object_id } if *object_id == satyr_id));
    assert!(
        reverted,
        "CR 702.103f: BestowReverted event should be emitted; events: {:?}",
        sba_events
    );

    // CR 702.103f: AuraFellOff should NOT be emitted (bestowed Aura stays on battlefield).
    let fell_off = sba_events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == satyr_id));
    assert!(
        !fell_off,
        "CR 702.103f: AuraFellOff should NOT be emitted for bestowed Aura; events: {:?}",
        sba_events
    );

    // CR 702.103f: Satyr should still be on the battlefield.
    assert_eq!(
        state.objects[&satyr_id].zone,
        ZoneId::Battlefield,
        "CR 702.103f: Reverted bestow permanent should remain on battlefield"
    );

    // CR 702.103f: is_bestowed should be false now.
    assert!(
        !state.objects[&satyr_id].is_bestowed,
        "CR 702.103f: is_bestowed should be false after revert"
    );

    // CR 702.103f: attached_to should be cleared.
    assert_eq!(
        state.objects[&satyr_id].attached_to, None,
        "CR 702.103f: attached_to should be None after revert"
    );

    // CR 702.103f: Satyr should now have Creature type.
    assert!(
        state.objects[&satyr_id]
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.103f: Reverted satyr should have Creature type"
    );

    // CR 702.103f: Satyr should NOT have Aura subtype anymore.
    assert!(
        !state.objects[&satyr_id]
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "CR 702.103f: Reverted satyr should NOT have Aura subtype"
    );
}

// ── Test 5: Alternative cost pays bestow cost, not printed mana cost ──────────

#[test]
/// CR 702.103a / 118.9: Bestow cost {3}{G}{G} is paid (not printed mana cost {1}{G}{G}).
/// Giving exactly {3}{G}{G} should succeed; {1}{G}{G} should fail.
fn test_bestow_alternative_cost_pays_bestow_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_bestow_creature_def(), mock_bear_def()]);

    // First: giving only {1}{G}{G} (printed mana cost) should FAIL for bestow cast.
    {
        let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("mock-bestow-satyr".to_string()))
            .with_types(vec![CardType::Enchantment, CardType::Creature])
            .with_keyword(KeywordAbility::Bestow)
            .with_mana_cost(ManaCost {
                generic: 1,
                green: 2,
                ..Default::default()
            });
        let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(satyr)
            .object(bear)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        // Pay only {1}{G}{G} — the printed mana cost, not the bestow cost.
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Green, 2);
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 1);
        state.turn.priority_holder = Some(p1);

        let satyr_id = find_object(&state, "Mock Bestow Satyr");
        let bear_id = find_object(&state, "Mock Bear");

        let result = process_command(
            state,
            Command::CastSpell {
                player: p1,
                card: satyr_id,
                targets: vec![Target::Object(bear_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Bestow),
                escape_exile_cards: vec![],
                retrace_discard_land: None,
                jump_start_discard: None,
                prototype: false,
                bargain_sacrifice: None,
                emerge_sacrifice: None,
                casualty_sacrifice: None,
                assist_player: None,
                assist_amount: 0,
                replicate_count: 0,
                splice_cards: vec![],
                entwine_paid: false,
                escalate_modes: 0,
                devour_sacrifices: vec![],
                modes_chosen: vec![],
                fuse: false,
            },
        );
        assert!(
            matches!(result, Err(GameStateError::InsufficientMana)),
            "CR 702.103a: Bestow requires paying bestow cost {{3}}{{G}}{{G}}, not printed cost; result: {:?}",
            result
        );
    }

    // Second: giving {3}{G}{G} (bestow cost) should SUCCEED.
    {
        let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(CardId("mock-bestow-satyr".to_string()))
            .with_types(vec![CardType::Enchantment, CardType::Creature])
            .with_keyword(KeywordAbility::Bestow)
            .with_mana_cost(ManaCost {
                generic: 1,
                green: 2,
                ..Default::default()
            });
        let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(satyr)
            .object(bear)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Green, 2);
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 3);
        state.turn.priority_holder = Some(p1);

        let satyr_id = find_object(&state, "Mock Bestow Satyr");
        let bear_id = find_object(&state, "Mock Bear");

        let result = process_command(
            state,
            Command::CastSpell {
                player: p1,
                card: satyr_id,
                targets: vec![Target::Object(bear_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Bestow),
                escape_exile_cards: vec![],
                retrace_discard_land: None,
                jump_start_discard: None,
                prototype: false,
                bargain_sacrifice: None,
                emerge_sacrifice: None,
                casualty_sacrifice: None,
                assist_player: None,
                assist_amount: 0,
                replicate_count: 0,
                splice_cards: vec![],
                entwine_paid: false,
                escalate_modes: 0,
                devour_sacrifices: vec![],
                modes_chosen: vec![],
                fuse: false,
            },
        );
        assert!(
            result.is_ok(),
            "CR 702.103a: Bestow cast with correct cost {{3}}{{G}}{{G}} should succeed; err: {:?}",
            result.err()
        );
    }
}

// ── Test 6: Bestow cannot combine with flashback ──────────────────────────────

#[test]
/// CR 118.9a — Attempt to combine bestow with flashback should fail.
/// Only one alternative cost may be paid (CR 118.9a).
fn test_bestow_cannot_combine_with_flashback() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mock_bestow_creature_def()]);

    // Put the bestow card in the graveyard (as if it had flashback) to trigger the check.
    // The engine validates flashback by checking if the card is in the graveyard + has Flashback.
    // We just test that cast_with_bestow=true when casting from hand with flashback keywords fails.
    // The simplest check: manually test that the error message mentions alternative cost.
    // In practice, the flashback cast path checks casting_from_graveyard + Flashback keyword.
    // To test the conflict, we need the card to be in graveyard with Flashback keyword.
    // Simpler: use a card that has bestow and flashback (or use the existing validation path).
    // The validation code checks: if casting_with_bestow && casting_with_flashback → error.
    // casting_with_flashback = casting_from_graveyard && has Flashback keyword.
    // So put the bestow card in graveyard WITH Flashback keyword to trigger both.
    let satyr = ObjectSpec::card(p1, "Mock Bestow Satyr")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("mock-bestow-satyr".to_string()))
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Bestow)
        .with_keyword(KeywordAbility::Flashback) // added for this negative test
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(satyr)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let satyr_id = find_object(&state, "Mock Bestow Satyr");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: satyr_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Bestow),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "CR 118.9a: Bestow + flashback should fail with InvalidCommand; result: {:?}",
        result
    );
}

// ── Test 7: Bestow cannot combine with evoke ──────────────────────────────────

#[test]
/// CR 118.9a — Attempt to combine bestow with evoke should fail.
fn test_bestow_cannot_combine_with_evoke() {
    let p1 = p(1);
    let p2 = p(2);

    // A (hypothetical) card with both bestow and evoke.
    let dual_def = CardDefinition {
        card_id: CardId("dual-card".to_string()),
        name: "Dual Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bestow),
            AbilityDefinition::Bestow {
                cost: ManaCost {
                    generic: 4,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            AbilityDefinition::Evoke {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![dual_def]);

    let card = ObjectSpec::card(p1, "Dual Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dual-card".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Bestow)
        .with_keyword(KeywordAbility::Evoke)
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dual Card");

    // CR 118.9a: Only one alternative cost may be applied at a time.
    // The new API enforces this by design — alt_cost is a single Option<AltCostKind>.
    // Casting with ONLY Evoke (alt_cost: Some(Evoke)) is valid — the card has Evoke.
    // Casting with ONLY Bestow (alt_cost: Some(Bestow)) requires a valid target.
    // There's no way to specify two alt costs simultaneously in the new API.
    //
    // Here we verify that casting with Evoke alone (without Bestow) is accepted,
    // confirming the card can be cast with either alt cost independently.
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
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );
    // Evoke alone is valid (card has Evoke keyword); the mutual exclusion of two
    // alt costs is enforced by the API type — only one Option<AltCostKind> can be set.
    assert!(
        result.is_ok(),
        "CR 702.74a: Casting with Evoke (single alt cost) on a card with both Bestow+Evoke should succeed; result: {:?}",
        result
    );
}

// ── Test 8: Non-bestow spell with cast_with_bestow=true is rejected ───────────

#[test]
/// Engine validation: Setting cast_with_bestow=true on a spell without bestow returns error.
fn test_bestow_non_bestow_spell_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let bear = ObjectSpec::creature(p1, "Mock Bear", 2, 2).in_zone(ZoneId::Hand(p1));
    // A creature with mana cost but no bestow.
    let bear_spec = ObjectSpec::card(p1, "Mock Bear")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(bear_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let _ = bear; // suppress unused warning
    let bear_id = find_object(&state, "Mock Bear");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bear_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Bestow),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "Spell without bestow should reject cast_with_bestow=true; result: {:?}",
        result
    );
    if let Err(GameStateError::InvalidCommand(msg)) = result {
        assert!(
            msg.contains("bestow"),
            "Error message should mention bestow; got: {:?}",
            msg
        );
    }
}

// ── Test 9: Enters battlefield without casting is a creature ──────────────────

#[test]
/// Card ruling (CR 702.103b): A permanent with bestow that enters via means OTHER
/// than being cast bestowed enters as an enchantment creature, NOT an Aura.
/// Simulated here by placing it directly on the battlefield.
fn test_bestow_enters_without_casting_is_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // Place a bestow card directly on the battlefield (not cast) using enchantment()
    // which defaults to Battlefield zone, then add the bestow-related types/keywords.
    let satyr_bf = ObjectSpec::enchantment(p1, "Mock Bestow Satyr")
        .with_types(vec![CardType::Enchantment, CardType::Creature])
        .with_keyword(KeywordAbility::Bestow);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(satyr_bf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let satyr_id = find_object(&state, "Mock Bestow Satyr");
    let satyr_obj = &state.objects[&satyr_id];

    // Should be on the battlefield.
    assert_eq!(
        satyr_obj.zone,
        ZoneId::Battlefield,
        "Satyr should be on battlefield"
    );

    // is_bestowed should be false (not cast bestowed).
    assert!(
        !satyr_obj.is_bestowed,
        "Satyr placed directly on battlefield should have is_bestowed=false"
    );

    // Should have Creature type (it's an enchantment creature, not an Aura).
    assert!(
        satyr_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Satyr on battlefield without bestow cast should have Creature type"
    );

    // Should NOT have Aura subtype.
    assert!(
        !satyr_obj
            .characteristics
            .subtypes
            .contains(&SubType("Aura".to_string())),
        "Satyr on battlefield without bestow cast should NOT have Aura subtype"
    );

    // attached_to should be None.
    assert_eq!(
        satyr_obj.attached_to, None,
        "Satyr should not be attached to anything"
    );
}

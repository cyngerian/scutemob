//! Equip keyword ability tests (CR 702.6).
//!
//! Equip is an activated ability of Equipment cards:
//! "[Cost]: Attach this permanent to target creature you control.
//! Activate only as a sorcery."
//!
//! Key rules:
//! - CR 702.6a: Equip is a sorcery-speed activated ability.
//! - CR 301.5c: Equipment can't equip more than one creature; can't equip itself.
//! - CR 701.3b: Attaching to the same target does nothing.
//! - CR 701.3c: New timestamp on reattach (layer ordering).
//! - CR 602.5d: Sorcery-speed means active player's main phase, empty stack.

use mtg_engine::state::{
    ActivatedAbility, ActivationCost, ContinuousEffect, EffectId, GameStateError,
};
use mtg_engine::{
    calculate_characteristics, process_command, CardEffectTarget, CardType, Command, Effect,
    EffectDuration, EffectFilter, EffectLayer, GameEvent, GameStateBuilder, KeywordAbility,
    LayerModification, ManaColor, ManaCost, ObjectId, PlayerId, ProtectionQuality, Step, SubType,
    Target, ZoneId,
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

/// Build an ActivatedAbility representing Equip {N} (sorcery-speed, no tap required).
fn equip_ability(generic_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: false,
            mana_cost: if generic_mana > 0 {
                Some(ManaCost {
                    generic: generic_mana,
                    ..Default::default()
                })
            } else {
                None
            },
            sacrifice_self: false,
        },
        description: format!("Equip {{{}}}", generic_mana),
        effect: Some(Effect::AttachEquipment {
            equipment: CardEffectTarget::Source,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
        }),
        sorcery_speed: true,
    }
}

// ── Test 1: Basic equip attaches to creature ──────────────────────────────────

#[test]
/// CR 702.6a — equip attaches the equipment to the targeted creature you control.
fn test_equip_basic_attaches_to_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 mana to pay Equip {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    // Activate equip targeting the creature.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    // Pass priority to resolve (p1 then p2 pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Equipment is attached to the creature.
    let equip_obj = state.objects.get(&equip_id).expect("equipment exists");
    assert_eq!(
        equip_obj.attached_to,
        Some(creature_id),
        "equipment.attached_to should be the creature"
    );

    let creature_obj = state.objects.get(&creature_id).expect("creature exists");
    assert!(
        creature_obj.attachments.contains(&equip_id),
        "creature.attachments should contain the equipment"
    );

    // EquipmentAttached event emitted.
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::EquipmentAttached { equipment_id, target_id, controller }
            if *equipment_id == equip_id && *target_id == creature_id && *controller == p1
        )),
        "EquipmentAttached event expected"
    );
}

// ── Test 2: Sorcery-speed restriction (not main phase) ────────────────────────

#[test]
/// CR 702.6a, CR 602.5d — equip can only be activated during main phase.
fn test_equip_sorcery_speed_only() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers) // NOT main phase
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::NotMainPhase)),
        "equip during non-main phase should return NotMainPhase, got {:?}",
        result
    );
}

// ── Test 3: Sorcery-speed restriction (not active player) ────────────────────

#[test]
/// CR 702.6a, CR 602.5d — equip can only be activated during your own turn.
fn test_equip_sorcery_speed_not_active_player() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p2) // p2's turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 priority during p2's main phase.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "equip on opponent's turn should return InvalidCommand, got {:?}",
        result
    );
}

// ── Test 4: Sorcery-speed restriction (stack not empty) ───────────────────────

#[test]
/// CR 602.5d — equip can't be activated with spells on the stack.
fn test_equip_sorcery_speed_stack_not_empty() {
    use mtg_engine::{AbilityDefinition, CardDefinition, CardId, CardRegistry};

    let p1 = p(1);
    let p2 = p(2);

    // A simple instant so we can have something on the stack.
    let instant_def = CardDefinition {
        card_id: CardId("test-instant".to_string()),
        name: "Test Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Test instant.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![instant_def]);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let instant = mtg_engine::ObjectSpec::card(p2, "Test Instant")
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
        .object(equipment)
        .object(creature)
        .object(instant)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 casts instant (going on the stack).
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p2);
    let instant_id = find_object(&state, "Test Instant");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: instant_id,
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
    .unwrap();

    // Stack is now non-empty. p1 tries to activate equip.
    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    // p1 has priority after the cast.
    let mut state = state;
    state.turn.priority_holder = Some(p1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::StackNotEmpty)),
        "equip with non-empty stack should return StackNotEmpty, got {:?}",
        result
    );
}

// ── Test 5: Target must be controlled by activating player ────────────────────

#[test]
/// CR 702.6a — equip can only target "a creature you control."
/// Targeting an opponent's creature is rejected at activation time (before mana is spent).
///
/// This test was updated from the original "no-op at resolution" behavior after
/// MEDIUM 2 fix: activation-time target validation now rejects illegal targets
/// per CR 601.2c / CR 115.1 (targets must be legal when the ability is put on
/// the stack), preventing mana from being wasted on an illegal activation.
fn test_equip_target_opponent_creature_rejected_at_activation() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    // Creature controlled by p2.
    let creature = mtg_engine::ObjectSpec::creature(p2, "Opponent Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
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
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Opponent Bear");

    // Activation must be rejected: equip can only target creatures you control.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "equip targeting opponent's creature should be rejected at activation with InvalidTarget, got {:?}",
        result
    );
}

// ── Test 6: Re-equip detaches from previous creature ─────────────────────────

#[test]
/// CR 301.5c — equipment can't equip more than one creature;
/// moving it detaches from the previous creature.
fn test_equip_reequip_detaches_from_previous() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature_a = mtg_engine::ObjectSpec::creature(p1, "Creature A", 2, 2);
    let creature_b = mtg_engine::ObjectSpec::creature(p1, "Creature B", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature_a)
        .object(creature_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Test Sword");
    let creature_a_id = find_object(&state, "Creature A");
    let creature_b_id = find_object(&state, "Creature B");

    // Manually pre-attach to creature A.
    state.objects.get_mut(&equip_id).unwrap().attached_to = Some(creature_a_id);
    state
        .objects
        .get_mut(&creature_a_id)
        .unwrap()
        .attachments
        .push_back(equip_id);

    // Capture timestamp before re-equipping to creature B.
    // CR 701.3c: reattaching to a different object must assign a new (greater) timestamp.
    let timestamp_before_reequip = state.objects.get(&equip_id).unwrap().timestamp;

    // Now equip targeting creature B.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_b_id)],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // Equipment is now attached to creature B.
    assert_eq!(
        state.objects.get(&equip_id).unwrap().attached_to,
        Some(creature_b_id),
        "equipment should be attached to creature B"
    );

    // Creature A no longer has the equipment in its attachments.
    assert!(
        !state
            .objects
            .get(&creature_a_id)
            .unwrap()
            .attachments
            .contains(&equip_id),
        "creature A should not have equipment in attachments"
    );

    // Creature B has the equipment.
    assert!(
        state
            .objects
            .get(&creature_b_id)
            .unwrap()
            .attachments
            .contains(&equip_id),
        "creature B should have equipment in attachments"
    );

    // CR 701.3c / CR 613.7e: reattaching to a different object gives a new, strictly
    // greater timestamp (required for correct layer ordering).
    let timestamp_after_reequip = state.objects.get(&equip_id).unwrap().timestamp;
    assert!(
        timestamp_after_reequip > timestamp_before_reequip,
        "equipment timestamp should increase after re-equip (CR 701.3c): before={}, after={}",
        timestamp_before_reequip,
        timestamp_after_reequip
    );
}

// ── Test 7: Equipment can't equip itself ──────────────────────────────────────

#[test]
/// CR 301.5c — an Equipment can't equip itself.
/// If equipment = target, the effect is skipped.
fn test_equip_cannot_equip_self() {
    let p1 = p(1);
    let p2 = p(2);

    // Equipment that is also a creature (hypothetical — e.g., Colossus Hammer variant).
    // We simulate this by making it have both Artifact and Creature card types.
    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Living Sword")
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(0));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Living Sword");

    // Manually set power/toughness so the object is a valid creature for SBA purposes.
    // (We just need to avoid SBAs killing it before we can test.)
    if let Some(obj) = state.objects.get_mut(&equip_id) {
        obj.characteristics.power = Some(3);
        obj.characteristics.toughness = Some(3);
    }

    state.turn.priority_holder = Some(p1);

    // Activate equip targeting itself.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(equip_id)],
        },
    )
    .unwrap();

    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Effect does nothing: equipment can't equip itself.
    let equip_obj = state.objects.get(&equip_id).expect("equipment exists");
    assert_eq!(
        equip_obj.attached_to, None,
        "equipment should NOT be attached to itself"
    );

    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::EquipmentAttached { .. })),
        "EquipmentAttached should not fire when equipping self"
    );
}

// ── Test 8: Already attached to same target — no-op ──────────────────────────

#[test]
/// CR 701.3b — attaching an equipment to the same target is a no-op.
fn test_equip_already_attached_to_same_target_no_op() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    // Pre-attach to the creature.
    state.objects.get_mut(&equip_id).unwrap().attached_to = Some(creature_id);
    state
        .objects
        .get_mut(&creature_id)
        .unwrap()
        .attachments
        .push_back(equip_id);

    let old_timestamp = state.objects.get(&equip_id).unwrap().timestamp;

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // Activate equip targeting the same creature.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // State unchanged: attached_to still same creature.
    assert_eq!(
        state.objects.get(&equip_id).unwrap().attached_to,
        Some(creature_id),
        "equipment should still be attached to the same creature"
    );

    // Creature still has exactly one attachment entry (no duplicate).
    let attachment_count = state
        .objects
        .get(&creature_id)
        .unwrap()
        .attachments
        .iter()
        .filter(|&&x| x == equip_id)
        .count();
    assert_eq!(
        attachment_count, 1,
        "creature should have exactly one attachment entry"
    );

    // No EquipmentAttached event for a no-op.
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::EquipmentAttached { .. })),
        "EquipmentAttached should NOT fire for a no-op re-equip"
    );

    // CR 701.3b / CR 701.3c: attaching to the same target is a no-op; the timestamp
    // must NOT be updated (no reattachment happened).
    let new_timestamp = state.objects.get(&equip_id).unwrap().timestamp;
    assert_eq!(
        new_timestamp, old_timestamp,
        "equipment timestamp should NOT change when re-equipping to the same target (CR 701.3b)"
    );
}

// ── Test 9: Pays mana cost ────────────────────────────────────────────────────

#[test]
/// CR 702.6a — equip activation pays the mana cost from the player's pool.
fn test_equip_pays_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 exactly 2 generic mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    // ManaCostPaid event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)),
        "ManaCostPaid event expected"
    );

    // Mana pool is now empty.
    let mana_pool = &state.players.get(&p1).unwrap().mana_pool;
    assert_eq!(
        mana_pool.colorless
            + mana_pool.white
            + mana_pool.blue
            + mana_pool.black
            + mana_pool.red
            + mana_pool.green,
        0,
        "mana pool should be empty after paying equip cost"
    );
}

// ── Test 10: Insufficient mana is rejected ────────────────────────────────────

#[test]
/// CR 702.6a — equip activation with insufficient mana returns InsufficientMana.
fn test_equip_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 only 1 mana (not enough for Equip {2}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Test Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InsufficientMana)),
        "equip with insufficient mana should return InsufficientMana, got {:?}",
        result
    );
}

// ── Test 11: Protection blocks targeting ─────────────────────────────────────

#[test]
/// CR 702.16d (via DEBT "E") — protection prevents equipping.
/// A creature with protection from artifacts cannot be targeted by an Equipment's equip ability.
fn test_equip_protection_blocks_targeting() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    // Creature with protection from artifacts.
    let protected_creature = mtg_engine::ObjectSpec::creature(p1, "Protected Paladin", 2, 2)
        .with_keyword(KeywordAbility::ProtectionFrom(
            ProtectionQuality::FromCardType(CardType::Artifact),
        ));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(protected_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Protected Paladin");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "equip targeting a creature with protection from artifacts should return InvalidTarget, got {:?}",
        result
    );
}

// ── Test 12: Fizzle if target leaves battlefield ──────────────────────────────

#[test]
/// CR 608.2b — if all targets become illegal before an activated ability resolves,
/// the ability has no effect (fizzle).
/// The AttachEquipment effect validates target legality at resolution time.
fn test_equip_fizzles_if_target_not_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Test Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Doomed Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Test Sword");
    let creature_id = find_object(&state, "Doomed Bear");

    // Give mana and activate equip.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    // Before resolving, destroy the creature (move to graveyard).
    let mut state = state;
    let owner = state.objects.get(&creature_id).map(|o| o.owner).unwrap();
    let _ = state.move_object_to_zone(creature_id, ZoneId::Graveyard(owner));

    // Pass priority — ability resolves, but target is gone.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Equipment still unattached.
    let equip_obj = state.objects.get(&equip_id).expect("equipment exists");
    assert_eq!(
        equip_obj.attached_to, None,
        "equipment should remain unattached when target left battlefield"
    );

    // No EquipmentAttached event.
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::EquipmentAttached { .. })),
        "EquipmentAttached should not fire when target left battlefield"
    );
}

// ── Test 13: Equip {0} (zero-cost equip succeeds without mana) ────────────────

#[test]
/// CR 702.6a — equip abilities with {0} cost require no mana payment.
/// Lightning Greaves is the most common Commander equipment with Equip {0}.
/// The equip_ability helper sets mana_cost: None for cost 0; handle_activate_ability
/// skips the mana check when ability_cost.mana_cost is None.
fn test_equip_zero_cost_succeeds() {
    let p1 = p(1);
    let p2 = p(2);

    // Equipment with Equip {0} (no mana cost).
    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Zero Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(0)); // 0 = None mana_cost

    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No mana added — Equip {0} should not require any.
    state.turn.priority_holder = Some(p1);

    let equip_id = find_object(&state, "Zero Sword");
    let creature_id = find_object(&state, "Test Bear");

    // Activation must succeed with empty mana pool.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    // No ManaCostPaid event (zero cost).
    assert!(
        !activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { .. })),
        "ManaCostPaid should NOT fire for Equip {{0}}"
    );

    // Resolve — both players pass priority.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Equipment is attached to the creature.
    let equip_obj = state.objects.get(&equip_id).expect("equipment exists");
    assert_eq!(
        equip_obj.attached_to,
        Some(creature_id),
        "Equip {{0}} should attach to creature without spending mana"
    );

    // EquipmentAttached event emitted.
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::EquipmentAttached { equipment_id, target_id, .. }
            if *equipment_id == equip_id && *target_id == creature_id
        )),
        "EquipmentAttached event expected for Equip {{0}}"
    );
}

// ── Test 14: Equipment keyword grant via layer system ─────────────────────────

#[test]
/// CR 702.6a + CR 604 (static abilities) + CR 613 (layer system).
/// When an equipment's static ability grants a keyword (e.g. Haste) via
/// EffectFilter::AttachedCreature, the equipped creature should have that keyword
/// reflected by calculate_characteristics after equipping.
///
/// This tests the end-to-end path: equip (set attached_to) -> layer system
/// EffectFilter::AttachedCreature matches the creature -> keyword appears.
fn test_equip_grants_keywords_via_layer_system() {
    let p1 = p(1);
    let p2 = p(2);

    // Equipment with a static ability granting Haste to the equipped creature.
    let equipment = mtg_engine::ObjectSpec::artifact(p1, "Haste Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(equip_ability(2));

    let creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let equip_id = find_object(&state, "Haste Sword");
    let creature_id = find_object(&state, "Vanilla Bear");

    // Manually register a continuous effect on the equipment, simulating what
    // register_static_continuous_effects does at ETB time for equipment with static abilities.
    // This grants Haste to whatever the equipment is attached to (AttachedCreature filter).
    {
        let eff_id = state.next_object_id().0;
        state.timestamp_counter += 1;
        let ts = state.timestamp_counter;
        state.continuous_effects.push_back(ContinuousEffect {
            id: EffectId(eff_id),
            source: Some(equip_id),
            timestamp: ts,
            layer: EffectLayer::Ability,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AttachedCreature,
            modification: LayerModification::AddKeywords(
                [KeywordAbility::Haste].into_iter().collect(),
            ),
            is_cda: false,
        });
    }

    // Before equipping: creature should NOT have Haste (equipment is unattached).
    let chars_before =
        calculate_characteristics(&state, creature_id).expect("creature exists before equipping");
    assert!(
        !chars_before.keywords.contains(&KeywordAbility::Haste),
        "creature should NOT have Haste before equipment is attached"
    );

    // Equip targeting the creature.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // Equipment is attached.
    assert_eq!(
        state.objects.get(&equip_id).unwrap().attached_to,
        Some(creature_id),
        "equipment should be attached to the creature"
    );

    // After equipping: creature should have Haste via layer-computed characteristics.
    // CR 613 (layer 6 / Ability layer): static ability from attached equipment applies.
    let chars_after =
        calculate_characteristics(&state, creature_id).expect("creature exists after equipping");
    assert!(
        chars_after.keywords.contains(&KeywordAbility::Haste),
        "creature should have Haste from equipment's static ability after equipping (CR 604 + CR 613)"
    );
}

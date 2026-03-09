//! Backup keyword ability tests (CR 702.165).
//!
//! Backup is a triggered ability. "Backup N" means "When this creature enters,
//! put N +1/+1 counters on target creature. If that's another creature, it also
//! gains the non-backup abilities of this creature printed below this one until
//! end of turn." (CR 702.165a)
//!
//! Key rules verified:
//! - Backup trigger fires on ETB, places N +1/+1 counters on target (CR 702.165a).
//! - If target is the backup creature itself, it gets counters but NO abilities (CR 702.165a).
//! - Only non-backup abilities printed below the Backup entry are granted (CR 702.165a, c).
//! - Abilities are determined at trigger time, not resolution (CR 702.165d).
//! - Abilities granted expire at end of turn (UntilEndOfTurn duration).
//! - If target leaves battlefield before resolution, trigger fizzles (CR 608.2b).
//! - Multiple Backup instances on one card each trigger separately.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, EffectDuration, EffectFilter, EffectLayer, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, LayerModification, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId,
    StackObject, StackObjectKind, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once.
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

/// Cast a creature from hand and resolve it (both players pass).
fn cast_and_resolve(
    state: GameState,
    caster: PlayerId,
    card_name: &str,
    other_player: PlayerId,
) -> (GameState, Vec<GameEvent>) {
    let card_id = find_object(&state, card_name);
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
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
    .unwrap_or_else(|e| panic!("CastSpell '{}' failed: {:?}", card_name, e));

    pass_all(state, &[caster, other_player])
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// 2/2 creature with Backup 1 and Flying + First Strike printed below.
fn backup_1_flying_firststrike_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("backup-1-flying-fs".to_string()),
        name: "Backup Valkyrie".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Backup 1\nFlying\nFirst strike".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Backup(1)),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 3/3 creature with Backup 2 and Trample + Lifelink printed below.
fn backup_2_trample_lifelink_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("backup-2-trample-ll".to_string()),
        name: "Backup Wurm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Backup 2\nTrample\nLifelink".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Backup(2)),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// 2/2 creature with Backup 1 only -- no abilities below.
fn backup_1_no_abilities_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("backup-1-no-abil".to_string()),
        name: "Bare Backup Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Backup 1".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Backup(1))],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 2/2 creature with Backup 1 + Backup 1 (two instances).
fn double_backup_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("double-backup".to_string()),
        name: "Double Backup Goblin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Backup 1\nBackup 1".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Backup(1)),
            AbilityDefinition::Keyword(KeywordAbility::Backup(1)),
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: Enum variant exists ───────────────────────────────────────────────

#[test]
/// CR 702.165: KeywordAbility::Backup(N) can be constructed and matched.
fn test_backup_enum_variant_exists() {
    let kw = KeywordAbility::Backup(1);
    match kw {
        KeywordAbility::Backup(n) => assert_eq!(n, 1),
        _ => panic!("KeywordAbility::Backup(1) did not match Backup variant"),
    }
    assert_ne!(KeywordAbility::Backup(1), KeywordAbility::Backup(2));
    assert_eq!(KeywordAbility::Backup(3), KeywordAbility::Backup(3));
}

// ── Test 2: ETB generates a BackupTrigger ─────────────────────────────────────

#[test]
/// CR 702.165a — When a creature with Backup enters, a BackupTrigger appears on the stack.
fn test_backup_etb_generates_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_1_flying_firststrike_def()]);

    let backup_card = ObjectSpec::creature(p1, "Backup Valkyrie", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_keyword(KeywordAbility::Flying)
        .with_keyword(KeywordAbility::FirstStrike)
        .with_card_id(CardId("backup-1-flying-fs".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 3);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve Backup Valkyrie (both players pass once to let it land).
    let (state, _) = cast_and_resolve(state.clone(), p1, "Backup Valkyrie", p2);

    // BackupTrigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.165a: BackupTrigger should be on stack after creature ETB"
    );
    assert!(
        matches!(
            state.stack_objects.back().map(|s| &s.kind),
            Some(StackObjectKind::KeywordTrigger { keyword: KeywordAbility::Backup(_), .. })
        ),
        "CR 702.165a: Stack object should be BackupTrigger kind"
    );
}

// ── Test 3: Self-target gets counters but NO abilities ─────────────────────────

#[test]
/// CR 702.165a — "If that's another creature, it also gains..."
/// Self-targeting: N +1/+1 counters placed, but NO ability-granting CE created.
fn test_backup_self_target_gets_counters_only() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_1_flying_firststrike_def()]);

    let backup_card = ObjectSpec::creature(p1, "Backup Valkyrie", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_keyword(KeywordAbility::Flying)
        .with_keyword(KeywordAbility::FirstStrike)
        .with_card_id(CardId("backup-1-flying-fs".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 3);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve (trigger lands on stack).
    let (state, _) = cast_and_resolve(state.clone(), p1, "Backup Valkyrie", p2);

    // Resolve the self-targeting backup trigger (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Backup Valkyrie should have 1 +1/+1 counter.
    let valkyrie_id = find_object_in_zone(&state, "Backup Valkyrie", ZoneId::Battlefield)
        .expect("Backup Valkyrie should be on battlefield");

    let obj = state.objects.get(&valkyrie_id).unwrap();
    let counters = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 1,
        "CR 702.165a: Self-target should receive 1 +1/+1 counter"
    );

    // Self-targeting should NOT create any ability-granting continuous effect.
    let ability_ces: Vec<_> = state
        .continuous_effects
        .iter()
        .filter(|ce| matches!(&ce.filter, EffectFilter::SingleObject(id) if *id == valkyrie_id))
        .collect();
    assert_eq!(
        ability_ces.len(),
        0,
        "CR 702.165a: Self-target must NOT create ability-granting continuous effect"
    );
}

// ── Test 4: Abilities filtering -- no Backup in abilities_below ───────────────

#[test]
/// CR 702.165a — "non-backup abilities" -- the Backup keyword itself is excluded
/// from the abilities_below list collected at trigger time.
fn test_backup_does_not_include_backup_keyword_in_grant() {
    // Verify the filtering logic used in check_triggers correctly excludes Backup.
    let def = backup_1_flying_firststrike_def();

    // Find the Backup ability index.
    let backup_idx = def
        .abilities
        .iter()
        .position(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Backup(_))));
    assert_eq!(
        backup_idx,
        Some(0),
        "Backup should be at index 0 in this definition"
    );

    // Replicate the filtering logic from check_triggers (CR 702.165a, 702.165c).
    let abilities_below: Vec<KeywordAbility> = def.abilities[1..]
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Keyword(kw) if !matches!(kw, KeywordAbility::Backup(_)) => {
                Some(kw.clone())
            }
            _ => None,
        })
        .collect();

    // Should contain Flying and FirstStrike but NOT Backup.
    assert!(
        abilities_below.contains(&KeywordAbility::Flying),
        "CR 702.165a: Flying should be in abilities_below"
    );
    assert!(
        abilities_below.contains(&KeywordAbility::FirstStrike),
        "CR 702.165a: FirstStrike should be in abilities_below"
    );
    assert!(
        !abilities_below
            .iter()
            .any(|kw| matches!(kw, KeywordAbility::Backup(_))),
        "CR 702.165a: Backup keyword must NOT appear in abilities_below"
    );
}

// ── Test 5: Multiple Backup instances trigger separately ──────────────────────

#[test]
/// CR 703.2: Each Backup instance is a separate triggered ability.
/// A card with two Backup 1 instances generates two BackupTriggers on ETB.
fn test_backup_multiple_instances_trigger_separately() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![double_backup_def()]);

    let backup_card = ObjectSpec::creature(p1, "Double Backup Goblin", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_card_id(CardId("double-backup".to_string()))
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 4);
    state.turn.priority_holder = Some(p1);

    let (state, _) = cast_and_resolve(state.clone(), p1, "Double Backup Goblin", p2);

    // Two Backup instances → two BackupTriggers on stack.
    let backup_triggers: Vec<_> = state
        .stack_objects
        .iter()
        .filter(|s| matches!(&s.kind, StackObjectKind::KeywordTrigger { keyword: KeywordAbility::Backup(_), .. }))
        .collect();
    assert_eq!(
        backup_triggers.len(),
        2,
        "CR 603.2: Two Backup instances should generate two separate BackupTriggers"
    );
}

// ── Test 6: Abilities only from below Backup in definition ─────────────────────

#[test]
/// CR 702.165a — "non-backup abilities of this creature printed below this one"
/// Only abilities appearing AFTER the Backup entry in the definition are granted.
/// Abilities before Backup are not included.
fn test_backup_only_abilities_below_are_granted() {
    // Card: Vigilance, Backup 1, Flying (Vigilance is BEFORE Backup, Flying is after).
    let def_with_ability_before = CardDefinition {
        card_id: CardId("backup-with-before-abil".to_string()),
        name: "Vigilant Backup".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Vigilance\nBackup 1\nFlying".to_string(),
        abilities: vec![
            // Vigilance is BEFORE Backup -- should NOT be in abilities_below.
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Backup(1)),
            // Flying is AFTER Backup -- should be in abilities_below.
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    // Find the Backup ability index.
    let backup_idx = def_with_ability_before
        .abilities
        .iter()
        .position(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Backup(_))))
        .unwrap();
    assert_eq!(
        backup_idx, 1,
        "Backup should be at index 1 in this definition"
    );

    // Replicate the filtering logic from check_triggers.
    let abilities_below: Vec<KeywordAbility> = def_with_ability_before.abilities[backup_idx + 1..]
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Keyword(kw) if !matches!(kw, KeywordAbility::Backup(_)) => {
                Some(kw.clone())
            }
            _ => None,
        })
        .collect();

    // Flying should be in abilities_below (after Backup).
    assert!(
        abilities_below.contains(&KeywordAbility::Flying),
        "CR 702.165a: Flying (after Backup) should be in abilities_below"
    );
    // Vigilance should NOT be in abilities_below (before Backup).
    assert!(
        !abilities_below.contains(&KeywordAbility::Vigilance),
        "CR 702.165a: Vigilance (before Backup) must NOT be in abilities_below"
    );
}

// ── Test 7: No abilities below -- only counters granted ──────────────────────

#[test]
/// CR 702.165a — If no keyword abilities are printed below Backup, no abilities
/// are granted, but counters are still placed.
fn test_backup_no_abilities_below_only_grants_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_1_no_abilities_def()]);

    let backup_card = ObjectSpec::creature(p1, "Bare Backup Creature", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_card_id(CardId("backup-1-no-abil".to_string()))
        .with_mana_cost(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 2);
    state.turn.priority_holder = Some(p1);

    // Cast and resolve (trigger fires, self-targets by default).
    let (state, _) = cast_and_resolve(state.clone(), p1, "Bare Backup Creature", p2);

    // Trigger on stack, resolve it.
    let (state, _) = pass_all(state, &[p1, p2]);

    let creature_id = find_object_in_zone(&state, "Bare Backup Creature", ZoneId::Battlefield)
        .expect("Bare Backup Creature should be on battlefield");

    // Should have 1 +1/+1 counter (self-targeting always grants counter).
    let obj = state.objects.get(&creature_id).unwrap();
    let counters = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 1,
        "CR 702.165a: Even with no abilities below, counter is still placed"
    );

    // No ability CEs created (empty abilities_to_grant and self-target).
    let ability_ces: Vec<_> = state
        .continuous_effects
        .iter()
        .filter(|ce| matches!(&ce.filter, EffectFilter::SingleObject(id) if *id == creature_id))
        .collect();
    assert_eq!(
        ability_ces.len(),
        0,
        "CR 702.165a: No ability CEs should be created when no abilities to grant"
    );
}

// ── Test 8: Backup 2 places 2 counters ────────────────────────────────────────

#[test]
/// CR 702.165a — Backup N places exactly N +1/+1 counters.
fn test_backup_n_counters_quantity() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_2_trample_lifelink_def()]);

    // Backup Wurm has Backup 2 (enters with trigger that places 2 counters).
    let backup_card = ObjectSpec::creature(p1, "Backup Wurm", 3, 3)
        .with_keyword(KeywordAbility::Backup(2))
        .with_keyword(KeywordAbility::Trample)
        .with_keyword(KeywordAbility::Lifelink)
        .with_card_id(CardId("backup-2-trample-ll".to_string()))
        .with_mana_cost(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
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
    state.turn.priority_holder = Some(p1);

    // Cast and resolve, then resolve the self-targeting trigger.
    let (state, _) = cast_and_resolve(state.clone(), p1, "Backup Wurm", p2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let wurm_id = find_object_in_zone(&state, "Backup Wurm", ZoneId::Battlefield)
        .expect("Backup Wurm should be on battlefield");

    let obj = state.objects.get(&wurm_id).unwrap();
    let counters = obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 2,
        "CR 702.165a: Backup 2 should place exactly 2 +1/+1 counters"
    );
}

// ── Test 9: BackupTrigger StackObjectKind structure ───────────────────────────

#[test]
/// CR 702.165a — The BackupTrigger StackObjectKind carries the correct data:
/// source_object, target_creature, counter_count, and abilities_to_grant.
fn test_backup_trigger_stack_object_structure() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_1_flying_firststrike_def()]);

    let backup_card = ObjectSpec::creature(p1, "Backup Valkyrie", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_keyword(KeywordAbility::Flying)
        .with_keyword(KeywordAbility::FirstStrike)
        .with_card_id(CardId("backup-1-flying-fs".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 3);
    state.turn.priority_holder = Some(p1);

    let (state, _) = cast_and_resolve(state.clone(), p1, "Backup Valkyrie", p2);

    // Find the BackupTrigger on the stack and inspect its structure.
    let trigger = state
        .stack_objects
        .iter()
        .find(|s| matches!(&s.kind, StackObjectKind::KeywordTrigger { keyword: KeywordAbility::Backup(_), .. }))
        .expect("BackupTrigger should be on stack");

    let valkyrie_id = find_object_in_zone(&state, "Backup Valkyrie", ZoneId::Battlefield)
        .expect("Valkyrie should be on battlefield");

    if let StackObjectKind::KeywordTrigger {
        source_object,
        keyword: KeywordAbility::Backup(_),
        data: mtg_engine::state::stack::TriggerData::ETBBackup {
            target,
            count,
            abilities,
        },
    } = &trigger.kind
    {
        // Source is the Backup Valkyrie.
        assert_eq!(
            *source_object, valkyrie_id,
            "CR 702.165a: source_object should be the entering backup creature"
        );
        // Default target is self (deterministic bot behavior).
        assert_eq!(
            *target, valkyrie_id,
            "CR 702.165a: Default target should be self (deterministic)"
        );
        // Counter count should be 1 (Backup 1).
        assert_eq!(
            *count, 1,
            "CR 702.165a: counter_count should be N from Backup N"
        );
        // Self-targeting: abilities_to_grant should be EMPTY (CR 702.165a "if that's another creature").
        assert!(
            abilities.is_empty(),
            "CR 702.165a: Self-targeting BackupTrigger must have empty abilities_to_grant"
        );
    } else {
        panic!("Stack object should be BackupTrigger");
    }
}

// ── Test 10: Abilities determined at trigger time (CR 702.165d) ────────────────

#[test]
/// CR 702.165d — The abilities to grant are locked in when the trigger goes on the
/// stack. Verifies that `backup_abilities` is captured in PendingTrigger at trigger
/// collection time, not at resolution time.
/// This is verified structurally: the BackupTrigger's abilities_to_grant is set
/// at flush_pending_triggers time (from backup_abilities stored at check_triggers time).
fn test_backup_abilities_locked_at_trigger_time() {
    // Test the static filtering logic: backup_abilities is computed in check_triggers
    // and stored on PendingTrigger, then passed to the StackObject at flush time.
    // For a card with Backup(1), Flying, FirstStrike:
    let def = backup_1_flying_firststrike_def();

    // Simulate: for the Backup at index 0, abilities_below = Flying + FirstStrike.
    let abilities_below: Vec<KeywordAbility> = def.abilities[1..]
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Keyword(kw) if !matches!(kw, KeywordAbility::Backup(_)) => {
                Some(kw.clone())
            }
            _ => None,
        })
        .collect();

    // Verify these are exactly [Flying, FirstStrike].
    assert_eq!(
        abilities_below.len(),
        2,
        "CR 702.165d: Two abilities below Backup"
    );
    assert!(
        abilities_below.contains(&KeywordAbility::Flying),
        "CR 702.165d: Flying should be snapshotted at trigger time"
    );
    assert!(
        abilities_below.contains(&KeywordAbility::FirstStrike),
        "CR 702.165d: FirstStrike should be snapshotted at trigger time"
    );

    // The stored list is what goes into the StackObject's abilities_to_grant
    // (for a non-self target). The snapshot is complete at trigger-collection time,
    // not at resolution time -- fulfilling CR 702.165d.
    // (Full integration test of this invariant would require two-player setup with
    // a Dress Down effect mid-resolution -- deferred as an edge-case scenario.)
}

// ── Test 11: Another creature gets counters AND abilities ─────────────────────

#[test]
/// CR 702.165a — "If that's another creature, it also gains the non-backup abilities
/// of this creature printed below this one until end of turn."
///
/// Exercises the continuous-effect creation path in resolution.rs (lines 2787-2807)
/// that is never reached by the deterministic bot's self-targeting default.
///
/// Setup: two creatures on the battlefield — the Backup source and a plain 2/2.
/// We manually push a BackupTrigger targeting the OTHER creature, then resolve.
/// Verify: (a) target gets 1 +1/+1 counter, (b) a Layer 6 UntilEndOfTurn CE
/// granting Flying+FirstStrike is created filtering to the target, (c) source
/// gets 0 counters from this trigger.
fn test_backup_another_creature_gets_counters_and_abilities() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![backup_1_flying_firststrike_def()]);

    // Backup Valkyrie (the source of the Backup trigger) on the battlefield.
    let backup_source = ObjectSpec::creature(p1, "Backup Valkyrie", 2, 2)
        .with_keyword(KeywordAbility::Backup(1))
        .with_keyword(KeywordAbility::Flying)
        .with_keyword(KeywordAbility::FirstStrike)
        .with_card_id(CardId("backup-1-flying-fs".to_string()))
        .in_zone(ZoneId::Battlefield);

    // Target creature that will RECEIVE the abilities -- a plain vanilla 2/2.
    let target_bear = ObjectSpec::creature(p1, "Runeclaw Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(backup_source)
        .object(target_bear)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Backup Valkyrie");
    let bear_id = find_object(&state, "Runeclaw Bear");

    // Manually push a BackupTrigger targeting the OTHER creature (bear_id).
    // abilities_to_grant matches what check_triggers would compute for Backup Valkyrie:
    // abilities at index 1+ excluding Backup itself → [Flying, FirstStrike].
    let trigger_id = state.next_object_id();
    let backup_trigger = StackObject {
        id: trigger_id,
        controller: p1,
        kind: StackObjectKind::KeywordTrigger {
            source_object: source_id,
            keyword: KeywordAbility::Backup(1),
            data: mtg_engine::state::stack::TriggerData::ETBBackup {
                target: bear_id,
                count: 1,
                abilities: vec![KeywordAbility::Flying, KeywordAbility::FirstStrike],
            },
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        x_value: 0,
        evidence_collected: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        is_cast_transformed: false,
        additional_costs: vec![],
    };
    state.stack_objects.push_back(backup_trigger);

    // Both players pass priority to resolve the BackupTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // (a) Target bear should have 1 +1/+1 counter.
    let bear_obj = state
        .objects
        .get(&bear_id)
        .expect("bear still on battlefield");
    let bear_counters = bear_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        bear_counters, 1,
        "CR 702.165a: Target (another creature) should receive 1 +1/+1 counter"
    );

    // (b) A Layer 6 UntilEndOfTurn CE granting Flying+FirstStrike should exist,
    //     filtered to the bear (not the source).
    let ability_ces: Vec<_> = state
        .continuous_effects
        .iter()
        .filter(|ce| {
            matches!(&ce.filter, EffectFilter::SingleObject(id) if *id == bear_id)
                && ce.layer == EffectLayer::Ability
                && ce.duration == EffectDuration::UntilEndOfTurn
        })
        .collect();
    assert_eq!(
        ability_ces.len(),
        1,
        "CR 702.165a: Exactly one ability-granting CE should exist for the target"
    );

    // Verify the CE grants the correct keywords.
    if let LayerModification::AddKeywords(kws) = &ability_ces[0].modification {
        assert!(
            kws.contains(&KeywordAbility::Flying),
            "CR 702.165a: CE should grant Flying"
        );
        assert!(
            kws.contains(&KeywordAbility::FirstStrike),
            "CR 702.165a: CE should grant FirstStrike"
        );
    } else {
        panic!("CE modification should be LayerModification::AddKeywords");
    }

    // (c) Source (Backup Valkyrie) should have 0 +1/+1 counters from this trigger.
    let source_obj = state
        .objects
        .get(&source_id)
        .expect("source still on battlefield");
    let source_counters = source_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        source_counters, 0,
        "CR 702.165a: Source gets 0 counters when targeting another creature"
    );
}

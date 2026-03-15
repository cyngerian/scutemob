//! Haunt keyword ability tests (CR 702.55).
//!
//! Haunt is a triggered ability that creates two linked triggered abilities:
//! 1. "When this creature is put into a graveyard from the battlefield, exile it haunting
//!    target creature." (CR 702.55a — creature path)
//! 2. "When the creature [this card] haunts dies, [effect]." (CR 702.55c — fires from exile)
//!
//! Key rules verified:
//! - Creature with Haunt that dies triggers a HauntExileTrigger (CR 702.55a).
//! - HauntExileTrigger resolution: haunt card moves from graveyard to exile; haunting_target set.
//! - HauntExiled event is emitted when the haunt relationship is established.
//! - When the haunted creature dies, HauntedCreatureDiesTrigger fires from exile (CR 702.55c).
//! - HauntedCreatureDiesTrigger resolution: effect from card registry executes.
//! - If no creatures available to haunt, HauntExileTrigger fizzles (card stays in graveyard).
//! - If haunt card is removed from exile before haunted creature dies, no trigger fires.
//! - Haunt does NOT trigger when the creature is exiled directly (only dies = graveyard).
//! - Multiplayer: controller of exiled haunt card controls the HauntedCreatureDies trigger.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, TriggerCondition, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in any zone", name))
}

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == zone)
        .count()
}

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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Synthetic haunt creature: "Haunt. When the creature it haunts dies, target player
/// loses 2 life." (Simplified Blind Hunter — drain omitted for test clarity.)
fn haunt_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-haunt-creature".to_string()),
        name: "Test Haunt Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Haunt. When the creature it haunts dies, target player loses 2 life."
            .to_string(),
        abilities: vec![
            // The Haunt keyword marks it as a haunt permanent (CR 702.55a).
            AbilityDefinition::Keyword(KeywordAbility::Haunt),
            // The second triggered ability fires from exile (CR 702.55c).
            // Uses a trivial GainLife(0) effect to keep the card definition simple.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::HauntedCreatureDies,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(0),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.55a — Creature with Haunt that dies triggers HauntExileTrigger.
/// The trigger goes on the stack; on resolution, the card moves to exile haunting
/// a target creature; HauntExiled event is emitted; haunting_target is set on the
/// exiled card.
#[test]
fn test_haunt_creature_dies_puts_haunt_exile_trigger_on_stack() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // P1's haunt creature has lethal damage (2 damage on a 2/2 → dies via SBA).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2), // lethal → SBA kills it
        )
        // P2 has a creature that the haunt card will target.
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority → SBA fires → haunt creature dies → HauntExileTrigger queued.
    let (state, events) = pass_all(state, &[p1, p2]);

    // Haunt creature should have died.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.55a: CreatureDied event expected when haunt creature dies"
    );

    // HauntExileTrigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.55a: HauntExileTrigger should be on the stack after haunt creature dies"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Haunt,
                ..
            }
        ),
        "CR 702.55a: stack object should be HauntExileTrigger"
    );

    // Card should be in the graveyard (not yet exiled — trigger not resolved).
    assert!(
        find_object_in_zone(&state, "Test Haunt Creature", ZoneId::Graveyard(p1)).is_some(),
        "CR 702.55a: haunt card should be in graveyard before trigger resolves"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        0,
        "CR 702.55a: exile should be empty before HauntExileTrigger resolves"
    );
}

/// CR 702.55a — HauntExileTrigger resolution: haunt card moves from graveyard to exile;
/// haunting_target is set; HauntExiled event fires.
#[test]
fn test_haunt_exile_trigger_resolution_exiles_card_with_haunting_target() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2),
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // First pass_all: SBA kills haunt creature → trigger on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Record the Target Creature's ObjectId before the trigger resolves.
    let target_id = find_object(&state, "Target Creature");

    // Second pass_all: resolve HauntExileTrigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // HauntExiled event should have fired.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::HauntExiled { .. })),
        "CR 702.55a: HauntExiled event should fire when HauntExileTrigger resolves"
    );

    // Haunt card should now be in exile.
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        1,
        "CR 702.55a: haunt card should be in exile after trigger resolves"
    );

    // Graveyard should be empty (card moved from graveyard to exile).
    assert_eq!(
        count_in_zone(&state, ZoneId::Graveyard(p1)),
        0,
        "CR 702.55a: graveyard should be empty after haunt card exiled"
    );

    // The exiled haunt card should have haunting_target set to Target Creature's ObjectId.
    let exiled_obj = state
        .objects
        .values()
        .find(|obj| obj.zone == ZoneId::Exile && obj.characteristics.name == "Test Haunt Creature")
        .expect("Test Haunt Creature should be in exile");
    assert_eq!(
        exiled_obj.haunting_target,
        Some(target_id),
        "CR 702.55b: haunting_target on exiled card should point to the haunted creature"
    );
}

/// CR 702.55c — When the haunted creature dies, HauntedCreatureDiesTrigger fires from exile.
/// The trigger goes on the stack; on resolution, the haunt effect fires.
#[test]
fn test_haunt_haunted_creature_dies_fires_trigger_from_exile() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // Place Target Creature on battlefield so we can get its ObjectId.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2), // lethal → dies
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Phase 1: haunt creature dies.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "setup: HauntExileTrigger should be on stack after haunt creature dies"
    );

    // Phase 2: HauntExileTrigger resolves → haunt card exiled haunting Target Creature.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        1,
        "setup: haunt card should be in exile after trigger resolves"
    );
    assert_eq!(
        state.stack_objects.len(),
        0,
        "setup: stack should be empty after HauntExileTrigger resolves"
    );

    // Mark Target Creature as dying (lethal damage).
    let target_id = find_object(&state, "Target Creature");
    let mut state = state;
    let target_obj = state.objects.get_mut(&target_id).unwrap();
    target_obj.damage_marked = 2; // 2/2 with 2 damage → lethal

    // Phase 3: Target Creature dies → HauntedCreatureDiesTrigger fires from exile.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.55c: Target Creature should die"
    );

    // HauntedCreatureDiesTrigger should be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.55c: HauntedCreatureDiesTrigger should be on the stack after haunted creature dies"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Haunt,
                ..
            }
        ),
        "CR 702.55c: stack object should be HauntedCreatureDiesTrigger"
    );
}

/// CR 702.55a — If no creatures are available to haunt when HauntExileTrigger resolves,
/// the trigger fizzles and the haunt card stays in the graveyard.
#[test]
fn test_haunt_no_creatures_available_fizzles() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // Only the haunt creature itself — no other creatures to target.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2),
        )
        // No other creatures on battlefield.
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Phase 1: SBA kills haunt creature → HauntExileTrigger on stack.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        1,
        "HauntExileTrigger should be on stack"
    );

    // Phase 2: trigger resolves — no legal targets → fizzles.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Haunt card should remain in the graveyard (not exiled).
    assert!(
        find_object_in_zone(&state, "Test Haunt Creature", ZoneId::Graveyard(p1)).is_some(),
        "CR 702.55a: haunt card should stay in graveyard when trigger fizzles (no targets)"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        0,
        "CR 702.55a: exile should be empty when HauntExileTrigger fizzles"
    );
}

/// CR 702.55c — If the haunt card leaves exile before the haunted creature dies,
/// no HauntedCreatureDiesTrigger fires (no object in exile with haunting_target set).
#[test]
fn test_haunt_card_removed_from_exile_no_trigger() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2),
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Phase 1+2: haunt creature dies, trigger resolves → haunt card in exile.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        1,
        "setup: haunt card should be in exile"
    );

    // Manually remove the haunt card from exile (simulate Riftsweeper effect).
    // Move it to p1's graveyard by directly manipulating state.
    let mut state = state;
    let exiled_haunt_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Exile)
        .map(|(&id, _)| id)
        .expect("haunt card should be in exile");
    let haunt_obj = state.objects.get_mut(&exiled_haunt_id).unwrap();
    haunt_obj.zone = ZoneId::Graveyard(p1);
    haunt_obj.haunting_target = None; // cleared when card leaves exile

    // Now mark Target Creature as dying.
    let target_id = find_object(&state, "Target Creature");
    let target_obj = state.objects.get_mut(&target_id).unwrap();
    target_obj.damage_marked = 2;

    // Phase 3: Target Creature dies. With no haunt card in exile, no trigger should fire.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.55c: no HauntedCreatureDiesTrigger should fire if haunt card is not in exile"
    );
}

/// CR 702.55a — Haunt only triggers when a creature "dies" (put into graveyard from
/// battlefield). If the creature is exiled directly (not dying), haunt does NOT trigger.
#[test]
fn test_haunt_creature_exiled_directly_does_not_trigger_haunt() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // Haunt creature placed directly in exile (simulating a replacement effect that
    // exiled it instead of letting it die normally).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .in_zone(ZoneId::Exile), // already in exile — never "died"
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pass priority — no SBA triggers (creature is in exile, not on battlefield).
    let (state, _) = pass_all(state, &[p1, p2]);

    // No haunt trigger should fire — the haunt exile trigger only fires on death.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.55a: haunt trigger does NOT fire when creature is exiled directly (not dying)"
    );

    // No haunting_target should be set on the exiled card.
    let exiled_obj = state
        .objects
        .values()
        .find(|obj| obj.zone == ZoneId::Exile && obj.characteristics.name == "Test Haunt Creature")
        .expect("Test Haunt Creature should be in exile");
    assert_eq!(
        exiled_obj.haunting_target, None,
        "CR 702.55a: haunting_target should be None when card was exiled without dying"
    );
}

/// CR 702.55 full lifecycle: haunt creature dies → exiled haunting target → haunted
/// creature dies → HauntedCreatureDiesTrigger fires and resolves.
#[test]
fn test_haunt_full_lifecycle() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2),
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Step 1: SBA kills haunt creature → HauntExileTrigger on stack.
    let (state, events1) = pass_all(state, &[p1, p2]);
    assert!(
        events1
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "lifecycle step 1: haunt creature should die"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "lifecycle step 1: trigger on stack"
    );

    // Step 2: HauntExileTrigger resolves → haunt card exiled haunting Target Creature.
    let target_id = find_object(&state, "Target Creature");
    let (state, events2) = pass_all(state, &[p1, p2]);
    assert!(
        events2
            .iter()
            .any(|e| matches!(e, GameEvent::HauntExiled { .. })),
        "lifecycle step 2: HauntExiled event should fire"
    );
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        1,
        "lifecycle step 2: haunt card in exile"
    );

    // Verify haunting_target points to Target Creature.
    let exiled_obj = state
        .objects
        .values()
        .find(|obj| obj.zone == ZoneId::Exile)
        .expect("haunt card should be in exile");
    assert_eq!(
        exiled_obj.haunting_target,
        Some(target_id),
        "lifecycle step 2: haunting_target should point to Target Creature"
    );

    // Step 3: Kill the haunted creature → HauntedCreatureDiesTrigger fires.
    let mut state = state;
    let target_id_current = find_object(&state, "Target Creature");
    let target_obj = state.objects.get_mut(&target_id_current).unwrap();
    target_obj.damage_marked = 2;

    let (state, events3) = pass_all(state, &[p1, p2]);
    assert!(
        events3
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "lifecycle step 3: Target Creature should die"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "lifecycle step 3: HauntedCreatureDiesTrigger on stack"
    );
    assert!(
        matches!(
            state.stack_objects[0].kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Haunt,
                ..
            }
        ),
        "lifecycle step 3: correct trigger kind"
    );

    // Step 4: HauntedCreatureDiesTrigger resolves → haunt effect fires.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects.len(),
        0,
        "lifecycle step 4: stack should be empty after haunt effect resolves"
    );

    // Haunt card remains in exile (does NOT leave exile after haunted creature dies).
    assert_eq!(
        count_in_zone(&state, ZoneId::Exile),
        1,
        "CR 702.55c: haunt card stays in exile after haunted creature dies"
    );
}

/// CR 702.55 multiplayer: P1's haunt creature dies and is exiled haunting P2's creature.
/// When P2's creature dies, the haunt trigger is controlled by P1 (the haunt card's controller).
#[test]
fn test_haunt_multiplayer_controller_of_trigger() {
    let p1 = p1();
    let p2 = p2();

    let def = haunt_creature_def();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Test Haunt Creature", 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2),
        )
        // P2 has a creature that P1's haunt card will target.
        .object(ObjectSpec::creature(p2, "P2 Creature", 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P1's haunt creature dies; HauntExileTrigger should be controlled by P1.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.stack_objects.len(), 1);
    let haunt_exile_trigger = &state.stack_objects[0];
    assert_eq!(
        haunt_exile_trigger.controller, p1,
        "CR 702.55a: HauntExileTrigger should be controlled by the haunt creature's controller"
    );

    // Resolve HauntExileTrigger → haunt card targets P2's creature.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Kill P2's creature to fire HauntedCreatureDiesTrigger.
    let mut state = state;
    let p2_creature_id = find_object(&state, "P2 Creature");
    let p2_obj = state.objects.get_mut(&p2_creature_id).unwrap();
    p2_obj.damage_marked = 2;

    let (state, _) = pass_all(state, &[p1, p2]);

    // HauntedCreatureDiesTrigger should be on the stack, controlled by P1.
    assert_eq!(state.stack_objects.len(), 1);
    let haunted_dies_trigger = &state.stack_objects[0];
    assert!(
        matches!(
            haunted_dies_trigger.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Haunt,
                ..
            }
        ),
        "CR 702.55c: HauntedCreatureDiesTrigger should be on stack"
    );
    assert_eq!(
        haunted_dies_trigger.controller,
        p1,
        "CR 702.55c: HauntedCreatureDiesTrigger should be controlled by P1 (haunt card's owner/controller)"
    );
}

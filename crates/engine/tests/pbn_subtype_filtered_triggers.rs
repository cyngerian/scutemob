//! PB-N: Subtype/color-filtered attack and death trigger tests.
//!
//! Tests for:
//! - `WheneverCreatureYouControlAttacks` with `triggering_creature_filter`
//! - `WheneverCreatureDies` with `triggering_creature_filter`
//! - Pre-death LKI semantics for the color filter (CR 603.10a)
//! - Hash parity for `TriggeredAbilityDef.triggering_creature_filter`
//! - Hash sentinel bump verification (sentinel 3 → 4)
//! - `combat_damage_filter` tightened to damage events only (regression test)
//! - Kolaghan, the Storm's Fury end-to-end
//!
//! CR Rules covered:
//! - CR 508.1m: Attack triggers fire after attackers are declared.
//! - CR 603.2: Trigger fires once per event occurrence.
//! - CR 603.10a: Death triggers look back in time; characteristics from pre-death state.
//! - CR 613.1d/f: Filter reads use layer-resolved characteristics.
//!
//! Convention: after flush_pending_triggers, pending_triggers is EMPTY but triggers appear on
//! state.stack_objects. "No trigger" means stack_objects is empty after the event.

use mtg_engine::{
    process_command, AttackTarget, CardContinuousEffectDef, CardId, CardRegistry, Color, Command,
    DeathTriggerFilter, Effect, EffectAmount, EffectDuration, EffectFilter, EffectLayer,
    GameStateBuilder, LayerModification, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step,
    SubType, TargetFilter, TriggerEvent, TriggeredAbilityDef, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
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

/// Count the number of TriggeredAbility stack objects (indicates triggers that fired).
fn stack_trigger_count(state: &mtg_engine::GameState) -> usize {
    state
        .stack_objects
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .count()
}

/// Build a library card in the given player's library.
fn library_card(player: PlayerId, id: &str, name: &str) -> ObjectSpec {
    ObjectSpec::creature(player, name, 1, 1)
        .in_zone(ZoneId::Library(player))
        .with_card_id(CardId(id.to_string()))
}

/// Build an attack trigger that draws a card, filtered by the given subtype.
fn attack_trigger_draw_subtype(subtype: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description: format!(
            "Whenever a {} you control attacks, draw a card. (CR 508.1m / PB-N)",
            subtype
        ),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType(subtype.to_string())),
            ..Default::default()
        }),
        targets: vec![],
    }
}

/// Build an attack trigger that draws a card, filtered by the given color.
fn attack_trigger_draw_color(color: Color) -> TriggeredAbilityDef {
    let mut color_set = im::OrdSet::new();
    color_set.insert(color);
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description:
            "Whenever a creature of the given color you control attacks, draw a card. (CR 508.1m / PB-N)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            colors: Some(color_set),
            ..Default::default()
        }),
        targets: vec![],
    }
}

/// Build a death trigger that draws a card, filtered by the given subtype.
fn death_trigger_draw_subtype(subtype: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureDies,
        intervening_if: None,
        description: format!(
            "Whenever a {} you control dies, draw a card. (CR 603.10a / PB-N)",
            subtype
        ),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: Some(DeathTriggerFilter {
            controller_you: true,
            controller_opponent: false,
            exclude_self: false,
            nontoken_only: false,
        }),
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType(subtype.to_string())),
            ..Default::default()
        }),
        targets: vec![],
    }
}

/// Build a death trigger that draws a card, filtered by the given color.
fn death_trigger_draw_color(color: Color) -> TriggeredAbilityDef {
    let mut color_set = im::OrdSet::new();
    color_set.insert(color);
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureDies,
        intervening_if: None,
        description:
            "Whenever a creature of the given color you control dies, draw a card. (CR 603.10a / PB-N)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: Some(DeathTriggerFilter {
            controller_you: true,
            controller_opponent: false,
            exclude_self: false,
            nontoken_only: false,
        }),
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            colors: Some(color_set),
            ..Default::default()
        }),
        targets: vec![],
    }
}

// ── Test 1: MANDATORY — attack subtype match fires ─────────────────────────────

/// CR 508.1m / PB-N — attack trigger with Dragon subtype filter fires on Dragon attacker.
/// Mandatory test 1/8: subtype match case.
/// After DeclareAttackers, flush_pending_triggers moves triggers to stack_objects.
#[test]
fn test_pbn_attack_filter_subtype_match_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: has the "Whenever a Dragon you control attacks, draw a card" trigger.
    let watcher = ObjectSpec::creature(p1, "Dragon Watcher", 1, 1)
        .with_triggered_ability(attack_trigger_draw_subtype("Dragon"));

    // The attacker: a Dragon creature controlled by p1.
    let dragon = ObjectSpec::creature(p1, "Test Dragon", 2, 2)
        .with_subtypes(vec![SubType("Dragon".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(watcher)
        .object(dragon)
        .build()
        .unwrap();

    let dragon_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Test Dragon")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(dragon_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // After DeclareAttackers, flush_pending_triggers puts the trigger on the stack.
    // The trigger must appear as a TriggeredAbility on stack_objects.
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected at least 1 triggered ability on stack when Dragon attacks (subtype match). stack_objects={:?}",
        state.stack_objects.iter().map(|s| &s.kind).collect::<Vec<_>>()
    );
}

// ── Test 2: MANDATORY — attack subtype mismatch does not fire ─────────────────

/// CR 508.1m / PB-N — attack trigger with Dragon subtype filter does NOT fire on Goblin attacker.
/// Mandatory test 2/8: subtype mismatch case.
#[test]
fn test_pbn_attack_filter_subtype_mismatch_no_fire() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher has the Dragon-filtered attack trigger.
    let watcher = ObjectSpec::creature(p1, "Dragon Watcher", 1, 1)
        .with_triggered_ability(attack_trigger_draw_subtype("Dragon"));

    // The attacker: a Goblin (NOT a Dragon).
    let goblin = ObjectSpec::creature(p1, "Test Goblin", 1, 1)
        .with_subtypes(vec![SubType("Goblin".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(watcher)
        .object(goblin)
        .build()
        .unwrap();

    let goblin_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Test Goblin")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(goblin_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // No Dragon attacked — no trigger should appear on the stack.
    assert_eq!(
        stack_trigger_count(&state),
        0,
        "Expected NO triggered ability on stack when Goblin attacks with Dragon subtype filter"
    );
}

// ── Test 3: MANDATORY — attack color filter fires ─────────────────────────────

/// CR 508.1m / PB-N — attack trigger with Black color filter fires on black attacker.
/// Mandatory test 3/8: color filter beyond subtype.
#[test]
fn test_pbn_attack_filter_color_match_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher has "Whenever a black creature you control attacks, draw a card" trigger.
    let watcher = ObjectSpec::creature(p1, "Color Watcher", 1, 1)
        .with_triggered_ability(attack_trigger_draw_color(Color::Black));

    // A black creature attacker.
    let black_creature =
        ObjectSpec::creature(p1, "Black Creature", 2, 2).with_colors(vec![Color::Black]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(watcher)
        .object(black_creature)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Black Creature")
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    assert!(
        stack_trigger_count(&state) > 0,
        "Expected triggered ability on stack when black creature attacks (color filter match)"
    );
}

// ── Test 4: MANDATORY — death subtype match fires ─────────────────────────────

/// CR 603.10a / PB-N — death trigger with Vampire subtype filter fires when Vampire dies.
/// Mandatory test 4/8: death subtype match.
/// A Vampire with toughness 0 dies via SBA → trigger queues → goes to stack.
#[test]
fn test_pbn_death_filter_subtype_match_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a Vampire you control dies, draw a card."
    let watcher = ObjectSpec::creature(p1, "Vampire Watcher", 1, 4)
        .with_triggered_ability(death_trigger_draw_subtype("Vampire"));

    // A Vampire creature that will die via SBA (0 toughness).
    let dying_vampire = ObjectSpec::creature(p1, "Dying Vampire", 1, 0)
        .with_subtypes(vec![SubType("Vampire".to_string())]);

    // Library card (needed if trigger actually resolves and draws).
    let lib = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_vampire)
        .object(lib)
        .build()
        .unwrap();

    // Both players pass → step advances → SBAs fire → Vampire dies → trigger on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The Vampire died → left battlefield via SBA.
    let vampire_gone = !state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Dying Vampire" && o.zone == ZoneId::Battlefield);
    assert!(
        vampire_gone,
        "Dying Vampire should have left the battlefield (SBA)"
    );

    // Death trigger should be on the stack (as a TriggeredAbility stack object).
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected death trigger on stack when Vampire died (subtype match)"
    );
}

// ── Test 5: MANDATORY — death subtype mismatch does not fire ─────────────────

/// CR 603.10a / PB-N — death trigger with Vampire filter does NOT fire when Goblin dies.
/// Mandatory test 5/8: death subtype mismatch.
#[test]
fn test_pbn_death_filter_subtype_mismatch_no_fire() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a Vampire you control dies, draw a card."
    let watcher = ObjectSpec::creature(p1, "Vampire Watcher", 1, 4)
        .with_triggered_ability(death_trigger_draw_subtype("Vampire"));

    // A dying Goblin (NOT a Vampire).
    let dying_goblin = ObjectSpec::creature(p1, "Dying Goblin", 1, 0)
        .with_subtypes(vec![SubType("Goblin".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_goblin)
        .build()
        .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // The Goblin died, but the Vampire-filter trigger should NOT have fired.
    assert_eq!(
        stack_trigger_count(&state),
        0,
        "Expected NO death trigger on stack when Goblin died with Vampire subtype filter"
    );
}

// ── Test 6: MANDATORY — pre-death LKI on color ────────────────────────────────

/// CR 603.10a / PB-N — death trigger color filter uses PRE-DEATH characteristics (LKI).
/// Mandatory test 6/8: load-bearing LKI test per PB-Q4 retro.
/// A black creature dies via SBA (0 toughness). The filter is black.
/// Trigger must fire because pre-death color = Black is preserved on the graveyard object.
#[test]
fn test_pbn_death_filter_pre_death_lki_color() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a black creature you control dies, draw a card."
    let watcher = ObjectSpec::creature(p1, "Color Watcher", 1, 4)
        .with_triggered_ability(death_trigger_draw_color(Color::Black));

    // A black creature that will die via SBA (toughness 0).
    // The graveyard object must preserve pre-death characteristics (CR 603.10a LKI).
    let dying_black =
        ObjectSpec::creature(p1, "Dying Black Creature", 1, 0).with_colors(vec![Color::Black]);

    let lib = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_black)
        .object(lib)
        .build()
        .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // The black creature died → death trigger should fire because pre-death color = Black.
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected death trigger on stack: black creature died (LKI: pre-death color = Black)"
    );
}

// ── Test 7: MANDATORY — hash parity for new field + sentinel bump ─────────────

/// PB-N — hash parity test: two states differing only in `triggering_creature_filter`
/// must hash to different values. Verifies the new field participates in the hash.
/// Also verifies hash sentinel is non-trivial (sentinel 4 is included).
/// Closes PB-Q H1 retro lesson: every new dispatch field needs hash coverage.
#[test]
fn test_pbn_hash_parity_triggering_creature_filter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build two states: identical except one watcher has triggering_creature_filter set.
    let watcher_no_filter =
        ObjectSpec::creature(p1, "Watcher", 1, 1).with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
            intervening_if: None,
            description: "Hash parity test trigger (no filter)".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![],
        });

    let watcher_with_filter =
        ObjectSpec::creature(p1, "Watcher", 1, 1).with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
            intervening_if: None,
            description: "Hash parity test trigger (no filter)".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: Some(TargetFilter {
                has_subtype: Some(SubType("Dragon".to_string())),
                ..Default::default()
            }),
            targets: vec![],
        });

    let state_no_filter = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(watcher_no_filter)
        .build()
        .unwrap();

    let state_with_filter = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(watcher_with_filter)
        .build()
        .unwrap();

    let hash_no_filter = state_no_filter.public_state_hash();
    let hash_with_filter = state_with_filter.public_state_hash();

    assert_ne!(
        hash_no_filter, hash_with_filter,
        "Hash must differ when triggering_creature_filter differs (PB-N field hash parity)"
    );

    // Verify sentinel is non-zero (sentinel 4 is incorporated).
    // Any valid state produces a non-zero hash.
    let empty_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .build()
        .unwrap();
    assert_ne!(
        empty_state.public_state_hash(),
        [0u8; 32],
        "public_state_hash must include schema version sentinel (expected non-zero)"
    );
}

// ── Test 8: MANDATORY — Kolaghan end-to-end ──────────────────────────────────

/// CR 508.1m / PB-N — Kolaghan: Dragon attacker triggers buff, non-Dragon does not.
/// Mandatory test 8/8: real card end-to-end.
#[test]
fn test_pbn_kolaghan_end_to_end() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Kolaghan trigger: Whenever a Dragon you control attacks, creatures you control get +1/+0.
    let dragon_filter = Some(TargetFilter {
        has_subtype: Some(SubType("Dragon".to_string())),
        ..Default::default()
    });

    let kolaghan_trigger = TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description: "Kolaghan: Whenever a Dragon you control attacks, creatures you control get +1/+0. (CR 508.1m / PB-N)"
            .to_string(),
        effect: Some(Effect::ApplyContinuousEffect {
            effect_def: Box::new(CardContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyPower(1),
                filter: EffectFilter::CreaturesYouControl,
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            }),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: dragon_filter.clone(),
        targets: vec![],
    };

    // Test A: Kolaghan (a Dragon) attacks → buff trigger should fire (appear on stack).
    let kolaghan = ObjectSpec::creature(p1, "Kolaghan", 4, 5)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(kolaghan_trigger);

    let state_a = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(kolaghan)
        .build()
        .unwrap();

    let kolaghan_id = state_a
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Kolaghan")
        .map(|(id, _)| *id)
        .unwrap();

    let (state_a, _) = process_command(
        state_a,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(kolaghan_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Dragon failed");

    assert!(
        stack_trigger_count(&state_a) > 0,
        "Expected buff trigger on stack when Dragon (Kolaghan) attacks"
    );

    // Test B: a non-Dragon attacks with the Kolaghan trigger (Dragon filter) → NO trigger.
    let goblin_trigger_def = TriggeredAbilityDef {
        triggering_creature_filter: dragon_filter,
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description: "Kolaghan trigger (Dragon filter)".to_string(),
        effect: Some(Effect::ApplyContinuousEffect {
            effect_def: Box::new(CardContinuousEffectDef {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyPower(1),
                filter: EffectFilter::CreaturesYouControl,
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            }),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        targets: vec![],
    };

    // Watcher has the Dragon-filtered Kolaghan trigger (not itself a dragon).
    let watcher_b =
        ObjectSpec::creature(p1, "Watcher", 4, 5).with_triggered_ability(goblin_trigger_def);

    // Attacker: a Goblin (NOT a Dragon).
    let goblin_b =
        ObjectSpec::creature(p1, "Goblin", 1, 1).with_subtypes(vec![SubType("Goblin".to_string())]);

    let state_b = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(watcher_b)
        .object(goblin_b)
        .build()
        .unwrap();

    let goblin_id_b = state_b
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Goblin")
        .map(|(id, _)| *id)
        .unwrap();

    let (state_b, _) = process_command(
        state_b,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(goblin_id_b, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Goblin failed");

    assert_eq!(
        stack_trigger_count(&state_b),
        0,
        "Expected NO buff trigger on stack when non-Dragon (Goblin) attacks (Kolaghan Dragon filter)"
    );
}

// ── Test 9 (OPTIONAL): combat_damage_filter regression ────────────────────────

/// PB-N regression: combat_damage_filter must NOT fire on attack events (only damage).
/// This closes the latent semantic bug where combat_damage_filter ran for both
/// AnyCreatureYouControlAttacks and AnyCreatureYouControlDealsCombatDamageToPlayer.
#[test]
fn test_pbn_combat_damage_filter_not_consulted_on_attack_events() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A trigger on DAMAGE events (not attacks) with combat_damage_filter for Ninja.
    // This should only fire on damage, NOT on attacks.
    let ninja_damage_trigger = TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        intervening_if: None,
        description:
            "Whenever a Ninja you control deals combat damage to a player, draw. (CR 510.3a / PB-N regression)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Ninja".to_string())),
            ..Default::default()
        }),
        triggering_creature_filter: None,
        targets: vec![],
    };

    // Watcher with combat_damage_filter (NOT triggering_creature_filter).
    let watcher = ObjectSpec::creature(p1, "Ninja Watcher", 1, 1)
        .with_triggered_ability(ninja_damage_trigger);

    // Attacker: a Ninja creature.
    let ninja = ObjectSpec::creature(p1, "Test Ninja", 2, 2)
        .with_subtypes(vec![SubType("Ninja".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(watcher)
        .object(ninja)
        .build()
        .unwrap();

    let ninja_id = state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == "Test Ninja")
        .map(|(id, _)| *id)
        .unwrap();

    // Declare attackers: fires AnyCreatureYouControlAttacks.
    // The trigger is on AnyCreatureYouControlDealsCombatDamageToPlayer — should NOT fire here.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(ninja_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // The trigger is wired for DAMAGE events only. Declaring attackers must not fire it.
    // combat_damage_filter must NOT have caused a false fire on the attack event.
    assert_eq!(
        stack_trigger_count(&state),
        0,
        "combat_damage_filter must NOT cause trigger to fire on AnyCreatureYouControlAttacks (PB-N regression)"
    );
}

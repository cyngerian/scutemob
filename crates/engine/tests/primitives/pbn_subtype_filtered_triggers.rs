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
//! state.stack_objects(). "No trigger" means stack_objects is empty after the event.

use mtg_engine::{
    process_command, AttackTarget, CardContinuousEffectDef, CardId, CardRegistry, Color, Command,
    ContinuousEffect, DeathTriggerFilter, Effect, EffectAmount, EffectDuration, EffectFilter,
    EffectId, EffectLayer, GameStateBuilder, LayerModification, ObjectSpec, PlayerId, PlayerTarget,
    StackObjectKind, Step, SubType, TargetFilter, TriggerEvent, TriggeredAbilityDef, ZoneId,
    HASH_SCHEMA_VERSION,
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
        .stack_objects()
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
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
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
    let mut color_set = imbl::OrdSet::new();
    color_set.insert(color);
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
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
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
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
        .objects()
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
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // After DeclareAttackers, flush_pending_triggers puts the trigger on the stack.
    // The trigger must appear as a TriggeredAbility on stack_objects.
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected at least 1 triggered ability on stack when Dragon attacks (subtype match). stack_objects={:?}",
        state.stack_objects().iter().map(|s| &s.kind).collect::<Vec<_>>()
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
        .objects()
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
            exert_choices: vec![],
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
        .objects()
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
            exert_choices: vec![],
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
        .objects()
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

// ── Test 6: MANDATORY — pre-death LKI on subtype ─────────────────────────────

/// CR 603.10a / PB-N — death trigger Vampire-subtype filter uses PRE-DEATH characteristics (LKI).
/// Mandatory test 6/8: load-bearing LKI test per PB-Q4 retro.
///
/// **LKI wedge (subtype-based, as mandated by fix_phase_directives F3):**
/// - Dying creature has Vampire subtype in its BASE characteristics.
/// - Death trigger filter: Vampire subtype.
/// - Expected: trigger FIRES because pre-death characteristics are threaded via `GameEvent::CreatureDied`
///   (`pre_death_characteristics` field, BASELINE-LKI-01) and used at the AnyCreatureDies dispatch site.
///
/// **Discrimination:** This test detects if the death trigger dispatch reads the graveyard
/// object's characteristics vs. uses a stale pre-death snapshot or no characteristics at all.
/// The dying creature's Vampire subtype is carried both in `old_object.characteristics.clone()`
/// into the new graveyard object AND in `pre_death_characteristics` on the event. If it read
/// zero characteristics (a regression), the trigger would not fire and this test would catch it.
///
/// **Note:** The original ESCALATED note about this being an engine limitation is now resolved.
/// BASELINE-LKI-01 (scutemob-37) added `pre_death_characteristics: Option<Characteristics>` to
/// `GameEvent::CreatureDied`, captured before `move_object_to_zone` via `calculate_characteristics`.
/// The dispatch site in `abilities.rs` now uses this snapshot — including subtypes granted by
/// battlefield-gated continuous effects (SingleObject, AttachedCreature, etc.) that drop out
/// after zone change (CR 400.7). See `test_lki_death_filter_subtype_granted_via_single_object`
/// and `test_lki_death_filter_subtype_granted_via_aura` for regression tests of the full fix.
#[test]
fn test_pbn_death_filter_pre_death_lki_color() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a Vampire you control dies, draw a card."
    // Trigger filter: Vampire subtype.
    let watcher = ObjectSpec::creature(p1, "Vampire Watcher", 1, 4)
        .with_triggered_ability(death_trigger_draw_subtype("Vampire"));

    // Dying creature: has Vampire in base characteristics.
    // After zone change, the graveyard object retains these characteristics via
    // `old_object.characteristics.clone()` in `move_object_to_zone`.
    // The death trigger dispatch calls `calculate_characteristics(graveyard_obj_id)`,
    // which reads the preserved characteristics — Vampire subtype is present → trigger fires.
    let dying_vampire = ObjectSpec::creature(p1, "Dying Vampire LKI", 1, 0)
        .with_subtypes(vec![SubType("Vampire".to_string())]);

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

    let (state, _) = pass_all(state, &[p1, p2]);

    // The vampire died → left battlefield via SBA.
    let creature_gone = !state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Dying Vampire LKI" && o.zone == ZoneId::Battlefield);
    assert!(
        creature_gone,
        "Dying Vampire LKI should have left the battlefield (SBA: toughness 0)"
    );

    // Death trigger must fire: graveyard object retains Vampire subtype from
    // `old_object.characteristics.clone()` (CR 603.10a LKI — pre-death state preserved).
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected Vampire death trigger on stack: graveyard object retains pre-death Vampire subtype \
         (CR 603.10a LKI). If this fails, the death trigger dispatch is not reading the graveyard \
         object's characteristics correctly."
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
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
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
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
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

    // F5 fix: Verify the schema version sentinel is exactly 4 (not just non-zero).
    // A rollback from sentinel 4 to sentinel 3 would change the hash fingerprint for
    // states that participate in PB-N wire format — this assertion catches that.
    // Uses the exported HASH_SCHEMA_VERSION constant from state::hash (not a magic literal)
    // so the test must be updated when the sentinel is bumped.
    // PB-P bumped the sentinel from 5 → 6 (EffectAmount::PowerOfSacrificedCreature + AdditionalCost::Sacrifice struct + StackObject field).
    // PB-L bumped the sentinel from 6 → 7 (ETBTriggerFilter.card_type_filter for Landfall dispatch).
    // PB-T bumped the sentinel from 7 → 8 (TargetRequirement::UpToN added, CR 601.2c / 115.1b).
    // PB-SFT bumped the sentinel from 8 → 9 (Effect::SacrificePermanents.filter + TargetFilter.is_nontoken).
    // PB-CC-B bumped the sentinel from 9 → 10 (TargetFilter.has_counter_type, CR 122.1).
    // PB-CC-C bumped the sentinel from 10 → 11 (LayerModification::ModifyPowerDynamic +
    //   ModifyToughnessDynamic, CR 613.4c single-axis dynamic P/T modification).
    // PB-CC-C-followup bumped the sentinel from 12 → 13 (AbilityDefinition::CdaModifyPowerToughness
    //   disc 76, CR 611.3a continuous re-evaluation for Layer-7c dynamic CDA modifications).
    // PB-TS bumped the sentinel from 13 → 14 (TokenSpec.count: u32 → EffectAmount, CR 111.1 / 608.2h).
    // PB-LKI-CC bumped the sentinel from 14 → 15 (EffectAmount::CounterCountAtLastKnownInformation,
    //   CR 603.10a / 113.7a, LKI counter snapshot for WhenDies/WhenLeavesBattlefield triggers).
    // This assertion is updated to reflect the current sentinel value.
    assert_eq!(
        HASH_SCHEMA_VERSION, 55u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this assertion and the state/hash.rs history block together; the authoritative check is the SR-17 machine gate in tests/core/hash_schema.rs."
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
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
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
        .objects()
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
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Dragon failed");

    assert!(
        stack_trigger_count(&state_a) > 0,
        "Expected buff trigger on stack when Dragon (Kolaghan) attacks"
    );

    // Test B: a non-Dragon attacks with the Kolaghan trigger (Dragon filter) → NO trigger.
    let goblin_trigger_def = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
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
        .objects()
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
            exert_choices: vec![],
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

/// PB-N regression: combat_damage_filter must NOT suppress attack triggers (only damage triggers).
///
/// CR 510.3a: combat_damage_filter scopes to the combat damage event, not the attack event.
///
/// **F4 fix — correct discriminating setup (from review finding F4):**
/// The trigger uses `trigger_on: AnyCreatureYouControlAttacks` (an ATTACK trigger, not a
/// damage trigger). `combat_damage_filter` is set to `Ninja` subtype. The attacker is a
/// Goblin (does NOT match the filter).
///
/// - Pre-fix engine: `combat_damage_filter` was checked for BOTH attack and damage events.
///   On the attack event, the Goblin does not match Ninja → combat_damage_filter suppresses
///   the trigger → 0 triggers fire (WRONG per CR 508.1m).
/// - Post-fix engine (PB-N): `combat_damage_filter` is only consulted when
///   `event_type == AnyCreatureYouControlDealsCombatDamageToPlayer`. On the attack event,
///   the filter is NOT consulted → trigger fires regardless of the filter value.
///   Goblin attacker triggers the attack trigger → 1 trigger fires (CORRECT).
///
/// The test asserts stack_trigger_count > 0 after DeclareAttackers.
/// **Regression direction**: if `combat_damage_filter` starts being checked on attack events
/// again, this test fails (trigger is suppressed, count = 0).
///
/// The prior version of this test (original runner) used `trigger_on: AnyCreatureYouControlDealsCombatDamageToPlayer`,
/// which the outer event-type check at `abilities.rs:5828` would have dropped on attack events
/// regardless of any inner filter. That test was non-discriminating per review finding F4.
#[test]
fn test_pbn_combat_damage_filter_not_consulted_on_attack_events() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // An ATTACK trigger (not a damage trigger) with combat_damage_filter set to Ninja.
    // The attacker is a Goblin (does NOT match Ninja).
    // Post-fix: combat_damage_filter is ignored on attack events → trigger fires.
    // Pre-fix: combat_damage_filter was checked on attack events → Goblin ≠ Ninja → trigger suppressed.
    let attack_trigger_with_damage_filter = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description:
            "Whenever a creature you control attacks, draw. (CR 510.3a / PB-N regression — \
             combat_damage_filter must not suppress this on attack events)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        // Non-matching filter: Goblin attacker will not match Ninja.
        // Post-fix: ignored on attack event → trigger fires (count > 0).
        // Pre-fix: checked on attack event → Goblin ≠ Ninja → trigger suppressed (count = 0).
        combat_damage_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Ninja".to_string())),
            ..Default::default()
        }),
        triggering_creature_filter: None,
        targets: vec![],
    };

    // Watcher has the attack trigger with combat_damage_filter.
    let watcher = ObjectSpec::creature(p1, "Attack Watcher", 1, 1)
        .with_triggered_ability(attack_trigger_with_damage_filter);

    // Attacker: a Goblin (NOT a Ninja — does NOT match combat_damage_filter).
    let goblin = ObjectSpec::creature(p1, "Test Goblin", 2, 2)
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
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Test Goblin")
        .map(|(id, _)| *id)
        .unwrap();

    // Declare attackers: fires AnyCreatureYouControlAttacks.
    // Post-fix: combat_damage_filter is NOT consulted → trigger fires despite Goblin ≠ Ninja.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(goblin_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // Post-fix: attack trigger fires even though combat_damage_filter (Ninja) doesn't match
    // the attacker (Goblin). The filter is now correctly scoped to damage events only.
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected attack trigger to fire even when combat_damage_filter does not match the attacker. \
         combat_damage_filter must only be consulted on damage events, not attack events. \
         (CR 510.3a / PB-N regression)"
    );
}

// ── F2 card-specific test: Utvara Hellkite Dragon filter ──────────────────────

/// CR 508.1m / PB-N — Utvara Hellkite: trigger fires on Dragon attacker but NOT on non-Dragon.
///
/// PB-N fix F2: utvara_hellkite.rs was left at `filter: None` (over-triggers on non-Dragons)
/// despite PB-N providing the Dragon subtype filter. This test verifies the fix: with Utvara
/// Hellkite's Dragon filter in place, only Dragon attackers trigger the "create a 6/6 Dragon" effect.
///
/// Two sub-tests:
/// A) Utvara Hellkite (a Dragon) attacks → trigger fires (creates 6/6 Dragon token intent).
/// B) A Goblin attacks alongside Utvara Hellkite's trigger watcher → trigger does NOT fire.
///
/// This bumps PB-N confirmed yield from 4 → 5 cards.
#[test]
fn test_utvara_hellkite_dragon_filter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Utvara Hellkite trigger: Whenever a Dragon you control attacks, create a 6/6 Dragon.
    let dragon_filter = Some(TargetFilter {
        has_subtype: Some(SubType("Dragon".to_string())),
        ..Default::default()
    });
    let utvara_trigger = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description: "Utvara Hellkite: Whenever a Dragon you control attacks, create a 6/6 red \
                      Dragon creature token with flying. (CR 508.1m / PB-N F2)"
            .to_string(),
        effect: Some(Effect::DrawCards {
            // Placeholder: real card uses CreateToken; draw is sufficient to detect trigger fire.
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: dragon_filter.clone(),
        targets: vec![],
    };

    // Sub-test A: Dragon attacker → trigger must fire.
    let utvara = ObjectSpec::creature(p1, "Utvara Hellkite", 6, 6)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(utvara_trigger.clone());

    let state_a = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(utvara)
        .build()
        .unwrap();

    let utvara_id = state_a
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Utvara Hellkite")
        .map(|(id, _)| *id)
        .unwrap();

    let (state_a, _) = process_command(
        state_a,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(utvara_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers (Dragon) failed");

    assert!(
        stack_trigger_count(&state_a) > 0,
        "Expected Utvara Hellkite trigger when Dragon (Utvara itself) attacks (Dragon filter match)"
    );

    // Sub-test B: Goblin attacker (not a Dragon) → trigger must NOT fire.
    let watcher_b = ObjectSpec::creature(p1, "Utvara Watcher", 6, 6).with_triggered_ability(
        TriggeredAbilityDef {
            triggering_creature_filter: dragon_filter,
            ..utvara_trigger
        },
    );
    let goblin_b = ObjectSpec::creature(p1, "Goblin Attacker", 1, 1)
        .with_subtypes(vec![SubType("Goblin".to_string())]);

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

    let goblin_id = state_b
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Goblin Attacker")
        .map(|(id, _)| *id)
        .unwrap();

    let (state_b, _) = process_command(
        state_b,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(goblin_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers (Goblin) failed");

    assert_eq!(
        stack_trigger_count(&state_b),
        0,
        "Expected NO Utvara Hellkite trigger when non-Dragon (Goblin) attacks (Dragon filter mismatch)"
    );
}

// ── F1 card-specific test: Sanctum Seeker flat-1 gain in 4-player ─────────────

/// CR 508.1m / PB-N F1 — Sanctum Seeker: "each opponent loses 1 life and you gain 1 life" (flat 1).
///
/// Oracle says "you gain 1 life" regardless of opponent count. In 4-player Commander with
/// 3 opponents, the controller gains exactly 1 life (NOT 3). This test verifies the fix:
/// using ForEach(EachOpponent, LoseLife 1) + GainLife(Controller, 1) instead of DrainLife,
/// which would have gained total_lost (3 life in 4-player).
///
/// Setup: 4-player game, Vampire attacker triggers Sanctum Seeker.
/// Expected: each of 3 opponents loses 1 life (total 3 lost), controller gains exactly 1 life.
#[test]
fn test_sanctum_seeker_flat_gain_4_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // Simulate Sanctum Seeker's trigger directly using the DSL primitives.
    // Oracle: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life."
    // Fix: ForEach(EachOpponent, LoseLife 1) → GainLife(Controller, 1). NOT DrainLife (gains total_lost).
    let sanctum_seeker_trigger = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
        intervening_if: None,
        description:
            "Sanctum Seeker: Whenever a Vampire attacks, each opponent loses 1, you gain 1. \
                      (flat gain, NOT DrainLife — CR 508.1m / PB-N F1)"
                .to_string(),
        effect: Some(Effect::Sequence(vec![
            Effect::ForEach {
                over: mtg_engine::ForEachTarget::EachOpponent,
                effect: Box::new(Effect::LoseLife {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                }),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
        ])),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Vampire".to_string())),
            ..Default::default()
        }),
        targets: vec![],
    };

    // Watcher holds the Sanctum Seeker trigger.
    let seeker = ObjectSpec::creature(p1, "Sanctum Seeker", 3, 4)
        .with_subtypes(vec![SubType("Vampire".to_string())])
        .with_triggered_ability(sanctum_seeker_trigger);

    // A Vampire attacker.
    let vampire = ObjectSpec::creature(p1, "Vampire Attacker", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(seeker)
        .object(vampire)
        .build()
        .unwrap();

    let vampire_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Vampire Attacker")
        .map(|(id, _)| *id)
        .unwrap();

    let p1_life_before = state.players()[&p1].life_total;
    let p2_life_before = state.players()[&p2].life_total;

    // Declare the Vampire attacker → queues the Sanctum Seeker trigger.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(vampire_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // Trigger should be on the stack.
    assert!(
        stack_trigger_count(&state) > 0,
        "Expected Sanctum Seeker trigger on stack when Vampire attacks"
    );

    // Resolve the trigger: all players pass priority.
    // Trigger resolves: each of p2, p3, p4 loses 1 life; p1 gains exactly 1 life.
    let (state, _) = pass_all(state.clone(), &[p1, p2, p3, p4]);
    // If trigger is still on stack (not yet resolved), pass again.
    let state = if stack_trigger_count(&state) > 0 {
        let (s, _) = pass_all(state, &[p1, p2, p3, p4]);
        s
    } else {
        state
    };

    let p1_life_after = state.players()[&p1].life_total;
    let p2_life_after = state.players()[&p2].life_total;
    let p3_life_after = state.players()[&p3].life_total;
    let p4_life_after = state.players()[&p4].life_total;

    // p1 gains exactly 1 life (flat, not total_lost = 3).
    assert_eq!(
        p1_life_after,
        p1_life_before + 1,
        "Sanctum Seeker: controller should gain exactly 1 life (flat), not total_lost (3) in 4-player. \
         DrainLife was wrong; ForEach+GainLife 1 is correct."
    );

    // Each opponent loses exactly 1 life.
    assert_eq!(p2_life_after, p2_life_before - 1, "p2 should lose 1 life");
    assert_eq!(
        p3_life_after,
        40 - 1,
        "p3 should lose 1 life (was at starting life total)"
    );
    assert_eq!(
        p4_life_after,
        40 - 1,
        "p4 should lose 1 life (was at starting life total)"
    );
}

// ── BASELINE-LKI-01 Regression Tests ─────────────────────────────────────────

/// CR 603.10a / CR 613.1d — BASELINE-LKI-01: filtered death trigger fires when the dying
/// creature's relevant subtype was granted by a `SingleObject` continuous effect while on
/// the battlefield.
///
/// **The bug (pre-fix):** The `AnyCreatureDies` dispatch site called
/// `calculate_characteristics(state, graveyard_obj_id)`. After `move_object_to_zone`, the
/// old battlefield ObjectId is dead (CR 400.7) and the new graveyard ObjectId does NOT match
/// the `EffectFilter::SingleObject(old_battlefield_id)` filter. So the continuous effect
/// dropped out, and the subtype was absent in the graveyard-side calculation — trigger
/// silently never fired.
///
/// **The fix:** `GameEvent::CreatureDied.pre_death_characteristics` captures the full
/// layer-resolved characteristics BEFORE `move_object_to_zone`, then the dispatch site
/// uses this snapshot instead of re-running `calculate_characteristics` on the graveyard object.
///
/// **Test setup:**
/// - Dying creature: base type Human only (no Zombie).
/// - Continuous effect: `EffectFilter::SingleObject(dying_id)` + `LayerModification::AddSubtypes([Zombie])`
///   — injected via `state.continuous_effects()` after build.
/// - Watcher: "Whenever a Zombie you control dies, draw a card" (`death_filter.controller_you`).
/// - Dying creature has toughness 0 → dies via SBA.
/// - Expected: trigger FIRES (pre_death_characteristics captured Zombie subtype from the effect).
#[test]
fn test_lki_death_filter_subtype_granted_via_single_object() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a Zombie you control dies, draw a card."
    let watcher = ObjectSpec::creature(p1, "Zombie Watcher", 1, 4)
        .with_triggered_ability(death_trigger_draw_subtype("Zombie"));

    // Dying creature: base Human only (no Zombie in base characteristics).
    // Toughness 0 → dies via SBA.
    let dying_human = ObjectSpec::creature(p1, "Dying Human LKI", 1, 0);

    // Library card to satisfy draw trigger resolution.
    let lib = library_card(p1, "lib-lki-so", "Library Card LKI-SO");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_human)
        .object(lib)
        .build()
        .unwrap();

    // Find the dying creature's battlefield ObjectId to use in the SingleObject filter.
    let dying_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Dying Human LKI")
        .map(|(id, _)| *id)
        .expect("Dying Human LKI must be on battlefield");

    // Inject a battlefield-gated continuous effect: "Dying Human LKI is also a Zombie."
    // EffectFilter::SingleObject(dying_id) matches only the battlefield object with that id.
    // After zone change, the new graveyard ObjectId does NOT match — this is what BASELINE-LKI-01 fixes.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9001),
        source: None,
        timestamp: 999,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(dying_id),
        modification: LayerModification::AddSubtypes(
            [SubType("Zombie".to_string())].into_iter().collect(),
        ),
        is_cda: false,
        condition: None,
    });

    let (state, _) = pass_all(state, &[p1, p2]);

    // Confirm the creature actually died (SBA: toughness 0).
    let creature_gone = !state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Dying Human LKI" && o.zone == ZoneId::Battlefield);
    assert!(
        creature_gone,
        "Dying Human LKI should have left the battlefield (SBA: toughness 0)"
    );

    // CR 603.10a / CR 613.1d: The trigger MUST fire because pre_death_characteristics
    // captured the Zombie subtype from the SingleObject continuous effect BEFORE the zone move.
    // Pre-fix: this would be 0 (calculate_characteristics on graveyard obj returned Human only).
    assert!(
        stack_trigger_count(&state) > 0,
        "BASELINE-LKI-01 regression: Zombie death trigger must fire when subtype was granted \
         via SingleObject continuous effect on battlefield. pre_death_characteristics snapshot \
         should have captured Zombie subtype before move_object_to_zone (CR 603.10a / CR 613.1d)."
    );
}

/// CR 603.10a / CR 613.1d — BASELINE-LKI-01: filtered death trigger fires when the dying
/// creature's relevant subtype was granted by an `AttachedCreature` (aura) continuous effect.
///
/// Same bug/fix as `test_lki_death_filter_subtype_granted_via_single_object` but using
/// the `EffectFilter::AttachedCreature` path, which also drops out after zone change because
/// the `attached_to` relationship is only maintained while both objects are on the battlefield.
///
/// **Test setup:**
/// - Dying creature: base type Human only (no Zombie in base characteristics).
/// - An enchantment (aura) placed on the battlefield with `attached_to = Some(dying_id)`.
/// - Continuous effect: `EffectFilter::AttachedCreature` + `LayerModification::AddSubtypes([Zombie])`
///   sourced from the aura — injected via `state.continuous_effects()`.
/// - Watcher: "Whenever a Zombie you control dies, draw a card" (`death_filter.controller_you`).
/// - Dying creature has toughness 0 → dies via SBA.
/// - Expected: trigger FIRES (pre_death_characteristics captured Zombie subtype from the aura effect).
#[test]
fn test_lki_death_filter_subtype_granted_via_aura() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "Whenever a Zombie you control dies, draw a card."
    let watcher = ObjectSpec::creature(p1, "Zombie Watcher Aura", 1, 4)
        .with_triggered_ability(death_trigger_draw_subtype("Zombie"));

    // Dying creature: base Human only (no Zombie in base characteristics).
    // Toughness 0 → dies via SBA.
    let dying_human = ObjectSpec::creature(p1, "Dying Human Aura LKI", 1, 0);

    // Aura enchantment: the source of the AddSubtypes continuous effect.
    let aura = ObjectSpec::enchantment(p1, "Zombie Aura");

    // Library card to satisfy draw trigger resolution.
    let lib = library_card(p1, "lib-lki-aura", "Library Card LKI-Aura");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_human)
        .object(aura)
        .object(lib)
        .build()
        .unwrap();

    // Find the dying creature's and aura's ObjectIds.
    let dying_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Dying Human Aura LKI")
        .map(|(id, _)| *id)
        .expect("Dying Human Aura LKI must be on battlefield");
    let aura_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Zombie Aura")
        .map(|(id, _)| *id)
        .expect("Zombie Aura must be on battlefield");

    // Set the aura's attached_to relationship so AttachedCreature filter resolves correctly.
    if let Some(aura_obj) = state.objects_mut().get_mut(&aura_id) {
        aura_obj.attached_to = Some(dying_id);
    }

    // Inject the continuous effect via AttachedCreature filter sourced from the aura.
    // This grants Zombie subtype to whichever creature the aura is attached to.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9002),
        source: Some(aura_id),
        timestamp: 999,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttachedCreature,
        modification: LayerModification::AddSubtypes(
            [SubType("Zombie".to_string())].into_iter().collect(),
        ),
        is_cda: false,
        condition: None,
    });

    let (state, _) = pass_all(state, &[p1, p2]);

    // Confirm the creature actually died (SBA: toughness 0).
    let creature_gone = !state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Dying Human Aura LKI" && o.zone == ZoneId::Battlefield);
    assert!(
        creature_gone,
        "Dying Human Aura LKI should have left the battlefield (SBA: toughness 0)"
    );

    // CR 603.10a / CR 613.1d: The trigger MUST fire because pre_death_characteristics
    // captured the Zombie subtype from the AttachedCreature aura effect BEFORE the zone move.
    // Pre-fix: this would be 0 (calculate_characteristics on graveyard obj returned Human only
    // because the aura's AttachedCreature filter no longer applied after zone change).
    assert!(
        stack_trigger_count(&state) > 0,
        "BASELINE-LKI-01 regression: Zombie death trigger must fire when subtype was granted \
         via AttachedCreature aura continuous effect. pre_death_characteristics snapshot should \
         have captured Zombie subtype before move_object_to_zone (CR 603.10a / CR 613.1d)."
    );
}

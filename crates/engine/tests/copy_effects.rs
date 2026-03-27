//! Copy effect tests: Layer 1 copiable values and clone chain (CR 707).
//!
//! Session 7 of M9.4 implements `LayerModification::CopyOf` in the layer system,
//! backed by `rules::copy::get_copiable_values` and its recursive clone-chain
//! resolution (CR 707.3).

use mtg_engine::{
    calculate_characteristics, CardType, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec,
    PlayerId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}

/// Build a `ContinuousEffect` with Layer 1 Copy semantics.
fn copy_effect_of(
    id: u64,
    copier_id: ObjectId,
    source_id: ObjectId,
    timestamp: u64,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: Some(copier_id),
        timestamp,
        layer: EffectLayer::Copy,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(copier_id),
        modification: LayerModification::CopyOf(source_id),
        is_cda: false,
        condition: None,
    }
}

// ── CR 707.2: Clone copies a Bear ────────────────────────────────────────────

/// CR 707.2 — A Clone enters copying a Grizzly Bears.
/// The Clone's calculated characteristics must equal the Bear's: 2/2 creature.
/// The Clone's printed characteristics (e.g., "Clone: 0/0 Creature") are replaced
/// by the Bear's copiable values.
#[test]
fn test_clone_copies_bear() {
    let p = p1();

    // The Bear: a 2/2 creature with subtype "Bear"
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);

    // The Clone: printed as a 0/0 creature (its copiable values will be replaced)
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Apply Layer 1 copy effect: Clone copies the Bear.
    let copy_eff = copy_effect_of(100, clone_id, bear_id, 50);
    state.continuous_effects.push_back(copy_eff);

    let chars = calculate_characteristics(&state, clone_id).unwrap();

    // The Clone should now have the Bear's name and stats.
    assert_eq!(chars.name, "Grizzly Bears", "Clone should have Bear's name");
    assert_eq!(chars.power, Some(2), "Clone should have Bear's power (2)");
    assert_eq!(
        chars.toughness,
        Some(2),
        "Clone should have Bear's toughness (2)"
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Clone should be a creature"
    );
}

// ── CR 707.3: Clone copying a Clone (clone chain) ────────────────────────────

/// CR 707.3 / CC#5 — A Clone-A copies Clone-B, and Clone-B is copying a Bear.
/// Clone-A must end up with the Bear's characteristics, NOT Clone-B's printed
/// characteristics.  This is the core of the clone chain rule.
#[test]
fn test_clone_copies_clone_chain() {
    let p = p1();

    // The original Bear: 2/2 Green creature with Flying keyword.
    let mut bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    bear_spec = bear_spec.with_keyword(KeywordAbility::Flying);

    // Clone-B: enters copying the Bear (its copiable values → Bear's values).
    let clone_b_spec = ObjectSpec::creature(p, "Clone-B", 0, 0);

    // Clone-A: enters copying Clone-B.
    let clone_a_spec = ObjectSpec::creature(p, "Clone-A", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_b_spec)
        .object(clone_a_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_b_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone-B")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_a_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone-A")
        .map(|(id, _)| *id)
        .unwrap();

    // Clone-B copies the Bear (timestamp 10).
    let eff_b = copy_effect_of(101, clone_b_id, bear_id, 10);
    // Clone-A copies Clone-B (timestamp 20, applied after eff_b).
    let eff_a = copy_effect_of(102, clone_a_id, clone_b_id, 20);

    state.continuous_effects.push_back(eff_b);
    state.continuous_effects.push_back(eff_a);

    let chars_a = calculate_characteristics(&state, clone_a_id).unwrap();

    // CR 707.3: Clone-A copying Clone-B sees Clone-B's copiable values AFTER
    // the copy effect on Clone-B is applied. Clone-B's copiable values = Bear's values.
    // So Clone-A should look like the Bear, NOT like Clone-B's printed form (0/0).
    assert_eq!(
        chars_a.name, "Grizzly Bears",
        "Clone-A should have Bear's name (not Clone-B's)"
    );
    assert_eq!(
        chars_a.power,
        Some(2),
        "Clone-A should have Bear's power via clone chain"
    );
    assert_eq!(
        chars_a.toughness,
        Some(2),
        "Clone-A should have Bear's toughness via clone chain"
    );
    assert!(
        chars_a.keywords.contains(&KeywordAbility::Flying),
        "Clone-A should inherit Flying from the Bear via clone chain"
    );

    // Also verify Clone-B independently looks like the Bear.
    let chars_b = calculate_characteristics(&state, clone_b_id).unwrap();
    assert_eq!(chars_b.name, "Grizzly Bears");
    assert_eq!(chars_b.power, Some(2));
}

// ── Layer 1 applies before other layers ──────────────────────────────────────

/// CR 613.1a — Layer 1 (Copy) applies before Layer 6 (Ability) and Layer 7 (P/T).
/// A Clone copies a Bear (Layer 1).  A separate +2/+2 continuous effect (Layer 7c)
/// applies to the Clone after the copy resolves.  The final characteristics must be
/// the Bear's base stats plus the modification — NOT the Clone's printed stats.
#[test]
fn test_copy_effect_layer_1_applies_before_other_layers() {
    use mtg_engine::{ContinuousEffect, EffectDuration, EffectId, EffectLayer, LayerModification};

    let p = p1();

    // Bear: 2/2
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    // Clone: printed as 0/0
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Layer 1: Clone copies the Bear (timestamp 10).
    let copy_eff = copy_effect_of(200, clone_id, bear_id, 10);

    // Layer 7c: +2/+2 on the Clone (timestamp 20, applied after Layer 1).
    let pump_eff = ContinuousEffect {
        id: EffectId(201),
        source: None,
        timestamp: 20,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(clone_id),
        modification: LayerModification::ModifyBoth(2),
        is_cda: false,
        condition: None,
    };

    state.continuous_effects.push_back(copy_eff);
    state.continuous_effects.push_back(pump_eff);

    let chars = calculate_characteristics(&state, clone_id).unwrap();

    // Layer 1 applies first: Clone gets Bear's 2/2 base.
    // Layer 7c applies after: +2/+2 brings it to 4/4.
    assert_eq!(
        chars.power,
        Some(4),
        "Layer 1 (copy) + Layer 7c (+2) should give 4 power"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "Layer 1 (copy) + Layer 7c (+2) should give 4 toughness"
    );
    assert_eq!(
        chars.name, "Grizzly Bears",
        "Name should be from the Bear (Layer 1 applied)"
    );
}

// ── Copy does not copy counters or status ─────────────────────────────────────

/// CR 707.2 — Copy effects copy COPIABLE VALUES only.
/// Counters, damage marked, status (tapped/summoning sickness), zone, and controller
/// are NOT copied (CR 707.2 exclusions).  This test verifies those exclusions.
#[test]
fn test_copy_does_not_copy_counters_or_status() {
    let p = p1();

    // Bear: 2/2 with two +1/+1 counters (should NOT be copied).
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);

    // Clone: 0/0 entering copying the Bear.
    let clone_spec = ObjectSpec::creature(p, "Clone", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Clone")
        .map(|(id, _)| *id)
        .unwrap();

    // Add +1/+1 counters to the Bear (should not be copied to Clone).
    {
        use mtg_engine::CounterType;
        let bear = state.objects.get_mut(&bear_id).unwrap();
        bear.counters.insert(CounterType::PlusOnePlusOne, 2);
    }

    // Tap the Bear (status should not be copied).
    {
        let bear = state.objects.get_mut(&bear_id).unwrap();
        bear.status.tapped = true;
    }

    // Apply Layer 1 copy effect.
    let copy_eff = copy_effect_of(300, clone_id, bear_id, 10);
    state.continuous_effects.push_back(copy_eff);

    // Verify Clone characteristics after copy.
    let clone_chars = calculate_characteristics(&state, clone_id).unwrap();

    // CR 707.2: the Clone copies Bear's PRINTED values, not the counters.
    // The Bear's printed P/T is 2/2. Counters are applied in Layer 7c to the Bear,
    // but the Clone's copy effect copies only the printed/copiable values (2/2),
    // not the counter-adjusted value (4/4).
    assert_eq!(
        clone_chars.name, "Grizzly Bears",
        "Name should be copied (copiable)"
    );
    assert_eq!(
        clone_chars.power,
        Some(2),
        "Power should be Bear's printed value (counters are NOT copiable)"
    );
    assert_eq!(
        clone_chars.toughness,
        Some(2),
        "Toughness should be Bear's printed value (counters are NOT copiable)"
    );

    // The Clone's own status (not tapped) should be independent of the Bear's status.
    let clone_obj = state.objects.get(&clone_id).unwrap();
    assert!(
        !clone_obj.status.tapped,
        "Clone's tapped status should be independent of Bear's (status is NOT copiable)"
    );

    // The Bear should still have its counters.
    let bear_chars = calculate_characteristics(&state, bear_id).unwrap();
    assert_eq!(
        bear_chars.power,
        Some(4),
        "Bear should still have its counter-boosted power (4)"
    );
}

// ── PB-22 S5: Effect::BecomeCopyOf and Effect::CreateTokenCopy ──────────────

/// CR 707.2: Effect::BecomeCopyOf — test the full effect execution path.
#[test]
fn test_effect_become_copy_of() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::EffectContext;
    use mtg_engine::state::{SpellTarget, Target};

    let p = p1();
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    let clone_spec = ObjectSpec::creature(p, "Shapeshifter", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shapeshifter")
        .map(|(id, _)| *id)
        .unwrap();

    let effect = Effect::BecomeCopyOf {
        copier: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
        duration: EffectDuration::UntilEndOfTurn,
    };

    let target = SpellTarget {
        target: Target::Object(bear_id),
        zone_at_cast: Some(mtg_engine::state::zone::ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p, clone_id, vec![target]);

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Verify BecameCopyOf event was emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            mtg_engine::GameEvent::BecameCopyOf { copier, source }
            if *copier == clone_id && *source == bear_id
        )),
        "should emit BecameCopyOf event"
    );

    // Verify the copy took effect via the layer system.
    let chars = calculate_characteristics(&state, clone_id).unwrap();
    assert_eq!(chars.name, "Grizzly Bears", "should have Bear's name");
    assert_eq!(chars.power, Some(2), "should have Bear's power");
}

/// CR 707.2: Effect::BecomeCopyOf with UntilEndOfTurn — verify reversion.
#[test]
fn test_effect_become_copy_reverts_at_eot() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::EffectContext;
    use mtg_engine::state::{SpellTarget, Target};

    let p = p1();
    let bear_spec = ObjectSpec::creature(p, "Grizzly Bears", 2, 2);
    let clone_spec = ObjectSpec::creature(p, "Shapeshifter", 0, 0);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(bear_spec)
        .object(clone_spec)
        .build()
        .unwrap();

    let bear_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Grizzly Bears")
        .map(|(id, _)| *id)
        .unwrap();
    let clone_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shapeshifter")
        .map(|(id, _)| *id)
        .unwrap();

    let effect = Effect::BecomeCopyOf {
        copier: EffectTarget::Source,
        target: EffectTarget::DeclaredTarget { index: 0 },
        duration: EffectDuration::UntilEndOfTurn,
    };

    let target = SpellTarget {
        target: Target::Object(bear_id),
        zone_at_cast: Some(mtg_engine::state::zone::ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p, clone_id, vec![target]);
    mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Confirm copy is active.
    let chars = calculate_characteristics(&state, clone_id).unwrap();
    assert_eq!(chars.name, "Grizzly Bears");

    // Simulate EOT cleanup: remove UntilEndOfTurn effects.
    state
        .continuous_effects
        .retain(|e| e.duration != EffectDuration::UntilEndOfTurn);

    // Should revert to original.
    let chars_after = calculate_characteristics(&state, clone_id).unwrap();
    assert_eq!(chars_after.name, "Shapeshifter", "should revert after EOT");
    assert_eq!(chars_after.power, Some(0));
}

/// CR 707.2 / CR 111.10: Effect::CreateTokenCopy — create a token copy.
#[test]
fn test_effect_create_token_copy() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::EffectContext;
    use mtg_engine::state::zone::ZoneId;
    use mtg_engine::state::{SpellTarget, Target};

    let p = p1();
    let dragon_spec = ObjectSpec::creature(p, "Shivan Dragon", 5, 5);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(dragon_spec)
        .build()
        .unwrap();

    let dragon_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shivan Dragon")
        .map(|(id, _)| *id)
        .unwrap();

    let effect = Effect::CreateTokenCopy {
        source: EffectTarget::DeclaredTarget { index: 0 },
        enters_tapped_and_attacking: false,
        except_not_legendary: false,
        gains_haste: false,
        delayed_action: None,
    };

    let target = SpellTarget {
        target: Target::Object(dragon_id),
        zone_at_cast: Some(mtg_engine::state::zone::ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p, dragon_id, vec![target]);

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Find the token.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.is_token && o.zone == ZoneId::Battlefield)
        .collect();
    assert_eq!(tokens.len(), 1, "should create one token");
    assert!(!tokens[0].status.tapped, "should not be tapped");

    // Verify copy characteristics.
    let chars = calculate_characteristics(&state, tokens[0].id).unwrap();
    assert_eq!(chars.name, "Shivan Dragon");
    assert_eq!(chars.power, Some(5));
    assert_eq!(chars.toughness, Some(5));

    // Verify events.
    assert!(events
        .iter()
        .any(|e| matches!(e, mtg_engine::GameEvent::TokenCreated { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, mtg_engine::GameEvent::PermanentEnteredBattlefield { .. })));
}

/// CR 508.4: CreateTokenCopy with enters_tapped_and_attacking.
#[test]
fn test_effect_create_token_copy_tapped_attacking() {
    use mtg_engine::cards::card_definition::{Effect, EffectTarget};
    use mtg_engine::effects::EffectContext;
    use mtg_engine::state::combat::{AttackTarget, CombatState};
    use mtg_engine::state::zone::ZoneId;
    use mtg_engine::state::{SpellTarget, Target};

    let p = p1();
    let p2 = PlayerId(2);
    let warrior_spec = ObjectSpec::creature(p, "Elite Warrior", 3, 3);
    let ninja_spec = ObjectSpec::creature(p, "Shadow Ninja", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .add_player(p2)
        .object(warrior_spec)
        .object(ninja_spec)
        .build()
        .unwrap();

    let warrior_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Elite Warrior")
        .map(|(id, _)| *id)
        .unwrap();
    let ninja_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shadow Ninja")
        .map(|(id, _)| *id)
        .unwrap();

    // Set up combat: ninja attacking p2.
    let mut combat = CombatState::new(p);
    combat.attackers.insert(ninja_id, AttackTarget::Player(p2));
    state.combat = Some(combat);

    let effect = Effect::CreateTokenCopy {
        source: EffectTarget::DeclaredTarget { index: 0 },
        enters_tapped_and_attacking: true,
        except_not_legendary: false,
        gains_haste: false,
        delayed_action: None,
    };

    let target = SpellTarget {
        target: Target::Object(warrior_id),
        zone_at_cast: Some(mtg_engine::state::zone::ZoneId::Battlefield),
    };
    let mut ctx = EffectContext::new(p, ninja_id, vec![target]);
    mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Find the token.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.is_token && o.zone == ZoneId::Battlefield)
        .collect();
    assert_eq!(tokens.len(), 1);

    // Token should be tapped and attacking p2.
    assert!(tokens[0].status.tapped, "token should be tapped");
    let combat = state.combat.as_ref().unwrap();
    assert!(combat.attackers.contains_key(&tokens[0].id));
    assert_eq!(
        *combat.attackers.get(&tokens[0].id).unwrap(),
        AttackTarget::Player(p2)
    );

    // Token should have warrior's characteristics.
    let chars = calculate_characteristics(&state, tokens[0].id).unwrap();
    assert_eq!(chars.name, "Elite Warrior");
    assert_eq!(chars.power, Some(3));
}

/// Condition::CardTypesInGraveyardAtLeast (Delirium) evaluation.
#[test]
fn test_delirium_condition_evaluation() {
    use mtg_engine::cards::card_definition::Condition;
    use mtg_engine::effects::EffectContext;

    let p = p1();

    let creature_spec = ObjectSpec::creature(p, "Test Creature", 1, 1);
    let land_spec = ObjectSpec::card(p, "Test Land").with_types(vec![CardType::Land]);
    let instant_spec = ObjectSpec::card(p, "Test Instant").with_types(vec![CardType::Instant]);
    let source_spec = ObjectSpec::creature(p, "Source", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p)
        .object(creature_spec)
        .object(land_spec)
        .object(instant_spec)
        .object(source_spec)
        .build()
        .unwrap();

    let source_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Source")
        .map(|(id, _)| *id)
        .unwrap();

    // Move 3 objects to graveyard for 3 distinct card types.
    let ids_to_move: Vec<ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name.starts_with("Test"))
        .map(|(id, _)| *id)
        .collect();
    for id in ids_to_move {
        let _ = state.move_object_to_zone(id, mtg_engine::state::zone::ZoneId::Graveyard(p));
    }

    let ctx = EffectContext::new(p, source_id, vec![]);

    // 3 card types (Creature, Land, Instant) — delirium(4) = false.
    assert!(
        !mtg_engine::effects::check_condition(
            &state,
            &Condition::CardTypesInGraveyardAtLeast(4),
            &ctx,
        ),
        "3 card types < 4"
    );

    // delirium(3) = true.
    assert!(
        mtg_engine::effects::check_condition(
            &state,
            &Condition::CardTypesInGraveyardAtLeast(3),
            &ctx,
        ),
        "3 card types >= 3"
    );
}

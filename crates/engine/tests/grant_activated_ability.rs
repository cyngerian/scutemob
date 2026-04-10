//! Tests for PB-S: GrantActivatedAbility / GrantManaAbility via Layer 6.
//!
//! LayerModification::AddManaAbility and AddActivatedAbility let static abilities
//! grant a single specific ability to filtered permanents via the Layer 6 path
//! (CR 613.1f). This file validates correctness, filter scoping, grant removal,
//! stacking (multiple Rites), summoning sickness, Humility interaction, face-down
//! inheritance, and card-integration smoke tests.

use im::OrdMap;
use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    calculate_characteristics, process_command, Command, ContinuousEffect, Effect, EffectAmount,
    EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameStateBuilder,
    KeywordAbility, LayerModification, ManaAbility, ManaColor, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

/// Build a Cryptolith-Rite-style grant effect (creatures you control get tap-for-any-color).
/// `source_obj` must be set for CreaturesYouControl filter to resolve the controller.
fn cryptolith_grant(eff_id: u64, source_obj: ObjectId, ts: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(source_obj),
        timestamp: ts,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddManaAbility(ManaAbility {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
        }),
        is_cda: false,
        condition: None,
    }
}

/// Build a Chromatic-Lantern-style grant effect (lands you control get tap-for-any-color).
fn lantern_grant(eff_id: u64, source_obj: ObjectId, ts: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(eff_id),
        source: Some(source_obj),
        timestamp: ts,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::LandsYouControl,
        modification: LayerModification::AddManaAbility(ManaAbility {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
        }),
        is_cda: false,
        condition: None,
    }
}

/// Build a Humility-style RemoveAllAbilities effect with a given timestamp.
fn humility_effect(eff_id: u64, ts: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(eff_id),
        source: None,
        timestamp: ts,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::RemoveAllAbilities,
        is_cda: false,
        condition: None,
    }
}

fn find_obj_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

// ---------------------------------------------------------------------------
// Test 1: Cryptolith Rite grants mana ability to creatures
// CR 613.1f, CR 605.1a
// ---------------------------------------------------------------------------

/// CR 613.1f + CR 605.1a: Layer 6 AddManaAbility grants tap-for-any-color to
/// every creature the source's controller controls. The ability is visible via
/// calculate_characteristics.
#[test]
fn test_cryptolith_rite_grants_mana_ability_to_creatures() {
    // Build state: grant source + two creatures.
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::artifact(p1(), "Cryptolith Rite Token"))
        .object(ObjectSpec::creature(p1(), "Bear Alpha", 2, 2))
        .object(ObjectSpec::creature(p1(), "Bear Beta", 2, 2))
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Cryptolith Rite Token");
    let alpha_id = find_obj_by_name(&state, "Bear Alpha");
    let beta_id = find_obj_by_name(&state, "Bear Beta");

    // Add the grant effect sourced from the token.
    let grant = cryptolith_grant(1, source_id, 10);
    state.continuous_effects.push_back(grant);

    // Both creatures should have the granted mana ability.
    let alpha_chars = calculate_characteristics(&state, alpha_id)
        .expect("calculate_characteristics should succeed");
    let beta_chars = calculate_characteristics(&state, beta_id)
        .expect("calculate_characteristics should succeed");

    assert!(
        !alpha_chars.mana_abilities.is_empty(),
        "Bear Alpha should have the granted tap-for-any-color mana ability (CR 613.1f)"
    );
    assert!(
        alpha_chars
            .mana_abilities
            .iter()
            .any(|a| a.any_color && a.requires_tap),
        "granted ability should be tap-for-any-color"
    );
    assert!(
        !beta_chars.mana_abilities.is_empty(),
        "Bear Beta should also have the granted mana ability (CR 613.5)"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Tapping a creature for granted mana works and adds to pool
// CR 605.1a — granted mana ability resolves immediately, no stack
// ---------------------------------------------------------------------------

/// CR 605.1a: A granted mana ability resolves immediately when activated.
/// Tapping a creature via the granted TapForMana command adds mana to the pool.
#[test]
fn test_granted_mana_ability_taps_and_produces_mana() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .object(ObjectSpec::artifact(p1(), "Rite Source"))
        .object(ObjectSpec::creature(p1(), "Mana Bear", 2, 2))
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1());

    let source_id = find_obj_by_name(&state, "Rite Source");
    let bear_id = find_obj_by_name(&state, "Mana Bear");

    let grant = cryptolith_grant(1, source_id, 10);
    state.continuous_effects.push_back(grant);

    // Verify bear has the granted mana ability at index 0.
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        !chars.mana_abilities.is_empty(),
        "creature should have granted mana ability"
    );

    // Tap for mana using the granted ability (index 0 = the granted one).
    let (state_after, events) = process_command(
        state,
        Command::TapForMana {
            player: p1(),
            source: bear_id,
            ability_index: 0,
        },
    )
    .expect("TapForMana via granted ability should succeed (CR 605.1a)");

    // Bear is now tapped.
    assert!(
        state_after.objects[&bear_id].status.tapped,
        "creature should be tapped after activating tap-for-mana grant"
    );

    // ManaAdded event should have fired.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaAdded { player, .. } if *player == p1())),
        "ManaAdded event should be emitted when creature taps for granted mana (CR 605.1a)"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Grant ends when source leaves battlefield
// CR 611.3a + CR 400.7 + EffectDuration::WhileSourceOnBattlefield
// ---------------------------------------------------------------------------

/// CR 611.3a + CR 400.7: When the Cryptolith Rite leaves the battlefield,
/// the continuous effect ends; creatures lose the granted mana ability.
/// Uses EffectDuration::WhileSourceOnBattlefield to confirm expiry logic.
#[test]
fn test_cryptolith_rite_grant_ends_when_source_leaves() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .object(ObjectSpec::artifact(p1(), "Cryptolith Rite"))
        .object(ObjectSpec::creature(p1(), "Forest Bear", 2, 2))
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1());

    let rite_id = find_obj_by_name(&state, "Cryptolith Rite");
    let bear_id = find_obj_by_name(&state, "Forest Bear");

    // Add the grant effect with WhileSourceOnBattlefield duration.
    let grant = ContinuousEffect {
        id: EffectId(1),
        source: Some(rite_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddManaAbility(ManaAbility {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
        }),
        is_cda: false,
        condition: None,
    };
    state.continuous_effects.push_back(grant);

    // Bear has the grant initially.
    let chars_before = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        chars_before.mana_abilities.iter().any(|a| a.any_color),
        "bear should have tap-for-any-color from Cryptolith Rite"
    );

    // Move Rite to graveyard — simulates it leaving the battlefield.
    // is_effect_active in layers.rs checks WhileSourceOnBattlefield against source zone.
    state.objects.get_mut(&rite_id).unwrap().zone = ZoneId::Graveyard(p1());

    // Now bear should lose the grant (source not on battlefield).
    let chars_after = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        !chars_after.mana_abilities.iter().any(|a| a.any_color),
        "bear should lose tap-for-any-color when Cryptolith Rite leaves (CR 611.3a)"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Two Cryptolith Rites grant two abilities, but only one fires per tap
// CR 613.5, Cryptolith Rite ruling 2016-04-08
// ---------------------------------------------------------------------------

/// CR 613.5 + Cryptolith Rite ruling 2016-04-08: Two separate grant sources each
/// push one mana ability entry onto the creature. But the `{T}` cost on the first
/// activation taps the creature; the second activation fails because it's tapped.
#[test]
fn test_two_cryptolith_rites_grant_two_abilities_but_one_tap() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .object(ObjectSpec::artifact(p1(), "Rite Source 1"))
        .object(ObjectSpec::artifact(p1(), "Rite Source 2"))
        .object(ObjectSpec::creature(p1(), "Dual Bear", 2, 2))
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1());

    let source1_id = find_obj_by_name(&state, "Rite Source 1");
    let source2_id = find_obj_by_name(&state, "Rite Source 2");
    let bear_id = find_obj_by_name(&state, "Dual Bear");

    // Two separate Cryptolith-Rite grants from two different sources.
    state
        .continuous_effects
        .push_back(cryptolith_grant(1, source1_id, 10));
    state
        .continuous_effects
        .push_back(cryptolith_grant(2, source2_id, 11));

    // Creature should have exactly 2 mana abilities (one per Rite).
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    let any_color_abilities: Vec<_> = chars
        .mana_abilities
        .iter()
        .filter(|a| a.any_color)
        .collect();
    assert_eq!(
        any_color_abilities.len(),
        2,
        "two Cryptolith Rites should produce two mana ability entries (CR 613.5)"
    );

    // First tap succeeds.
    let (state_after_first, _) = process_command(
        state,
        Command::TapForMana {
            player: p1(),
            source: bear_id,
            ability_index: 0,
        },
    )
    .expect("first TapForMana should succeed");

    assert!(
        state_after_first.objects[&bear_id].status.tapped,
        "creature should be tapped after first activation"
    );

    // Second tap fails — creature is already tapped.
    let second_result = process_command(
        state_after_first,
        Command::TapForMana {
            player: p1(),
            source: bear_id,
            ability_index: 1,
        },
    );
    assert!(
        second_result.is_err(),
        "second TapForMana should fail because creature is already tapped (2016-04-08 ruling)"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Chromatic Lantern grants only to lands, not creatures
// CR 613.1f filter scope
// ---------------------------------------------------------------------------

/// CR 613.1f: A land-filtered grant (Chromatic Lantern) does not affect creatures.
/// Only objects matching the filter receive the ability.
#[test]
fn test_chromatic_lantern_grants_only_lands_not_creatures() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::artifact(p1(), "Lantern Source"))
        .object(
            ObjectSpec::land(p1(), "Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .object(ObjectSpec::creature(p1(), "Elvish Warrior", 2, 2))
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Lantern Source");
    let forest_id = find_obj_by_name(&state, "Forest");
    let warrior_id = find_obj_by_name(&state, "Elvish Warrior");

    state
        .continuous_effects
        .push_back(lantern_grant(1, source_id, 10));

    let forest_chars = calculate_characteristics(&state, forest_id).unwrap();
    let warrior_chars = calculate_characteristics(&state, warrior_id).unwrap();

    // Forest should have both its Green ability AND the any-color grant.
    let any_color_count = forest_chars
        .mana_abilities
        .iter()
        .filter(|a| a.any_color)
        .count();
    assert_eq!(
        any_color_count, 1,
        "Chromatic Lantern should grant one any-color ability to Forest (CR 613.1f)"
    );

    // Warrior should NOT have an any-color mana ability.
    assert!(
        !warrior_chars.mana_abilities.iter().any(|a| a.any_color),
        "creature should NOT receive Lantern grant (filter is LandsYouControl, not creatures)"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Chromatic Lantern grant is additive — lands keep existing abilities
// Chromatic Lantern ruling 2018-10-05
// ---------------------------------------------------------------------------

/// Chromatic Lantern ruling 2018-10-05: "Lands you control won't lose any other
/// abilities they had." The Layer 6 grant appends; it does not replace.
/// A Forest under Chromatic Lantern still has {T}: Add {G} PLUS the any-color grant.
#[test]
fn test_chromatic_lantern_lands_keep_existing_abilities() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::artifact(p1(), "Lantern Source"))
        .object(
            ObjectSpec::land(p1(), "Ancient Forest")
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        )
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Lantern Source");
    let forest_id = find_obj_by_name(&state, "Ancient Forest");

    state
        .continuous_effects
        .push_back(lantern_grant(1, source_id, 10));

    let chars = calculate_characteristics(&state, forest_id).unwrap();

    // Should have the Green ability AND the any-color ability (total = 2).
    assert_eq!(
        chars.mana_abilities.len(),
        2,
        "land should have 2 mana abilities: original Green + any-color grant (additive)"
    );
    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|a| { !a.any_color && a.produces.get(&ManaColor::Green) == Some(&1) }),
        "original Green mana ability should be preserved"
    );
    assert!(
        chars.mana_abilities.iter().any(|a| a.any_color),
        "any-color grant should be appended"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Paradise Mantle grants only the equipped creature
// EffectFilter::AttachedCreature
// ---------------------------------------------------------------------------

/// CR 613.1f: AttachedCreature filter means only the object the equipment is
/// attached to receives the ability. An unequipped creature gets nothing.
#[test]
fn test_paradise_mantle_grants_only_equipped_creature() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Equipped Creature", 2, 2))
        .object(ObjectSpec::creature(p1(), "Unequipped Creature", 2, 2))
        .object(ObjectSpec::artifact(p1(), "Mantle"))
        .build()
        .unwrap();

    let equipped_id = find_obj_by_name(&state, "Equipped Creature");
    let unequipped_id = find_obj_by_name(&state, "Unequipped Creature");
    let mantle_id = find_obj_by_name(&state, "Mantle");

    // Attach mantle to the equipped creature.
    state.objects.get_mut(&mantle_id).unwrap().attached_to = Some(equipped_id);
    state.objects.get_mut(&equipped_id).unwrap().attachments = im::vector![mantle_id];

    // Register the grant effect sourced from the mantle.
    let grant = ContinuousEffect {
        id: EffectId(1),
        source: Some(mantle_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AttachedCreature,
        modification: LayerModification::AddManaAbility(ManaAbility {
            produces: OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
        }),
        is_cda: false,
        condition: None,
    };
    state.continuous_effects.push_back(grant);

    let equipped_chars = calculate_characteristics(&state, equipped_id).unwrap();
    let unequipped_chars = calculate_characteristics(&state, unequipped_id).unwrap();

    assert!(
        equipped_chars.mana_abilities.iter().any(|a| a.any_color),
        "equipped creature should have the granted any-color mana ability"
    );
    assert!(
        !unequipped_chars.mana_abilities.iter().any(|a| a.any_color),
        "unequipped creature should NOT have the ability (filter = AttachedCreature)"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Granted mana ability respects summoning sickness
// CR 302.1 / CR 302.6 + CR 613.1f
// ---------------------------------------------------------------------------

/// CR 302.1 / CR 302.6: Even if a creature gains a {T}: Add mana ability from
/// a continuous effect, it still cannot tap for mana if it has summoning sickness
/// and does not have haste.
#[test]
fn test_granted_mana_ability_respects_summoning_sickness() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .at_step(Step::PreCombatMain)
        .active_player(p1())
        .object(ObjectSpec::artifact(p1(), "Rite Source"))
        .object(ObjectSpec::creature(p1(), "New Creature", 2, 2))
        .build()
        .unwrap();
    state.turn.priority_holder = Some(p1());

    let source_id = find_obj_by_name(&state, "Rite Source");
    let creature_id = find_obj_by_name(&state, "New Creature");

    state
        .continuous_effects
        .push_back(cryptolith_grant(1, source_id, 10));

    // Mark creature as having summoning sickness.
    state
        .objects
        .get_mut(&creature_id)
        .unwrap()
        .has_summoning_sickness = true;

    // Verify the grant IS present in layer-resolved characteristics.
    let chars = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars.mana_abilities.iter().any(|a| a.any_color),
        "creature should still see the granted mana ability in calc'd characteristics"
    );

    // But activating it should fail due to summoning sickness.
    let result = process_command(
        state,
        Command::TapForMana {
            player: p1(),
            source: creature_id,
            ability_index: 0,
        },
    );
    assert!(
        result.is_err(),
        "creature with summoning sickness should not tap for mana (CR 302.6)"
    );
}

// ---------------------------------------------------------------------------
// Test 9: Humility removes the granted mana ability (later timestamp wins)
// CR 613.1f — Layer 6 timestamp ordering
// ---------------------------------------------------------------------------

/// CR 613.1f + CR 613.7: If Humility (RemoveAllAbilities, later timestamp) is
/// applied after Cryptolith Rite's grant (earlier timestamp), the grant is
/// stripped. Layer 6 effects apply in timestamp order: Rite adds, Humility removes.
#[test]
fn test_humility_removes_granted_mana_ability() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::artifact(p1(), "Rite Source"))
        .object(ObjectSpec::creature(p1(), "Bear Under Humility", 2, 2))
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Rite Source");
    let bear_id = find_obj_by_name(&state, "Bear Under Humility");

    // Rite grant at timestamp 10 (earlier).
    state
        .continuous_effects
        .push_back(cryptolith_grant(1, source_id, 10));
    // Humility at timestamp 20 (later) — wipes all abilities including grants.
    state.continuous_effects.push_back(humility_effect(2, 20));

    let chars = calculate_characteristics(&state, bear_id).unwrap();

    // Humility (ts=20) runs after Rite grant (ts=10) in Layer 6.
    // RemoveAllAbilities clears mana_abilities → creature ends up with no mana abilities.
    assert!(
        chars.mana_abilities.is_empty(),
        "Humility (later timestamp) should remove the Cryptolith Rite grant (CR 613.7)"
    );
    assert!(
        chars.keywords.is_empty(),
        "Humility should also remove all keywords"
    );
}

// ---------------------------------------------------------------------------
// Test 10: Face-down creature inherits granted mana ability (CR 708.2)
// ---------------------------------------------------------------------------

/// CR 708.2: Face-down creatures lose their printed characteristics but external
/// continuous effects (Layer 6 grants) still apply. A face-down creature under
/// Cryptolith Rite gains the mana ability from the grant, but does NOT gain
/// front-face characteristics. After turning face-up, it has BOTH its front-face
/// abilities AND the grant.
#[test]
fn test_face_down_creature_inherits_granted_mana_ability() {
    use mtg_engine::state::FaceDownKind;

    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::artifact(p1(), "Rite Source"))
        // A creature with a keyword (Flying) to verify face-down strips printed abilities.
        .object(
            ObjectSpec::creature(p1(), "Exalted Morph", 2, 2).with_keyword(KeywordAbility::Flying),
        )
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Rite Source");
    let creature_id = find_obj_by_name(&state, "Exalted Morph");

    state
        .continuous_effects
        .push_back(cryptolith_grant(1, source_id, 10));

    // (a) Verify grant is present on the face-up creature.
    let chars_face_up = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars_face_up.mana_abilities.iter().any(|a| a.any_color),
        "(a) face-up creature should have the grant from Cryptolith Rite"
    );
    assert!(
        chars_face_up.keywords.contains(&KeywordAbility::Flying),
        "(a) face-up creature should have Flying from its own characteristics"
    );

    // Turn the creature face-down.
    // layers.rs face-down override requires BOTH status.face_down AND face_down_as.
    state
        .objects
        .get_mut(&creature_id)
        .unwrap()
        .status
        .face_down = true;
    state.objects.get_mut(&creature_id).unwrap().face_down_as = Some(FaceDownKind::Morph);

    // (b) Face-down creature: grant should still be present (Layer 6 re-adds it after
    // the face-down override clears printed abilities). Front-face Flying should be gone.
    let chars_face_down = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars_face_down.mana_abilities.iter().any(|a| a.any_color),
        "(b) face-down creature should still inherit the Layer 6 grant (CR 708.2)"
    );
    assert!(
        !chars_face_down.keywords.contains(&KeywordAbility::Flying),
        "(b) face-down creature should NOT have Flying (printed ability stripped by face-down)"
    );

    // (c) Turn face-up: front-face abilities return AND grant is still present.
    state
        .objects
        .get_mut(&creature_id)
        .unwrap()
        .status
        .face_down = false;
    state.objects.get_mut(&creature_id).unwrap().face_down_as = None;
    let chars_face_up_again = calculate_characteristics(&state, creature_id).unwrap();
    assert!(
        chars_face_up_again
            .mana_abilities
            .iter()
            .any(|a| a.any_color),
        "(c) face-up again: Layer 6 grant should still be present"
    );
    assert!(
        chars_face_up_again
            .keywords
            .contains(&KeywordAbility::Flying),
        "(c) face-up again: printed Flying should be restored"
    );
}

// ---------------------------------------------------------------------------
// Test 11: LayerModification::AddActivatedAbility — variant 23 positive test
// CR 602.5b — granted once_per_turn restriction must be preserved + enforced
// ---------------------------------------------------------------------------

/// CR 613.1f + CR 602.5b: A Layer 6 grant of an ActivatedAbility must preserve
/// the `once_per_turn` flag on the granted ability and enforce the restriction.
///
/// This test exercises `LayerModification::AddActivatedAbility` (hash discriminant 23)
/// which was untested by the other 10 PB-S tests (all of which use AddManaAbility).
/// It also exercises the H1 hash fix path: `once_per_turn` is now hashed by
/// `HashInto for ActivatedAbility`, so two grants differing only in that flag
/// produce distinct hashes. We can't observe the hash directly from a unit test,
/// but we can prove the flag survives the grant round-trip and drives engine
/// behavior — which is what the hash is meant to track.
#[test]
fn test_granted_once_per_turn_activated_ability_is_preserved_and_enforced() {
    let source_spec = ObjectSpec::artifact(p1(), "Grant Source").in_zone(ZoneId::Battlefield);
    // Creature with vigilance so we don't need to worry about summoning sickness
    // semantics for cost-free activated abilities (activation does not need a tap cost).
    let creature_spec = ObjectSpec::creature(p1(), "Granted Creature", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_keyword(KeywordAbility::Haste);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1())
        .at_step(Step::PreCombatMain)
        .object(source_spec)
        .object(creature_spec)
        .build()
        .unwrap();

    let source_id = find_obj_by_name(&state, "Grant Source");
    let creature_id = find_obj_by_name(&state, "Granted Creature");

    // The ability being granted: "0: you gain 1 life. Activate only once each turn."
    // Cost-free so we can isolate the once_per_turn failure mode from tap/mana/etc.
    let granted = ActivatedAbility {
        cost: ActivationCost::default(),
        description: "0: Gain 1 life. Activate only once each turn.".to_string(),
        effect: Some(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: true,
    };

    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddActivatedAbility(Box::new(granted.clone())),
        is_cda: false,
        condition: None,
    });

    // Assertion (a): the grant appears in calculated characteristics.
    let chars = calculate_characteristics(&state, creature_id)
        .expect("calculate_characteristics should succeed");
    assert_eq!(
        chars.activated_abilities.len(),
        1,
        "granted creature should have exactly 1 activated ability (the grant)"
    );

    // Assertion (b): the once_per_turn flag is preserved on the granted ability.
    // If the Layer 6 apply path dropped or defaulted the flag, this would fail.
    // Also proves the runner's claim that specialized vecs carry full ability state.
    let granted_copy = &chars.activated_abilities[0];
    assert!(
        granted_copy.once_per_turn,
        "granted ability must preserve once_per_turn=true through Layer 6 (CR 602.5b)"
    );
    assert_eq!(
        granted_copy.description, granted.description,
        "granted ability description must match source"
    );

    // Assertion (c): first activation succeeds.
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1(),
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("first activation of granted once-per-turn ability should succeed");

    // After first activation, the counter should be > 0, preventing a second activation.
    let counter = state
        .objects
        .get(&creature_id)
        .map(|o| o.abilities_activated_this_turn)
        .unwrap_or(0);
    assert!(
        counter > 0,
        "abilities_activated_this_turn should increment after once_per_turn activation (CR 602.5b)"
    );

    // Assertion (d): second activation in the same turn fails with the once-per-turn error.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1(),
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "second activation of granted once-per-turn ability should fail (CR 602.5b)"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("once per turn"),
        "failure should cite once-per-turn restriction, got: {}",
        err_msg
    );
}

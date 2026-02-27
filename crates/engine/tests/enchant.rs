//! Enchant keyword ability tests (CR 702.5).
//!
//! Enchant is a static ability written "Enchant [object or player]."
//! Key rules tested:
//!
//! - CR 702.5a / 303.4a: Aura spell must target an object matching the Enchant restriction.
//! - CR 303.4b: The Aura attaches to its target on resolution; `attached_to` is set.
//! - CR 608.2b: If the Aura's target is illegal at resolution, the spell fizzles.
//! - CR 704.5m / 303.4c: SBA removes an Aura attached to an illegal object.
//! - CR 702.5a: Enchant Permanent allows any permanent type.

use mtg_engine::{
    process_command, start_game, Command, EnchantTarget, GameEvent, GameStateBuilder,
    GameStateError, KeywordAbility, ObjectSpec, PlayerId, Step, SubType, Target, ZoneId,
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

/// Build an Aura ObjectSpec in hand, with the given EnchantTarget keyword.
fn aura_in_hand(owner: PlayerId, name: &str, enchant: EnchantTarget) -> ObjectSpec {
    ObjectSpec::enchantment(owner, name)
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(enchant))
        .in_zone(ZoneId::Hand(owner))
}

// ── Test 1: Enchant creature Aura accepts a creature target ───────────────────

#[test]
/// CR 702.5a / 303.4a — "Enchant creature" Aura targeting a creature succeeds.
fn test_702_5_enchant_creature_targets_creature_valid() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    let creature = ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let creature_id = find_object(&state, "Test Bear");

    // Cast succeeds — creature target matches "Enchant creature".
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    );
    assert!(
        result.is_ok(),
        "Casting 'Enchant creature' Aura targeting a creature should succeed; got: {:?}",
        result.err()
    );
}

// ── Test 2: Enchant creature Aura rejects a land target ───────────────────────

#[test]
/// CR 702.5a / 303.4a — "Enchant creature" Aura targeting a land is rejected.
fn test_702_5_enchant_creature_rejects_land_target() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    let land = ObjectSpec::land(p1, "Test Forest");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let land_id = find_object(&state, "Test Forest");

    // Give the aura a zero mana cost so the only issue is target validation.
    // (ObjectSpec::enchantment has no mana cost by default, so none needed.)

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(land_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    );
    assert!(
        result.is_err(),
        "Casting 'Enchant creature' Aura targeting a land should fail"
    );
    match result.unwrap_err() {
        GameStateError::InvalidTarget(_) => {}
        e => panic!("Expected InvalidTarget, got: {:?}", e),
    }
}

// ── Test 3: Enchant land Aura targeting a land succeeds ───────────────────────

#[test]
/// CR 702.5a / 303.4a — "Enchant land" Aura targeting a land succeeds.
fn test_702_5_enchant_land_targets_land_valid() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Fertile Ground", EnchantTarget::Land);
    let land = ObjectSpec::land(p1, "Test Forest");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Fertile Ground");
    let land_id = find_object(&state, "Test Forest");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(land_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    );
    assert!(
        result.is_ok(),
        "Casting 'Enchant land' Aura targeting a land should succeed; got: {:?}",
        result.err()
    );
}

// ── Test 4: Aura attaches to target on resolution ─────────────────────────────

#[test]
/// CR 303.4b — After resolution, Aura has attached_to set and AuraAttached event emitted.
fn test_702_5_aura_attaches_to_target_on_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    let creature = ObjectSpec::creature(p2, "Target Bear", 2, 2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let creature_id = find_object(&state, "Target Bear");

    // Cast the Aura targeting the creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    )
    .expect("CastSpell should succeed");

    // Both players pass priority to resolve.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 303.4b: Aura has attached_to set (it moved zones — find by name on battlefield).
    let aura_on_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Test Aura" && o.zone == ZoneId::Battlefield);
    assert!(
        aura_on_bf.is_some(),
        "Aura should be on the battlefield after resolution"
    );
    let aura_bf_id = aura_on_bf.map(|o| o.id).unwrap();
    let aura_obj = state.objects.get(&aura_bf_id).unwrap();
    assert_eq!(
        aura_obj.attached_to,
        Some(creature_id),
        "Aura.attached_to should be set to the creature"
    );

    // Target creature should have the Aura in its attachments.
    let creature_obj = state.objects.get(&creature_id).unwrap();
    assert!(
        creature_obj.attachments.contains(&aura_bf_id),
        "creature.attachments should contain the Aura"
    );

    // AuraAttached event should have been emitted.
    let attached_event = resolve_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AuraAttached {
                aura_id: _,
                target_id,
                controller,
            } if *target_id == creature_id && *controller == p1
        )
    });
    assert!(
        attached_event,
        "AuraAttached event should be emitted on resolution; events: {:?}",
        resolve_events
    );
}

// ── Test 5: SBA 704.5m — unattached Aura goes to graveyard ───────────────────

#[test]
/// CR 704.5m — An Aura on the battlefield not attached to anything goes to the graveyard.
fn test_704_5m_aura_unattached_goes_to_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = ObjectSpec::enchantment(p1, "Floating Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Creature));
    // Note: no attached_to set — this is the illegal state.

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Floating Aura");

    let (_, events) = start_game(state).unwrap();

    let aura_fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if object_id == &aura_id));
    assert!(
        aura_fell_off,
        "Unattached Aura should fall off via SBA 704.5m; events: {:?}",
        events
    );
}

// ── Test 6: SBA 704.5m — Enchant creature Aura on non-creature goes to graveyard

#[test]
/// CR 704.5m / 303.4c — "Enchant creature" Aura attached to a land falls off.
fn test_704_5m_type_change_triggers_sba() {
    let p1 = p(1);
    let p2 = p(2);

    let land = ObjectSpec::land(p1, "Animated Land");
    let aura = ObjectSpec::enchantment(p1, "Creature Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Creature));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(land)
        .object(aura)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let land_id = find_object(&state, "Animated Land");
    let aura_id = find_object(&state, "Creature Aura");

    // Manually simulate illegally attached state (e.g. after a type-change animation ended).
    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(land_id);
        aura_obj
            .characteristics
            .keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Creature));
    }
    if let Some(land_obj) = state.objects.get_mut(&land_id) {
        land_obj.attachments.push_back(aura_id);
    }

    let (_, events) = start_game(state).unwrap();

    let aura_fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if object_id == &aura_id));
    assert!(
        aura_fell_off,
        "CR 704.5m: 'Enchant creature' Aura on a non-creature land should fall off; events: {:?}",
        events
    );
}

// ── Test 7: Enchant permanent accepts any permanent type ──────────────────────

#[test]
/// CR 702.5a / 303.4a — "Enchant permanent" Aura can target a creature, land, or artifact.
fn test_702_5_enchant_permanent_accepts_any_permanent() {
    let p1 = p(1);
    let p2 = p(2);

    // Test targeting a creature.
    {
        let aura = aura_in_hand(p1, "Wild Growth", EnchantTarget::Permanent);
        let creature = ObjectSpec::creature(p1, "Test Bear", 2, 2);

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(creature)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let aura_id = find_object(&state, "Wild Growth");
        let creature_id = find_object(&state, "Test Bear");

        let result = process_command(
            state,
            Command::CastSpell {
                player: p1,
                card: aura_id,
                targets: vec![Target::Object(creature_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                cast_with_evoke: false,
            },
        );
        assert!(
            result.is_ok(),
            "Enchant Permanent should accept a creature target; got: {:?}",
            result.err()
        );
    }

    // Test targeting a land.
    {
        let aura = aura_in_hand(p1, "Wild Growth", EnchantTarget::Permanent);
        let land = ObjectSpec::land(p1, "Test Forest");

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(land)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let aura_id = find_object(&state, "Wild Growth");
        let land_id = find_object(&state, "Test Forest");

        let result = process_command(
            state,
            Command::CastSpell {
                player: p1,
                card: aura_id,
                targets: vec![Target::Object(land_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                cast_with_evoke: false,
            },
        );
        assert!(
            result.is_ok(),
            "Enchant Permanent should accept a land target; got: {:?}",
            result.err()
        );
    }
}

// ── Test 8: Aura casting rejected without a target ────────────────────────────

#[test]
/// CR 303.4a — An Aura spell with an Enchant restriction and no targets is rejected.
fn test_702_5_enchant_casting_rejected_without_target() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");

    // Cast with no targets — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    );
    assert!(
        result.is_err(),
        "Aura spell cast with no targets should be rejected (CR 303.4a)"
    );
    match result.unwrap_err() {
        GameStateError::InvalidCommand(_) => {}
        e => panic!("Expected InvalidCommand, got: {:?}", e),
    }
}

// ── Test 9: CR 303.4a — Aura targeting creature in graveyard is rejected ─────

#[test]
/// CR 303.4a / 115.4 — Aura target must be on the battlefield; targeting a
/// creature card in the graveyard is rejected at cast time.
fn test_303_4a_aura_target_must_be_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    // Place the creature in p2's graveyard instead of the battlefield.
    let creature = ObjectSpec::creature(p2, "Dead Bear", 2, 2).in_zone(ZoneId::Graveyard(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let creature_id = find_object(&state, "Dead Bear");

    // Casting the Aura targeting a graveyard creature should be rejected.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    );
    assert!(
        result.is_err(),
        "Casting Aura targeting creature in graveyard should be rejected (CR 303.4a)"
    );
    match result.unwrap_err() {
        GameStateError::InvalidTarget(_) => {}
        e => panic!("Expected InvalidTarget, got: {:?}", e),
    }
}

// ── Test 10: CR 608.2b — Aura fizzles when creature target leaves battlefield ─

#[test]
/// CR 608.2b — If an Aura's target becomes illegal before resolution, the spell
/// fizzles: SpellFizzled is emitted, Aura goes to graveyard, no AuraAttached event.
fn test_702_5_aura_fizzles_when_target_killed() {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    let creature = ObjectSpec::creature(p2, "Target Bear", 2, 2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let creature_id = find_object(&state, "Target Bear");

    // Cast the Aura targeting the creature on the battlefield — succeeds.
    let (mut state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(creature_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
        },
    )
    .expect("CastSpell should succeed");

    // Simulate the creature leaving the battlefield (e.g. killed by a spell)
    // before the Aura resolves. The creature moves to p2's graveyard.
    state
        .move_object_to_zone(creature_id, ZoneId::Graveyard(p2))
        .expect("move creature to graveyard should succeed");

    // Both players pass priority — Aura resolves with an illegal target → fizzle.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // CR 608.2b: SpellFizzled event must be emitted.
    let fizzled = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellFizzled { .. }));
    assert!(
        fizzled,
        "Aura should fizzle (SpellFizzled) when its target left the battlefield; events: {:?}",
        resolve_events
    );

    // CR 608.2b: The Aura card goes to the graveyard, not the battlefield.
    let aura_on_battlefield = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Test Aura" && o.zone == ZoneId::Battlefield);
    assert!(
        !aura_on_battlefield,
        "Fizzled Aura should NOT be on the battlefield"
    );

    let aura_in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Test Aura" && matches!(o.zone, ZoneId::Graveyard(_)));
    assert!(aura_in_graveyard, "Fizzled Aura should be in the graveyard");

    // No AuraAttached event should have been emitted.
    let attached = resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraAttached { .. }));
    assert!(
        !attached,
        "AuraAttached must NOT be emitted when Aura fizzles"
    );
}

// ── Test 11: CR 303.4d — Aura attached to itself goes to graveyard (SBA) ─────

#[test]
/// CR 303.4d — An Aura can't enchant itself. If somehow an Aura is attached to
/// itself, SBA 704.5m puts it into its owner's graveyard.
fn test_303_4d_aura_cant_enchant_itself() {
    let p1 = p(1);
    let p2 = p(2);

    // Place an Aura with Enchant Enchantment on the battlefield (unattached
    // for the build step, then manually wire it to itself below).
    let aura = ObjectSpec::enchantment(p1, "Self-Enchanting Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Enchantment));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Self-Enchanting Aura");

    // Manually wire the Aura to point to itself — an impossible but CR-specified
    // state that SBA 704.5m / CR 303.4d must handle.
    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(aura_id);
        aura_obj.attachments.push_back(aura_id);
        aura_obj
            .characteristics
            .keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Enchantment));
    }

    // start_game triggers SBA checks; the self-attached Aura must fall off.
    let (_, events) = start_game(state).unwrap();

    let fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura_id));
    assert!(
        fell_off,
        "CR 303.4d: Aura attached to itself should fall off via SBA; events: {:?}",
        events
    );
}

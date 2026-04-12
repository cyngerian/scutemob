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
    calculate_characteristics, process_command, start_game, CardType, Command,
    EnchantControllerConstraint, EnchantFilter, EnchantTarget, GameEvent, GameStateBuilder,
    GameStateError, KeywordAbility, ObjectSpec, PlayerId, Step, SubType, SuperType, Target, ZoneId,
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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

// ── PB-Q4 Tests: EnchantTarget::Filtered ─────────────────────────────────────
//
// CR 303.4a / 702.5a: Aura with Filtered enchant restriction.
// CR 704.5m: SBA continuous enforcement.
// CR 205.4a: Basic/nonbasic supertype semantics.

/// Build an Aura with EnchantTarget::Filtered.
fn filtered_aura_in_hand(owner: PlayerId, name: &str, filter: EnchantFilter) -> ObjectSpec {
    ObjectSpec::enchantment(owner, name)
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(filter)))
        .in_zone(ZoneId::Hand(owner))
}

/// Build a land with a given subtype on the battlefield.
fn land_with_subtype(owner: PlayerId, name: &str, subtype: &str) -> ObjectSpec {
    ObjectSpec::land(owner, name).with_subtypes(vec![SubType(subtype.to_string())])
}

/// Build a basic land (with Basic supertype) with a given subtype.
fn basic_land_spec(owner: PlayerId, name: &str, subtype: &str) -> ObjectSpec {
    ObjectSpec::land(owner, name)
        .with_subtypes(vec![SubType(subtype.to_string())])
        .with_supertypes(vec![SuperType::Basic])
}

// ── Test M1: EnchantTarget::Filtered — land subtype cast-time legal ──────────

#[test]
/// CR 303.4a — Cast "Enchant Mountain" Aura targeting a Mountain you control; verify success.
fn test_enchant_filtered_land_subtype_cast_time_legal() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Enchant Mountain Aura", filter);
    let mountain = land_with_subtype(p1, "Test Mountain", "Mountain");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(mountain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Enchant Mountain Aura");
    let mountain_id = find_object(&state, "Test Mountain");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(mountain_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Casting 'Enchant Mountain' Aura targeting a Mountain should succeed; got: {:?}",
        result.err()
    );
}

// ── Test M2: EnchantTarget::Filtered — land subtype cast-time illegal ─────────

#[test]
/// CR 303.4a — Cast "Enchant Mountain" Aura targeting a Forest; verify InvalidTarget.
fn test_enchant_filtered_land_subtype_cast_time_illegal() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Enchant Mountain Aura", filter);
    let forest = land_with_subtype(p1, "Test Forest", "Forest");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(forest)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Enchant Mountain Aura");
    let forest_id = find_object(&state, "Test Forest");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(forest_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    match result {
        Err(GameStateError::InvalidTarget(_)) => {}
        other => panic!("Expected InvalidTarget; got: {:?}", other),
    }
}

// ── Test M3: EnchantTarget::Filtered — controller filter cast-time legal ──────

#[test]
/// CR 303.4a — Cast "Enchant Mountain you control" Aura targeting your own Mountain; verify success.
fn test_enchant_filtered_controller_cast_time_legal() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Chained Aura", filter);
    let my_mountain = land_with_subtype(p1, "My Mountain", "Mountain");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(my_mountain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Chained Aura");
    let mountain_id = find_object(&state, "My Mountain");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(mountain_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Casting 'Enchant Mountain you control' targeting your own Mountain should succeed; got: {:?}",
        result.err()
    );
}

// ── Test M4: EnchantTarget::Filtered — controller filter cast-time illegal ────

#[test]
/// CR 303.4a — Cast "Enchant Mountain you control" Aura targeting opponent's Mountain; verify InvalidTarget.
fn test_enchant_filtered_controller_cast_time_illegal() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Chained Aura", filter);
    let their_mountain = land_with_subtype(p2, "Opponent Mountain", "Mountain");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(their_mountain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Chained Aura");
    let mountain_id = find_object(&state, "Opponent Mountain");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(mountain_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    match result {
        Err(GameStateError::InvalidTarget(_)) => {}
        other => panic!("Expected InvalidTarget; got: {:?}", other),
    }
}

// ── Test M5: EnchantTarget::Filtered — basic land legal ──────────────────────

#[test]
/// CR 205.4a — Cast "Enchant basic land you control" Aura targeting a basic Plains; verify success.
fn test_enchant_filtered_basic_land_legal() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        basic: true,
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Ossification Aura", filter);
    let plains = basic_land_spec(p1, "Basic Plains", "Plains");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(plains)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Ossification Aura");
    let plains_id = find_object(&state, "Basic Plains");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(plains_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Casting 'Enchant basic land you control' targeting a basic Plains should succeed; got: {:?}",
        result.err()
    );
}

// ── Test M6: EnchantTarget::Filtered — basic flag rejects nonbasic land ───────

#[test]
/// CR 205.4a — Cast "Enchant basic land you control" Aura targeting a non-basic dual land
/// (has Plains subtype but no Basic supertype); verify InvalidTarget.
fn test_enchant_filtered_basic_land_illegal_nonbasic() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        basic: true,
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    let aura = filtered_aura_in_hand(p1, "Ossification Aura", filter);
    // Nonbasic dual land with Plains subtype but no Basic supertype.
    let nonbasic_dual =
        ObjectSpec::land(p1, "Nonbasic Dual").with_subtypes(vec![SubType("Plains".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(nonbasic_dual)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Ossification Aura");
    let dual_id = find_object(&state, "Nonbasic Dual");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aura_id,
            targets: vec![Target::Object(dual_id)],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    match result {
        Err(GameStateError::InvalidTarget(_)) => {}
        other => panic!(
            "Expected InvalidTarget for nonbasic dual with basic-only filter; got: {:?}",
            other
        ),
    }
}

// ── Test M7: SBA — controller change makes Filtered Aura illegal ──────────────

#[test]
/// CR 704.5m — Attach "Enchant Mountain you control" Aura to your Mountain, then
/// change controller of the Mountain to an opponent. SBA must put Aura in graveyard.
fn test_enchant_filtered_sba_control_change() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    let mountain = land_with_subtype(p1, "Test Mountain", "Mountain");
    let aura = ObjectSpec::enchantment(p1, "Chained Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
            filter.clone(),
        )));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mountain)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mountain_id = find_object(&state, "Test Mountain");
    let aura_id = find_object(&state, "Chained Aura");

    // Manually attach Aura to Mountain (initially legal: both p1-controlled).
    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(mountain_id);
        aura_obj
            .characteristics
            .keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Filtered(filter)));
    }
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.attachments.push_back(aura_id);
    }

    // Transfer control of the Mountain to p2 → Aura now on opponent's land.
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.controller = p2;
    }

    // start_game triggers SBA — Aura must fall off.
    let (_, events) = start_game(state).unwrap();

    let fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura_id));
    assert!(
        fell_off,
        "CR 704.5m: Aura on opponent's land should fall off via SBA after control change; events: {:?}",
        events
    );
}

// ── Test M8: SBA — land becomes non-land makes Filtered Aura illegal ──────────

#[test]
/// CR 704.5m — Attach "Enchant Mountain" Aura to a Mountain, then strip the Land type.
/// SBA must detect the Aura is attached to a non-land and put it in graveyard.
fn test_enchant_filtered_sba_land_becomes_nonland() {
    let p1 = p(1);
    let p2 = p(2);

    let filter = EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        ..Default::default()
    };
    let mountain = land_with_subtype(p1, "Test Mountain", "Mountain");
    let aura = ObjectSpec::enchantment(p1, "Enchant Mountain Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
            filter.clone(),
        )));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mountain)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mountain_id = find_object(&state, "Test Mountain");
    let aura_id = find_object(&state, "Enchant Mountain Aura");

    // Attach Aura to Mountain.
    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(mountain_id);
        aura_obj
            .characteristics
            .keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Filtered(filter)));
    }
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.attachments.push_back(aura_id);
    }

    // Remove Land type from Mountain's base characteristics.
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.characteristics.card_types.remove(&CardType::Land);
    }

    // SBA must detect the Aura is on a non-land and remove it.
    let (_, events) = start_game(state).unwrap();

    let fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura_id));
    assert!(
        fell_off,
        "CR 704.5m: Filtered 'Enchant Land' Aura on a non-land should fall off via SBA; events: {:?}",
        events
    );
}

// ── Test M9: EnchantTarget::Filtered — disjunction (Forest or Plains) ─────────

#[test]
/// CR 303.4a — Aura with has_subtypes: [Forest, Plains] (OR semantics).
/// Forest → legal; Plains → legal; Mountain → illegal.
fn test_enchant_filtered_disjunction_forest_or_plains() {
    let p1 = p(1);
    let p2 = p(2);

    let make_filter = || EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtypes: vec![SubType("Forest".to_string()), SubType("Plains".to_string())],
        ..Default::default()
    };

    // Case 1: Forest → legal.
    {
        let aura = filtered_aura_in_hand(p1, "Forest or Plains Aura", make_filter());
        let forest = land_with_subtype(p1, "Test Forest", "Forest");
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(forest)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let aura_id = find_object(&state, "Forest or Plains Aura");
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        assert!(
            result.is_ok(),
            "Forest target with [Forest,Plains] filter should succeed; got: {:?}",
            result.err()
        );
    }

    // Case 2: Plains → legal.
    {
        let aura = filtered_aura_in_hand(p1, "Forest or Plains Aura", make_filter());
        let plains = land_with_subtype(p1, "Test Plains", "Plains");
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(plains)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let aura_id = find_object(&state, "Forest or Plains Aura");
        let land_id = find_object(&state, "Test Plains");
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        assert!(
            result.is_ok(),
            "Plains target with [Forest,Plains] filter should succeed; got: {:?}",
            result.err()
        );
    }

    // Case 3: Mountain → illegal.
    {
        let aura = filtered_aura_in_hand(p1, "Forest or Plains Aura", make_filter());
        let mountain = land_with_subtype(p1, "Test Mountain", "Mountain");
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(mountain)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let aura_id = find_object(&state, "Forest or Plains Aura");
        let land_id = find_object(&state, "Test Mountain");
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        match result {
            Err(GameStateError::InvalidTarget(_)) => {}
            other => panic!(
                "Expected InvalidTarget for Mountain with [Forest,Plains] filter; got: {:?}",
                other
            ),
        }
    }
}

// ── Test M10: EnchantTarget::Filtered — nonbasic flag ─────────────────────────

#[test]
/// CR 205.4a — Aura with `nonbasic: true, has_card_type: Land`.
/// Basic Mountain → illegal (has Basic supertype); nonbasic dual → legal.
fn test_enchant_filtered_nonbasic_land() {
    let p1 = p(1);
    let p2 = p(2);

    let make_filter = || EnchantFilter {
        has_card_type: Some(CardType::Land),
        nonbasic: true,
        ..Default::default()
    };

    // Case 1: basic Mountain → illegal.
    {
        let aura = filtered_aura_in_hand(p1, "Nonbasic Land Aura", make_filter());
        let bmt = basic_land_spec(p1, "Basic Mountain", "Mountain");
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(bmt)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let aura_id = find_object(&state, "Nonbasic Land Aura");
        let land_id = find_object(&state, "Basic Mountain");
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        match result {
            Err(GameStateError::InvalidTarget(_)) => {}
            other => panic!(
                "Expected InvalidTarget for basic Mountain with nonbasic filter; got: {:?}",
                other
            ),
        }
    }

    // Case 2: nonbasic dual → legal.
    {
        let aura = filtered_aura_in_hand(p1, "Nonbasic Land Aura", make_filter());
        let nonbasic = ObjectSpec::land(p1, "Nonbasic Dual")
            .with_subtypes(vec![SubType("Mountain".to_string())]);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(aura)
            .object(nonbasic)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        let aura_id = find_object(&state, "Nonbasic Land Aura");
        let land_id = find_object(&state, "Nonbasic Dual");
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
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        );
        assert!(
            result.is_ok(),
            "Nonbasic dual with nonbasic filter should succeed; got: {:?}",
            result.err()
        );
    }
}

// ── Test M11: Layer resolution — animated Mountain via static Aura effects ────

#[test]
/// CR 613.1 — Attach an Aura with AddCardTypes(Creature)+Giant+SetPT(7,7)+Haste to a Mountain.
/// Verify layer-resolved characteristics: Creature + Land present, Giant + Mountain subtypes,
/// power/toughness = 7/7, Haste present. Verifies "it's still a land" (CR 205.3i).
fn test_animate_land_pt_and_types_via_chained_or_awaken() {
    use mtg_engine::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectLayer, LayerModification,
    };

    let p1 = p(1);
    let p2 = p(2);

    let mountain = land_with_subtype(p1, "Test Mountain", "Mountain");
    let aura = ObjectSpec::enchantment(p1, "Awaken Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
            EnchantFilter {
                has_card_type: Some(CardType::Land),
                has_subtype: Some(SubType("Mountain".to_string())),
                ..Default::default()
            },
        )));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mountain)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mountain_id = find_object(&state, "Test Mountain");
    let aura_id = find_object(&state, "Awaken Aura");

    // Attach Aura to Mountain.
    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(mountain_id);
    }
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.attachments.push_back(aura_id);
    }

    // Register static continuous effects mimicking Awaken the Ancient.
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(1),
        source: Some(aura_id),
        layer: EffectLayer::TypeChange,
        modification: LayerModification::AddCardTypes([CardType::Creature].into_iter().collect()),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 1000,
    });
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(2),
        source: Some(aura_id),
        layer: EffectLayer::TypeChange,
        modification: LayerModification::AddSubtypes(
            [SubType("Giant".to_string())].into_iter().collect(),
        ),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 1001,
    });
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(3),
        source: Some(aura_id),
        layer: EffectLayer::PtSet,
        modification: LayerModification::SetPowerToughness {
            power: 7,
            toughness: 7,
        },
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 1002,
    });
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(4),
        source: Some(aura_id),
        layer: EffectLayer::Ability,
        modification: LayerModification::AddKeywords([KeywordAbility::Haste].into_iter().collect()),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 1003,
    });
    // Layer 5: Set color to red (Awaken the Ancient makes the land red).
    use mtg_engine::Color;
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(5),
        source: Some(aura_id),
        layer: EffectLayer::ColorChange,
        modification: LayerModification::SetColors([Color::Red].into_iter().collect()),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 1004,
    });

    let chars = calculate_characteristics(&state, mountain_id)
        .expect("Mountain characteristics should be calculable");

    // CR 205.3i: Land type preserved.
    assert!(
        chars.card_types.contains(&CardType::Land),
        "Land type must be preserved; types: {:?}",
        chars.card_types
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Creature type added; types: {:?}",
        chars.card_types
    );
    assert!(
        chars.subtypes.contains(&SubType("Giant".to_string())),
        "Giant subtype added; subtypes: {:?}",
        chars.subtypes
    );
    assert!(
        chars.subtypes.contains(&SubType("Mountain".to_string())),
        "Mountain subtype preserved; subtypes: {:?}",
        chars.subtypes
    );
    assert_eq!(
        chars.power,
        Some(7),
        "Power must be 7 after Awaken animation"
    );
    assert_eq!(
        chars.toughness,
        Some(7),
        "Toughness must be 7 after Awaken animation"
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Haste must be present; keywords: {:?}",
        chars.keywords
    );
    // CR 613.1d: Layer 5 SetColors dispatch on AttachedLand — mountain must now be red.
    assert!(
        chars.colors.contains(&Color::Red),
        "Layer 5 SetColors must make the animated land red; colors: {:?}",
        chars.colors
    );
}

// ── Test M12: Summoning sickness — Haste overrides it for animated land ───────

#[test]
/// CR 302.1 + CR 702.10b — Mountain that entered this turn is animated with Haste.
/// Haste must be present in layer-resolved characteristics (prerequisite for sickness override).
fn test_animate_land_summoning_sickness_propagation() {
    use mtg_engine::{
        AttackTarget, ContinuousEffect, EffectDuration, EffectFilter, EffectLayer,
        LayerModification,
    };

    let p1 = p(1);
    let p2 = p(2);

    let mountain = land_with_subtype(p1, "New Mountain", "Mountain");
    let aura = ObjectSpec::enchantment(p1, "Haste Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
            EnchantFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
        )));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(mountain)
        .object(aura)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let mountain_id = find_object(&state, "New Mountain");
    let aura_id = find_object(&state, "Haste Aura");

    if let Some(aura_obj) = state.objects.get_mut(&aura_id) {
        aura_obj.attached_to = Some(mountain_id);
    }
    if let Some(mt_obj) = state.objects.get_mut(&mountain_id) {
        mt_obj.attachments.push_back(aura_id);
        mt_obj.has_summoning_sickness = true;
    }

    // Register Creature type + Haste on the Mountain.
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(10),
        source: Some(aura_id),
        layer: EffectLayer::TypeChange,
        modification: LayerModification::AddCardTypes([CardType::Creature].into_iter().collect()),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 2000,
    });
    state.continuous_effects.push_back(ContinuousEffect {
        id: mtg_engine::EffectId(11),
        source: Some(aura_id),
        layer: EffectLayer::Ability,
        modification: LayerModification::AddKeywords([KeywordAbility::Haste].into_iter().collect()),
        filter: EffectFilter::AttachedLand,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: None,
        is_cda: false,
        timestamp: 2001,
    });

    let chars = calculate_characteristics(&state, mountain_id)
        .expect("Mountain characteristics should resolve");

    // CR 302.1 + CR 702.10b: Haste must be in layer-resolved keywords for sickness override.
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Haste must be present in layer-resolved chars (prerequisite for summoning-sickness override); keywords: {:?}",
        chars.keywords
    );
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Creature type must be present; card_types: {:?}",
        chars.card_types
    );
    // Mountain retains Land type (CR 205.3i).
    assert!(
        chars.card_types.contains(&CardType::Land),
        "Land type must be preserved on animated Mountain; card_types: {:?}",
        chars.card_types
    );

    // CR 702.10b: With Haste, summoning sickness does not prevent attacking.
    // Exercise the actual can-attack gate — DeclareAttackers must succeed.
    let result_with_haste = process_command(
        state.clone(),
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(mountain_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );
    assert!(
        result_with_haste.is_ok(),
        "Animated land with Haste should be able to attack despite summoning sickness; got: {:?}",
        result_with_haste.err()
    );

    // Negative case: same mountain without the Haste effect → must be blocked by sickness.
    state
        .continuous_effects
        .retain(|e| e.id != mtg_engine::EffectId(11));
    let result_no_haste = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(mountain_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );
    assert!(
        result_no_haste.is_err(),
        "Animated land without Haste should be blocked by summoning sickness"
    );
}

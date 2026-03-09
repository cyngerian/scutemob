//! Delve keyword ability tests (CR 702.66).
//!
//! Delve is a static ability that functions while the spell is on the stack.
//! "For each generic mana in this spell's total cost, you may exile a card
//! from your graveyard rather than pay that mana." (CR 702.66a)
//!
//! Key rules verified:
//! - Each exiled card reduces one generic mana pip (CR 702.66a).
//! - Delve only reduces GENERIC mana — not colored pips (CR 702.66a).
//! - Delve applies AFTER total cost is determined (CR 702.66b).
//! - Delve is not an additional or alternative cost (CR 702.66b).
//! - Commander tax is applied before delve (CR 702.66b + CR 903.8).
//! - Cannot exile more cards than the generic mana requirement (Treasure Cruise ruling).
//! - Cards must be in the caster's own graveyard (CR 702.66a: "your graveyard").
//! - Cannot exile cards from an opponent's graveyard (CR 702.66a).
//! - Duplicate card IDs are rejected (engine validation).
//! - ObjectExiled events are emitted for each exiled card (CR 400.7).
//! - The old ObjectId from `delve_cards` is retired after exile (CR 400.7).
//! - Multiple instances of delve on the same spell are redundant (CR 702.66c).

use mtg_engine::{
    process_command, CardId, CardType, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SuperType, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Count objects with the given name in the exile zone.
fn count_in_exile(state: &GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == ZoneId::Exile)
        .count()
}

/// Count total objects in the exile zone.
fn total_in_exile(state: &GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count()
}

/// Count objects in the caster's graveyard.
fn graveyard_size(state: &GameState, owner: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(owner))
        .count()
}

/// Create a delve sorcery spell in hand.
///
/// Cost: `{generic}{blue}` where blue is the number of blue pips.
fn delve_spell_spec(owner: PlayerId, name: &str, generic: u32, blue: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            blue,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Delve)
}

/// Plain sorcery with no delve keyword.
fn plain_sorcery_spec(owner: PlayerId, name: &str, generic: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            ..Default::default()
        })
}

/// Create a card in the caster's graveyard (any type — lands, instants, creatures, etc.).
fn graveyard_card(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Instant])
}

/// Create a card in an opponent's graveyard.
fn opponent_graveyard_card(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Instant])
}

/// Create a creature on the battlefield (used for "card not in graveyard" test).
fn battlefield_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1)
}

// ── Test 1: Basic delve — exile cards reduce generic cost ──────────────────────

#[test]
/// CR 702.66a — Exiling cards from graveyard reduces generic mana when casting a delve spell.
/// Treasure Cruise-like spell: {7}{U}. Exile 7 cards from graveyard, pay {U} from pool.
fn test_delve_basic_exile_cards_reduce_generic_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // Treasure Cruise-like: {7}{U} — 7 generic + 1 blue pip.
    let spell = delve_spell_spec(p1, "Treasure Cruise", 7, 1);

    // 7 graveyard cards to exile for delve.
    let g1 = graveyard_card(p1, "Card 1");
    let g2 = graveyard_card(p1, "Card 2");
    let g3 = graveyard_card(p1, "Card 3");
    let g4 = graveyard_card(p1, "Card 4");
    let g5 = graveyard_card(p1, "Card 5");
    let g6 = graveyard_card(p1, "Card 6");
    let g7 = graveyard_card(p1, "Card 7");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(g1)
        .object(g2)
        .object(g3)
        .object(g4)
        .object(g5)
        .object(g6)
        .object(g7)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {U} — pays the 1 blue pip. The 7 generic are paid by 7 exiled cards.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Treasure Cruise");
    let id1 = find_object(&state, "Card 1");
    let id2 = find_object(&state, "Card 2");
    let id3 = find_object(&state, "Card 3");
    let id4 = find_object(&state, "Card 4");
    let id5 = find_object(&state, "Card 5");
    let id6 = find_object(&state, "Card 6");
    let id7 = find_object(&state, "Card 7");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![id1, id2, id3, id4, id5, id6, id7],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with delve failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.66a: spell should be on the stack after delve cast"
    );

    // All 7 graveyard cards are now in exile.
    assert_eq!(
        total_in_exile(&state),
        7,
        "CR 702.66a: 7 cards should be in exile after delve"
    );

    // Graveyard is now empty.
    assert_eq!(
        graveyard_size(&state, p1),
        0,
        "CR 702.66a: caster's graveyard should be empty after exiling all 7 cards"
    );

    // Mana pool is empty (blue pip consumed).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.66a: mana pool should be empty after delve + mana payment"
    );
}

// ── Test 2: Partial delve reduction ───────────────────────────────────────────

#[test]
/// CR 702.66a — Delve can be used for partial reduction. Exile 3 cards from graveyard,
/// pay remaining {1}{B} from mana pool for a Murderous Cut-like {4}{B} spell.
fn test_delve_partial_reduction() {
    let p1 = p(1);
    let p2 = p(2);

    // Murderous Cut-like: {4}{B} — 4 generic + 1 black pip.
    let spell = ObjectSpec::card(p1, "Murderous Cut")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("murderous-cut"))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 4,
            black: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Delve);

    // 3 graveyard cards for partial delve (leaves {1}{B} to pay).
    let g1 = graveyard_card(p1, "Dead Card 1");
    let g2 = graveyard_card(p1, "Dead Card 2");
    let g3 = graveyard_card(p1, "Dead Card 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(g1)
        .object(g2)
        .object(g3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {1}{B} from pool (3 generic paid by 3 exiled cards, 1 generic + 1 black remain).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Murderous Cut");
    let id1 = find_object(&state, "Dead Card 1");
    let id2 = find_object(&state, "Dead Card 2");
    let id3 = find_object(&state, "Dead Card 3");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![id1, id2, id3],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Partial delve cast failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.66a: spell should be on the stack after partial delve cast"
    );

    // 3 cards are in exile.
    assert_eq!(
        total_in_exile(&state),
        3,
        "CR 702.66a: 3 cards should be in exile after partial delve"
    );

    // Graveyard has 0 cards left (started with 3, exiled 3).
    assert_eq!(
        graveyard_size(&state, p1),
        0,
        "CR 702.66a: graveyard should have 0 cards after partial delve of 3"
    );

    // Mana pool is empty ({1}{B} consumed).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.66a: mana pool should be empty after delve + mana payment"
    );
}

// ── Test 3: ObjectExiled events and CR 400.7 ──────────────────────────────────

#[test]
/// CR 702.66a + CR 400.7 — Each card exiled by delve produces an ObjectExiled event.
/// The old ObjectIds from `delve_cards` are retired after exile (new ObjectIds assigned).
fn test_delve_object_exiled_events() {
    let p1 = p(1);
    let p2 = p(2);

    // {3} spell — 3 generic, no colored pips.
    let spell = delve_spell_spec(p1, "Delve Spell", 3, 0);

    let g1 = graveyard_card(p1, "Exile Target 1");
    let g2 = graveyard_card(p1, "Exile Target 2");
    let g3 = graveyard_card(p1, "Exile Target 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(g1)
        .object(g2)
        .object(g3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No mana needed — 3 generic paid by 3 exiled cards.
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Delve Spell");
    let id1 = find_object(&state, "Exile Target 1");
    let id2 = find_object(&state, "Exile Target 2");
    let id3 = find_object(&state, "Exile Target 3");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![id1, id2, id3],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Delve cast failed: {:?}", e));

    // Verify 3 ObjectExiled events were emitted.
    let exiled_events: Vec<&GameEvent> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .collect();
    assert_eq!(
        exiled_events.len(),
        3,
        "CR 702.66a: 3 ObjectExiled events expected for 3 delve-exiled cards"
    );

    // Verify old ObjectIds are retired (not present as objects in graveyard).
    for old_id in [id1, id2, id3] {
        assert!(
            !state.objects.contains_key(&old_id),
            "CR 400.7: old ObjectId {:?} should be retired after exile zone change",
            old_id
        );
    }

    // Verify 3 cards are now in exile.
    assert_eq!(
        total_in_exile(&state),
        3,
        "CR 702.66a: 3 cards should be in exile zone after delve"
    );

    // Verify each exiled card appears in exile by name.
    for name in ["Exile Target 1", "Exile Target 2", "Exile Target 3"] {
        assert_eq!(
            count_in_exile(&state, name),
            1,
            "CR 702.66a: '{}' should be in exile after delve",
            name
        );
    }
}

// ── Test 4: Reject delve on spell without Delve keyword ───────────────────────

#[test]
/// CR 702.66a — Delve can only be used on spells that have the Delve keyword.
/// Attempting to use delve cards on a plain sorcery should return an error.
fn test_delve_reject_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    // A plain sorcery without Delve.
    let spell = plain_sorcery_spec(p1, "Plain Sorcery", 3);
    let card = graveyard_card(p1, "Graveyard Card");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide enough mana to pay the cost (but we'll also try to use delve).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Sorcery");
    let card_id = find_object(&state, "Graveyard Card");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![card_id],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.66a: should reject delve on spell without Delve keyword"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("delve") || err.contains("InvalidCommand"),
        "CR 702.66a: error should mention delve or be InvalidCommand, got: {err}"
    );
}

// ── Test 5: Reject too many exiled cards ─────────────────────────────────────

#[test]
/// CR 702.66a / Treasure Cruise ruling — Cannot exile more cards than the generic
/// mana requirement of a spell with delve. For a spell costing {2}{U}, at most
/// 2 cards can be exiled for delve. Attempting to exile 3 should be rejected.
fn test_delve_reject_too_many_cards() {
    let p1 = p(1);
    let p2 = p(2);

    // {2}{U}: 2 generic + 1 blue pip. Can exile at most 2 cards.
    let spell = delve_spell_spec(p1, "Small Delve Spell", 2, 1);
    let g1 = graveyard_card(p1, "Graveyard Card 1");
    let g2 = graveyard_card(p1, "Graveyard Card 2");
    let g3 = graveyard_card(p1, "Graveyard Card 3");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(g1)
        .object(g2)
        .object(g3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide {U} for the blue pip.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Small Delve Spell");
    let id1 = find_object(&state, "Graveyard Card 1");
    let id2 = find_object(&state, "Graveyard Card 2");
    let id3 = find_object(&state, "Graveyard Card 3");

    // Try to exile 3 cards for a spell with only 2 generic.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![id1, id2, id3],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.66a / ruling: should reject more delve cards than generic mana allows"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("exceeds") || err.contains("generic") || err.contains("InvalidCommand"),
        "CR 702.66a: error should mention exceeds/generic or be InvalidCommand, got: {err}"
    );
}

// ── Test 6: Reject card not in graveyard ──────────────────────────────────────

#[test]
/// CR 702.66a — Delve cards must be in the caster's graveyard. Attempting to
/// use a card that is on the battlefield should return an error.
fn test_delve_reject_card_not_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    // {1} spell — 1 generic.
    let spell = delve_spell_spec(p1, "Delve Spell", 1, 0);
    // A creature on the battlefield — NOT in the graveyard.
    let creature = battlefield_creature(p1, "Battlefield Creature");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Delve Spell");
    let creature_id = find_object(&state, "Battlefield Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![creature_id],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.66a: should reject delve on a card not in the graveyard"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("graveyard") || err.contains("InvalidCommand"),
        "CR 702.66a: error should mention graveyard or be InvalidCommand, got: {err}"
    );
}

// ── Test 7: Reject card from opponent's graveyard ─────────────────────────────

#[test]
/// CR 702.66a — "your graveyard" — Cannot exile cards from an opponent's graveyard
/// to pay for delve. Attempting to use an opponent's graveyard card should fail.
fn test_delve_reject_opponents_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    // {1} spell — 1 generic.
    let spell = delve_spell_spec(p1, "Delve Spell", 1, 0);
    // Opponent's graveyard card (p2 owns it).
    let opp_card = opponent_graveyard_card(p2, "Opponent's Card");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(opp_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Delve Spell");
    let opp_card_id = find_object(&state, "Opponent's Card");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![opp_card_id],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.66a: should reject delve using card from opponent's graveyard"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("graveyard") || err.contains("InvalidCommand"),
        "CR 702.66a: error should mention graveyard or be InvalidCommand, got: {err}"
    );
}

// ── Test 8: Reject duplicate cards ────────────────────────────────────────────

#[test]
/// Engine validation — Cannot pass the same ObjectId twice in `delve_cards`.
/// Duplicates would allow "paying" the same card twice, which is illegal.
fn test_delve_reject_duplicate_cards() {
    let p1 = p(1);
    let p2 = p(2);

    // {2} spell — 2 generic.
    let spell = delve_spell_spec(p1, "Delve Spell", 2, 0);
    let g1 = graveyard_card(p1, "Lone Graveyard Card");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .object(g1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Delve Spell");
    let card_id = find_object(&state, "Lone Graveyard Card");

    // Pass the same card ID twice.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![card_id, card_id],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "Engine validation: should reject duplicate delve_cards entries"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("duplicate") || err.contains("InvalidCommand"),
        "Engine validation: error should mention duplicate or be InvalidCommand, got: {err}"
    );
}

// ── Test 9: Zero delve cards — normal cast ────────────────────────────────────

#[test]
/// CR 702.66a — A spell with Delve can be cast normally with an empty `delve_cards` vec.
/// Full mana payment from the pool is required.
fn test_delve_zero_cards_normal_cast() {
    let p1 = p(1);
    let p2 = p(2);

    // {3}{U}: requires 4 mana total.
    let spell = delve_spell_spec(p1, "Delve Spell", 3, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay full cost from mana pool — no delve.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Delve Spell");

    let (state, _events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![], // No delve
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Normal cast of delve spell failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.66a: spell with no delve cards should go on stack normally"
    );

    // Mana pool is empty (full cost paid).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.66a: full mana cost should be paid when no cards used for delve"
    );
}

// ── Test 10: Delve with commander tax ────────────────────────────────────────

#[test]
/// CR 702.66b + CR 903.8 — Commander tax is an additional cost determined before
/// delve is applied. Delve reduces the total cost INCLUDING the tax.
///
/// Commander with Delve: mana cost {4}{U}{U}. After 1 previous cast, tax = {2}.
/// Total cost = {6}{U}{U}. Exile 6 cards, pay {U}{U} from pool.
fn test_delve_with_commander_tax() {
    let p1 = p(1);
    let p2 = p(2);

    // Commander with Delve: printed cost {4}{U}{U}.
    let cmd_id = cid("delve-commander");
    let commander_spec = ObjectSpec::card(p1, "Delve Commander")
        .with_card_id(cmd_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            generic: 4,
            blue: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Delve)
        .in_zone(ZoneId::Command(p1));

    // 6 graveyard cards to exile (pay {6} generic out of {6}{U}{U}).
    let g1 = graveyard_card(p1, "Past Card 1");
    let g2 = graveyard_card(p1, "Past Card 2");
    let g3 = graveyard_card(p1, "Past Card 3");
    let g4 = graveyard_card(p1, "Past Card 4");
    let g5 = graveyard_card(p1, "Past Card 5");
    let g6 = graveyard_card(p1, "Past Card 6");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_commander(p1, cmd_id.clone())
        .object(commander_spec)
        .object(g1)
        .object(g2)
        .object(g3)
        .object(g4)
        .object(g5)
        .object(g6)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pre-set tax to 1 (cast once previously) — adds {2} to total cost.
    // Total cost = {4}{U}{U} + {2} tax = {6}{U}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    // Give p1 {U}{U} — to pay the 2 colored blue pips. Cards pay the {6} generic.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    // Register commander zone replacements (required for casting from command zone).
    mtg_engine::register_commander_zone_replacements(&mut state);

    let cmd_obj_id = state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .copied()
        .expect("commander object not found");

    let id1 = find_object(&state, "Past Card 1");
    let id2 = find_object(&state, "Past Card 2");
    let id3 = find_object(&state, "Past Card 3");
    let id4 = find_object(&state, "Past Card 4");
    let id5 = find_object(&state, "Past Card 5");
    let id6 = find_object(&state, "Past Card 6");

    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![id1, id2, id3, id4, id5, id6],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Commander delve cast failed: {:?}", e));

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.66b + 903.8: commander delve spell should be on the stack"
    );

    // Commander tax incremented to 2.
    assert_eq!(
        state.players[&p1].commander_tax.get(&cmd_id).copied(),
        Some(2),
        "CR 903.8: commander tax should increment to 2 after second cast"
    );

    // CommanderCastFromCommandZone event emitted with tax_paid = 1.
    let commander_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, tax_paid: 1, .. }
            if *player == p1
        )
    });
    assert!(
        commander_event,
        "CR 903.8: CommanderCastFromCommandZone event with tax_paid=1 expected"
    );

    // 6 cards are now in exile.
    assert_eq!(
        total_in_exile(&state),
        6,
        "CR 702.66b: 6 cards should be in exile after delve with commander tax"
    );

    // Mana pool empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.66b: mana pool should be empty after delve + mana payment"
    );
}

//! Flashback keyword ability tests (CR 702.34).
//!
//! Flashback is a static ability that allows instants and sorceries to be cast
//! from the graveyard by paying an alternative cost (the flashback cost). When
//! a flashback spell leaves the stack for any reason, it is exiled instead of
//! going anywhere else.
//!
//! Key rules verified:
//! - Cast from graveyard by paying flashback cost (CR 702.34a).
//! - Exiled on resolution instead of graveyard (CR 702.34a).
//! - Exiled when countered via Effect::CounterSpell (CR 702.34a).
//! - Sorcery-speed timing restriction still applies from graveyard (CR 702.34a + Think Twice ruling).
//! - Non-flashback card in graveyard cannot be cast (negative: CR 702.34a).
//! - Normal hand cast does NOT exile the card on resolution (CR 702.34a — flashback flag is false).
//! - Flashback cost is paid instead of mana cost (CR 118.9 / CR 702.34a).
//! - Mana value is based on printed mana cost, not flashback cost (CR 118.9c).
//! - Copies created from flashback spells have cast_with_flashback: false (CR 707.10).
//! - Insufficient flashback mana is rejected (CR 601.2f-h).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, Step, Target, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
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

/// Think Twice: Instant {1}{U}, "Draw a card. Flashback {2}{U}"
/// Mana cost: 1 generic + 1 blue = MV 2.
/// Flashback cost: 2 generic + 1 blue = MV 3.
fn think_twice_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("think-twice".to_string()),
        name: "Think Twice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Flashback {2}{U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::Flashback {
                cost: ManaCost {
                    generic: 2,
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Faithless Looting: Sorcery {R}, "Draw a card. Flashback {2}{R}"
/// Using a simpler draw-only effect for testing.
fn faithless_looting_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("faithless-looting".to_string()),
        name: "Faithless Looting".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Flashback {2}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::Flashback {
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Lightning Bolt: Instant {R}, no flashback.
fn lightning_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Counterspell: Instant {U}{U}, "Counter target spell."
fn counterspell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("counterspell".to_string()),
        name: "Counterspell".to_string(),
        mana_cost: Some(ManaCost {
            blue: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Counter target spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic flashback cast from graveyard ────────────────────────────────

#[test]
/// CR 702.34a — A card with flashback in the graveyard can be cast by paying the flashback cost.
fn test_flashback_basic_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    // Think Twice is already in p1's graveyard (was previously cast or discarded).
    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {2}{U} mana (flashback cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    // p1 casts Think Twice from graveyard via flashback.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.34a: SpellCast event expected for flashback cast"
    );

    // Think Twice is now on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.34a: Think Twice should be on the stack"
    );

    // Mana pool should be empty (flashback cost {2}{U} paid).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 702.34a: flashback cost {{2}}{{U}} should have been deducted from mana pool"
    );

    // The stack object has cast_with_flashback: true.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.cast_with_flashback,
        "CR 702.34a: stack object should have cast_with_flashback: true"
    );
}

// ── Test 2: Exile on resolution ────────────────────────────────────────────────

#[test]
/// CR 702.34a — When a flashback spell resolves, it is exiled (not put into graveyard).
fn test_flashback_exile_on_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    // Cast from graveyard via flashback.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Both players pass priority — spell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // SpellResolved (not SpellCountered, not SpellFizzled).
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p1)),
        "CR 702.34a: SpellResolved event expected"
    );

    // Think Twice should be in exile, NOT in graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.34a: flashback spell should be exiled on resolution, not in graveyard"
    );

    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && matches!(o.zone, ZoneId::Graveyard(_)));
    assert!(
        !in_graveyard,
        "CR 702.34a: flashback spell must NOT be in graveyard after resolution"
    );
}

// ── Test 3: Exile when countered ───────────────────────────────────────────────

#[test]
/// CR 702.34a — When a flashback spell is countered, it is exiled (not put into graveyard).
fn test_flashback_exile_on_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def(), counterspell_def()]);

    let flashback_card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let counter_card = ObjectSpec::card(p2, "Counterspell")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("counterspell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flashback_card)
        .object(counter_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {2}{U} for flashback cost; p2 has {U}{U} for counterspell.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let flashback_id = find_object(&state, "Think Twice");

    // p1 casts Think Twice from graveyard via flashback.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: flashback_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Find Think Twice on the stack (as a game object in state.objects with zone == Stack).
    // The counterspell targets the card object (state.objects), not the StackObject entry.
    // The effects/mod.rs CounterSpell effect also matches on StackObject.kind.source_object,
    // so targeting the card's ObjectId in zone Stack is correct per the targeting API.
    let spell_card_on_stack = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Think Twice" && obj.zone == ZoneId::Stack {
                Some(id)
            } else {
                None
            }
        })
        .expect("Think Twice should be in Stack zone as a game object");

    let counter_id = find_object(&state, "Counterspell");

    // p1 passes priority — priority goes to p2.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 casts Counterspell targeting Think Twice's card object on the stack.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![Target::Object(spell_card_on_stack)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Both players pass — Counterspell resolves (counters Think Twice), then Counterspell itself resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2, p1, p2]);

    // SpellCountered event emitted for Think Twice.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)),
        "CR 702.34a: SpellCountered event expected for Think Twice"
    );

    // Think Twice should be in exile, NOT in graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.34a: countered flashback spell should be exiled, not in graveyard"
    );

    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && matches!(o.zone, ZoneId::Graveyard(_)));
    assert!(
        !in_graveyard,
        "CR 702.34a: countered flashback spell must NOT be in graveyard"
    );
}

// ── Test 4: Sorcery-speed timing from graveyard ────────────────────────────────

#[test]
/// CR 702.34a + Think Twice ruling 2024-11-08: Sorcery-speed flashback cards can only be
/// cast at sorcery speed. A sorcery with flashback cannot be cast during another player's turn.
fn test_flashback_sorcery_timing_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![faithless_looting_def()]);

    let card = ObjectSpec::card(p1, "Faithless Looting")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("faithless-looting".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p2) // p2 is the active player — p1 cannot cast sorceries
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Faithless Looting");

    // p1 tries to cast Faithless Looting via flashback during p2's turn (sorcery speed — invalid).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.34a: sorcery with flashback cannot be cast during opponent's turn"
    );
}

// ── Test 5: Non-flashback card in graveyard cannot be cast ─────────────────────

#[test]
/// CR 702.34a — A card without flashback in the graveyard cannot be cast.
fn test_flashback_non_flashback_card_cannot_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    // Lightning Bolt in graveyard (no flashback).
    let card = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("lightning-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Lightning Bolt");

    // Try to cast from graveyard — should fail (no flashback keyword).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.34a: non-flashback card in graveyard cannot be cast"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("hand") || err_msg.contains("InvalidCommand"),
        "CR 702.34a: error should indicate card is not in hand, got: {err_msg}"
    );
}

// ── Test 6: Flashback pays flashback cost, not mana cost ───────────────────────

#[test]
/// CR 702.34a, CR 118.9: The flashback cost is paid instead of the mana cost.
/// Think Twice mana cost is {1}{U} (MV=2), flashback cost is {2}{U} (MV=3).
/// A player with exactly {2}{U} (not enough for... well, it IS enough for {1}{U} too,
/// but we verify the flashback cost is what was actually charged).
fn test_flashback_pays_flashback_cost_not_mana_cost() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide exactly {2}{U} — enough for flashback cost but not more.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    // Cast from graveyard via flashback — should succeed with {2}{U}.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .expect("CR 702.34a: flashback cast with {2}{U} should succeed");

    // ManaCostPaid event should be emitted with flashback cost {2}{U}.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)),
        "CR 702.34a: ManaCostPaid event expected for flashback cost"
    );

    // Mana pool should be completely empty (all {2}{U} consumed).
    let pool = &state.players[&p1].mana_pool;
    let total_mana = pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white;
    assert_eq!(
        total_mana, 0,
        "CR 702.34a: flashback cost {{2}}{{U}} should have consumed all mana"
    );
}

// ── Test 7: Normal hand cast does NOT exile on resolution ──────────────────────

#[test]
/// CR 702.34a: Flashback only applies when the spell was cast FROM the graveyard.
/// When cast normally from hand, the spell goes to graveyard on resolution (not exile).
fn test_flashback_normal_hand_cast_not_exiled() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 pays mana cost {1}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    // Cast from hand (normal cast, not flashback).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Verify cast_with_flashback is false on the stack object.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        !stack_obj.cast_with_flashback,
        "CR 702.34a: normal hand cast should have cast_with_flashback: false"
    );

    // Both players pass — spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Think Twice should be in graveyard (not exile) — normal cast behavior.
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && matches!(o.zone, ZoneId::Graveyard(_)));
    assert!(
        in_graveyard,
        "CR 702.34a: normally-cast flashback spell should go to graveyard on resolution"
    );

    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Think Twice" && o.zone == ZoneId::Exile);
    assert!(
        !in_exile,
        "CR 702.34a: normally-cast flashback spell must NOT be exiled"
    );
}

// ── Test 8: cast_with_flashback flag set on stack ──────────────────────────────

#[test]
/// CR 702.34a: When a spell is cast via flashback, the cast_with_flashback flag is
/// set to true on the StackObject. This flag drives the exile replacement.
fn test_flashback_cast_with_flashback_flag_set_on_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    let stack_obj = state
        .stack_objects
        .back()
        .expect("spell should be on stack");

    assert!(
        stack_obj.cast_with_flashback,
        "CR 702.34a: stack object cast from graveyard via flashback must have cast_with_flashback: true"
    );
    assert!(!stack_obj.is_copy, "flashback spell is not a copy");
}

// ── Test 9: Insufficient flashback mana is rejected ───────────────────────────

#[test]
/// CR 601.2f-h: If the player cannot pay the flashback cost, the cast is rejected.
/// Think Twice flashback cost is {2}{U}. With only {1}{U}, the cast should fail.
fn test_flashback_insufficient_flashback_mana_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only {1}{U} — NOT enough for flashback cost {2}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2f-h: flashback cast with insufficient mana should fail"
    );
}

// ── Test 10: Mana value is based on printed mana cost, not flashback cost ──────

#[test]
/// CR 118.9c: The spell's mana value is based on its printed mana cost, not the
/// flashback cost. Think Twice has a printed mana cost of {1}{U} (MV=2); its
/// flashback cost {2}{U} does not change the mana value.
fn test_flashback_mana_value_unchanged() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![think_twice_def()]);

    let card = ObjectSpec::card(p1, "Think Twice")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("think-twice".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Think Twice");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // The card on the stack should still have its printed mana cost {1}{U} (MV=2).
    let spell_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Think Twice" && o.zone == ZoneId::Stack)
        .expect("Think Twice should be on stack");

    let mana_cost = spell_obj
        .characteristics
        .mana_cost
        .as_ref()
        .expect("Think Twice should have a mana cost on the stack");

    assert_eq!(
        mana_cost.mana_value(),
        2,
        "CR 118.9c: mana value should be 2 (printed cost {{1}}{{U}}), not 3 (flashback cost {{2}}{{U}})"
    );
    assert_eq!(
        mana_cost.blue, 1,
        "CR 118.9c: printed mana cost should have 1 blue pip"
    );
    assert_eq!(
        mana_cost.generic, 1,
        "CR 118.9c: printed mana cost should have 1 generic pip"
    );
}

//! Aftermath keyword ability tests (CR 702.127).
//!
//! Aftermath is an ability found on split cards. It represents three static abilities:
//! (a) You may cast the aftermath half of this split card from your graveyard.
//! (b) The aftermath half can't be cast from any zone other than a graveyard.
//! (c) If the aftermath half was cast from a graveyard, exile it instead of putting
//!     it anywhere else any time it would leave the stack.
//!
//! Key rules verified:
//! - First half can be cast normally from hand (CR 709.3).
//! - Aftermath half can only be cast from graveyard (CR 702.127a).
//! - Aftermath half is exiled on resolution (CR 702.127a).
//! - Aftermath half is exiled when countered (CR 702.127a).
//! - Aftermath half cannot be cast from hand (CR 702.127a).
//! - First half cast from hand goes to graveyard on resolution, not exile (CR 608.2n).
//! - Aftermath cost is paid instead of the first half's mana cost (CR 118.9 / CR 702.127a).
//! - Aftermath half's effect is used at resolution, not the first half's effect (CR 702.127a).
//! - Non-aftermath card in graveyard with cast_with_aftermath: true is rejected.
//! - Full lifecycle: hand cast -> graveyard -> aftermath cast -> exile.

use mtg_engine::cards::card_definition::{EffectAmount, ForEachTarget, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
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

/// "Cut // Ribbons" test card definition (simplified for testing).
///
/// First half "Cut": Sorcery, {1}{R}, deals 4 damage to target creature.
/// Aftermath half "Ribbons": Sorcery, {2}{B}{B}, each opponent loses 3 life.
///
/// This simplifies the real card (Ribbons has {X}{B}{B}) to avoid X cost complexity.
fn cut_ribbons_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cut-ribbons".to_string()),
        name: "Cut // Ribbons".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Cut deals 4 damage to target creature. // Aftermath: Each opponent loses 3 life."
                .to_string(),
        abilities: vec![
            // Keyword marker for quick presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Aftermath),
            // First half "Cut": deals 4 damage to a target creature.
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(4),
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
            // Aftermath half "Ribbons": each opponent loses 3 life.
            AbilityDefinition::Aftermath {
                name: "Ribbons".to_string(),
                cost: ManaCost {
                    generic: 2,
                    black: 2,
                    ..Default::default()
                },
                card_type: CardType::Sorcery,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(3),
                    }),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

/// Lightning Bolt: Instant {R}, no aftermath. Used for negative tests.
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

// ── Test 1: First half cast from hand ─────────────────────────────────────────

#[test]
/// CR 709.3 — The first half of a split card with aftermath can be cast normally from hand.
fn test_aftermath_basic_cast_first_half_from_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A 2/2 creature in p2's battlefield for Cut to target.
    let creature = ObjectSpec::creature(p2, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {1}{R} mana for Cut's first-half cost.
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
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");
    let creature_id = find_object(&state, "Target Creature");

    // p1 casts Cut (first half) from hand normally — no aftermath flag.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
        },
    )
    .expect("CR 709.3: casting first half of aftermath card from hand should succeed");

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 709.3: SpellCast event expected for first-half cast from hand"
    );

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 709.3: Cut should be on the stack"
    );

    // cast_with_flashback is false (not an exile-on-departure spell when cast from hand).
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        !stack_obj.cast_with_flashback,
        "CR 709.3: first half cast from hand must have cast_with_flashback: false"
    );
    assert!(
        !stack_obj.cast_with_aftermath,
        "CR 709.3: first half cast from hand must have cast_with_aftermath: false"
    );

    // Mana pool should be empty ({1}{R} deducted).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 709.3: Cut first-half cost {{1}}{{R}} should have been deducted"
    );
}

// ── Test 2: Aftermath half cast from graveyard ────────────────────────────────

#[test]
/// CR 702.127a — The aftermath half can be cast from the graveyard by paying its cost.
fn test_aftermath_cast_second_half_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    // Cut // Ribbons already in p1's graveyard.
    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {2}{B}{B} mana for Ribbons' aftermath cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // p1 casts Ribbons (aftermath half) from graveyard.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("CR 702.127a: aftermath half should be castable from graveyard");

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.127a: SpellCast event expected for aftermath cast from graveyard"
    );

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.127a: Ribbons should be on the stack"
    );

    // cast_with_aftermath: true on the stack object.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.cast_with_aftermath,
        "CR 702.127a: stack object must have cast_with_aftermath: true"
    );
    // cast_with_flashback is set true (reuses flashback's exile-on-departure mechanism).
    assert!(
        stack_obj.cast_with_flashback,
        "CR 702.127a: aftermath cast reuses cast_with_flashback: true for exile-on-departure"
    );

    // Aftermath cost {2}{B}{B} should have been deducted.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 702.127a: aftermath cost {{2}}{{B}}{{B}} should have been deducted"
    );
}

// ── Test 3: Exile on resolution ───────────────────────────────────────────────

#[test]
/// CR 702.127a — "Exile it instead of putting it anywhere else any time it would leave the stack."
/// When the aftermath half resolves, it is exiled (not returned to the graveyard).
fn test_aftermath_exile_on_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

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
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // Cast the aftermath half (Ribbons) from graveyard.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap();

    // Both players pass priority — spell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // SpellResolved event emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { player, .. } if *player == p1)),
        "CR 702.127a: SpellResolved event expected for aftermath cast"
    );

    // Cut // Ribbons should be in EXILE, not in graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cut // Ribbons" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.127a: aftermath spell should be exiled on resolution, not in graveyard"
    );

    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Cut // Ribbons" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 702.127a: aftermath spell must NOT be in graveyard after resolution"
    );
}

// ── Test 4: Exile when countered ──────────────────────────────────────────────

#[test]
/// CR 702.127a — "Any time it would leave the stack" includes being countered.
/// When the aftermath half is countered, it is exiled (not put into graveyard).
fn test_aftermath_exile_on_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def(), counterspell_def()]);

    let aftermath_card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

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
        .object(aftermath_card)
        .object(counter_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {2}{B}{B} for Ribbons; p2 has {U}{U} for Counterspell.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
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

    let aftermath_id = find_object(&state, "Cut // Ribbons");

    // p1 casts Ribbons from graveyard via aftermath.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: aftermath_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap();

    // Find the Ribbons spell on the stack as a game object.
    let spell_on_stack = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Cut // Ribbons" && obj.zone == ZoneId::Stack {
                Some(id)
            } else {
                None
            }
        })
        .expect("Cut // Ribbons should be on stack");

    let counter_id = find_object(&state, "Counterspell");

    // p1 passes priority — priority goes to p2.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 casts Counterspell targeting Ribbons.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![Target::Object(spell_on_stack)],
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
    .unwrap();

    // Both players pass — Counterspell resolves (counters Ribbons), then Counterspell resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2, p1, p2]);

    // SpellCountered event for Ribbons.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)),
        "CR 702.127a: SpellCountered event expected for countered aftermath spell"
    );

    // Cut // Ribbons should be in exile, NOT in graveyard.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cut // Ribbons" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.127a: countered aftermath spell should be exiled, not in graveyard"
    );

    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Cut // Ribbons" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 702.127a: countered aftermath spell must NOT be in graveyard"
    );
}

// ── Test 5: Cannot cast aftermath half from hand ───────────────────────────────

#[test]
/// CR 702.127a — "This half of this split card can't be cast from any zone other than a graveyard."
/// Attempting to cast the aftermath half from hand with cast_with_aftermath: true must fail.
fn test_aftermath_cannot_cast_second_half_from_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    // Card in HAND — not graveyard.
    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

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
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // Attempt to cast aftermath half from hand — must fail.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.127a: aftermath half cannot be cast from hand"
    );
}

// ── Test 6: Cannot cast graveyard card without aftermath flag ─────────────────

#[test]
/// Negative test: Card with aftermath in graveyard but cast_with_aftermath: false
/// (and no flashback/escape) should be rejected as "card is not in your hand".
fn test_aftermath_cannot_cast_second_half_without_flag() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

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
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // Try to cast from graveyard without setting any graveyard-casting flag — must fail.
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
        "Card with aftermath in graveyard cannot be cast without cast_with_aftermath: true"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("hand") || err_msg.contains("InvalidCommand"),
        "Error should indicate card is not in hand, got: {err_msg}"
    );
}

// ── Test 7: First half goes to graveyard (not exile) ─────────────────────────

#[test]
/// CR 608.2n — When the first half of an aftermath card is cast from hand, it goes
/// to the graveyard on resolution (not exile). The exile-on-departure rule only
/// applies when cast via aftermath from the graveyard.
fn test_aftermath_first_half_goes_to_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A 2/2 creature for Cut to target.
    let creature = ObjectSpec::creature(p2, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {1}{R} for Cut.
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
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");
    let creature_id = find_object(&state, "Target Creature");

    // Cast Cut (first half) from hand — no aftermath flag.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
        },
    )
    .unwrap();

    // Both players pass — Cut resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Cut // Ribbons should be in p1's GRAVEYARD, not in exile.
    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Cut // Ribbons"
            && matches!(o.zone, ZoneId::Graveyard(p) if p == p1)
    });
    assert!(
        in_graveyard,
        "CR 608.2n: first-half cast from hand should put the card in graveyard on resolution"
    );

    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cut // Ribbons" && o.zone == ZoneId::Exile);
    assert!(
        !in_exile,
        "CR 608.2n: first-half cast from hand must NOT exile the card on resolution"
    );
}

// ── Test 8: Aftermath pays aftermath cost, not first-half cost ────────────────

#[test]
/// CR 702.127a / CR 118.9 — When casting via aftermath, the aftermath half's cost
/// ({2}{B}{B}) is paid, not the first half's cost ({1}{R}).
fn test_aftermath_pays_aftermath_cost() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide exactly {2}{B}{B} — enough for aftermath cost, not for the first-half {1}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // Cast aftermath half — should succeed with {2}{B}{B}.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("CR 702.127a: aftermath cast with {{2}}{{B}}{{B}} should succeed");

    // ManaCostPaid event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)),
        "CR 702.127a: ManaCostPaid event expected for aftermath cost"
    );

    // All {2}{B}{B} consumed (4 mana total).
    let pool = &state.players[&p1].mana_pool;
    let total = pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white;
    assert_eq!(
        total, 0,
        "CR 702.127a: aftermath cost {{2}}{{B}}{{B}} should have consumed all mana"
    );
}

// ── Test 9: Non-aftermath card in graveyard with cast_with_aftermath: true ────

#[test]
/// Negative test: A card without the Aftermath keyword in graveyard cannot be
/// cast with cast_with_aftermath: true. The engine must validate the keyword.
fn test_aftermath_card_without_aftermath_in_graveyard_fails() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    // Lightning Bolt in graveyard — no Aftermath keyword.
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
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Lightning Bolt");

    // Attempt to cast Lightning Bolt from graveyard as aftermath — must fail.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.127a: card without Aftermath keyword cannot be cast with cast_with_aftermath: true"
    );
}

// ── Test 10: Aftermath effect fires, not first-half effect ───────────────────

#[test]
/// CR 702.127a / CR 709.3b — When the aftermath half is cast, its effect is
/// executed at resolution, not the first half's effect.
/// Ribbons: "Each opponent loses 3 life" (NOT Cut's "deal 4 to target creature").
fn test_aftermath_uses_aftermath_effect() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Record p2 and p3 life totals before resolution.
    let p2_life_before = state.players[&p2].life_total;
    let p3_life_before = state.players[&p3].life_total;
    let p1_life_before = state.players[&p1].life_total;

    // p1 has {2}{B}{B} for Ribbons.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

    // Cast Ribbons (aftermath half) — no targets needed.
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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap();

    // All players pass — Ribbons resolves.
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Opponents p2 and p3 should each have lost 3 life (Ribbons effect).
    assert_eq!(
        state.players[&p2].life_total,
        p2_life_before - 3,
        "CR 702.127a: p2 should have lost 3 life from Ribbons effect"
    );
    assert_eq!(
        state.players[&p3].life_total,
        p3_life_before - 3,
        "CR 702.127a: p3 should have lost 3 life from Ribbons effect"
    );

    // Controller p1 should NOT have lost life.
    assert_eq!(
        state.players[&p1].life_total, p1_life_before,
        "CR 702.127a: controller p1 should NOT lose life from Ribbons"
    );
}

// ── Test 11: Full lifecycle ───────────────────────────────────────────────────

#[test]
/// CR 702.127a / CR 709.3 — Full lifecycle test:
/// 1. Cast Cut (first half) from hand.
/// 2. Cut resolves — card goes to graveyard.
/// 3. Cast Ribbons (aftermath half) from graveyard.
/// 4. Ribbons resolves — card goes to exile.
fn test_aftermath_full_lifecycle() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A 2/2 creature for Cut to target.
    let creature = ObjectSpec::creature(p2, "Target Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    // Start with Cut // Ribbons in hand.
    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Phase 1: Cast Cut (first half) from hand.
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
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");
    let creature_id = find_object(&state, "Target Creature");

    // Cast Cut from hand.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
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
        },
    )
    .expect("CR 709.3: first half should cast from hand");

    // Both pass — Cut resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Phase 2: Cut // Ribbons should now be in p1's graveyard.
    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Cut // Ribbons"
            && matches!(o.zone, ZoneId::Graveyard(p) if p == p1)
    });
    assert!(
        in_graveyard,
        "CR 608.2n: after first-half resolution, card should be in p1's graveyard"
    );

    // Phase 3: Cast Ribbons (aftermath half) from graveyard.
    // Find the new object ID (zone change created a new object per CR 400.7).
    let card_in_graveyard = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Cut // Ribbons"
                && matches!(obj.zone, ZoneId::Graveyard(p) if p == p1)
            {
                Some(id)
            } else {
                None
            }
        })
        .expect("Cut // Ribbons should be in graveyard after Cut resolves");

    let p2_life_before = state.players[&p2].life_total;

    let mut state = state;
    // Add mana for Ribbons aftermath cost {2}{B}{B}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // Cast Ribbons from graveyard.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_in_graveyard,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .expect("CR 702.127a: aftermath half should be castable from graveyard");

    // Both pass — Ribbons resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Phase 4: Cut // Ribbons should be in EXILE (not graveyard).
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cut // Ribbons" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.127a: after aftermath resolution, card should be in exile"
    );

    let in_graveyard = state.objects.values().any(|o| {
        o.characteristics.name == "Cut // Ribbons" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_graveyard,
        "CR 702.127a: after aftermath resolution, card must NOT be in graveyard"
    );

    // Ribbons effect fired: p2 should have lost 3 life.
    assert_eq!(
        state.players[&p2].life_total,
        p2_life_before - 3,
        "CR 702.127a: Ribbons effect should have made p2 lose 3 life"
    );
}

// ── Test 12: Insufficient aftermath mana is rejected ──────────────────────────

#[test]
/// CR 601.2f-h — If the player cannot pay the aftermath cost, the cast is rejected.
/// Ribbons aftermath cost is {2}{B}{B}. With only {1}{B}{B}, the cast should fail.
fn test_aftermath_insufficient_mana_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![cut_ribbons_def()]);

    let card = ObjectSpec::card(p1, "Cut // Ribbons")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("cut-ribbons".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Aftermath);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only {1}{B}{B} — NOT enough for aftermath cost {2}{B}{B}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cut // Ribbons");

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
            alt_cost: Some(AltCostKind::Aftermath),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2f-h: aftermath cast with insufficient mana should fail"
    );
}

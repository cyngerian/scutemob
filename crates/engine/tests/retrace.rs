//! Retrace keyword ability tests (CR 702.81).
//!
//! Retrace is a static ability that allows instants and sorceries to be cast
//! from the graveyard by discarding a land card as an ADDITIONAL cost (not
//! alternative — the normal mana cost is still paid). Unlike Flashback, a
//! spell cast via retrace returns to the graveyard on resolution (not exiled),
//! making it re-castable in subsequent turns.
//!
//! Key rules verified:
//! - Cast from graveyard by paying normal mana cost + discarding a land (CR 702.81a).
//! - Card returns to graveyard on resolution, not exile (ruling 2008-08-01).
//! - Card returns to graveyard when countered (ruling 2008-08-01).
//! - Retrace card can be recast after resolution (ruling 2008-08-01).
//! - Sorcery-speed timing restriction still applies from graveyard (ruling 2008-08-01).
//! - Discarded card must be a land type (CR 702.81a).
//! - Discarded card must be in player's hand (CR 702.81a).
//! - Card without Retrace keyword cannot be cast via retrace.
//! - Pays normal mana cost, NOT a separate retrace cost (CR 702.81a).
//! - Normal hand cast does not require land discard (retrace_discard_land: None).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, Effect,
    GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, Step,
    Target, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

#[allow(dead_code)]
fn find_object_opt(state: &mtg_engine::GameState, name: &str) -> Option<mtg_engine::ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
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

// ── Card definitions ───────────────────────────────────────────────────────────

/// Flame Jab: Sorcery {R}, "Flame Jab deals 1 damage to any target. Retrace."
/// (Simplified for testing — single target + damage + retrace keyword.)
fn flame_jab_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("flame-jab".to_string()),
        name: "Flame Jab".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Flame Jab deals 1 damage to any target. Retrace".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Retrace),
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    // Use DrawCards as a simple no-target effect for most tests.
                    // For targeting tests we rely on the damage variant below.
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Simple sorcery without Retrace for negative tests.
fn simple_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("simple-sorcery".to_string()),
        name: "Simple Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A basic land card (Mountain) for use as the retrace discard cost.
fn mountain_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mountain".to_string()),
        name: "Mountain".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "{T}: Add {R}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

/// Counterspell: Instant {U}{U}, "Counter target spell."
fn counterspell_def() -> CardDefinition {
    use mtg_engine::cards::card_definition::EffectTarget;
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
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic retrace cast from graveyard ─────────────────────────────────

/// CR 702.81a — A card with Retrace in the graveyard can be cast by paying
/// its normal mana cost and discarding a land card from hand.
#[test]
fn test_retrace_basic_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    // Flame Jab is already in p1's graveyard (was previously cast/discarded).
    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    // Mountain is in p1's hand (to be discarded as the retrace additional cost).
    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {R} mana (normal mana cost for Flame Jab).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");

    // p1 casts Flame Jab from graveyard via retrace, discarding Mountain.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(
        result.is_ok(),
        "Retrace cast from graveyard should succeed: {:?}",
        result.err()
    );

    let (state, cast_events) = result.unwrap();

    // Verify SpellCast event was emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "SpellCast event should be emitted"
    );

    // Verify CardDiscarded event was emitted (land was discarded as cost).
    assert!(
        cast_events.iter().any(
            |e| matches!(e, GameEvent::CardDiscarded { player, object_id, .. }
                if *player == p1 && *object_id == mountain_id)
        ),
        "CardDiscarded event should be emitted for the land"
    );

    // Flame Jab should now be on the stack (moved out of graveyard).
    let jab_on_stack = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Stack);
    assert!(
        jab_on_stack,
        "Flame Jab should be on the stack after retrace cast"
    );

    // Mountain should be in p1's graveyard (discarded).
    let mountain_in_graveyard = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Mountain" && obj.zone == ZoneId::Graveyard(p1));
    assert!(
        mountain_in_graveyard,
        "Mountain should be in p1's graveyard after retrace discard"
    );

    // p1's mana pool should be empty ({R} was paid).
    let p1_mana = &state.players[&p1].mana_pool;
    assert_eq!(
        p1_mana.total(),
        0,
        "p1's mana pool should be empty after paying {{R}}"
    );
}

// ── Test 2: Card returns to graveyard on resolution ───────────────────────────

/// CR 702.81a + ruling 2008-08-01 — When a retrace spell resolves, it is put
/// into its owner's graveyard (NOT exiled). This is the critical difference
/// from Flashback.
#[test]
fn test_retrace_card_returns_to_graveyard_on_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain)
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

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");

    // Cast Flame Jab via retrace.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    )
    .unwrap();

    // Both players pass priority to resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Flame Jab should be back in p1's graveyard (not exiled, not in hand).
    let jab_in_graveyard = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Graveyard(p1));
    assert!(
        jab_in_graveyard,
        "Flame Jab should return to graveyard after retrace resolution (not exiled)"
    );

    // Flame Jab should NOT be in exile.
    let jab_in_exile = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Exile);
    assert!(
        !jab_in_exile,
        "Flame Jab should NOT be in exile after retrace (different from flashback)"
    );
}

// ── Test 3: Card returns to graveyard when countered ─────────────────────────

/// CR 702.81a + ruling 2008-08-01 — When a retrace spell is countered, it
/// goes to the graveyard (not exile). This allows it to be recast via retrace again.
#[test]
fn test_retrace_card_returns_to_graveyard_when_countered() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def(), counterspell_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let counter = ObjectSpec::card(p2, "Counterspell")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("counterspell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flash);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain)
        .object(counter)
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
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");
    let counter_id = find_object(&state, "Counterspell");

    // p1 casts Flame Jab via retrace.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    )
    .unwrap();

    // Find Flame Jab on the stack (new object id after zone change).
    let jab_stack_id = state
        .stack_objects
        .back()
        .map(|so| {
            if let mtg_engine::StackObjectKind::Spell { source_object } = so.kind {
                source_object
            } else {
                panic!("expected Spell on stack")
            }
        })
        .expect("Flame Jab should be on the stack");

    // After p1 casts, priority resets to the active player (p1).
    // p1 must pass priority before p2 can act.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 counters with Counterspell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![Target::Object(jab_stack_id)],
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
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    )
    .unwrap();

    // Both players pass priority to resolve Counterspell (which counters Flame Jab).
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve the Counterspell itself.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Flame Jab should be back in p1's graveyard (not exiled).
    let jab_in_graveyard = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Graveyard(p1));
    assert!(
        jab_in_graveyard,
        "Countered retrace spell should go to graveyard (not exile)"
    );

    let jab_in_exile = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Exile);
    assert!(
        !jab_in_exile,
        "Countered retrace spell should NOT be in exile"
    );
}

// ── Test 4: Normal timing for sorceries (cannot cast on opponent's turn) ───────

/// CR 702.81a + ruling 2008-08-01 — Sorceries with Retrace follow normal
/// timing rules. A sorcery with Retrace cannot be cast on an opponent's turn.
#[test]
fn test_retrace_normal_timing_sorcery_cannot_cast_on_opponents_turn() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain)
        .active_player(p2) // p2 is the active player — it's their turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1); // p1 has priority

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");

    // Attempt to cast Flame Jab via retrace during p2's turn — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(
        result.is_err(),
        "Retrace sorcery should fail when cast on opponent's turn"
    );
}

// ── Test 5: Discard must be a land card ───────────────────────────────────────

/// CR 702.81a — The card discarded as the retrace additional cost must be a
/// land card. Attempting to discard a non-land (e.g., an instant) should fail.
#[test]
fn test_retrace_discard_must_be_land() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), simple_sorcery_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    // A non-land card in hand — should not be valid as retrace cost.
    let non_land = ObjectSpec::card(p1, "Simple Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("simple-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(non_land)
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

    let jab_id = find_object(&state, "Flame Jab");
    let non_land_id = find_object(&state, "Simple Sorcery");

    // Attempt to discard a non-land as retrace cost — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(non_land_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(result.is_err(), "Retrace should reject a non-land discard");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("must be a land"),
        "Error should mention 'must be a land': {}",
        err_msg
    );
}

// ── Test 6: Discard must be in hand ───────────────────────────────────────────

/// CR 702.81a — The land card discarded as the retrace additional cost must
/// be in the player's hand. A land on the battlefield is not a valid discard target.
#[test]
fn test_retrace_discard_must_be_in_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    // Mountain on the battlefield (not in hand) — invalid retrace target.
    let mountain_on_battlefield = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain_on_battlefield)
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

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");

    // Attempt to use a land on the battlefield as retrace cost — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(result.is_err(), "Retrace should reject a land not in hand");
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("must be in your hand"),
        "Error should mention 'must be in your hand': {}",
        err_msg
    );
}

// ── Test 7: Card without Retrace keyword cannot be cast from graveyard ────────

/// CR 702.81a — A card without the Retrace keyword cannot be cast from the
/// graveyard just because the player provides a retrace_discard_land.
#[test]
fn test_retrace_no_retrace_keyword_cannot_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![simple_sorcery_def(), mountain_def()]);

    // Simple Sorcery in graveyard — no Retrace keyword.
    let non_retrace = ObjectSpec::card(p1, "Simple Sorcery")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("simple-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        });
    // Note: no .with_keyword(KeywordAbility::Retrace)

    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(non_retrace)
        .object(mountain)
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
    state.turn.priority_holder = Some(p1);

    let sorcery_id = find_object(&state, "Simple Sorcery");
    let mountain_id = find_object(&state, "Mountain");

    // Attempt to cast from graveyard with retrace_discard_land — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: sorcery_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(
        result.is_err(),
        "A card without Retrace keyword cannot be cast from graveyard via retrace"
    );
}

// ── Test 8: Pays normal mana cost (not a separate retrace cost) ───────────────

/// CR 702.81a — Retrace uses the card's NORMAL mana cost, not a separate
/// retrace cost. Verify the exact mana cost is deducted.
#[test]
fn test_retrace_pays_normal_mana_cost() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    let mountain = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 exactly {R} — the normal mana cost of Flame Jab.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let jab_id = find_object(&state, "Flame Jab");
    let mountain_id = find_object(&state, "Mountain");

    // Cast via retrace — should succeed with exactly {R}.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(
        result.is_ok(),
        "Retrace should succeed with exactly the normal mana cost: {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();

    // Mana pool should now be empty (exactly {R} was paid).
    assert_eq!(
        state.players[&p1].mana_pool.total(),
        0,
        "Exactly the normal mana cost ({{R}}) should be paid"
    );
}

// ── Test 9: Retrace without land in hand fails ───────────────────────────────

/// CR 702.81a — If the player has no land cards in hand, they cannot use retrace.
/// Providing None for retrace_discard_land when the card is in the graveyard
/// with Retrace should fail (treated as a regular graveyard cast attempt).
#[test]
fn test_retrace_without_land_provided_cannot_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);
    // No land in hand.

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
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

    let jab_id = find_object(&state, "Flame Jab");

    // No land provided — should fail as "card is not in your hand".
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None, // No land provided = no retrace permission
        },
    );

    assert!(
        result.is_err(),
        "Retrace without providing a land should fail"
    );
}

// ── Test 10: Normal hand cast does not require land discard ───────────────────

/// CR 702.81a — When casting a Retrace card from hand (normal cast),
/// no land discard is required. The retrace additional cost only applies when
/// casting from the graveyard.
#[test]
fn test_retrace_normal_hand_cast_no_land_discard_needed() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def()]);

    // Flame Jab is in p1's hand (normal cast, not graveyard).
    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
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

    let jab_id = find_object(&state, "Flame Jab");

    // Normal cast from hand — no retrace_discard_land needed.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: None,
            cast_with_jump_start: false,
            jump_start_discard: None, // No land required for hand cast
        },
    );

    assert!(
        result.is_ok(),
        "Normal hand cast of Retrace card should succeed without land discard: {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();

    // Flame Jab should be on the stack.
    let jab_on_stack = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Stack);
    assert!(
        jab_on_stack,
        "Flame Jab should be on the stack after normal hand cast"
    );
}

// ── Test 11: Recast after resolution ─────────────────────────────────────────

/// CR 702.81a + ruling 2008-08-01 — After a retrace spell resolves and returns
/// to the graveyard, it can be cast again via retrace (using another land).
#[test]
fn test_retrace_recast_after_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![flame_jab_def(), mountain_def()]);

    let flame_jab = ObjectSpec::card(p1, "Flame Jab")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("flame-jab".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Retrace);

    // Two mountains in hand for two retrace casts.
    let mountain1 = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mountain2 = ObjectSpec::card(p1, "Mountain")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mountain".to_string()))
        .with_types(vec![CardType::Land]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(flame_jab)
        .object(mountain1)
        .object(mountain2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2); // {R}{R} for two casts
    state.turn.priority_holder = Some(p1);

    let jab_id = find_object(&state, "Flame Jab");
    // Find both mountains (same name — pick distinct IDs).
    let mountains: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == "Mountain")
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(mountains.len(), 2, "should have 2 Mountains in hand");
    let (mountain1_id, mountain2_id) = (mountains[0], mountains[1]);

    // First retrace cast.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain1_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    )
    .expect("First retrace cast should succeed");

    // Resolve the first retrace cast.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Flame Jab should be back in graveyard.
    let jab_in_graveyard = state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == "Flame Jab" && obj.zone == ZoneId::Graveyard(p1));
    assert!(
        jab_in_graveyard,
        "Flame Jab should be back in graveyard after first resolution"
    );

    // For the second cast, the state has a new object id for the jab (zone change).
    let new_jab_id = find_object(&state, "Flame Jab");

    // Reset priority for second cast.
    state.turn.priority_holder = Some(p1);

    // Second retrace cast (using the second mountain).
    let result2 = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: new_jab_id,
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
            cast_with_overload: false,
            retrace_discard_land: Some(mountain2_id),
            cast_with_jump_start: false,
            jump_start_discard: None,
        },
    );

    assert!(
        result2.is_ok(),
        "Second retrace cast should succeed after resolution: {:?}",
        result2.err()
    );
}

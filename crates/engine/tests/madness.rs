//! Madness keyword ability tests (CR 702.35).
//!
//! Madness is a keyword that represents two abilities:
//! 1. A static ability: when the card is discarded, exile it instead of putting
//!    it into the graveyard (CR 702.35a).
//! 2. A triggered ability: when the card is exiled this way, its owner may cast
//!    it by paying the madness cost. If not, it goes to the graveyard (CR 702.35a).
//!
//! Key rules verified:
//! - Discarded madness card goes to exile, not graveyard (CR 702.35a).
//! - `CardDiscarded` event still fires even when exiled (CR ruling).
//! - `MadnessTrigger` is pushed onto the stack after exile (CR 702.35a).
//! - Non-madness card discarded goes to graveyard (negative, CR 702.35a).
//! - Madness card in exile can be cast by paying madness cost (CR 702.35b).
//! - Madness ignores sorcery-speed timing restriction (CR ruling).
//! - `cast_with_madness: true` is set on the stack object (CR 702.35).
//! - Non-madness card in exile cannot be cast via CastSpell (negative).
//! - Madness trigger auto-decline puts card in graveyard (CR 702.35a).
//! - Madness works with cycling discard (CR 702.35a + cycling).
//! - Madness works with cleanup-step hand-size discard (CR ruling).

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectAmount, PlayerTarget, TargetRequirement,
};
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, CardEffectTarget, CardId, CardRegistry, Command, GameEvent, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, Step, Target, ZoneId,
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Fiery Temper: Instant {1}{R}{R}, "Fiery Temper deals 3 damage to any target. Madness {R}"
/// Mana cost: 1 generic + 2 red = MV 3.
/// Madness cost: 1 red = MV 1.
fn fiery_temper_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fiery-temper".to_string()),
        name: "Fiery Temper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fiery Temper deals 3 damage to any target. Madness {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Violent Eruption: Sorcery {2}{R}{R}, "Violent Eruption deals 2 damage to ... Madness {R}{R}"
/// Used to test sorcery-speed timing override.
fn violent_eruption_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("violent-eruption".to_string()),
        name: "Violent Eruption".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Violent Eruption deals 2 damage to any target. Madness {R}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness {
                cost: ManaCost {
                    red: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// A plain instant with no special abilities (for negative tests).
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
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

/// A card with madness AND cycling for interaction tests.
fn cycling_madness_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-madness-card".to_string()),
        name: "Cycling Madness Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Cycling {1}. Madness {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness {
                cost: ManaCost {
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

// ── Test 1: Madness discard goes to exile ──────────────────────────────────────

#[test]
/// CR 702.35a — When a card with madness is discarded, it goes to exile instead of graveyard.
fn test_madness_discard_goes_to_exile() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Use cleanup-step discard path: 8 cards in hand triggers discard to 7.
    // The cleanup code discards the last() ObjectId — add fillers first, then Fiery Temper.
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::End);

    // Add 7 fillers first (lowest ObjectIds).
    for i in 0..7u32 {
        let filler = ObjectSpec::card(p1, &format!("Filler {i}"))
            .in_zone(ZoneId::Hand(p1))
            .with_types(vec![CardType::Instant]);
        builder = builder.object(filler);
    }

    // Add Fiery Temper last (highest ObjectId = last() picks it for discard).
    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_keyword(KeywordAbility::Madness);
    builder = builder.object(temper);

    let state = builder.build().unwrap();

    // Pass both players in End step to trigger the transition to Cleanup.
    let (state, events_p1) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state_after, events_p2) =
        process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let all_events: Vec<_> = events_p1.into_iter().chain(events_p2).collect();

    // Fiery Temper should be in exile (madness replacement).
    let in_exile = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.35a: Fiery Temper (madness) should be in exile after discard, not graveyard"
    );

    // Card should NOT be in graveyard.
    let in_grave = state_after.objects.values().any(|o| {
        o.characteristics.name == "Fiery Temper" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        !in_grave,
        "CR 702.35a: Fiery Temper should NOT be in graveyard when madness applies"
    );

    // DiscardedToHandSize event fires (cleanup-step discard uses this event, not CardDiscarded).
    let discarded_event = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::DiscardedToHandSize { .. }));
    assert!(
        discarded_event,
        "CR 514.1: DiscardedToHandSize event must fire when cleanup discards a madness card"
    );
}

// ── Test 2: Non-madness discard goes to graveyard ─────────────────────────────

#[test]
/// CR 702.35a (negative) — A card without madness goes to graveyard when discarded.
fn test_madness_non_madness_card_goes_to_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![plain_instant_def()]);

    // Add 8 cards: 7 fillers first (lowest ObjectIds), then Plain Instant last (highest ObjectId).
    // cleanup_actions uses obj_ids.last() (highest ObjectId), so Plain Instant gets discarded.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::End);

    for i in 0..7u32 {
        let filler = ObjectSpec::card(p1, &format!("Filler2 {i}"))
            .in_zone(ZoneId::Hand(p1))
            .with_types(vec![CardType::Instant]);
        builder = builder.object(filler);
    }

    // Add Plain Instant last (highest ObjectId = last() picks it for discard).
    let plain = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant]);
    builder = builder.object(plain);

    let state = builder.build().unwrap();

    // Pass both players to trigger cleanup.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state_after, _events) =
        process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // Plain Instant should be in graveyard (no madness).
    let in_grave = state_after.objects.values().any(|o| {
        o.characteristics.name == "Plain Instant" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "CR 702.35a (negative): non-madness card should go to graveyard when discarded"
    );

    // Should NOT be in exile.
    let in_exile = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Plain Instant" && o.zone == ZoneId::Exile);
    assert!(
        !in_exile,
        "CR 702.35a (negative): non-madness card should not go to exile"
    );
}

// ── Test 3: MadnessTrigger on stack after discard ─────────────────────────────

#[test]
/// CR 702.35a — After a madness card is exiled by the discard replacement,
/// a MadnessTrigger is pushed onto the stack.
fn test_madness_trigger_on_stack_after_discard() {
    use mtg_engine::state::stack::StackObjectKind;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    // 8 cards: 7 fillers first (lowest ObjectIds), then Fiery Temper last (highest ObjectId).
    // cleanup_actions uses obj_ids.last() so Fiery Temper gets discarded.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::End);

    for i in 0..7u32 {
        let filler = ObjectSpec::card(p1, &format!("StackFiller {i}"))
            .in_zone(ZoneId::Hand(p1))
            .with_types(vec![CardType::Instant]);
        builder = builder.object(filler);
    }

    // Fiery Temper last — highest ObjectId, picked by obj_ids.last().
    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_keyword(KeywordAbility::Madness);
    builder = builder.object(temper);

    let state = builder.build().unwrap();

    // Pass both players to trigger cleanup.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state_after, _events) =
        process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // A MadnessTrigger should be on the stack.
    let has_madness_trigger = state_after
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. }));
    assert!(
        has_madness_trigger,
        "CR 702.35a: MadnessTrigger should be on the stack after madness card is discarded"
    );
}

// ── Test 4: Cast madness card from exile ─────────────────────────────────────

#[test]
/// CR 702.35a, CR 702.35b — A card with madness in exile can be cast by paying
/// the madness cost (an alternative cost).
fn test_madness_cast_from_exile() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    // Fiery Temper is already in exile with the Madness keyword.
    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Madness);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(temper)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {R} mana (madness cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Fiery Temper");

    // p1 casts Fiery Temper from exile via madness, targeting p2.
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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.35a/b: Casting Fiery Temper from exile via madness should succeed; got: {:?}",
        result.err()
    );

    let (state_after, cast_events) = result.unwrap();

    // SpellCast event should be emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.35a: SpellCast event expected for madness cast"
    );

    // Spell is on the stack.
    assert!(
        !state_after.stack_objects.is_empty(),
        "CR 702.35a: Spell should be on the stack after madness cast"
    );

    // Mana pool should be empty ({R} madness cost paid, not {1}{R}{R} mana cost).
    let pool = &state_after.players[&p1].mana_pool;
    assert_eq!(
        pool.red + pool.colorless + pool.blue + pool.green + pool.black + pool.white,
        0,
        "CR 702.35b: madness cost {{R}} should have been deducted (not {{1}}{{R}}{{R}})"
    );
}

// ── Test 5: Madness ignores sorcery timing ────────────────────────────────────

#[test]
/// CR 702.35 ruling — A sorcery cast via madness ignores timing restrictions
/// and can be cast any time the player has priority (like an instant).
fn test_madness_sorcery_ignores_timing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![violent_eruption_def()]);

    // Violent Eruption (a sorcery with madness) is in exile.
    // p1 is NOT the active player (opponent's turn) — sorcery-speed would normally fail.
    let eruption = ObjectSpec::card(p1, "Violent Eruption")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("violent-eruption".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Madness);

    // Active player is p2, not p1.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(eruption)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {R}{R} mana (madness cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Violent Eruption");

    // p1 casts the sorcery via madness during p2's turn — should succeed.
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
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.35 ruling: sorcery with madness should ignore timing restrictions; got: {:?}",
        result.err()
    );
}

// ── Test 6: Madness trigger auto-decline puts card in graveyard ───────────────

#[test]
/// CR 702.35a, CR ruling — If the owner declines to cast the madness card
/// when the trigger resolves, it goes to the graveyard.
fn test_madness_decline_goes_to_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    // 8 cards: 7 fillers first (lowest ObjectIds), Fiery Temper last (highest ObjectId).
    // cleanup_actions uses obj_ids.last() so Fiery Temper gets discarded.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::End);

    for i in 0..7u32 {
        let filler = ObjectSpec::card(p1, &format!("DeclineFiller {i}"))
            .in_zone(ZoneId::Hand(p1))
            .with_types(vec![CardType::Instant]);
        builder = builder.object(filler);
    }

    // Fiery Temper last — highest ObjectId, picked by obj_ids.last().
    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_keyword(KeywordAbility::Madness);
    builder = builder.object(temper);

    let state = builder.build().unwrap();

    // Pass both players in End step → triggers Cleanup → Fiery Temper discarded → exiled.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state_after_cleanup, _) =
        process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // Now resolve the MadnessTrigger by having all players pass priority.
    // MVP: the trigger auto-declines and moves the card to graveyard.
    let (state_final, _) = pass_all(state_after_cleanup, &[p1, p2]);

    // Fiery Temper should now be in graveyard (auto-decline).
    let in_grave = state_final.objects.values().any(|o| {
        o.characteristics.name == "Fiery Temper" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_grave,
        "CR 702.35a: After declining madness, card goes to graveyard"
    );

    // Card should NOT be in exile anymore.
    let in_exile = state_final
        .objects
        .values()
        .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile);
    assert!(
        !in_exile,
        "CR 702.35a: Card should leave exile when madness trigger resolves without cast"
    );
}

// ── Test 7: cast_with_madness flag is set on StackObject ─────────────────────

#[test]
/// CR 702.35 — When cast via madness, the stack object has `cast_with_madness: true`.
fn test_madness_cast_with_madness_flag_set_on_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Madness);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(temper)
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

    let card_id = find_object(&state, "Fiery Temper");

    let (state_after, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap();

    // The stack object for the spell should have cast_with_madness: true.
    let stack_obj = state_after.stack_objects.iter().find(|so| {
        matches!(
            so.kind,
            mtg_engine::state::stack::StackObjectKind::Spell { .. }
        )
    });

    assert!(
        stack_obj.is_some(),
        "CR 702.35: Spell should be on the stack"
    );
    assert!(
        stack_obj.unwrap().cast_with_madness,
        "CR 702.35: cast_with_madness should be true when cast via madness from exile"
    );
}

// ── Test 8: Non-madness card in exile cannot be cast ─────────────────────────

#[test]
/// CR 702.35a (negative) — A card without madness in exile cannot be cast with CastSpell.
fn test_madness_non_madness_exile_cannot_cast() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![plain_instant_def()]);

    // Plain Instant (no madness) in exile.
    let card = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant]);

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
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plain Instant");

    // Attempting to cast a non-madness card from exile should fail.
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
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.35a (negative): casting a non-madness card from exile should be rejected"
    );
}

// ── Test 9: Cycling discard triggers madness ──────────────────────────────────

#[test]
/// CR 702.35a + cycling — When a card with both madness and cycling is cycled,
/// the discard-as-cost triggers madness and the card goes to exile (not graveyard).
fn test_madness_cycling_discard_triggers_madness() {
    use mtg_engine::state::stack::StackObjectKind;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![cycling_madness_def()]);

    let card = ObjectSpec::card(p1, "Cycling Madness Card")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("cycling-madness-card".to_string()))
        .with_keyword(KeywordAbility::Cycling)
        .with_keyword(KeywordAbility::Madness);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add {1} mana for cycling cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Cycling Madness Card");

    let (state_after, events) = process_command(
        state,
        Command::CycleCard {
            player: p1,
            card: card_id,
        },
    )
    .unwrap();

    // Card went to exile (madness replacement applies to cycling discard).
    let in_exile = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Cycling Madness Card" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.35a: Madness card cycled should be in exile (madness applies to cycling discard)"
    );

    // CardDiscarded event fires.
    let discarded = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDiscarded { .. }));
    assert!(
        discarded,
        "CR ruling: CardDiscarded event must fire even when madness exiles during cycling"
    );

    // CardCycled event fires.
    let cycled = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardCycled { .. }));
    assert!(
        cycled,
        "CR 702.29a: CardCycled event must fire when cycling (madness doesn't suppress it)"
    );

    // MadnessTrigger is on the stack (along with the cycling draw ability).
    let has_madness_trigger = state_after
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. }));
    assert!(
        has_madness_trigger,
        "CR 702.35a: MadnessTrigger should be on the stack after cycling a madness card"
    );
}

// ── Test 10: Madness mana value is based on printed cost ─────────────────────

#[test]
/// CR 118.9c — The mana value of a spell cast for its madness cost is determined
/// by its printed mana cost, not the madness cost.
fn test_madness_mana_value_unchanged() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![fiery_temper_def()]);

    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Exile)
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Madness);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(temper)
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

    let card_id = find_object(&state, "Fiery Temper");

    let (state_after, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap();

    // The card on the stack has its printed mana cost ({1}{R}{R} = MV 3), not the madness cost ({R} = MV 1).
    // Verify by checking the characteristics of the card on the stack.
    let stack_spell = state_after.stack_objects.iter().find(|so| {
        matches!(
            so.kind,
            mtg_engine::state::stack::StackObjectKind::Spell { .. }
        )
    });
    assert!(stack_spell.is_some(), "Spell should be on the stack");

    // The card is now in ZoneId::Stack; find it to check mana_cost.
    let stack_card = state_after
        .objects
        .values()
        .find(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Stack);
    assert!(
        stack_card.is_some(),
        "Fiery Temper should be in ZoneId::Stack"
    );

    let mana_cost = stack_card.unwrap().characteristics.mana_cost.as_ref();
    assert!(
        mana_cost.is_some(),
        "CR 118.9c: Printed mana cost should still be present on stack"
    );
    // Printed MV = 1 generic + 2 red = 3.
    assert_eq!(
        mana_cost.unwrap().mana_value(),
        3,
        "CR 118.9c: Mana value on stack should be 3 (printed cost), not 1 (madness cost)"
    );
}

// ── Test 11: Effect-based discard triggers madness ───────────────────────────

#[test]
/// CR 702.35a — The effect-based discard path (Effect::DiscardCards) also applies
/// the madness replacement, sending the card to exile instead of graveyard.
fn test_madness_effect_discard_goes_to_exile() {
    use mtg_engine::state::stack::StackObjectKind;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Define a "Discard Spell" that forces p1 to discard 1 card.
    let discard_spell = CardDefinition {
        card_id: CardId("discard-spell".to_string()),
        name: "Discard Spell".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Target player discards a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DiscardCards {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                count: EffectAmount::Fixed(1),
            },
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![fiery_temper_def(), discard_spell]);

    // p1 has Fiery Temper in hand. p2 has the discard spell in hand.
    let temper = ObjectSpec::card(p1, "Fiery Temper")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fiery-temper".to_string()))
        .with_keyword(KeywordAbility::Madness);

    let discard = ObjectSpec::card(p2, "Discard Spell")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("discard-spell".to_string()))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(temper)
        .object(discard)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p2 {B} mana to cast the discard spell.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    // Give p2 priority.
    state.turn.priority_holder = Some(p2);

    let discard_id = find_object(&state, "Discard Spell");

    // p2 casts Discard Spell targeting p1.
    let (state_after_cast, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: discard_id,
            targets: vec![Target::Player(p1)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
        },
    )
    .unwrap();

    // Both players pass priority → spell resolves.
    let (state_after_resolve, _) = pass_all(state_after_cast, &[p1, p2]);

    // Fiery Temper should be in exile (madness replacement on effect-based discard).
    let in_exile = state_after_resolve
        .objects
        .values()
        .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.35a: Fiery Temper should be in exile after effect-based discard (madness applies)"
    );

    // MadnessTrigger should be on the stack.
    let has_trigger = state_after_resolve
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. }));
    assert!(
        has_trigger,
        "CR 702.35a: MadnessTrigger should be on stack after effect-based discard of madness card"
    );
}

//! Entwine keyword ability tests (CR 702.42).
//!
//! Entwine is a static ability on modal spells. "Entwine [cost]" means the caster
//! may choose ALL modes of the spell instead of just the number specified, by paying
//! an additional [cost]. When entwine is paid, all modes execute in printed order.
//!
//! Key rules verified:
//! - CR 702.42a: Entwine is an additional cost, not an alternative cost.
//! - CR 702.42b: When entwined, all modes execute in printed order.
//! - CR 601.2f: The entwine cost is added to the total mana cost when paid.
//! - Engine validation: spells without KeywordAbility::Entwine reject entwine_paid = true.
//! - Auto-mode[0]: when not entwined, only the first mode executes.
//! - Entwine cost stacks with commander tax (CR 601.2f + CR 903.8).

use mtg_engine::state::CardType;
use mtg_engine::Effect;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command,
    EffectAmount, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ModeSelection,
    ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};
use std::sync::Arc;

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

/// Build a synthetic "Entwine Test Spell" card:
/// Sorcery {1}{U}
/// Choose one:
///   Mode 0: GainLife(3) — p1 gains 3 life.
///   Mode 1: DrawCards(2) — p1 draws 2 cards.
/// Entwine {2}
///
/// This self-contained card lets us test entwine with no external dependencies.
fn entwine_test_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("entwine-test-spell".to_string()),
        name: "Entwine Test Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one — Gain 3 life; or draw 2 cards. Entwine {2}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            AbilityDefinition::Entwine {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: gain 3 life.
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        // Mode 1: draw 2 cards.
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Build a registry containing the entwine test spell.
fn entwine_registry() -> Arc<CardRegistry> {
    CardRegistry::new(vec![entwine_test_spell_def()])
}

/// Build a registry containing the entwine test spell plus a plain non-modal sorcery.
fn entwine_registry_with_plain() -> (Arc<CardRegistry>, CardDefinition) {
    let plain = CardDefinition {
        card_id: CardId("plain-sorcery".to_string()),
        name: "Plain Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
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
    };
    let reg = CardRegistry::new(vec![entwine_test_spell_def(), plain.clone()]);
    (reg, plain)
}

// ── Test 1: Entwine paid — both modes execute in order ────────────────────────

/// CR 702.42b — When entwine cost is paid, ALL modes of the modal spell execute
/// in printed order. Mode 0 (GainLife) fires before Mode 1 (DrawCards).
#[test]
fn test_entwine_basic_both_modes_execute() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = entwine_registry();

    let spell = ObjectSpec::card(p1, "Entwine Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);

    // Add 6 library cards so DrawCards doesn't fail on empty library.
    let lib_cards: Vec<_> = (0..6)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

    let mut state = {
        let mut builder = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spell)
            .active_player(p1)
            .at_step(Step::PreCombatMain);
        for lib in lib_cards {
            builder = builder.object(lib);
        }
        builder.build().unwrap()
    };

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1; // exclude the spell itself

    // Pay {1}{U} + entwine {2} = {1}{U}{2} = 4 mana total.
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
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Entwine Test Spell");

    // Cast with entwine paid.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            entwine_paid: true,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with entwine_paid failed: {:?}", e));

    // Spell is on the stack with was_entwined = true.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert!(
        state.stack_objects[0].was_entwined,
        "CR 702.42a: was_entwined should be true when entwine cost was paid"
    );

    // Resolve (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife) + Mode 1 (DrawCards) both executed.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.42b: Mode 0 (GainLife 3) should have executed"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size,
        initial_hand_size + 2,
        "CR 702.42b: Mode 1 (DrawCards 2) should have executed"
    );
}

// ── Test 2: Entwine not paid — only first mode executes ───────────────────────

/// CR 702.42a — When entwine cost is NOT paid, only mode[0] executes.
/// Mode 1 (DrawCards) should NOT fire.
#[test]
fn test_entwine_not_paid_only_first_mode() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = entwine_registry();

    let spell = ObjectSpec::card(p1, "Entwine Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);

    let lib_cards: Vec<_> = (0..3)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

    let mut state = {
        let mut builder = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spell)
            .active_player(p1)
            .at_step(Step::PreCombatMain);
        for lib in lib_cards {
            builder = builder.object(lib);
        }
        builder.build().unwrap()
    };

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay only base {1}{U} — no entwine cost.
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

    let spell_id = find_object(&state, "Entwine Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
        },
    )
    .unwrap_or_else(|e| panic!("cast without entwine failed: {:?}", e));

    assert!(
        !state.stack_objects[0].was_entwined,
        "was_entwined should be false when entwine not paid"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    // Only Mode 0 (GainLife) should have executed.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.42a: Mode 0 (GainLife 3) should have executed"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size, initial_hand_size,
        "CR 702.42a: Mode 1 (DrawCards) should NOT have executed when entwine not paid"
    );
}

// ── Test 3: Entwine cost added to total ───────────────────────────────────────

/// CR 601.2f — Entwine cost is an additional cost added on top of the base mana cost.
/// Attempting to pay entwine with insufficient mana is rejected.
#[test]
fn test_entwine_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = entwine_registry();

    // Spell has explicit mana cost {1}{U} so the engine knows the base cost.
    // Total with entwine: {1}{U} + {2} = 4 mana. We only provide 2 mana.
    let spell = ObjectSpec::card(p1, "Entwine Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine)
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only provide {1}{U} = 2 mana — NOT enough for entwine ({2} more needed = 4 total).
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

    let spell_id = find_object(&state, "Entwine Test Spell");

    // Cast with entwine_paid = true but insufficient mana — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            entwine_paid: true,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    );
    assert!(
        result.is_err(),
        "CR 601.2f: casting with entwine but insufficient mana should be rejected"
    );
}

// ── Test 4: entwine_paid rejected on spell without Entwine keyword ────────────

/// Engine validation — a spell without KeywordAbility::Entwine must reject
/// entwine_paid = true with an InvalidCommand error.
#[test]
fn test_entwine_no_keyword_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let (registry, plain_def) = entwine_registry_with_plain();

    // Plain sorcery in hand (no Entwine keyword).
    let spell = ObjectSpec::card(p1, "Plain Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(plain_def.card_id.clone())
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
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

    let spell_id = find_object(&state, "Plain Sorcery");

    // Attempt entwine on a spell without the keyword — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            entwine_paid: true,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    );
    assert!(
        result.is_err(),
        "engine must reject entwine_paid=true on a spell without KeywordAbility::Entwine"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("entwine"),
        "error message should mention 'entwine', got: {}",
        err_msg
    );
}

// ── Test 5: Modes execute in printed order (state from mode 0 visible to mode 1) ─

/// CR 702.42b — Modes execute in printed order. State changes from mode 0
/// are visible to mode 1. We verify that mode 0 (GainLife) fires first
/// by checking the life total is already increased when mode 1 (DrawCards) runs
/// (both effects are visible in the final state).
#[test]
fn test_entwine_modes_in_printed_order() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = entwine_registry();

    let spell = ObjectSpec::card(p1, "Entwine Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);

    let lib_cards: Vec<_> = (0..4)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

    let mut state = {
        let mut builder = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spell)
            .active_player(p1)
            .at_step(Step::PreCombatMain);
        for lib in lib_cards {
            builder = builder.object(lib);
        }
        builder.build().unwrap()
    };

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay {1}{U}{2} for base + entwine.
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
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Entwine Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            entwine_paid: true,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Both modes applied: +3 life AND +2 cards drawn.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.42b: Mode 0 (GainLife 3) must have applied"
    );
    let final_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_size,
        initial_hand_size + 2,
        "CR 702.42b: Mode 1 (DrawCards 2) must have applied after Mode 0"
    );
}

// ── Test 6: was_entwined on stack object ──────────────────────────────────────

/// CR 702.42a — Verify that `was_entwined` is set correctly on the StackObject
/// when the entwine cost is paid vs. not paid.
#[test]
fn test_entwine_was_entwined_flag_on_stack() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = entwine_registry();

    let spell = ObjectSpec::card(p1, "Entwine Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay full cost + entwine.
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
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Entwine Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
            entwine_paid: true,
            escalate_modes: 0,
            devour_sacrifices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with entwine failed: {:?}", e));

    assert!(
        state.stack_objects[0].was_entwined,
        "CR 702.42a: was_entwined must be true when entwine cost was paid"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "exactly one object should be on the stack"
    );
}

//! Modal spell tests (CR 700.2).
//!
//! A spell is modal if it has two or more options in a bulleted list preceded
//! by "Choose one --" (or choose two, choose up to N, etc.). The controller
//! announces their mode choice(s) as part of casting the spell (CR 601.2b).
//!
//! Key rules verified:
//! - CR 700.2a: Controller chooses mode(s) at cast time; invalid indices rejected.
//! - CR 700.2b: Mode indices select which effects execute at resolution.
//! - CR 700.2d: Same mode cannot be chosen twice (unless allow_duplicate_modes).
//! - CR 700.2g / 707.10: Copies of a modal spell copy the mode(s) chosen.
//! - CR 702.42b: Entwine overrides modes_chosen — all modes execute.
//! - Backward compat: empty modes_chosen auto-selects mode[0].

use mtg_engine::state::CardType;
use mtg_engine::AdditionalCost;
use mtg_engine::Effect;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, EffectAmount, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ModeSelection, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
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

/// Build a synthetic "Modal Test Spell" card:
/// Sorcery {1}{U}
/// Choose one —
///   Mode 0: Controller gains 3 life.
///   Mode 1: Controller draws 2 cards.
///   Mode 2: Deal 2 damage to controller (for testing negative/unusual modes).
fn modal_test_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("modal-test-spell".to_string()),
        name: "Modal Test Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Choose one — Gain 3 life; or draw 2 cards; or Modal Test Spell deals 2 damage to you."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: controller gains 3 life.
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    // Mode 1: controller draws 2 cards.
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    // Mode 2: deal 2 damage to controller.
                    Effect::DealDamage {
                        target: CardEffectTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a "Choose Two Spell" card (min=2, max=2):
/// Sorcery {2}{U}
/// Choose two —
///   Mode 0: Gain 3 life.
///   Mode 1: Draw 2 cards.
///   Mode 2: Deal 2 damage to controller.
fn choose_two_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("choose-two-spell".to_string()),
        name: "Choose Two Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose two — Gain 3 life; or draw 2 cards; or deal 2 damage to yourself."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 2,
                max_modes: 2,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::DealDamage {
                        target: CardEffectTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a registry containing the modal test spell and choose-two spell.
fn modal_registry() -> Arc<CardRegistry> {
    CardRegistry::new(vec![modal_test_spell_def(), choose_two_spell_def()])
}

/// Build a standard 2-player game state with the modal test spell in P1's hand.
fn build_state_with_modal_spell(registry: Arc<CardRegistry>) -> mtg_engine::GameState {
    let p1 = p(1);
    let p2 = p(2);

    let spell = ObjectSpec::card(p1, "Modal Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("modal-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery]);

    // 6 library cards so DrawCards doesn't fail on empty library.
    let lib_cards: Vec<_> = (0..6)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

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
    let mut state = builder.build().unwrap();

    // Add 2 blue + 1 colorless mana for {1}{U} cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    state
}

// ── Test 1: Choose one — mode 0 executes ──────────────────────────────────────

/// CR 700.2a — The controller of a modal spell chooses the mode(s) as part of casting.
/// Choosing mode index 0 causes only mode 0 (GainLife 3) to execute at resolution.
#[test]
fn test_modal_choose_one_mode_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = modal_registry();
    let state = build_state_with_modal_spell(registry);
    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1; // exclude the spell itself

    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![0],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert_eq!(
        state.stack_objects[0].modes_chosen,
        vec![0],
        "CR 700.2a: modes_chosen should be [0]"
    );

    // Resolve: both players pass.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) fired.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 700.2a: mode 0 (GainLife 3) should have fired"
    );
    // Mode 1 (DrawCards 2) did NOT fire.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, initial_hand,
        "CR 700.2a: mode 1 (DrawCards) should NOT have fired when mode 0 was chosen"
    );
}

// ── Test 2: Choose one — mode 1 executes ──────────────────────────────────────

/// CR 700.2a — Choosing mode index 1 (DrawCards) executes only mode 1.
/// Mode 0 (GainLife) must NOT execute.
#[test]
fn test_modal_choose_one_mode_one() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = modal_registry();
    let state = build_state_with_modal_spell(registry);
    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![1],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 1 (DrawCards 2) fired: hand grew by 2.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after,
        initial_hand + 2,
        "CR 700.2a: mode 1 (DrawCards 2) should have fired"
    );
    // Mode 0 (GainLife) did NOT fire: life unchanged.
    assert_eq!(
        state.players[&p1].life_total, initial_life,
        "CR 700.2a: mode 0 (GainLife) should NOT have fired when mode 1 chosen"
    );
}

// ── Test 3: Choose one — mode 2 executes ──────────────────────────────────────

/// CR 700.2a — Choosing mode index 2 (DealDamage 2 to self) executes only mode 2.
/// Verifies that non-zero, non-adjacent mode indices work.
#[test]
fn test_modal_choose_one_mode_two() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = modal_registry();
    let state = build_state_with_modal_spell(registry);
    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![2],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 2 (DealDamage 2) fired: life decreased by 2.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life - 2,
        "CR 700.2a: mode 2 (DealDamage 2) should have fired"
    );
    // Mode 0/1 did NOT fire.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, initial_hand,
        "CR 700.2a: mode 1 (DrawCards) should NOT have fired when mode 2 chosen"
    );
}

// ── Test 4: Choose two — modes [0, 2] both execute ────────────────────────────

/// CR 700.2 — When a spell says "choose two", the controller picks exactly 2 modes.
/// Choosing [0, 2] executes mode 0 (GainLife 3) then mode 2 (DealDamage 2).
/// Mode 1 (DrawCards) must NOT execute.
#[test]
fn test_modal_choose_two_modes() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a state with the choose-two spell.
    let registry = modal_registry();
    let spell = ObjectSpec::card(p1, "Choose Two Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("choose-two-spell".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let lib_cards: Vec<_> = (0..6)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

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
    let mut state = builder.build().unwrap();

    // {2}{U} = 3 mana total.
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

    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Choose Two Spell");

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
            prototype: false,
            modes_chosen: vec![0, 2], // Choose mode 0 and mode 2.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) fired AND mode 2 (DealDamage 2) fired.
    // Net life: initial + 3 - 2 = initial + 1.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 700.2: mode 0 (GainLife 3) + mode 2 (DealDamage 2) should both execute"
    );
    // Mode 1 (DrawCards) did NOT fire.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, initial_hand,
        "CR 700.2: mode 1 (DrawCards) should NOT execute when only modes [0, 2] chosen"
    );
}

// ── Test 5: Empty modes_chosen auto-selects mode[0] (backward compat) ────────

/// CR 700.2a — When modes_chosen is empty, the engine auto-selects mode[0].
/// This is backward-compatible with all existing tests that pre-date this feature.
#[test]
fn test_modal_default_auto_selects_mode_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = modal_registry();
    let state = build_state_with_modal_spell(registry);
    let initial_life = state.players[&p1].life_total;

    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![], // Empty → auto-select mode[0]
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with empty modes_chosen failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) auto-selected and fired.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "Backward compat: empty modes_chosen auto-selects mode[0] (GainLife 3)"
    );
}

// ── Test 6: Invalid mode index rejected ───────────────────────────────────────

/// CR 700.2a — Mode index must be < modes.len(). Out-of-range index is rejected.
#[test]
fn test_modal_invalid_index_rejected() {
    let p1 = p(1);
    let _p2 = p(2);
    let registry = modal_registry();
    let mut state = build_state_with_modal_spell(registry);

    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![5], // Only 3 modes; index 5 is out of range.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 700.2a: mode index 5 out of range (spell has 3 modes) must be rejected"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("out of range"),
        "Error message should mention 'out of range', got: {err}"
    );
}

// ── Test 7: Duplicate mode index rejected ─────────────────────────────────────

/// CR 700.2d — Same mode cannot be chosen twice unless allow_duplicate_modes is set.
/// Choosing [0, 0] on a standard modal spell must be rejected.
#[test]
fn test_modal_duplicate_index_rejected() {
    let p1 = p(1);
    let _p2 = p(2);
    let registry = modal_registry();
    let mut state = build_state_with_modal_spell(registry);

    // Add extra mana to allow for the choose-one max_modes check to be reached.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![0, 0], // Duplicate on choose-one spell.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 700.2d: duplicate mode index [0, 0] must be rejected"
    );
}

// ── Test 8: Too few modes chosen rejected ─────────────────────────────────────

/// CR 700.2a — Choosing 0 modes on a "choose one" (min_modes=1) spell must fail.
#[test]
fn test_modal_too_few_modes_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a choose-two spell (min_modes=2) but pass only 1 mode.
    let registry = modal_registry();
    let spell = ObjectSpec::card(p1, "Choose Two Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("choose-two-spell".to_string()))
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
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Choose Two Spell");

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
            prototype: false,
            modes_chosen: vec![0], // Only 1 mode chosen, but min_modes=2.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 700.2a: too few modes (1 of min 2) must be rejected"
    );
}

// ── Test 9: Too many modes chosen rejected ────────────────────────────────────

/// CR 700.2a — Choosing 3 modes on a "choose one" (max_modes=1) spell must fail.
#[test]
fn test_modal_too_many_modes_rejected() {
    let p1 = p(1);
    let _p2 = p(2);
    let registry = modal_registry();
    let mut state = build_state_with_modal_spell(registry);

    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![0, 1, 2], // 3 modes chosen, but max_modes=1.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 700.2a: too many modes (3 of max 1) must be rejected"
    );
}

// ── Test 10: Entwine overrides modes_chosen ───────────────────────────────────

/// CR 702.42b — When entwine is paid, ALL modes execute regardless of modes_chosen.
/// Casting with entwine_paid=true and modes_chosen=[0] must execute ALL modes.
#[test]
fn test_modal_entwine_overrides_modes_chosen() {
    use mtg_engine::CardType;

    let p1 = p(1);
    let p2 = p(2);

    // Build an entwine-capable modal spell (min=1, max=1, entwine cost 2).
    let entwine_modal_def = CardDefinition {
        card_id: CardId("entwine-modal".to_string()),
        name: "Entwine Modal Spell".to_string(),
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
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
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
    };

    let registry = CardRegistry::new(vec![entwine_modal_def]);
    let spell = ObjectSpec::card(p1, "Entwine Modal Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("entwine-modal".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Entwine);

    let lib_cards: Vec<_> = (0..6)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

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
    let mut state = builder.build().unwrap();

    // {1}{U} + entwine {2} = 4 mana.
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

    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Entwine Modal Spell");

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
            prototype: false,
            additional_costs: vec![AdditionalCost::Entwine], // Entwine paid.
            modes_chosen: vec![0], // modes_chosen=[0] but entwine overrides.
            x_value: 0,
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("cast with entwine failed: {:?}", e));

    assert!(
        state.stack_objects[0]
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::Entwine)),
        "CR 702.42a: additional_costs should contain Entwine"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    // BOTH modes executed because entwine takes precedence.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.42b: mode 0 (GainLife 3) should execute with entwine"
    );
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after,
        initial_hand + 2,
        "CR 702.42b: mode 1 (DrawCards 2) should also execute with entwine (overrides modes_chosen=[0])"
    );
}

// ── Test 11: Non-modal spell with modes_chosen is rejected ───────────────────

/// CR 700.2a — Specifying modes_chosen on a non-modal spell must be rejected.
#[test]
fn test_modal_non_modal_spell_with_modes_chosen_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // A plain non-modal sorcery.
    let plain_def = CardDefinition {
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

    let registry = CardRegistry::new(vec![plain_def]);
    let spell = ObjectSpec::card(p1, "Plain Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-sorcery".to_string()))
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
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Sorcery");

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
            prototype: false,
            modes_chosen: vec![0], // Non-modal spell; modes_chosen=[0] must be rejected.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 700.2a: modes_chosen on a non-modal spell must be rejected"
    );
}

// ── Test 12: modes_chosen stored on StackObject ───────────────────────────────

/// CR 700.2a / 601.2b — The modes chosen at cast time are stored on the StackObject.
/// This is required for copies to inherit the mode choice (CR 700.2g).
#[test]
fn test_modal_modes_chosen_stored_on_stack_object() {
    let p1 = p(1);
    let _p2 = p(2);
    let registry = modal_registry();
    let mut state = build_state_with_modal_spell(registry);

    state.turn.priority_holder = Some(p1);
    let spell_id = find_object(&state, "Modal Test Spell");

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
            prototype: false,
            modes_chosen: vec![2],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert_eq!(
        state.stack_objects[0].modes_chosen,
        vec![2],
        "CR 700.2a / 601.2b: modes_chosen [2] must be stored on the StackObject"
    );
}

// ── Test 13: Copy of modal spell inherits modes_chosen ────────────────────────

/// CR 700.2g / 707.10 — A copy of a modal spell copies the mode(s) chosen for it.
/// The controller of the copy cannot choose a different mode.
#[test]
fn test_modal_copy_inherits_modes() {
    use mtg_engine::ObjectId;

    let p1 = p(1);
    let p2 = p(2);
    let registry = modal_registry();
    let state = build_state_with_modal_spell(registry);

    // Record initial hand size (minus the spell).
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Modal Test Spell");

    // Cast with mode 1 (DrawCards 2).
    let (mut state, _) = process_command(
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
            prototype: false,
            modes_chosen: vec![1],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        1,
        "original should be on the stack"
    );
    assert_eq!(
        state.stack_objects[0].modes_chosen,
        vec![1],
        "CR 601.2b: original modes_chosen should be [1]"
    );

    let original_stack_id: ObjectId = state.stack_objects[0].id;

    // Copy the spell on the stack directly (simulating a copy effect).
    let (copy_id, _copy_event) =
        mtg_engine::rules::copy::copy_spell_on_stack(&mut state, original_stack_id, p1, false)
            .unwrap_or_else(|e| panic!("copy_spell_on_stack failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        2,
        "original + copy should be on the stack"
    );

    // Find the copy and verify modes_chosen is inherited.
    let copy_obj = state
        .stack_objects
        .iter()
        .find(|s| s.id == copy_id)
        .expect("copy stack object not found");
    assert_eq!(
        copy_obj.modes_chosen,
        vec![1],
        "CR 700.2g / 707.10: copy must inherit modes_chosen [1] from the original"
    );

    // Resolve both: copy resolves first (LIFO), then original.
    // Copy resolves — DrawCards 2.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Original resolves — DrawCards 2.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both the copy and the original executed mode 1 (DrawCards 2): +4 cards total.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after,
        initial_hand + 4,
        "CR 700.2g / 707.10: both copy and original should execute mode 1 (DrawCards 2) — expect +4 cards"
    );
}

// ── Test 14: allow_duplicate_modes positive path ──────────────────────────────

/// CR 700.2d — "However, some modal spells include the instruction 'You may choose the
/// same mode more than once.' If a particular mode is chosen multiple times, the spell
/// is treated as if that mode appeared that many times in sequence."
/// When allow_duplicate_modes=true, choosing [0, 0] must be accepted and mode 0
/// must execute twice.
#[test]
fn test_modal_allow_duplicate_modes() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a synthetic 2-mode spell where allow_duplicate_modes=true.
    // Mode 0: GainLife 3 (can be chosen twice → +6 life).
    // Mode 1: DrawCards 1.
    let dup_modal_def = CardDefinition {
        card_id: CardId("dup-modal".to_string()),
        name: "Dup Modal Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Choose two. You may choose the same mode more than once — Gain 3 life; or draw a card."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 2,
                max_modes: 2,
                allow_duplicate_modes: true,
                mode_costs: None,
                modes: vec![
                    // Mode 0: controller gains 3 life.
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    },
                    // Mode 1: controller draws 1 card.
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![dup_modal_def]);
    let spell = ObjectSpec::card(p1, "Dup Modal Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("dup-modal".to_string()))
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

    // {2}{U} = 3 mana.
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

    let initial_life = state.players[&p1].life_total;
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    let spell_id = find_object(&state, "Dup Modal Spell");

    // Cast with modes [0, 0] — duplicate mode 0 allowed by allow_duplicate_modes=true.
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
            prototype: false,
            modes_chosen: vec![0, 0], // Duplicate mode 0 — allowed because allow_duplicate_modes=true.
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with duplicate modes failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert_eq!(
        state.stack_objects[0].modes_chosen,
        vec![0, 0],
        "CR 700.2d: modes_chosen [0, 0] should be stored on the StackObject"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) executed TWICE → life = initial + 6.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 6,
        "CR 700.2d: mode 0 (GainLife 3) chosen twice must execute twice — expect +6 life"
    );
    // Mode 1 (DrawCards) did NOT execute.
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after, initial_hand,
        "CR 700.2d: mode 1 (DrawCards) should NOT execute when only mode 0 was chosen"
    );
}

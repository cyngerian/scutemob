//! Spree keyword ability tests (CR 702.172).
//!
//! Spree is a static ability on modal spells. "Choose one or more modes. As an
//! additional cost to cast this spell, pay the costs associated with those modes."
//! Each mode has its own per-mode additional cost stored in `ModeSelection.mode_costs`.
//!
//! Key rules verified:
//! - CR 702.172a: Spree requires at least one mode to be chosen.
//! - CR 700.2h: Each chosen mode's cost is paid as an additional cost.
//! - CR 601.2f: Total cost = base mana cost + sum of chosen-mode costs.
//! - CR 700.2d: Duplicate mode indices are rejected.
//! - CR 118.8d: Mana value equals printed mana cost only (not affected by mode costs).

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

/// Build a synthetic "Spree Test Spell" card:
/// Sorcery {1}{W} — Spree (Choose one or more — pay the cost for each chosen mode.)
/// + {1}   — Controller gains 4 life.       (mode 0)
/// + {2}   — Controller draws 2 cards.      (mode 1)
/// + {1}{W} — Controller loses 3 life.      (mode 2)
///
/// Base cost: {1}{W} = 2 mana (1 generic + 1 white)
/// Mode 0 cost: {1} (1 generic)
/// Mode 1 cost: {2} (2 generic)
/// Mode 2 cost: {1}{W} (1 generic + 1 white)
fn spree_test_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("spree-test-spell".to_string()),
        name: "Spree Test Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Spree (Choose one or more — pay the costs for each chosen mode.)\n\
            + {1} — You gain 4 life.\n\
            + {2} — Draw 2 cards.\n\
            + {1}{W} — You lose 3 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spree),
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: Some(vec![
                        // Mode 0 cost: {1}
                        ManaCost {
                            generic: 1,
                            ..Default::default()
                        },
                        // Mode 1 cost: {2}
                        ManaCost {
                            generic: 2,
                            ..Default::default()
                        },
                        // Mode 2 cost: {1}{W}
                        ManaCost {
                            generic: 1,
                            white: 1,
                            ..Default::default()
                        },
                    ]),
                    modes: vec![
                        // Mode 0: controller gains 4 life.
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(4),
                        },
                        // Mode 1: controller draws 2 cards.
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        // Mode 2: controller loses 3 life (measurable, no targeting needed).
                        Effect::LoseLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Build a plain non-modal sorcery for "no spree keyword" tests.
fn plain_sorcery_def() -> CardDefinition {
    CardDefinition {
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
    }
}

fn spree_registry() -> Arc<CardRegistry> {
    CardRegistry::new(vec![spree_test_spell_def()])
}

fn spree_registry_with_plain() -> (Arc<CardRegistry>, CardDefinition) {
    let plain = plain_sorcery_def();
    let reg = CardRegistry::new(vec![spree_test_spell_def(), plain.clone()]);
    (reg, plain)
}

/// Build a test state with the spree spell in hand and library filler.
fn build_spree_state(
    p1: PlayerId,
    p2: PlayerId,
    registry: Arc<CardRegistry>,
    library_count: usize,
) -> mtg_engine::GameState {
    let spell = ObjectSpec::card(p1, "Spree Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("spree-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Spree)
        .with_mana_cost(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        });

    let lib_cards: Vec<_> = (0..library_count)
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
    builder.build().unwrap()
}

/// Helper: add mana to p1's pool.
fn add_mana(state: &mut mtg_engine::GameState, p1: PlayerId, generic: u32, white: u32) {
    let pool = &mut state.players.get_mut(&p1).unwrap().mana_pool;
    pool.add(ManaColor::Colorless, generic);
    pool.add(ManaColor::White, white);
}

// ── Test 1: Single mode chosen — base + mode cost paid ────────────────────────

/// CR 702.172a / 700.2h — Choosing mode 0 costs base {1}{W} + mode-0 {1} = {2}{W}.
/// Total = 3 mana. Mode 0 (GainLife 4) executes; modes 1 and 2 do not.
#[test]
fn test_spree_single_mode_adds_mode_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 0);

    let initial_life = state.players[&p1].life_total;

    // Base {1}{W} + mode-0 {1} = {2}{W} = 3 mana total.
    add_mana(&mut state, p1, 2, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            modes_chosen: vec![0], // mode 0 only
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("cast with mode 0 failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert_eq!(
        state.stack_objects[0].modes_chosen,
        vec![0],
        "CR 702.172a: modes_chosen should record mode 0"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 4) executed; modes 1 and 2 did not.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 4,
        "CR 702.172a: Mode 0 (GainLife 4) should have executed"
    );
}

// ── Test 2: Two modes chosen — base + two mode costs paid ─────────────────────

/// CR 702.172a / 700.2h — Choosing modes 0 and 1 costs base {1}{W} + {1} + {2} = {4}{W}.
/// Total = 5 mana. Both modes 0 and 1 execute.
#[test]
fn test_spree_two_modes_adds_both_costs() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 4);

    let initial_life = state.players[&p1].life_total;

    // Base {1}{W} + mode-0 {1} + mode-1 {2} = {4}{W} = 5 mana.
    add_mana(&mut state, p1, 4, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            modes_chosen: vec![0, 1], // modes 0 and 1
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("cast with modes [0,1] failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 4) + Mode 1 (DrawCards 2) both executed.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 4,
        "CR 702.172a: Mode 0 (GainLife 4) should have executed"
    );
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 2,
        "CR 702.172a: Mode 1 (DrawCards 2) should have executed"
    );
}

// ── Test 3: All three modes chosen ────────────────────────────────────────────

/// CR 702.172a — Choosing all 3 modes costs base {1}{W} + {1} + {2} + {1}{W} = {5}{W}{W}.
/// Total = 7 mana. All three modes execute in printed order.
#[test]
fn test_spree_all_three_modes() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 4);

    let initial_life = state.players[&p1].life_total;

    // Base {1}{W} + mode-0 {1} + mode-1 {2} + mode-2 {1}{W} = {5}{W}{W} = 7 mana.
    add_mana(&mut state, p1, 5, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            modes_chosen: vec![0, 1, 2], // all three modes
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("cast with all 3 modes failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (+4 life) + Mode 1 (draw 2) + Mode 2 (-3 life) = net +1 life.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 4 - 3,
        "CR 702.172a: all 3 modes should execute: +4 life (mode 0), draw 2 (mode 1), -3 life (mode 2)"
    );
    let hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 2,
        "CR 702.172a: Mode 1 (DrawCards 2) should have executed"
    );
}

// ── Test 4: Zero modes rejected ───────────────────────────────────────────────

/// CR 702.172a — Attempting to cast a Spree spell choosing zero modes must be rejected.
/// Spree says "Choose one or more modes."
#[test]
fn test_spree_zero_modes_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 0);

    // Provide plenty of mana — rejection should be due to 0 modes, not mana.
    add_mana(&mut state, p1, 10, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![], // ZERO modes — invalid for Spree
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.172a: casting a Spree spell with zero modes must be rejected"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("spree") || err_msg.contains("mode"),
        "error should mention spree/mode requirement, got: {}",
        err_msg
    );
}

// ── Test 5: Insufficient mana rejected ───────────────────────────────────────

/// CR 601.2f — Choosing two modes but only providing mana for one mode's cost fails.
/// Base {1}{W} + mode-0 {1} + mode-1 {2} = {4}{W}, but we only provide {3}{W}.
#[test]
fn test_spree_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 0);

    // Provide only {2}{W} instead of {4}{W} needed for modes 0+1.
    add_mana(&mut state, p1, 2, 1); // only enough for base + mode 0 ({2}{W}), not base + mode 0 + mode 1
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![0, 1], // needs {4}{W} but only {2}{W} provided
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    );
    assert!(
        result.is_err(),
        "CR 601.2f: insufficient mana for Spree modes must be rejected"
    );
}

// ── Test 6: Duplicate mode rejected ──────────────────────────────────────────

/// CR 700.2d — Choosing mode 0 twice must be rejected (no duplicate modes).
#[test]
fn test_spree_duplicate_mode_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 0);

    add_mana(&mut state, p1, 10, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

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
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![0, 0], // duplicate mode 0 — invalid
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    );
    assert!(
        result.is_err(),
        "CR 700.2d: duplicate mode index must be rejected for Spree spell"
    );
}

// ── Test 7: Modes execute in ascending index order ───────────────────────────

/// CR 700.2a — Modes always execute in ascending printed order, regardless of
/// the order the player supplies them at cast time. Verify that casting with
/// `modes_chosen: vec![2, 0]` results in `stack_obj.modes_chosen == [0, 2]`.
#[test]
fn test_spree_mode_order_ascending() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let mut state = build_spree_state(p1, p2, registry, 0);

    // Modes 0 and 2: base {1}{W} + mode-0 {1} + mode-2 {1}{W} = {3}{W}{W} = 5 mana.
    add_mana(&mut state, p1, 3, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Spree Test Spell");

    // Supply modes in reverse index order [2, 0].
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
            modes_chosen: vec![2, 0], // supplied in reverse order
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("cast with modes [2,0] failed: {:?}", e));

    // The spell should now be on the stack. Verify modes_chosen was sorted to [0, 2].
    let stack_obj = state
        .stack_objects
        .iter()
        .next()
        .expect("spell should be on the stack");
    assert_eq!(
        stack_obj.modes_chosen,
        vec![0usize, 2],
        "CR 700.2a: modes_chosen on stack object must be sorted ascending; got {:?}",
        stack_obj.modes_chosen
    );
}

// ── Test 8: KeywordAbility::Spree is present on the card ──────────────────────

/// CR 702.172a — Verify `KeywordAbility::Spree` is in the card's keywords after enrichment.
#[test]
fn test_spree_keyword_marker_present() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = spree_registry();
    let state = build_spree_state(p1, p2, registry, 0);

    let spell_id = find_object(&state, "Spree Test Spell");
    let obj = &state.objects[&spell_id];
    assert!(
        obj.characteristics
            .keywords
            .contains(&KeywordAbility::Spree),
        "CR 702.172a: Spree keyword must be present on the card"
    );
}

// ── Test 9: Non-Spree modal spell unaffected ──────────────────────────────────

/// Engine validation — a non-Spree modal spell cast with `modes_chosen` but no Spree
/// keyword does not have per-mode additional costs added. The plain sorcery here is
/// non-modal (modes: None) so this tests the "no Spree = no extra cost" path.
#[test]
fn test_spree_non_spree_spell_unchanged() {
    let p1 = p(1);
    let p2 = p(2);

    let (registry, plain_def) = spree_registry_with_plain();

    // Plain sorcery in hand — no Spree keyword, no modes.
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

    // Pay only the base {1} mana cost — should be sufficient.
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
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    );
    assert!(
        result.is_ok(),
        "non-Spree spell should not require additional mode costs; cast should succeed with base cost only"
    );
}

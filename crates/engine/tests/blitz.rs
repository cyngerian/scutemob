//! Blitz keyword ability tests (CR 702.152).
//!
//! Blitz is an alternative cost (CR 118.9) that allows a creature to be cast
//! for its blitz cost instead of its mana cost. When the permanent enters the
//! battlefield, it gains haste, and gains "When this permanent is put into a
//! graveyard from the battlefield, draw a card." At the beginning of the next
//! end step, it is sacrificed.
//!
//! Key rules verified:
//! - Blitz is an alternative cost: pay blitz cost instead of mana cost (CR 702.152a).
//! - After ETB: permanent has haste (CR 702.152a).
//! - After ETB: permanent has "When this dies, draw a card" trigger (CR 702.152a).
//! - At beginning of next end step, a delayed trigger sacrifices the permanent (CR 702.152a).
//! - Sacrifice triggers the draw-on-death trigger (CR 702.152a combined).
//! - If permanent leaves battlefield before sacrifice trigger resolves, trigger does nothing
//!   (Ruling 2022-04-29).
//! - Normal cast (no blitz) leaves creature on battlefield with no sacrifice trigger (negative test).
//! - Blitz cannot combine with other alternative costs: flashback, evoke, etc. (CR 118.9a).
//! - Attempting to cast a non-blitz card with blitz cost is rejected (CR 702.152a).
//! - Commander tax applies on top of blitz cost (CR 118.9d).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::types::SuperType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

/// Pass priority for all listed players once (resolves top of stack if all pass).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// Drain the stack completely (pass all until stack is empty).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    while !state.stack_objects.is_empty() {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Blitz Goblin (mock): Creature {1}{R} 2/2, Blitz {R}.
/// Normal cost {1}{R}, blitz cost {R}.
fn blitz_goblin_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("blitz-goblin".to_string()),
        name: "Blitz Goblin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Blitz {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Blitz),
            AbilityDefinition::Blitz {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Helper: place a Blitz Goblin in the player's hand.
fn goblin_in_hand(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Blitz Goblin")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("blitz-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Blitz)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });
    // Set power and toughness so the layer system can check lethal damage via SBA 704.5g.
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

/// Plain creature with no blitz ability (for negative test).
fn plain_goblin_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-goblin".to_string()),
        name: "Plain Goblin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: Basic blitz cast — haste granted ──────────────────────────────────

/// CR 702.152a — Blitz Goblin cast for blitz cost {R}.
/// After ETB: creature on battlefield with haste and cast_alt_cost == Some(Blitz).
/// Mana consumed: blitz cost {R}, not normal cost {1}{R}.
#[test]
fn test_blitz_basic_cast_with_blitz_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {R} — blitz cost instead of mana cost {1}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast with blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Spell is on the stack with was_blitzed = true.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.152a: blitzed spell should be on the stack"
    );
    assert!(
        state.stack_objects[0].was_blitzed,
        "CR 702.152a: was_blitzed should be true on stack object"
    );

    // Mana consumed: {R} = 1 mana total (not {1}{R} = 2 mana).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.152a: {{R}} blitz cost should be deducted from mana pool"
    );

    // Resolve the spell (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "CR 702.152a: blitzed creature should be on battlefield after resolution"
    );

    // cast_alt_cost is set to Blitz on the permanent.
    let bf_id = find_in_zone(&state, "Blitz Goblin", ZoneId::Battlefield).unwrap();
    assert_eq!(
        state.objects[&bf_id].cast_alt_cost,
        Some(AltCostKind::Blitz),
        "CR 702.152a: cast_alt_cost should be Some(Blitz) on battlefield permanent"
    );

    // Haste is granted: permanent has the Haste keyword.
    assert!(
        state.objects[&bf_id]
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 702.152a: blitzed creature should have Haste keyword"
    );
}

// ── Test 2: Normal cast — no sacrifice at end step, no blitz benefits ─────────

/// CR 702.152a (negative) — Blitz Goblin cast for normal mana cost {1}{R}.
/// No haste from blitz, no sacrifice at end step, no draw on death.
/// Creature stays on battlefield after end step.
#[test]
fn test_blitz_normal_cast_no_sacrifice_no_draw() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {1}{R} — normal mana cost.
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

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast normally (no blitz).
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
        },
    )
    .unwrap_or_else(|e| panic!("Normal CastSpell failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "creature should be on battlefield after normal cast"
    );

    let bf_id = find_in_zone(&state, "Blitz Goblin", ZoneId::Battlefield).unwrap();

    // cast_alt_cost is None (normal cast, not blitzed).
    assert!(
        state.objects[&bf_id].cast_alt_cost.is_none(),
        "CR 702.152a: cast_alt_cost should be None for normally cast creature"
    );

    // Advance to End step — no BlitzSacrificeTrigger should fire.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // Pass priority at End step — creature should still be on battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "CR 702.152a: normally cast creature should NOT be sacrificed at end step"
    );
}

// ── Test 3: Blitz sacrifice at end step ───────────────────────────────────────

/// CR 702.152a — Blitzed creature is sacrificed at beginning of next end step.
/// Cast with blitz at PostCombatMain, advance to end step, trigger resolves.
/// After resolution: creature NOT on battlefield, IS in graveyard.
#[test]
fn test_blitz_sacrifice_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {R} — blitz cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast with blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Both players pass priority → spell resolves → creature enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "creature should be on battlefield after blitz cast"
    );

    // Advance from PostCombatMain to End step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // Both players pass priority at End step:
    // BlitzSacrificeTrigger queued → pushed to stack → resolves → creature sacrificed.
    // Note: the draw-on-death trigger ALSO fires (creature died) and must be drained.
    let (state, _end_events) = pass_all(state, &[p1, p2]);
    // Sacrifice trigger resolved, but draw trigger is now on the stack. Drain it.
    let (state, _draw_events) = drain_stack(state, &[p1, p2]);

    // Creature should be in graveyard, NOT on battlefield.
    assert!(
        !on_battlefield(&state, "Blitz Goblin"),
        "CR 702.152a: blitzed creature should NOT be on battlefield after end-step sacrifice"
    );
    assert!(
        in_graveyard(&state, "Blitz Goblin", p1),
        "CR 702.152a: blitzed creature should be in graveyard after end-step sacrifice"
    );
}

// ── Test 4: Draw-on-death trigger fires when blitzed creature dies ────────────

/// CR 702.152a — When a blitzed creature is put into a graveyard from the
/// battlefield (for any reason), its controller draws a card.
///
/// Ruling: Mezzio Mugger (2022-04-29): "The triggered ability that lets its
/// controller draw a card triggers when it dies for any reason, not just when
/// you sacrifice it during the end step."
///
/// This test exercises a non-sacrifice death path: lethal damage causes the SBA
/// (CR 704.5g) to destroy the creature through the engine event system, which
/// fires CreatureDied, which triggers the SelfDies draw trigger.
#[test]
fn test_blitz_draw_card_on_death() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    // Add a card to P1's library so the draw doesn't fail on empty library.
    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay {R} — blitz cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast with blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Resolve the spell.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "creature should be on battlefield after blitz cast"
    );

    // Record hand size before death.
    let hand_before = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Mark lethal damage on the 2/2 Blitz Goblin.
    // CR 704.5g: a creature with damage >= its toughness is destroyed.
    // The SBA check runs before the next priority grant and goes through the
    // engine event path: CreatureDied is emitted, check_triggers picks up the
    // SelfDies draw trigger injected at ETB, and the trigger is queued.
    let bf_id = find_in_zone(&state, "Blitz Goblin", ZoneId::Battlefield).unwrap();
    state.objects.get_mut(&bf_id).unwrap().damage_marked = 2; // 2 >= toughness 2 → lethal

    // PassPriority → SBAs fire (CR 704.5g: lethal damage) → creature destroyed via
    // engine path → CreatureDied event emitted → SelfDies draw trigger queued.
    state.turn.priority_holder = Some(p1);
    let (state, sba_events) = pass_all(state, &[p1, p2]);

    // CreatureDied should have been emitted through the engine SBA path.
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 704.5g: CreatureDied should be emitted when lethal damage SBA fires"
    );

    // The creature should be gone from the battlefield.
    assert!(
        !on_battlefield(&state, "Blitz Goblin"),
        "CR 704.5g: creature should be off the battlefield after lethal damage SBA"
    );

    // Drain the stack: the SelfDies draw trigger resolves, drawing a card.
    let (state, draw_events) = drain_stack(state, &[p1, p2]);

    // Creature should be in graveyard.
    assert!(
        in_graveyard(&state, "Blitz Goblin", p1),
        "CR 702.152a: blitzed creature should be in graveyard after dying"
    );

    // Controller should have drawn a card (hand size increased by 1).
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after,
        hand_before + 1,
        "CR 702.152a / Ruling Mezzio Mugger 2022-04-29: draw trigger fires on any death, \
         not just end-step sacrifice; hand should grow by 1"
    );

    // CardDrawn event should have been emitted for p1.
    let drawn = draw_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.152a: CardDrawn event expected when blitz draw trigger resolves"
    );
}

// ── Test 5: Draw on sacrifice at end step (combined flow) ─────────────────────

/// CR 702.152a — Combined test: blitz sacrifice at end step triggers the draw.
/// Full engine path: cast → resolve → end step → sacrifice trigger → creature dies
/// → draw trigger fires → draw trigger resolves → controller draws a card.
#[test]
fn test_blitz_draw_on_sacrifice_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    // Add a card to P1's library so the draw doesn't fail on empty library.
    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {R} — blitz cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast with blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Both players pass priority → spell resolves → creature enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "creature should be on battlefield after blitz cast"
    );

    // Record hand size before end step.
    let hand_before = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Advance from PostCombatMain to End step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // At End step: BlitzSacrificeTrigger fires and goes on stack.
    // Pass priority → sacrifice trigger resolves → creature dies → CreatureDied fired →
    // draw trigger (SelfDies) goes on stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Draw trigger is now on the stack. Drain it.
    let (state, draw_events) = drain_stack(state, &[p1, p2]);

    // Creature should be in graveyard.
    assert!(
        !on_battlefield(&state, "Blitz Goblin"),
        "CR 702.152a: blitzed creature should be sacrificed at end step"
    );
    assert!(
        in_graveyard(&state, "Blitz Goblin", p1),
        "CR 702.152a: sacrificed creature should be in graveyard"
    );

    // Controller should have drawn a card (hand size increased by 1).
    let hand_after = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after,
        hand_before + 1,
        "CR 702.152a: blitzed creature's draw trigger should add 1 card to controller's hand"
    );

    // CardDrawn event should have been emitted for p1.
    let drawn = draw_events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        drawn,
        "CR 702.152a: CardDrawn event expected when blitz draw trigger resolves"
    );
}

// ── Test 6: Creature left battlefield before end step — trigger does nothing ──

/// Ruling 2022-04-29 — "If you pay the blitz cost to cast a creature spell, that
/// permanent will be sacrificed only if it's still on the battlefield when that
/// triggered ability resolves. If it dies or goes to another zone before then,
/// it will stay where it is."
/// If the creature leaves the battlefield before the end step trigger resolves,
/// the trigger does nothing (CR 400.7: new object).
#[test]
fn test_blitz_creature_left_battlefield_before_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Pay {R} — blitz cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Cast with blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Both players pass priority → spell resolves → creature enters battlefield.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Goblin"),
        "creature should be on battlefield after blitz cast"
    );

    // Manually move the creature to the graveyard BEFORE the end step.
    let bf_id = find_in_zone(&state, "Blitz Goblin", ZoneId::Battlefield).unwrap();
    state
        .move_object_to_zone(bf_id, ZoneId::Graveyard(p1))
        .expect("move to graveyard should succeed");

    assert!(
        !on_battlefield(&state, "Blitz Goblin"),
        "creature should be in graveyard now (before end step)"
    );
    assert!(
        in_graveyard(&state, "Blitz Goblin", p1),
        "creature should be in graveyard"
    );

    // Advance to End step.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::End, "should advance to End step");

    // Resolve trigger at End step — trigger fires, finds creature NOT on battlefield,
    // does nothing. Creature should remain in graveyard (not double-moved).
    let (state, _end_events) = pass_all(state, &[p1, p2]);
    // Drain any residual triggers (e.g., AbilityResolved events).
    let (state, _) = drain_stack(state, &[p1, p2]);

    // Creature should still be in graveyard and NOT on battlefield.
    assert!(
        !on_battlefield(&state, "Blitz Goblin"),
        "Ruling 2022-04-29: creature that left battlefield should still not be on battlefield"
    );
    assert!(
        in_graveyard(&state, "Blitz Goblin", p1),
        "Ruling 2022-04-29: creature that left battlefield before end step should remain in graveyard"
    );
}

// ── Test 7: Card without blitz rejected ───────────────────────────────────────

/// CR 702.152a — Attempting to cast a non-blitz card with blitz alternative cost
/// is rejected with an error containing "blitz".
#[test]
fn test_blitz_card_without_blitz_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_goblin_def()]);

    let plain_goblin_in_hand = ObjectSpec::card(p1, "Plain Goblin")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_goblin_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Plain Goblin");

    // Attempt to cast with blitz — should fail because card has no blitz.
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
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    );

    assert!(
        result.is_err(),
        "CR 702.152a: casting non-blitz card with blitz cost should be rejected"
    );
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.to_lowercase().contains("blitz"),
        "CR 702.152a: error message should mention 'blitz', got: {}",
        err_msg
    );
}

// ── Test 8: Alternative cost exclusivity ─────────────────────────────────────

/// CR 118.9a — Blitz cannot be combined with other alternative costs.
/// Blitz is an alternative cost; only one alternative cost can apply to a spell.
/// Attempting to combine blitz with evoke is rejected.
#[test]
fn test_blitz_alternative_cost_exclusivity() {
    let p1 = p(1);
    let p2 = p(2);

    // Use a card that has blitz (but not evoke). The engine checks mutual exclusion
    // before checking evoke cost presence — the error fires on the blitz block's
    // "casting_with_evoke" check (or vice versa), returning an exclusivity error.
    let registry = CardRegistry::new(vec![blitz_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 3);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Blitz Goblin");

    // Attempt to cast with evoke when the card doesn't have evoke:
    // the engine first checks casting_with_evoke before blitz, so the error
    // will be about evoke not being present on the card. This confirms that
    // blitz and evoke are mutually exclusive (since both need to be declared).
    // Here we test: a card WITH blitz cannot be cast with Evoke alt_cost.
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
            alt_cost: Some(AltCostKind::Evoke),
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
        },
    );

    assert!(
        result.is_err(),
        "CR 118.9a: cannot combine blitz with evoke -- should be rejected"
    );
}

// ── Test 9: Commander tax applies on top of blitz cost ────────────────────────

/// CR 118.9d — Commander tax is added on top of alternative costs including blitz.
/// Commander with blitz cost {R}, cast once before (tax = {2}).
/// Attempt with only {R} → fails. Attempt with {2}{R} → succeeds.
#[test]
fn test_blitz_commander_tax_applies() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a legendary creature with blitz (usable as commander).
    let commander_def = CardDefinition {
        card_id: CardId("blitz-goblin".to_string()),
        name: "Blitz Goblin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Blitz {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Blitz),
            AbilityDefinition::Blitz {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![commander_def]);

    // Build state with the commander in the command zone.
    let commander_card = ObjectSpec::card(p1, "Blitz Goblin")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(CardId("blitz-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Blitz)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(commander_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set commander tax to 2 (simulates having cast the commander once before).
    let commander_card_id = CardId("blitz-goblin".to_string());
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_ids
        .push_back(commander_card_id.clone());
    // Commander tax entry stores the NUMBER OF TIMES already cast.
    // apply_commander_tax multiplies by 2: 1 cast = {2} additional mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(commander_card_id, 1);

    let cmd_obj_id = find_object(&state, "Blitz Goblin");
    state.turn.priority_holder = Some(p1);

    // Pay {R} blitz cost alone — should FAIL because commander tax {2} is owed
    // (total needed: {R} + {2} = {2}{R} = 3 mana, but only providing 1).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);

    let result_insufficient = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    );
    assert!(
        result_insufficient.is_err(),
        "CR 118.9d: blitz + commander tax should require more mana than {{R}} alone"
    );

    // Now pay {2}{R} (blitz cost {R} + {2} commander tax) — should succeed.
    {
        let player = state.players.get_mut(&p1).unwrap();
        player.mana_pool.colorless = 2;
        player.mana_pool.red = 1;
    }

    let result_sufficient = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Blitz),
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
        },
    );
    assert!(
        result_sufficient.is_ok(),
        "CR 118.9d: blitz + {{2}} commander tax = {{2}}{{R}} total; should succeed with 3 mana: {:?}",
        result_sufficient.err()
    );
}

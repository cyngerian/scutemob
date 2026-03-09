//! Replicate keyword ability tests (CR 702.56).
//!
//! Replicate represents two abilities:
//! 1. A static ability: "As an additional cost to cast this spell, you may pay [cost]
//!    any number of times." (CR 702.56a)
//! 2. A triggered ability: "When you cast this spell, if a replicate cost was paid for it,
//!    copy it for each time its replicate cost was paid. If the spell has any targets, you
//!    may choose new targets for any of the copies." (CR 702.56a)
//!
//! Key rules verified:
//! - Replicate is optional — cast with replicate_count=0 to not pay it.
//! - Paying replicate_count=N adds the replicate cost N times to the total mana cost.
//! - When replicate_count > 0, a ReplicateTrigger appears on the stack; resolving it
//!   creates N copies via `create_storm_copies`.
//! - Copies have `is_copy: true` and do NOT increment `spells_cast_this_turn`
//!   (ruling 2024-01-12 for Shattering Spree).
//! - Providing replicate_count > 0 for a spell without Replicate keyword is rejected.
//! - Insufficient mana (not enough to pay for replicate cost) causes rejection.
//! - Multiple copies (replicate_count=2 → 2 copies + original = 3 resolutions).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::AdditionalCost;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId,
    Step, ZoneId,
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

/// Pass priority for all listed players once (round-robin, one pass each).
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

/// Synthetic Replicate {U} instant: controller gains 1 life.
/// Using GainLife to make the effect easy to verify without targeting.
///
/// Mana cost: {1}{U}. Replicate {U}: pay {U} any number of times as additional cost.
fn replicate_life_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("replicate-life".to_string()),
        name: "Replicate Life".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Replicate {U} (As an additional cost to cast this spell, you may pay {U} any number of times. When you cast this spell, if a replicate cost was paid for it, copy it for each time its replicate cost was paid.)\nYou gain 1 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Replicate),
            AbilityDefinition::Replicate {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic spell without Replicate keyword — used to verify providing replicate_count > 0
/// for a non-replicate spell is rejected.
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
        oracle_text: "You gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Helper: build a 2-player state with Replicate Life in p1's hand and mana to cast it.
/// `extra_blue` is additional blue mana to add (for paying replicate costs).
fn setup_replicate_state(
    extra_blue: u32,
) -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![replicate_life_def()]);

    let spell = ObjectSpec::card(p1, "Replicate Life")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("replicate-life".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Replicate);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let ps = state.players.get_mut(&p1).unwrap();
    // Base cost: {1}{U}
    ps.mana_pool.add(ManaColor::Colorless, 1);
    ps.mana_pool.add(ManaColor::Blue, 1);
    // Extra blue for replicate payments
    for _ in 0..extra_blue {
        ps.mana_pool.add(ManaColor::Blue, 1);
    }
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Replicate Life");
    (state, p1, p2, spell_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.56a — Replicate with replicate_count=0: cast without paying the replicate cost.
/// Verify no trigger is created, the spell resolves once, and life is gained exactly once.
#[test]
fn test_replicate_zero_copies() {
    let (state, p1, p2, spell_id) = setup_replicate_state(0);
    let initial_life = state.players[&p1].life_total;

    // Cast without paying replicate (replicate_count = 0).
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
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without replicate should succeed: {:?}", e));

    // Only the spell should be on the stack -- no trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.56a: no ReplicateTrigger when replicate cost was not paid"
    );

    // Resolve -- both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Spell resolves once -- gain 1 life.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 702.56a: non-replicate cast should gain life exactly once"
    );
}

/// CR 702.56a — Replicate with replicate_count=1: pay once, verify exactly one copy created.
/// Trigger fires, resolves to produce 1 copy. Original + 1 copy = 2 total resolutions.
#[test]
fn test_replicate_one_copy() {
    // Extra blue: 1 (for replicate_count=1: pay {U} once).
    let (state, p1, p2, spell_id) = setup_replicate_state(1);
    let initial_life = state.players[&p1].life_total;

    // Cast with replicate_count=1.
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
            additional_costs: vec![AdditionalCost::Replicate { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with replicate_count=1 failed: {:?}", e));

    // Stack should have 2 objects: spell + trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.56a: after paying replicate cost once, stack should have spell + ReplicateTrigger"
    );

    // Top of stack should be the ReplicateTrigger.
    use mtg_engine::StackObjectKind;
    let top = state.stack_objects.back().expect("stack must have objects");
    assert!(
        matches!(
            top.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Replicate,
                data: mtg_engine::state::stack::TriggerData::SpellCopy { copy_count: 1, .. },
                ..
            }
        ),
        "CR 702.56a: Replicate KeywordTrigger should be on top with copy_count=1"
    );

    // Both players pass priority -- trigger resolves, creating 1 copy.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After trigger resolves: 1 copy pushed above original → stack = [original, copy].
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.56a: after ReplicateTrigger resolves with count=1, both copy and original on stack"
    );

    let copy = state
        .stack_objects
        .iter()
        .find(|o| o.is_copy)
        .expect("CR 702.56a: copy (is_copy=true) should be on stack");
    assert!(copy.is_copy, "copy should have is_copy=true");

    // Resolve copy (both pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 702.56a: copy resolves first, gaining 1 life"
    );

    // Resolve original (both pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 2,
        "CR 702.56a: original resolves, total life gain = 2 (1 copy + 1 original)"
    );
}

/// CR 702.56a — Replicate with replicate_count=2: pay twice, verify two copies created.
/// Trigger fires, resolves to produce 2 copies. Original + 2 copies = 3 total resolutions.
#[test]
fn test_replicate_basic_two_copies() {
    // Extra blue: 2 (for replicate_count=2: pay {U} twice).
    let (state, p1, p2, spell_id) = setup_replicate_state(2);
    let initial_life = state.players[&p1].life_total;

    // Cast with replicate_count=2.
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
            additional_costs: vec![AdditionalCost::Replicate { count: 2 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with replicate_count=2 failed: {:?}", e));

    // Stack should have 2 objects: spell + trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.56a: after paying replicate cost twice, stack should have spell + ReplicateTrigger"
    );

    // Top of stack should be the ReplicateTrigger with replicate_count=2.
    use mtg_engine::StackObjectKind;
    let top = state.stack_objects.back().expect("stack must have objects");
    assert!(
        matches!(
            top.kind,
            StackObjectKind::KeywordTrigger {
                keyword: KeywordAbility::Replicate,
                data: mtg_engine::state::stack::TriggerData::SpellCopy { copy_count: 2, .. },
                ..
            }
        ),
        "CR 702.56a: Replicate KeywordTrigger should be on top with copy_count=2"
    );

    // Both players pass priority -- trigger resolves, creating 2 copies.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After trigger resolves: 2 copies pushed → stack = [original, copy1, copy2].
    assert_eq!(
        state.stack_objects.len(),
        3,
        "CR 702.56a: after ReplicateTrigger resolves with count=2, 2 copies + original on stack"
    );

    // Count copies (is_copy=true).
    let copy_count = state.stack_objects.iter().filter(|o| o.is_copy).count();
    assert_eq!(
        copy_count, 2,
        "CR 702.56a: exactly 2 copies should be on stack after replicate trigger resolves"
    );

    // Resolve copy 1 (top of stack).
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.players[&p1].life_total, initial_life + 1);

    // Resolve copy 2.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.players[&p1].life_total, initial_life + 2);

    // Resolve original.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.56a: 2 copies + 1 original = 3 total resolutions, gaining 3 life"
    );
}

/// Engine validation — providing replicate_count > 0 for a spell without Replicate keyword
/// must be rejected.
#[test]
fn test_replicate_no_keyword_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_instant_def()]);

    let spell = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
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

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Instant");

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
            additional_costs: vec![AdditionalCost::Replicate { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "Engine validation: replicate_count > 0 for a spell without Replicate keyword must be rejected (CR 702.56a)"
    );
}

/// Ruling 2024-01-12 (Shattering Spree) — Replicate copies are NOT cast: the `is_copy` flag
/// should be true on copies, and `spells_cast_this_turn` should only be incremented once
/// (for the original).
#[test]
fn test_replicate_copies_not_cast() {
    // Extra blue: 1 (for replicate_count=1).
    let (state, p1, p2, spell_id) = setup_replicate_state(1);
    let spells_before = state.players[&p1].spells_cast_this_turn;

    // Cast with replicate_count=1.
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
            additional_costs: vec![AdditionalCost::Replicate { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with replicate_count=1 failed: {:?}", e));

    // After cast, spells_cast_this_turn should be +1 (original only).
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "Ruling 2024-01-12: only the original spell is 'cast'; spells_cast_this_turn should be +1"
    );

    // Resolve the replicate trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After trigger resolves, spells_cast_this_turn should still be +1, not +2.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "Ruling 2024-01-12: replicate copy is not cast; spells_cast_this_turn should remain +1"
    );

    // Verify the copy exists on stack with is_copy = true.
    let copy = state.stack_objects.iter().find(|o| o.is_copy);
    assert!(
        copy.is_some(),
        "Ruling 2024-01-12: copy created by replicate trigger should have is_copy = true"
    );
}

/// CR 702.56a / CR 601.2f — Replicate cost is added N times to the total mana cost.
/// If the player has insufficient mana for replicate_count=1 (needs base cost + 1 replicate),
/// the cast must fail.
#[test]
fn test_replicate_mana_cost_added() {
    // Setup with NO extra blue -- only enough mana for base cost {1}{U}.
    // Attempting replicate_count=1 (needs additional {U}) must fail.
    let (state, p1, _p2, spell_id) = setup_replicate_state(0);

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
            additional_costs: vec![AdditionalCost::Replicate { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.56a / CR 601.2f: replicate cost is added to total; insufficient mana for replicate_count=1 must be rejected"
    );
}

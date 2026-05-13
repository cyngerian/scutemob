//! PB-CD: counter-doubling/extra-counter replacement effects gated on counter type
//! and "creature you control" receiver (CR 122.6 / CR 614.1).
//!
//! Covers Hardened Scales, Corpsejack Menace, and the replacement half of
//! Conclave Mentor. Validates:
//!   * counter_filter gate (Some(PlusOnePlusOne)) — +1/+1-only.
//!   * ObjectFilter::CreatureControlledBy — creature you control, not any
//!     permanent, not an opponent's creature.
//!   * Isolation: -1/-1 / loyalty / charge counters on a creature you control
//!     are NOT modified by Hardened Scales.
//!   * Stacking: two Hardened Scales → +2; Hardened Scales × Corpsejack Menace.

use mtg_engine::cards::card_definition::{AbilityDefinition, CardDefinition, TypeLine};
use mtg_engine::cards::defs;
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::replacement_effect::{
    ObjectFilter, PlayerFilter, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::state::{
    CardId, CardType, CounterType, GameStateBuilder, ObjectSpec, PlayerId, ZoneId,
};
use mtg_engine::CardRegistry;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn make_target_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pb-cd-target-creature".to_string()),
        name: "PB-CD Target Creature".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

fn make_target_enchantment_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pb-cd-target-enchantment".to_string()),
        name: "PB-CD Target Enchantment".to_string(),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Re-register replacement effects on all battlefield permanents that have a card_id
/// (the builder doesn't run ETB hooks).
fn register_replacement_effects(state: &mut mtg_engine::state::GameState) {
    use mtg_engine::state::game_object::ObjectId;
    use mtg_engine::state::zone::ZoneId;

    let registry = state.card_registry.clone();
    let battlefield_objects: Vec<(
        ObjectId,
        PlayerId,
        Option<mtg_engine::state::player::CardId>,
    )> = state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();

    for (obj_id, controller, card_id) in &battlefield_objects {
        mtg_engine::rules::replacement::register_permanent_replacement_abilities(
            state,
            *obj_id,
            *controller,
            card_id.as_ref(),
            &registry,
        );
    }
}

fn find_named(
    state: &mtg_engine::state::GameState,
    name: &str,
) -> mtg_engine::state::game_object::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap()
}

/// Build a state where p1 controls Hardened Scales + a 2/2 creature and p2 controls
/// a 2/2 creature. Both creatures share the same target def so we can find them by
/// owner.
fn build_scales_state() -> (mtg_engine::state::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![
        defs::hardened_scales::card(),
        make_target_creature_def(),
        make_target_enchantment_def(),
    ]);

    let mut scales_spec =
        ObjectSpec::enchantment(p1, "Hardened Scales").in_zone(ZoneId::Battlefield);
    scales_spec.card_id = Some(CardId("hardened-scales".to_string()));

    let mut own_creature =
        ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    own_creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut opp_creature =
        ObjectSpec::creature(p2, "Opp Creature", 2, 2).in_zone(ZoneId::Battlefield);
    opp_creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut own_enchantment =
        ObjectSpec::enchantment(p1, "Own Enchantment").in_zone(ZoneId::Battlefield);
    own_enchantment.card_id = Some(CardId("pb-cd-target-enchantment".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(scales_spec)
        .object(own_creature)
        .object(opp_creature)
        .object(own_enchantment)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    (state, p1, p2)
}

/// Build a state with Corpsejack Menace + own/opponent creatures.
fn build_corpsejack_state() -> (mtg_engine::state::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![
        defs::corpsejack_menace::card(),
        make_target_creature_def(),
    ]);

    let mut corpsejack_spec =
        ObjectSpec::creature(p1, "Corpsejack Menace", 4, 4).in_zone(ZoneId::Battlefield);
    corpsejack_spec.card_id = Some(CardId("corpsejack-menace".to_string()));

    let mut own_creature =
        ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    own_creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(corpsejack_spec)
        .object(own_creature)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    (state, p1, p2)
}

/// Build a state with Conclave Mentor + own/opponent creatures.
fn build_conclave_state() -> (mtg_engine::state::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![
        defs::conclave_mentor::card(),
        make_target_creature_def(),
    ]);

    let mut mentor_spec =
        ObjectSpec::creature(p1, "Conclave Mentor", 2, 2).in_zone(ZoneId::Battlefield);
    mentor_spec.card_id = Some(CardId("conclave-mentor".to_string()));

    let mut own_creature =
        ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    own_creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(mentor_spec)
        .object(own_creature)
        .build()
        .unwrap();

    register_replacement_effects(&mut state);
    (state, p1, p2)
}

// ── Positive tests ──────────────────────────────────────────────────────────

/// CR 122.6 / 614.1 — Hardened Scales adds +1 to +1/+1 counters on a creature you control.
#[test]
fn pb_cd_hardened_scales_adds_extra_on_creature_you_control() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 4, "Hardened Scales should add +1: 3 → 4");
    assert_eq!(
        events.len(),
        1,
        "exactly one ReplacementEffectApplied event"
    );
}

/// CR 122.6 / 614.1 — Corpsejack Menace doubles +1/+1 counters on a creature you control.
#[test]
fn pb_cd_corpsejack_doubles_plus1_plus1_on_creature_you_control() {
    let (state, p1, _) = build_corpsejack_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 6, "Corpsejack Menace should double: 3 → 6");
    assert_eq!(events.len(), 1);
}

/// CR 122.6 / 614.1 — Conclave Mentor (replacement half) adds +1 to +1/+1 counters on
/// a creature you control.
#[test]
fn pb_cd_conclave_mentor_replacement_adds_extra_on_creature_you_control() {
    let (state, p1, _) = build_conclave_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        2,
    );
    assert_eq!(modified, 3, "Conclave Mentor should add +1: 2 → 3");
    assert_eq!(events.len(), 1);
}

// ── Counter-type isolation tests ────────────────────────────────────────────

/// PB-CD Gap 1 — Hardened Scales must NOT apply to -1/-1 counters.
/// Without counter_filter gating, Scales would (wrongly) push 3 → 4 here.
#[test]
fn pb_cd_hardened_scales_does_not_affect_minus1_minus1() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::MinusOneMinusOne,
        3,
    );
    assert_eq!(modified, 3, "Hardened Scales must NOT touch -1/-1 counters");
    assert!(events.is_empty(), "no replacement events should fire");
}

/// PB-CD Gap 1 — Hardened Scales must NOT apply to loyalty counters.
#[test]
fn pb_cd_hardened_scales_does_not_affect_loyalty_counters() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::Loyalty,
        3,
    );
    assert_eq!(
        modified, 3,
        "Hardened Scales must NOT touch loyalty counters"
    );
    assert!(events.is_empty());
}

/// PB-CD Gap 1 — Hardened Scales must NOT apply to charge / oil / quest etc.
#[test]
fn pb_cd_hardened_scales_does_not_affect_charge_counters() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::Charge,
        3,
    );
    assert_eq!(modified, 3);
    assert!(events.is_empty());
}

// ── Receiver-filter isolation tests ─────────────────────────────────────────

/// PB-CD Gap 2 — Hardened Scales must NOT apply to opponent's creature.
#[test]
fn pb_cd_hardened_scales_does_not_affect_opponent_creature() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Opp Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(
        modified, 3,
        "Hardened Scales must NOT touch opponent's creature"
    );
    assert!(events.is_empty());
}

/// PB-CD Gap 2 — Hardened Scales must NOT apply to a non-creature permanent (enchantment)
/// you control. Distinguishes CreatureControlledBy from ControlledBy.
#[test]
fn pb_cd_hardened_scales_does_not_affect_own_enchantment() {
    let (state, p1, _) = build_scales_state();
    let target = find_named(&state, "Own Enchantment");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(
        modified, 3,
        "Hardened Scales must NOT touch a non-creature permanent (enchantment) you control"
    );
    assert!(events.is_empty());
}

// ── Stacking tests ──────────────────────────────────────────────────────────

/// CR 614.5 / ruling — each additional Hardened Scales adds one more counter.
#[test]
fn pb_cd_two_hardened_scales_add_two_extra() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![
        defs::hardened_scales::card(),
        make_target_creature_def(),
    ]);

    let mut scales1 =
        ObjectSpec::enchantment(p1, "Hardened Scales #1").in_zone(ZoneId::Battlefield);
    scales1.card_id = Some(CardId("hardened-scales".to_string()));

    let mut scales2 =
        ObjectSpec::enchantment(p1, "Hardened Scales #2").in_zone(ZoneId::Battlefield);
    scales2.card_id = Some(CardId("hardened-scales".to_string()));

    let mut creature = ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(scales1)
        .object(scales2)
        .object(creature)
        .build()
        .unwrap();
    register_replacement_effects(&mut state);

    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 5, "Two Hardened Scales: 3 → 4 → 5");
    assert_eq!(events.len(), 2);
}

/// CR 614.5 + 616.1 — Hardened Scales × Corpsejack Menace stack.
/// Deterministic order: each replacement fires once; the resulting count after
/// both applications is what matters. Engine applies in registration order;
/// with Corpsejack added before Scales, doubling fires first (3→6), then +1 (6→7).
#[test]
fn pb_cd_scales_and_corpsejack_stack() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![
        defs::corpsejack_menace::card(),
        defs::hardened_scales::card(),
        make_target_creature_def(),
    ]);

    let mut corpsejack_spec =
        ObjectSpec::creature(p1, "Corpsejack Menace", 4, 4).in_zone(ZoneId::Battlefield);
    corpsejack_spec.card_id = Some(CardId("corpsejack-menace".to_string()));

    let mut scales_spec =
        ObjectSpec::enchantment(p1, "Hardened Scales").in_zone(ZoneId::Battlefield);
    scales_spec.card_id = Some(CardId("hardened-scales".to_string()));

    let mut creature = ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(corpsejack_spec)
        .object(scales_spec)
        .object(creature)
        .build()
        .unwrap();
    register_replacement_effects(&mut state);

    let target = find_named(&state, "Own Creature");
    let (modified, events) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    // Both replacements apply (CR 614.5 — each fires once). Final count must be one of
    // 7 (double-then-add, 3→6→7) or 8 (add-then-double, 3→4→8) depending on order.
    // The current engine applies in registration order (deterministic). Both yield ≥ 7
    // and both differ from the no-stacking baseline; assert ≥ 7 to keep this resilient
    // to future CR 616.1 interactive-ordering work.
    assert!(
        modified == 7 || modified == 8,
        "Corpsejack × Scales should yield 7 or 8, got {}",
        modified
    );
    assert_eq!(
        events.len(),
        2,
        "both replacements should fire exactly once"
    );
}

// ── Cross-existing-test sanity: Vorinclex still doubles any-type counters. ──

/// Pre-PB-CD behavior preserved: a `counter_filter: None` effect (Vorinclex shape)
/// continues to apply to ALL counter types, not only +1/+1. Ensures the new
/// gating is OPT-IN and backward compatible.
#[test]
fn pb_cd_counter_filter_none_matches_any_counter_type() {
    // Construct an inline Vorinclex-shape def matching any counter type.
    fn make_any_counter_doubler() -> CardDefinition {
        CardDefinition {
            card_id: CardId("pb-cd-any-counter-doubler".to_string()),
            name: "Any-Counter Doubler".to_string(),
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            mana_cost: Some(ManaCost {
                generic: 4,
                green: 2,
                ..Default::default()
            }),
            power: Some(6),
            toughness: Some(6),
            abilities: vec![AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Specific(PlayerId(0)),
                    receiver_filter: ObjectFilter::Any,
                    counter_filter: None,
                },
                modification: ReplacementModification::DoubleCounters,
                is_self: false,
                unless_condition: None,
            }],
            ..Default::default()
        }
    }

    let p1 = p(1);
    let registry = CardRegistry::new(vec![make_any_counter_doubler(), make_target_creature_def()]);

    let mut doubler_spec =
        ObjectSpec::creature(p1, "Any-Counter Doubler", 6, 6).in_zone(ZoneId::Battlefield);
    doubler_spec.card_id = Some(CardId("pb-cd-any-counter-doubler".to_string()));

    let mut creature = ObjectSpec::creature(p1, "Own Creature", 2, 2).in_zone(ZoneId::Battlefield);
    creature.card_id = Some(CardId("pb-cd-target-creature".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(doubler_spec)
        .object(creature)
        .build()
        .unwrap();
    register_replacement_effects(&mut state);

    let target = find_named(&state, "Own Creature");

    // +1/+1 → doubled
    let (modified, _) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::PlusOnePlusOne,
        3,
    );
    assert_eq!(modified, 6, "any-counter doubler should double +1/+1");

    // Loyalty → doubled (counter_filter None should still match)
    let (modified, _) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::Loyalty,
        3,
    );
    assert_eq!(
        modified, 6,
        "any-counter doubler should also double Loyalty"
    );

    // Charge → doubled
    let (modified, _) = mtg_engine::rules::replacement::apply_counter_replacement(
        &state,
        p1,
        target,
        &CounterType::Charge,
        2,
    );
    assert_eq!(modified, 4, "any-counter doubler should also double Charge");
}

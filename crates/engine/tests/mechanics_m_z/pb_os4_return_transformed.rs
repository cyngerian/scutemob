//! PB-OS4 (scutemob-130, OOS-EF5-3): return-transformed / enters-transformed as a
//! NEW object (CR 400.7 / 712.18 / 603.7).
//!
//! Three new unit `Effect` variants model a permanent that LEAVES the battlefield
//! (exiled or dies) and RETURNS already showing its back face -- a fundamentally
//! different mechanism from PB-EF5's `Effect::TransformSelf` (which flips a DFC
//! IN PLACE, same `ObjectId`, CR 712.18). Here the permanent becomes a new object
//! per CR 400.7 when it leaves, and a *further* new object enters the battlefield
//! transformed: counters, Auras/Equipment, and damage do NOT carry over.
//!
//! - `Effect::ExileSourceAndReturnTransformed` -- immediate: exile `ctx.source`
//!   (a battlefield permanent), then return it transformed. Used by Fable of the
//!   Mirror-Breaker's Saga chapter III.
//! - `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` -- delayed: register
//!   a `DelayedTrigger` (CR 603.7) that returns `ctx.source` (already off the
//!   battlefield) transformed at the beginning of the next end step. No roster
//!   card in this PB uses this exact timing (see below), but it is a real,
//!   distinct CR 603.7 primitive and is exercised directly here.
//! - `Effect::ReturnSourceToBattlefieldTransformed` -- immediate, NO exile step:
//!   `ctx.source` has already left the battlefield via some other event (e.g.
//!   death) and returns transformed on the SAME resolution. Added mid-PB when
//!   Edgar, Charmed Groom's real oracle text (confirmed against `cards.sqlite`)
//!   turned out to return immediately, with neither an exile step nor an
//!   "at the next end step" delay -- a genuine divergence from the plan's
//!   reconstruction (which assumed the delayed shape). Mirrors Persist/Undying's
//!   `Effect::MoveZone { Source -> Battlefield }` idiom, plus the back-face flip
//!   and static/ETB registration.
//!
//! Key rules verified:
//! - CR 400.7: the pre-departure `ObjectId` is dead; the returned object is a
//!   NEW `ObjectId`.
//! - CR 400.7: counters do NOT carry over (decoy: `TransformSelf` on the same
//!   setup keeps them -- proves this is the new-object path, not the in-place
//!   flip).
//! - CR 400.7 / 704.5m: an Aura enchanting the departing permanent falls off (SBA)
//!   -- it does not follow the permanent to its new object.
//! - CR 712.18: the returned object's characteristics are the BACK face's,
//!   layer-resolved (decoy: the front face's name is not what's read).
//! - CR ruling (DFC-copy edge): a non-DFC source exiled this way stays in exile
//!   -- it does not return (decoy: a DFC source in the same shape DOES return).
//! - CR 603.7: the delayed variant's return happens at the next end step, not
//!   immediately (decoy: not on the battlefield before the end step).
//! - CR 400.7 / 603.7c: the delayed-returned object is also a new `ObjectId`.
//! - CR 714.4: a Saga whose chapter III returns it transformed is NOT swept by
//!   the "sacrifice after final chapter" SBA -- it's no longer a Saga (decoy: a
//!   Saga whose chapter III does a plain effect IS sacrificed).
//! - Card integration: `fable_of_the_mirror_breaker` (chapter III) and
//!   `edgar_charmed_groom` (WhenDies, full trigger-dispatch path).
//! - Integrity guard: `nicol_bolas_the_ravager` / `grist_voracious_larva` are NOT
//!   force-flipped to `Complete` while the planeswalker-back starting-loyalty gap
//!   (OOS-OS4-1) stands.

use mtg_engine::cards::card_definition::Effect;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, calculate_characteristics, check_and_apply_sbas, enrich_spec_from_def,
    AbilityDefinition, CardDefinition, CardFace, CardId, CardRegistry, CardType, Completeness,
    CounterType, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerId, Step, SubType, TypeLine, ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: &ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && &obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Find the (single) `is_transformed` object on the battlefield controlled by
/// `owner`. Base `characteristics.name` does NOT change on transform (only
/// `calculate_characteristics` resolves the back face per CR 712.18), so a
/// returned-transformed object can't be found by its back-face NAME via
/// `find_in_zone` -- it must be found by the `is_transformed` flag, then its
/// resolved characteristics checked separately.
fn find_transformed_on_battlefield(state: &GameState, owner: PlayerId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield && obj.is_transformed && obj.owner == owner
        })
        .map(|(id, _)| *id)
}

/// Drain the stack completely by passing priority in turn order.
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 200, "drain_stack exceeded safety guard");
        for &pl in players {
            let (s, _) = mtg_engine::process_command(
                state,
                mtg_engine::Command::PassPriority { player: pl },
            )
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
            state = s;
        }
    }
    state
}

fn advance_to_step(mut state: GameState, target: Step, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while state.turn().step != target {
        guard += 1;
        assert!(guard < 500, "advance_to_step exceeded safety guard");
        for &pl in players {
            let (s, _) = mtg_engine::process_command(
                state,
                mtg_engine::Command::PassPriority { player: pl },
            )
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
            state = s;
        }
    }
    state
}

// ── Mock DFC definitions ──────────────────────────────────────────────────────

/// Front: "Mock RT Front" 2/2 Creature. Back: "Mock RT Back" 4/4 Creature with Flying.
fn mock_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-return-transformed-dfc".to_string()),
        name: "Mock RT Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock RT Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Flying".to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Flying)],
            power: Some(4),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// A single-faced (non-DFC) card -- the DFC-copy-edge decoy (CR ruling: a non-DFC
/// told to enter transformed stays wherever it was moved to).
fn mock_nondfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-return-transformed-nondfc".to_string()),
        name: "Mock RT Non-DFC".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        ..Default::default()
    }
}

fn mock_dfc_on_battlefield(owner: PlayerId, name: &str) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-return-transformed-dfc".to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

fn mock_dfc_in_graveyard(owner: PlayerId, name: &str) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("mock-return-transformed-dfc".to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

fn mock_nondfc_on_battlefield(owner: PlayerId, name: &str) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-return-transformed-nondfc".to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

fn registry_with(defs: Vec<CardDefinition>) -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(defs)
}

// ── 1: ExileSourceAndReturnTransformed -- new-object identity (CR 400.7) ──────

#[test]
fn test_return_transformed_new_object_identity() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let old_id = find_by_name(&state, "Mock RT Front");
    let mut ctx = EffectContext::new(p1, old_id, vec![]);
    let events = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    // CR 400.7: the old ObjectId is dead.
    assert!(
        !state.objects().contains_key(&old_id),
        "CR 400.7: the pre-departure ObjectId must be dead"
    );
    let new_id = find_transformed_on_battlefield(&state, p1)
        .expect("returned object should be on the battlefield as the back face");
    assert_ne!(
        new_id, old_id,
        "CR 400.7: the returned object is a NEW ObjectId"
    );
    assert!(state.objects()[&new_id].is_transformed);
    assert_eq!(
        calculate_characteristics(&state, new_id).unwrap().name,
        "Mock RT Back"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { object_id, .. } if *object_id == new_id)
        ),
        "should emit PermanentEnteredBattlefield for the new object"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::ObjectExiled { object_id, .. } if *object_id == old_id)
        ),
        "should emit ObjectExiled for the departing object (CR 400.7j)"
    );
}

// ── 2: counters do NOT carry (decoy: TransformSelf keeps them) ───────────────

#[test]
fn test_return_transformed_counters_do_not_carry() {
    let p1 = p(1);
    let p2 = p(2);

    // Case A: ExileSourceAndReturnTransformed -- counters do NOT carry.
    let registry = registry_with(vec![mock_dfc_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            mock_dfc_on_battlefield(p1, "Mock RT Front")
                .with_counter(CounterType::PlusOnePlusOne, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let old_id = find_by_name(&state, "Mock RT Front");
    assert_eq!(
        state.objects()[&old_id]
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        3,
        "sanity: counters present before the effect"
    );
    let mut ctx = EffectContext::new(p1, old_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );
    let new_id = find_transformed_on_battlefield(&state, p1).unwrap();
    assert!(
        state.objects()[&new_id].counters.is_empty(),
        "CR 400.7: counters must NOT carry over to the new object"
    );

    // Case B (decoy): TransformSelf on the same setup KEEPS the counters -- proves
    // the field-under-test really is "which path was taken", not an unrelated bug.
    let registry2 = registry_with(vec![mock_dfc_def()]);
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object(
            mock_dfc_on_battlefield(p1, "Mock RT Front")
                .with_counter(CounterType::PlusOnePlusOne, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id2 = find_by_name(&state2, "Mock RT Front");
    let mut ctx2 = EffectContext::new(p1, id2, vec![]);
    let _ = execute_effect(&mut state2, &Effect::TransformSelf, &mut ctx2);
    assert_eq!(
        state2.objects()[&id2]
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        3,
        "decoy: TransformSelf's in-place flip keeps counters (CR 712.18) -- contrasts with case A"
    );
}

// ── 3: Aura falls off (CR 400.7 / 704.5m) ─────────────────────────────────────

#[test]
fn test_return_transformed_aura_falls_off() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock RT Front"))
        .object(
            ObjectSpec::enchantment(p1, "Mock Aura")
                .with_subtypes(vec![SubType("Aura".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_by_name(&state, "Mock RT Front");
    let aura_id = find_by_name(&state, "Mock Aura");
    state.objects_mut().get_mut(&aura_id).unwrap().attached_to = Some(creature_id);
    state
        .objects_mut()
        .get_mut(&creature_id)
        .unwrap()
        .attachments
        .push_back(aura_id);

    let mut ctx = EffectContext::new(p1, creature_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    // The new object is unenchanted (it never had the Aura attached).
    let new_id = find_transformed_on_battlefield(&state, p1).unwrap();
    assert!(state.objects()[&new_id].attachments.is_empty());

    // SBA 704.5m: the Aura, still pointing at the dead old ObjectId, falls off.
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        sba_events.iter().any(
            |e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura_id)
        ),
        "CR 400.7/704.5m: Aura enchanting the departed permanent should fall off; events: {:?}",
        sba_events
    );
    assert!(
        find_in_zone(&state, "Mock Aura", &ZoneId::Graveyard(p1)).is_some(),
        "fallen-off Aura should be in its owner's graveyard"
    );
}

// ── 4: back-face characteristics, layer-resolved (CR 712.18) ────────────────

#[test]
fn test_return_transformed_back_face_characteristics() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let old_id = find_by_name(&state, "Mock RT Front");
    let mut ctx = EffectContext::new(p1, old_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    let new_id = find_transformed_on_battlefield(&state, p1).unwrap();
    let chars = calculate_characteristics(&state, new_id).expect("should have chars");
    assert_eq!(
        chars.name, "Mock RT Back",
        "CR 712.18: back-face name, layer-resolved"
    );
    assert_eq!(chars.power, Some(4));
    assert_eq!(chars.toughness, Some(4));
    assert!(chars.keywords.contains(&KeywordAbility::Flying));
    // Decoy: the FRONT face's name is not what's read.
    assert_ne!(chars.name, "Mock RT Front");
}

// ── 5: non-DFC stays in exile (decoy: DFC variant DOES return) ──────────────

#[test]
fn test_return_transformed_non_dfc_stays_in_exile() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_nondfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_nondfc_on_battlefield(p1, "Mock RT Non-DFC"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let old_id = find_by_name(&state, "Mock RT Non-DFC");
    let mut ctx = EffectContext::new(p1, old_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    assert!(
        find_in_zone(&state, "Mock RT Non-DFC", &ZoneId::Exile).is_some(),
        "CR ruling: a non-DFC exiled this way stays in exile"
    );
    assert!(
        find_in_zone(&state, "Mock RT Non-DFC", &ZoneId::Battlefield).is_none(),
        "a non-DFC must NOT return to the battlefield"
    );

    // Decoy: the DFC variant, same shape, DOES return.
    let registry2 = registry_with(vec![mock_dfc_def()]);
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object(mock_dfc_on_battlefield(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id2 = find_by_name(&state2, "Mock RT Front");
    let mut ctx2 = EffectContext::new(p1, id2, vec![]);
    let _ = execute_effect(
        &mut state2,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx2,
    );
    assert!(
        find_transformed_on_battlefield(&state2, p1).is_some(),
        "decoy: the DFC variant should return transformed"
    );
}

// ── 6: delayed timing -- returns at next end step, not immediately (CR 603.7) ──

#[test]
fn test_delayed_return_transformed_timing() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_in_graveyard(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let gy_id = find_by_name(&state, "Mock RT Front");
    let mut ctx = EffectContext::new(p1, gy_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ReturnSourceToBattlefieldTransformedNextEndStep,
        &mut ctx,
    );

    // Immediately after resolving: still in the graveyard, NOT on the battlefield.
    // Decoy: a bug that returned immediately would fail this assertion.
    assert!(
        find_in_zone(&state, "Mock RT Front", &ZoneId::Graveyard(p1)).is_some(),
        "CR 603.7: must still be in the graveyard immediately after the trigger resolves"
    );
    assert!(
        find_transformed_on_battlefield(&state, p1).is_none(),
        "CR 603.7: must NOT be on the battlefield before the end step"
    );
    assert_eq!(
        state.delayed_triggers().len(),
        1,
        "one pending delayed trigger"
    );

    // Advance to the end step and drain the stack -- NOW it returns transformed.
    let state = advance_to_step(state, Step::End, &[p1, p2]);
    let state = drain_stack(state, &[p1, p2]);

    assert!(
        find_transformed_on_battlefield(&state, p1).is_some(),
        "CR 603.7: should return transformed at the beginning of the next end step"
    );
}

// ── 7: delayed return is also a new object (CR 400.7 / 603.7c) ──────────────

#[test]
fn test_delayed_return_transformed_is_new_object() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_in_graveyard(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let gy_id = find_by_name(&state, "Mock RT Front");
    let mut ctx = EffectContext::new(p1, gy_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ReturnSourceToBattlefieldTransformedNextEndStep,
        &mut ctx,
    );

    let state = advance_to_step(state, Step::End, &[p1, p2]);
    let state = drain_stack(state, &[p1, p2]);

    let new_id =
        find_transformed_on_battlefield(&state, p1).expect("should have returned transformed");
    assert_ne!(
        new_id, gy_id,
        "CR 400.7/603.7c: the returned object is a NEW ObjectId"
    );
    assert!(state.objects()[&new_id].is_transformed);
    assert!(
        !state.objects().contains_key(&gy_id),
        "the old graveyard ObjectId is dead"
    );
}

// ── 8: immediate no-exile return (Edgar's real shape) ────────────────────────

/// `Effect::ReturnSourceToBattlefieldTransformed` -- the source is ALREADY off the
/// battlefield (e.g. a graveyard object from a WhenDies trigger) and returns
/// transformed IMMEDIATELY, with no exile step. Decoy: no `ObjectExiled` event is
/// emitted (contrasts with `ExileSourceAndReturnTransformed`, which always does).
#[test]
fn test_return_transformed_immediate_no_exile_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_in_graveyard(p1, "Mock RT Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let gy_id = find_by_name(&state, "Mock RT Front");
    let mut ctx = EffectContext::new(p1, gy_id, vec![]);
    let events = execute_effect(
        &mut state,
        &Effect::ReturnSourceToBattlefieldTransformed,
        &mut ctx,
    );

    // Immediate: already on the battlefield, transformed, new object -- no delay.
    let new_id = find_transformed_on_battlefield(&state, p1)
        .expect("should return transformed immediately, no next-end-step delay");
    assert_ne!(new_id, gy_id, "CR 400.7: new ObjectId");
    assert!(state.objects()[&new_id].is_transformed);
    assert!(
        !state.objects().contains_key(&gy_id),
        "old ObjectId is dead"
    );

    // Decoy: no exile step -- no ObjectExiled event (contrasts with
    // ExileSourceAndReturnTransformed, which always emits one).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { .. })),
        "the no-exile variant must not emit ObjectExiled"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { object_id, .. } if *object_id == new_id)));
}

// ── 9: Saga chapter III no-sacrifice (CR 714.4) ──────────────────────────────

fn mock_saga_return_transformed_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-saga-return-transformed".to_string()),
        name: "Mock Saga RT".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            subtypes: [SubType("Saga".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "III -- Exile this Saga, then return it to the battlefield transformed."
            .to_string(),
        abilities: vec![AbilityDefinition::SagaChapter {
            chapter: 3,
            effect: Effect::ExileSourceAndReturnTransformed,
            targets: vec![],
        }],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock Saga RT Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: vec![],
            power: Some(3),
            toughness: Some(3),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// Decoy Saga: chapter III does a plain effect (no return-transformed) -- stays a
/// Saga at 3+ lore counters, so CR 714.4 DOES sacrifice it.
fn mock_saga_plain_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-saga-plain".to_string()),
        name: "Mock Saga Plain".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            subtypes: [SubType("Saga".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "III -- Gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::SagaChapter {
            chapter: 3,
            effect: Effect::GainLife {
                player: mtg_engine::PlayerTarget::Controller,
                amount: mtg_engine::EffectAmount::Fixed(1),
            },
            targets: vec![],
        }],
        ..Default::default()
    }
}

#[test]
fn test_saga_return_transformed_chapter_three_no_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_saga_return_transformed_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock Saga RT")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-saga-return-transformed".to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_counter(CounterType::Lore, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let saga_id = find_by_name(&state, "Mock Saga RT");
    let mut ctx = EffectContext::new(p1, saga_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    let new_id = find_transformed_on_battlefield(&state, p1)
        .expect("the back-face creature should be on the battlefield");
    let chars = calculate_characteristics(&state, new_id).unwrap();
    assert_eq!(chars.name, "Mock Saga RT Back");
    assert_eq!(chars.power, Some(3), "returned as the back-face creature");

    // CR 714.4: the object leaving-and-returning is no longer a Saga, so the
    // "sacrifice after final chapter" SBA does NOT sweep it.
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        state.objects().contains_key(&new_id),
        "CR 714.4 must NOT sacrifice the returned object -- it's no longer a Saga"
    );
    assert!(
        !sba_events.iter().any(
            |e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == new_id)
        ),
        "no CR 714.4 sacrifice event for the returned object"
    );

    // Decoy: a Saga whose chapter III does a PLAIN effect stays a Saga and IS
    // sacrificed by CR 714.4.
    let registry2 = registry_with(vec![mock_saga_plain_def()]);
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object(
            ObjectSpec::card(p1, "Mock Saga Plain")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-saga-plain".to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_counter(CounterType::Lore, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let plain_id = find_by_name(&state2, "Mock Saga Plain");
    let mut ctx2 = EffectContext::new(p1, plain_id, vec![]);
    let _ = execute_effect(
        &mut state2,
        &Effect::GainLife {
            player: mtg_engine::PlayerTarget::Controller,
            amount: mtg_engine::EffectAmount::Fixed(1),
        },
        &mut ctx2,
    );
    let _ = check_and_apply_sbas(&mut state2);
    assert!(
        !state2.objects().contains_key(&plain_id),
        "decoy: a Saga whose chapter III leaves it a Saga at 3+ lore IS sacrificed by CR 714.4"
    );
}

// ── Card-def integration: fable_of_the_mirror_breaker ────────────────────────

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

/// Card integration: Fable of the Mirror-Breaker's chapter III
/// (`Effect::ExileSourceAndReturnTransformed`) returns it to the battlefield as
/// Reflection of Kiki-Jiki, and CR 714.4 does not sacrifice it.
#[test]
fn test_fable_transforms_at_chapter_three() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            real_card_spec(
                p1,
                "Fable of the Mirror-Breaker",
                ZoneId::Battlefield,
                &defs,
            )
            .with_counter(CounterType::Lore, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fable_id = find_by_name(&state, "Fable of the Mirror-Breaker");
    let mut ctx = EffectContext::new(p1, fable_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    let new_id = find_transformed_on_battlefield(&state, p1)
        .expect("Fable's chapter III should return it as Reflection of Kiki-Jiki");
    let chars = calculate_characteristics(&state, new_id).unwrap();
    assert_eq!(chars.name, "Reflection of Kiki-Jiki");
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));

    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        state.objects().contains_key(&new_id),
        "CR 714.4 must NOT sacrifice Reflection of Kiki-Jiki -- it's no longer a Saga"
    );
    assert!(!sba_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == new_id)));
}

// ── Card-def integration: edgar_charmed_groom (full trigger-dispatch path) ───

/// Card integration: Edgar, Charmed Groom's WhenDies trigger
/// (`Effect::ReturnSourceToBattlefieldTransformed`) fires through the REAL
/// production trigger-dispatch pipeline (check_triggers -> flush_pending_triggers
/// -> stack resolution), returning it as Edgar Markov's Coffin IMMEDIATELY --
/// no "at the next end step" delay (the plan's brief was wrong about this; the
/// real oracle text has no such clause, confirmed against cards.sqlite).
#[test]
fn test_edgar_returns_transformed_immediately() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Edgar, Charmed Groom",
            ZoneId::Battlefield,
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let edgar_id = find_by_name(&state, "Edgar, Charmed Groom");

    // Edgar "dies": zone change produces a new (graveyard) ObjectId.
    let (grave_id, _) = test_util::move_object_to_zone(&mut state, edgar_id, ZoneId::Graveyard(p1))
        .expect("move Edgar to graveyard");

    // Drive the real WhenDies dispatch path: check_triggers -> pending_triggers ->
    // flush -> stack resolution (mirrors delayed_triggers.rs's
    // test_exile_until_source_leaves pattern).
    let events = vec![GameEvent::CreatureDied {
        object_id: edgar_id,
        new_grave_id: grave_id,
        controller: p1,
        pre_death_counters: imbl::OrdMap::new(),
        pre_death_power: None,
        pre_death_characteristics: None,
    }];
    let triggers = mtg_engine::rules::abilities::check_triggers(&state, &events);
    assert!(!triggers.is_empty(), "Edgar's WhenDies trigger should fire");
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let _ = mtg_engine::rules::abilities::flush_pending_triggers(&mut state);
    let state = drain_stack(state, &[p1, p2]);

    // Immediate: no "next end step" delay -- returns as soon as the trigger resolves.
    let new_id = find_transformed_on_battlefield(&state, p1)
        .expect("Edgar should return transformed as Edgar Markov's Coffin, immediately");
    let chars = calculate_characteristics(&state, new_id).unwrap();
    assert_eq!(chars.name, "Edgar Markov's Coffin");
    assert_ne!(new_id, edgar_id, "CR 400.7: new ObjectId");
    assert_ne!(
        new_id, grave_id,
        "CR 400.7: new ObjectId (not the graveyard object either)"
    );
    assert!(state.objects()[&new_id].is_transformed);
    assert!(
        state.objects()[&new_id].counters.is_empty(),
        "CR 400.7: no counters carried over onto the returned Coffin"
    );
}

// ── Integrity guard: nicol_bolas_the_ravager / grist_voracious_larva ─────────

/// Integrity guard (OOS-OS4-1): neither `nicol_bolas_the_ravager` nor
/// `grist_voracious_larva` may be `Complete` while the planeswalker-back
/// starting-loyalty gap stands (`CardFace` has no `starting_loyalty`; a
/// return-transformed planeswalker back face would enter with 0 loyalty and die
/// to SBA 704.5i). PB-OS4 leaves both cards unauthored (matching PB-EF5's
/// precedent) -- this test pins their absence from the registry so a future
/// session cannot silently force-flip them without also fixing the loyalty gap.
#[test]
fn test_nicol_bolas_and_grist_not_complete() {
    let defs = defs_map();
    for name in ["Nicol Bolas, the Ravager", "Grist, Voracious Larva"] {
        match defs.get(name) {
            None => {
                // Expected: PB-OS4 leaves these unauthored (loyalty gap, OOS-OS4-1).
            }
            Some(def) => {
                assert!(
                    !def.completeness.is_complete(),
                    "{} must NOT be Complete while the planeswalker-back starting-loyalty gap \
                     (OOS-OS4-1) stands",
                    name
                );
            }
        }
    }
}

// ── Integrity guard: fable_of_the_mirror_breaker is marked partial ───────────

/// Integrity guard: Fable's chapter III (this PB's primitive) is fully wired and
/// correct, but chapter I's token-attached triggered ability and chapter II's
/// "discard up to two, draw that many" are not expressible in the current DSL --
/// the card must be `Partial`, not force-flipped to `Complete`.
#[test]
fn test_fable_marked_partial() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Fable of the Mirror-Breaker")
        .expect("Fable of the Mirror-Breaker should have a CardDefinition");
    assert!(
        matches!(def.completeness, Completeness::Partial(_)),
        "fable_of_the_mirror_breaker should be Partial (chapters I/II residuals) -- not Complete"
    );
}

/// Integrity guard: Edgar, Charmed Groom IS fully expressible and must be Complete.
#[test]
fn test_edgar_marked_complete() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Edgar, Charmed Groom")
        .expect("Edgar, Charmed Groom should have a CardDefinition");
    assert!(
        def.completeness.is_complete(),
        "edgar_charmed_groom should be Complete -- every clause is expressible"
    );
}

//! Tests for PB-LKI-Power: `EffectAmount::SourcePowerAtLastKnownInformation` (CR 603.10a).
//!
//! This primitive unblocks "when ~ dies / leaves the battlefield, [effect] equal to its power"
//! patterns. Without LKI power snapshots, the effect would resolve to 0 because
//! `move_object_to_zone` rebuilds `Characteristics` from printed face (CR 400.7), losing
//! battlefield-only continuous effects (anthems, +1/+1 counters, equipment) per CR 122.2.
//!
//! Implementation: Path A — layer-resolved power snapshot threaded through the trigger pipeline:
//!   `pre_death_power` (sba.rs, via `calculate_characteristics`) →
//!   `PendingTrigger.lki_power` (abilities.rs check_triggers) →
//!   `StackObject.lki_power` (flush_pending_triggers) →
//!   `EffectContext.lki_power` (resolution.rs) →
//!   `resolve_amount` arm (effects/mod.rs → `ctx.lki_power.unwrap_or(0)`).
//!
//! Cards exercised:
//!   - Conclave Mentor (WhenDies: gain life = LKI power)
//!   - Juri, Master of the Revue (WhenDies: deal damage = LKI power to any target)
//!
//! Tests:
//!   (a) Conclave Mentor death trigger gains life = LKI power (boosted, not printed 2).
//!   (b) Juri Master death trigger deals damage = LKI power (boosted, not printed 1).
//!   (c) Discriminating LKI test: graveyard object has printed power (CR 400.7/122.2);
//!       trigger still uses pre-death LKI power correctly.
//!   (d) Hash schema sentinel + variant-discriminant determinism + Option tag-byte encoding.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::stubs::{PendingTrigger, PendingTriggerKind};
use mtg_engine::state::types::CounterType;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    Step, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
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
    while !state.stack_objects().is_empty() {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Build the full card defs map + registry (used by enrich_spec_from_def).
fn build_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

/// Enrich an ObjectSpec from the all_cards registry (resolves triggers, abilities, P/T).
fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

// ── Test (a): Conclave Mentor death trigger gains life = LKI power ────────────

/// CR 603.10a — When Conclave Mentor dies, its controller gains life equal to its power.
///
/// Ruling 2020-06-23: "Use Conclave Mentor's power as it last existed on the battlefield
/// to determine how much life you gain."
///
/// The gained amount must come from LKI (the layer-resolved power snapshot captured before
/// `move_object_to_zone`), NOT the printed power (2). With 2 +1/+1 counters added, the
/// LKI power is 4; the controller must gain exactly 4 life.
#[test]
fn test_conclave_mentor_death_trigger_gains_life_from_lki_power() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    let mentor_spec = enrich(p1, "Conclave Mentor", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mentor_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add 2 +1/+1 counters: Conclave Mentor becomes 4/4 on battlefield (printed 2/2).
    let mentor_id = find_by_name(&state, "Conclave Mentor");
    state
        .objects_mut()
        .get_mut(&mentor_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 2);

    // Record p1's life total before death.
    let life_before = state.players().get(&p1).unwrap().life_total;

    // Mark lethal damage: Conclave Mentor is 2/2 base + 2 counters = 4/4;
    // 4 damage >= 4 toughness → SBA (CR 704.5g) destroys it.
    state
        .objects_mut()
        .get_mut(&mentor_id)
        .unwrap()
        .damage_marked = 4;

    // Grant priority so PassPriority triggers SBA check.
    state.turn_mut().priority_holder = Some(p1);
    let (state, sba_events) = pass_all(state, &[p1, p2]);

    // Verify death occurred.
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 704.5g: CreatureDied must be emitted when lethal damage SBA fires"
    );
    assert!(
        !on_battlefield(&state, "Conclave Mentor"),
        "CR 704.5g: Conclave Mentor must be off the battlefield after lethal damage SBA"
    );

    // Drain the stack — the WhenDies trigger resolves → gain life = LKI power.
    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let life_after = state.players().get(&p1).unwrap().life_total;
    let life_gained = life_after - life_before;
    assert_eq!(
        life_gained, 4,
        "CR 603.10a / Conclave Mentor 2020-06-23 ruling: WhenDies must gain life = \
         LKI power (printed 2 + 2 counters = 4); got {} life. If 2, lki_power was \
         not captured at sba.rs:540 (read printed power instead). If 0, lki_power \
         was not threaded through PendingTrigger → StackObject → EffectContext.",
        life_gained
    );
}

// ── Test (b): Juri Master death trigger deals damage = LKI power ──────────────

/// CR 603.10a — When Juri, Master of the Revue dies, it deals damage equal to its power
/// to any target.
///
/// Ruling 2020-11-10: "For Juri's second ability, use its power from when it was last
/// on the battlefield to determine how much damage is dealt."
///
/// Scenario: Juri (printed 1/1) has 3 +1/+1 counters → 4/4 on battlefield.
/// After death, the WhenDies trigger targets p2; p2 must take exactly 4 damage.
#[test]
fn test_juri_master_death_trigger_deals_damage_from_lki_power() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    let juri_spec = enrich(p1, "Juri, Master of the Revue", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(juri_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add 3 +1/+1 counters: Juri becomes 4/4 on battlefield (printed 1/1).
    let juri_id = find_by_name(&state, "Juri, Master of the Revue");
    state
        .objects_mut()
        .get_mut(&juri_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 3);

    let p2_life_before = state.players().get(&p2).unwrap().life_total;

    // Mark lethal damage: Juri is 1/1 base + 3 counters = 4/4;
    // 4 damage >= 4 toughness → SBA (CR 704.5g) destroys it.
    state.objects_mut().get_mut(&juri_id).unwrap().damage_marked = 4;

    // Grant priority so PassPriority triggers SBA check.
    state.turn_mut().priority_holder = Some(p1);
    let (state, sba_events) = pass_all(state, &[p1, p2]);

    // Verify death occurred.
    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 704.5g: CreatureDied must be emitted when lethal damage SBA fires"
    );
    assert!(
        !on_battlefield(&state, "Juri, Master of the Revue"),
        "CR 704.5g: Juri must be off the battlefield after lethal damage SBA"
    );

    // Drain the stack — WhenDies trigger resolves; engine auto-selects p2 as target.
    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let p2_life_after = state.players().get(&p2).unwrap().life_total;
    let damage_taken = p2_life_before - p2_life_after;
    assert_eq!(
        damage_taken, 4,
        "CR 603.10a / Juri 2020-11-10 ruling: WhenDies trigger must deal damage = \
         LKI power (printed 1 + 3 counters = 4); got {} damage. If 1, lki_power read \
         printed value. If 0, lki_power was None (not threaded).",
        damage_taken
    );
}

// ── Test (c): Discriminating LKI test — graveyard object proves snapshot was pre-death ────

/// CR 603.10a — the power snapshot must be from BEFORE `move_object_to_zone` runs.
/// CR 122.2 — counters cease on zone change (graveyard object has empty counters).
/// CR 400.7 — zone change creates a new object (graveyard characteristics = printed face).
///
/// This test proves the snapshot is pre-death by checking three invariants simultaneously:
///   1. Graveyard Juri has no counters (CR 122.2 — counters cease on zone change).
///   2. Graveyard Juri's printed power == 1 (CR 400.7 — new object, base characteristics).
///   3. p2 took 6 damage from the death trigger (pre-death LKI power = 1 + 5 counters = 6).
///
/// If the engine reads graveyard power instead of LKI snapshot, assertion #3 would fail
/// (would get 1). If no snapshot is captured, #3 fails with 0.
#[test]
fn test_lki_power_resolves_to_pre_death_value_not_printed_value() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    let juri_spec = enrich(p1, "Juri, Master of the Revue", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(juri_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add 5 +1/+1 counters: Juri becomes 6/6 on battlefield (printed 1/1).
    let juri_id = find_by_name(&state, "Juri, Master of the Revue");
    state
        .objects_mut()
        .get_mut(&juri_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 5);

    // Pre-death invariant: counters are present before SBA.
    let pre_death_counter_count = state
        .objects()
        .get(&juri_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        pre_death_counter_count, 5,
        "Pre-condition: Juri must have 5 +1/+1 counters before death"
    );

    let p2_life_before = state.players().get(&p2).unwrap().life_total;

    // Mark lethal damage (6 = toughness with counters).
    state.objects_mut().get_mut(&juri_id).unwrap().damage_marked = 6;

    state.turn_mut().priority_holder = Some(p1);
    let (state, _sba_events) = pass_all(state, &[p1, p2]);

    // Verify Juri is no longer on battlefield.
    assert!(
        !on_battlefield(&state, "Juri, Master of the Revue"),
        "Juri must be off the battlefield after SBA"
    );

    // Find Juri's graveyard copy and verify CR 122.2 / CR 400.7 invariants.
    let grave_id = find_in_zone(&state, "Juri, Master of the Revue", ZoneId::Graveyard(p1))
        .expect("Juri must be in p1's graveyard after dying");

    let grave_obj = &state.objects()[&grave_id];

    // CR 122.2: counters cease on zone change — graveyard object must have empty counters.
    assert!(
        grave_obj.counters.is_empty(),
        "CR 122.2: graveyard Juri must have NO counters (counters cease on zone change). \
         Found {:?}. If non-empty, move_object_to_zone is not resetting counters and the \
         LKI threading may be redundant.",
        grave_obj.counters
    );

    // CR 400.7: graveyard object has printed characteristics.
    assert_eq!(
        grave_obj.characteristics.power,
        Some(1),
        "CR 400.7: graveyard Juri's printed power must be 1; got {:?}. If different, \
         move_object_to_zone is preserving battlefield characteristics.",
        grave_obj.characteristics.power
    );

    // Drain the stack to resolve the WhenDies trigger.
    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let p2_life_after = state.players().get(&p2).unwrap().life_total;
    let damage_taken = p2_life_before - p2_life_after;

    // The damage must reflect LKI (6), NOT printed (1) AND NOT zero.
    assert_eq!(
        damage_taken, 6,
        "CR 603.10a: Juri's WhenDies trigger must deal damage = LKI power (6 = 1 printed + 5 \
         counters), NOT printed (1) and NOT zero. Got {}.",
        damage_taken
    );
}

// ── Test (d): Hash determinism + sentinel + variant-discrimination ────────────

/// Hash schema sentinel, variant-discriminant determinism, and Option tag-byte encoding.
///
/// This test is the mechanical safety net for the PB-LKI-Power implementation.
/// If any of the three sub-assertions fail, the hash infrastructure is broken.
#[test]
fn test_pb_lki_power_hash_schema_version_and_determinism() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;
    use mtg_engine::{CardEffectTarget, EffectAmount};

    // Sub-assertion 1: HASH_SCHEMA_VERSION sentinel.
    assert_eq!(
        HASH_SCHEMA_VERSION, 36u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );

    // Sub-assertion 2: variant-discriminant determinism.
    let h = |a: &EffectAmount| {
        let mut hh = Hasher::new();
        a.hash_into(&mut hh);
        *hh.finalize().as_bytes()
    };

    let lki_a = EffectAmount::SourcePowerAtLastKnownInformation;
    let lki_b = EffectAmount::SourcePowerAtLastKnownInformation;
    let live = EffectAmount::PowerOf(CardEffectTarget::Source);
    let cc = EffectAmount::CounterCountAtLastKnownInformation {
        counter: CounterType::PlusOnePlusOne,
    };

    assert_eq!(
        h(&lki_a),
        h(&lki_b),
        "EffectAmount::SourcePowerAtLastKnownInformation must have deterministic hash"
    );
    assert_ne!(
        h(&lki_a),
        h(&live),
        "EffectAmount::SourcePowerAtLastKnownInformation must be discriminated from \
         EffectAmount::PowerOf(Source)"
    );
    assert_ne!(
        h(&lki_a),
        h(&cc),
        "EffectAmount::SourcePowerAtLastKnownInformation must be discriminated from \
         EffectAmount::CounterCountAtLastKnownInformation"
    );

    // Sub-assertion 3: PendingTrigger.lki_power Option tag-byte encoding.
    // Verifies that Some(0) != None (the generic Option<i32> HashInto uses a tag byte).
    let hp = |t: &PendingTrigger| {
        let mut hh = Hasher::new();
        t.hash_into(&mut hh);
        *hh.finalize().as_bytes()
    };

    let dummy_source = ObjectId(0);
    let trig_none = PendingTrigger {
        lki_power: None,
        ..PendingTrigger::blank(dummy_source, PlayerId(1), PendingTriggerKind::Normal)
    };
    let trig_some_zero = PendingTrigger {
        lki_power: Some(0),
        ..PendingTrigger::blank(dummy_source, PlayerId(1), PendingTriggerKind::Normal)
    };
    let trig_some_one = PendingTrigger {
        lki_power: Some(1),
        ..PendingTrigger::blank(dummy_source, PlayerId(1), PendingTriggerKind::Normal)
    };

    assert_ne!(
        hp(&trig_none),
        hp(&trig_some_zero),
        "PendingTrigger: lki_power None vs Some(0) must produce distinct hashes \
         (Option tag-byte encoding — 0=None, 1=Some). Failure means lki_power is \
         not hashed or the tag byte is missing."
    );
    assert_ne!(
        hp(&trig_some_zero),
        hp(&trig_some_one),
        "PendingTrigger: lki_power Some(0) vs Some(1) must produce distinct hashes"
    );
    assert_ne!(
        hp(&trig_none),
        hp(&trig_some_one),
        "PendingTrigger: lki_power None vs Some(1) must produce distinct hashes"
    );
}

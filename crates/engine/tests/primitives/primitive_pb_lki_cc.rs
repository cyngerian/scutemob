//! Tests for PB-LKI-CC: `EffectAmount::CounterCountAtLastKnownInformation` (CR 603.10a).
//!
//! This primitive unblocks "when ~ dies / leaves the battlefield, do X for each counter on it"
//! patterns. Without LKI counter snapshots, the counter count resolves to 0 because
//! `move_object_to_zone` resets `GameObject.counters` to empty (CR 122.2 / CR 400.7).
//!
//! Implementation: Path A — counter snapshot threaded through the trigger pipeline:
//!   `pre_death_counters` (sba.rs) →
//!   `PendingTrigger.lki_counters` (abilities.rs) →
//!   `StackObject.lki_counters` (flush_pending_triggers) →
//!   `EffectContext.lki_counters` (resolution.rs) →
//!   `resolve_amount` arm (effects/mod.rs).
//!
//! Cards exercised:
//!   - Chasm Skulker (WhenDies: create X Squid tokens, X = +1/+1 counter count)
//!   - Toothy, Imaginary Friend (WhenLeavesBattlefield: draw X cards, X = +1/+1 counter count)
//!
//! Tests:
//!   (a) Chasm Skulker death trigger creates correct token count from LKI (3 counters → 3 Squids).
//!   (b) Toothy leaves-battlefield trigger draws correct card count from LKI (4 counters → 4 draws).
//!   (c) Zero counters → 0 tokens/draws, no panic.
//!   (d) Mixed counter types: only PlusOnePlusOne counters are counted (not Loyalty, etc.).
//!   (e) Hash schema sentinel: HASH_SCHEMA_VERSION == 15 (PB-LKI-CC bump 14→15).

use std::collections::HashMap;
use std::sync::Arc;

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

fn count_objects_named_in_zone(state: &GameState, name: &str, zone: ZoneId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == zone)
        .count()
}

fn count_tokens_named(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.is_token && o.characteristics.name == name)
        .count()
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

// ── Test (a): Chasm Skulker death trigger creates correct Squid tokens ────────

/// CR 603.10a — When Chasm Skulker dies, it creates X 1/1 Squid tokens with
/// islandwalk, where X is the number of +1/+1 counters it had.
///
/// The token count must come from LKI (the snapshot taken before zone transition),
/// NOT from the graveyard object whose counters are reset to empty by
/// `move_object_to_zone` per CR 122.2.
///
/// Scenario: Chasm Skulker has 3 +1/+1 counters, takes lethal damage, dies via
/// the SBA (CR 704.5g). The WhenDies trigger resolves and creates exactly 3 Squid
/// tokens on the battlefield.
#[test]
fn test_chasm_skulker_death_trigger_creates_squid_tokens_from_lki() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    // Library filler so draws from any concurrent trigger don't fail on empty library.
    let lib_card = ObjectSpec::card(p1, "Library Filler").in_zone(ZoneId::Library(p1));

    let skulker_spec = enrich(p1, "Chasm Skulker", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(skulker_spec)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add 3 +1/+1 counters to Chasm Skulker (simulates draw-trigger accumulation).
    let skulker_id = find_by_name(&state, "Chasm Skulker");
    state
        .objects_mut()
        .get_mut(&skulker_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 3);

    // Mark lethal damage: Chasm Skulker is 1/1 base + 3 counters = 4/4;
    // 4 damage >= 4 toughness → SBA (CR 704.5g) destroys it.
    state
        .objects_mut()
        .get_mut(&skulker_id)
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
        !on_battlefield(&state, "Chasm Skulker"),
        "CR 704.5g: Chasm Skulker must be off the battlefield after lethal damage SBA"
    );

    // Drain the stack — the WhenDies trigger resolves → 3 Squid tokens created.
    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let squid_count = count_tokens_named(&state, "Squid");
    assert_eq!(
        squid_count, 3,
        "CR 603.10a: WhenDies trigger must read LKI counter count (3) and create 3 Squid \
         tokens; got {} tokens. If 0, lki_counters were not threaded through the trigger pipeline.",
        squid_count
    );

    // All Squid tokens should be on the battlefield.
    let bf_squids = count_objects_named_in_zone(&state, "Squid", ZoneId::Battlefield);
    assert_eq!(
        bf_squids, 3,
        "All 3 Squid tokens should be on the battlefield"
    );
}

// ── Test (b): Toothy leaves-battlefield trigger draws cards from LKI ──────────

/// CR 603.10a — When Toothy, Imaginary Friend leaves the battlefield, its
/// controller draws a card for each +1/+1 counter that was on it.
///
/// This is the regression sentinel: the trigger MUST use
/// `EffectAmount::CounterCountAtLastKnownInformation` not `EffectAmount::CounterCount`
/// (which resolves to 0 post-move because CR 122.2 resets counters).
///
/// Scenario: Toothy has 4 +1/+1 counters and dies via lethal damage. After the
/// WhenLeavesBattlefield trigger resolves, P1's hand should contain 4 more cards
/// than before Toothy died.
#[test]
fn test_toothy_leaves_battlefield_draws_cards_from_lki_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    // Build a library with 5 cards so the draws don't fail on empty library.
    let lib_cards: Vec<ObjectSpec> = (0..5)
        .map(|i| ObjectSpec::card(p1, &format!("Library Card {i}")).in_zone(ZoneId::Library(p1)))
        .collect();

    let toothy_spec = enrich(p1, "Toothy, Imaginary Friend", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(toothy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib_card in lib_cards {
        builder = builder.object(lib_card);
    }
    let mut state = builder.build().unwrap();

    // Add 4 +1/+1 counters to Toothy.
    let toothy_id = find_by_name(&state, "Toothy, Imaginary Friend");
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 4);

    // Record hand size before Toothy dies.
    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Mark lethal damage: Toothy is 1/1 + 4 counters = 5/5; 5 damage >= 5 toughness.
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .damage_marked = 5;

    state.turn_mut().priority_holder = Some(p1);
    let (state, sba_events) = pass_all(state, &[p1, p2]);

    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 704.5g: CreatureDied must fire for Toothy"
    );
    assert!(
        !on_battlefield(&state, "Toothy, Imaginary Friend"),
        "Toothy must leave the battlefield"
    );

    // Drain the stack — WhenLeavesBattlefield trigger resolves → draw 4 cards.
    let (state, draw_events) = drain_stack(state, &[p1, p2]);

    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let cards_drawn = hand_after.saturating_sub(hand_before);
    assert_eq!(
        cards_drawn, 4,
        "CR 603.10a: Toothy WhenLeavesBattlefield trigger must draw 4 cards (= 4 LKI \
         +1/+1 counters); drew {} cards. If 0, EffectAmount::CounterCountAtLastKnownInformation \
         is not resolving from lki_counters correctly.",
        cards_drawn
    );

    // CardDrawn events should have been emitted for p1 (4 separate draws).
    let draw_count = draw_events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        draw_count, 4,
        "CR 603.10a: exactly 4 CardDrawn events expected for P1; got {}",
        draw_count
    );
}

// ── Test (c): Zero counters → 0 tokens, no panic ─────────────────────────────

/// CR 603.10a / CR 608.2h — When the source had 0 +1/+1 counters at death,
/// `CounterCountAtLastKnownInformation` resolves to 0. The effect executes
/// normally: 0 tokens created, no panic, no negative-count truncation.
///
/// Defensive regression: ensures `lki_counters` with no entry for the requested
/// counter type falls through to `unwrap_or(0)` cleanly.
#[test]
fn test_lki_counter_count_zero_counters_returns_zero_no_panic() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    // Chasm Skulker with NO counters.
    let skulker_spec = enrich(p1, "Chasm Skulker", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(skulker_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Chasm Skulker has 0 counters. Mark lethal damage on the 1/1 (1 >= 1 toughness).
    let skulker_id = find_by_name(&state, "Chasm Skulker");
    state
        .objects_mut()
        .get_mut(&skulker_id)
        .unwrap()
        .damage_marked = 1;

    state.turn_mut().priority_holder = Some(p1);
    let (state, _sba_events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Chasm Skulker"),
        "Chasm Skulker must die from 1 damage (1/1 creature)"
    );

    // Drain the WhenDies trigger — should create 0 Squid tokens (no panic).
    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let squid_count = count_tokens_named(&state, "Squid");
    assert_eq!(
        squid_count, 0,
        "CR 603.10a / CR 608.2h: 0 counters at death → 0 tokens; no panic expected. Got {}",
        squid_count
    );
}

// ── Test (d): Mixed counter types — only PlusOnePlusOne counted ───────────────

/// CR 603.10a / CR 122.2 — `CounterCountAtLastKnownInformation` is parameterized
/// by `counter: CounterType`. It must read ONLY the specified counter type from
/// the LKI snapshot, ignoring all other counter types.
///
/// Scenario: Chasm Skulker has 2 +1/+1 counters AND 5 Loyalty counters at death.
/// The WhenDies trigger specifies `counter: PlusOnePlusOne` → must create exactly
/// 2 Squid tokens, not 7 (sum) or 5 (wrong type).
#[test]
fn test_lki_counter_count_multi_type_returns_requested_counter_type_only() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_registry();

    let skulker_spec = enrich(p1, "Chasm Skulker", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(skulker_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let skulker_id = find_by_name(&state, "Chasm Skulker");
    // Add 2 +1/+1 counters AND 5 Loyalty counters to verify type discrimination.
    {
        let obj = state.objects_mut().get_mut(&skulker_id).unwrap();
        obj.counters.insert(CounterType::PlusOnePlusOne, 2);
        obj.counters.insert(CounterType::Loyalty, 5);
    }

    // Chasm Skulker is now 3/3 (1+2/1+2 from counters). Mark 3 damage (lethal).
    state
        .objects_mut()
        .get_mut(&skulker_id)
        .unwrap()
        .damage_marked = 3;

    state.turn_mut().priority_holder = Some(p1);
    let (state, _sba_events) = pass_all(state, &[p1, p2]);

    assert!(
        !on_battlefield(&state, "Chasm Skulker"),
        "Chasm Skulker (3/3 with 3 damage) must die"
    );

    let (state, _trigger_events) = drain_stack(state, &[p1, p2]);

    let squid_count = count_tokens_named(&state, "Squid");
    assert_eq!(
        squid_count, 2,
        "CR 603.10a: CounterCountAtLastKnownInformation {{PlusOnePlusOne}} must count only \
         +1/+1 counters (2), not Loyalty (5) or total (7). Got {}",
        squid_count
    );
}

// ── Test (e): HASH_SCHEMA_VERSION sentinel ────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. See the `state/hash.rs` history block.
#[test]
fn test_pb_lki_cc_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 58u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this assertion and the state/hash.rs history block together; the authoritative check is the SR-17 machine gate in tests/core/hash_schema.rs."
    );
}

/// CR N/A (hash infrastructure) — Hash determinism sub-test:
/// identical game states must produce the same hash; different states must not.
///
/// Specifically: a state where Toothy just died (with LKI snapshot on PendingTrigger)
/// must produce a different hash from the initial state with Toothy alive.
#[test]
fn test_pb_lki_cc_hash_determinism() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let lib_cards: Vec<ObjectSpec> = (0..5)
        .map(|i| ObjectSpec::card(p1, &format!("Hash Det Card {i}")).in_zone(ZoneId::Library(p1)))
        .collect();
    let toothy_spec = enrich(p1, "Toothy, Imaginary Friend", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(toothy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib_card in lib_cards {
        builder = builder.object(lib_card);
    }
    let mut state = builder.build().unwrap();

    // Add 2 +1/+1 counters to Toothy.
    let toothy_id = find_by_name(&state, "Toothy, Imaginary Friend");
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 2);

    // Hash of initial state (Toothy alive, 2 counters).
    let hash_initial = state.public_state_hash();

    // Two identical states must hash the same.
    let hash_initial_again = state.public_state_hash();
    assert_eq!(
        hash_initial, hash_initial_again,
        "CR N/A: identical states must produce identical hashes (determinism)"
    );

    // Kill Toothy — state changes.
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .damage_marked = 3;
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    let hash_after_death = state.public_state_hash();
    assert_ne!(
        hash_initial, hash_after_death,
        "CR N/A: state after Toothy dies must produce a different hash from initial state"
    );
}

// ── Fix-phase regression tests (E1): LKI on non-death leaves-battlefield ──────

/// CR 603.10a — Mandatory E1 regression test.
///
/// Toothy with 3 +1/+1 counters is BOUNCED to its owner's hand via
/// `Effect::BounceAll`. The `WhenLeavesBattlefield` trigger must read
/// the LKI counter snapshot from `ObjectReturnedToHand.pre_lba_counters`
/// (populated by the fix), not from the now-reset `counters` field on the
/// hand object. P1 must draw exactly 3 cards.
///
/// Before E1 fix: `ObjectReturnedToHand` arm in `check_triggers` used
/// `..PendingTrigger::blank(...)` which defaulted `lki_counters` to empty,
/// producing 0 draws regardless of counter count.
#[test]
fn test_toothy_bounced_to_hand_draws_lki_counter_count() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
    use mtg_engine::{CardType, Effect, ObjectId, TargetFilter};

    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let lib_cards: Vec<ObjectSpec> = (0..5)
        .map(|i| ObjectSpec::card(p1, &format!("Bounce Lib {i}")).in_zone(ZoneId::Library(p1)))
        .collect();
    let toothy_spec = enrich(p1, "Toothy, Imaginary Friend", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(toothy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib_card in lib_cards {
        builder = builder.object(lib_card);
    }
    let mut state = builder.build().unwrap();

    // Add 3 +1/+1 counters to Toothy.
    let toothy_id = find_by_name(&state, "Toothy, Imaginary Friend");
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 3);

    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Bounce all creatures — Toothy is the only creature on the battlefield.
    let mut ctx = EffectContext::new(p1, ObjectId(0), vec![]);
    let bounce_events = execute_effect(
        &mut state,
        &Effect::BounceAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            max_toughness_amount: None,
        },
        &mut ctx,
    );

    // Verify Toothy bounced to hand.
    assert!(
        bounce_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectReturnedToHand { .. })),
        "BounceAll must emit ObjectReturnedToHand for Toothy"
    );
    assert!(
        !on_battlefield(&state, "Toothy, Imaginary Friend"),
        "Toothy must be off the battlefield after BounceAll"
    );

    // Fire check_triggers on the bounce events, queue resulting triggers.
    let triggers = check_triggers(&state, &bounce_events);
    assert!(
        !triggers.is_empty(),
        "WhenLeavesBattlefield trigger must be queued after Toothy bounces"
    );
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }

    // Flush pending triggers onto the stack (sets priority).
    let flush_events = flush_pending_triggers(&mut state);
    assert!(
        !flush_events.is_empty(),
        "flush_pending_triggers must emit events (trigger pushed to stack)"
    );

    // Drain the stack — WhenLeavesBattlefield trigger resolves → draw 3 cards.
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Note: Toothy itself is now in hand (the bounced card), so we subtract 1 for that.
    // hand_before + 1 (Toothy bounced) + 3 (draws) = hand_after.
    let cards_drawn = hand_after.saturating_sub(hand_before + 1); // +1 for Toothy itself
    assert_eq!(
        cards_drawn, 3,
        "CR 603.10a (E1 regression): Toothy bounced with 3 counters must draw 3 cards; \
         drew {} cards. If 0, pre_lba_counters was not threaded from \
         ObjectReturnedToHand through check_triggers → PendingTrigger.lki_counters.",
        cards_drawn
    );
}

/// CR 603.10a — Toothy with 3 +1/+1 counters is DESTROYED by a non-SBA path
/// (direct `Effect::DestroyAll`). The `WhenLeavesBattlefield` trigger must use
/// `PermanentDestroyed.pre_lba_counters` to draw 3 cards.
///
/// This tests the `PermanentDestroyed` arm in `check_triggers` (distinct from
/// the `CreatureDied` arm tested by (b)).
#[test]
fn test_toothy_destroyed_draws_lki_counter_count() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
    use mtg_engine::{CardType, Effect, ObjectId, TargetFilter};

    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let lib_cards: Vec<ObjectSpec> = (0..5)
        .map(|i| ObjectSpec::card(p1, &format!("Destroy Lib {i}")).in_zone(ZoneId::Library(p1)))
        .collect();
    let toothy_spec = enrich(p1, "Toothy, Imaginary Friend", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(toothy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib_card in lib_cards {
        builder = builder.object(lib_card);
    }
    let mut state = builder.build().unwrap();

    // Add 3 +1/+1 counters to Toothy.
    let toothy_id = find_by_name(&state, "Toothy, Imaginary Friend");
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 3);

    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Destroy all creatures — Toothy is the only creature.
    let mut ctx = EffectContext::new(p1, ObjectId(0), vec![]);
    let destroy_events = execute_effect(
        &mut state,
        &Effect::DestroyAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            cant_be_regenerated: false,
        },
        &mut ctx,
    );

    // DestroyAll on a creature emits CreatureDied (the SBA path handled separately);
    // via the effect path it emits PermanentDestroyed for indestructible filtering.
    // Either event type should fire the LTB trigger.
    let died = destroy_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CreatureDied { .. } | GameEvent::PermanentDestroyed { .. }
        )
    });
    assert!(
        died,
        "DestroyAll must emit a death or destroy event for Toothy"
    );
    assert!(
        !on_battlefield(&state, "Toothy, Imaginary Friend"),
        "Toothy must be off the battlefield after DestroyAll"
    );

    // Fire check_triggers and drain stack.
    let triggers = check_triggers(&state, &destroy_events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let _ = flush_pending_triggers(&mut state);
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let cards_drawn = hand_after.saturating_sub(hand_before);
    assert_eq!(
        cards_drawn, 3,
        "CR 603.10a: Toothy destroyed with 3 counters must draw 3 cards; drew {}. \
         If 0, pre_lba_counters not threaded from destroy event through trigger pipeline.",
        cards_drawn
    );
}

/// CR 603.10a — Toothy with 3 +1/+1 counters is EXILED by `Effect::ExileAll`.
/// The `WhenLeavesBattlefield` trigger must use `ObjectExiled.pre_lba_counters`
/// to draw 3 cards.
///
/// This tests the `ObjectExiled` arm in `check_triggers` (distinct from
/// the `CreatureDied` arm tested by (b)).
#[test]
fn test_toothy_exiled_draws_lki_counter_count() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
    use mtg_engine::{CardType, Effect, ObjectId, TargetFilter};

    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let lib_cards: Vec<ObjectSpec> = (0..5)
        .map(|i| ObjectSpec::card(p1, &format!("Exile Lib {i}")).in_zone(ZoneId::Library(p1)))
        .collect();
    let toothy_spec = enrich(p1, "Toothy, Imaginary Friend", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(toothy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib_card in lib_cards {
        builder = builder.object(lib_card);
    }
    let mut state = builder.build().unwrap();

    // Add 3 +1/+1 counters to Toothy.
    let toothy_id = find_by_name(&state, "Toothy, Imaginary Friend");
    state
        .objects_mut()
        .get_mut(&toothy_id)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 3);

    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Exile all creatures — Toothy is the only creature.
    let mut ctx = EffectContext::new(p1, ObjectId(0), vec![]);
    let exile_events = execute_effect(
        &mut state,
        &Effect::ExileAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
        },
        &mut ctx,
    );

    assert!(
        exile_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { .. })),
        "ExileAll must emit ObjectExiled for Toothy"
    );
    assert!(
        !on_battlefield(&state, "Toothy, Imaginary Friend"),
        "Toothy must be off the battlefield after ExileAll"
    );

    // Fire check_triggers and drain stack.
    let triggers = check_triggers(&state, &exile_events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let _ = flush_pending_triggers(&mut state);
    state.turn_mut().priority_holder = Some(p1);
    let (state, _) = drain_stack(state, &[p1, p2]);

    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let cards_drawn = hand_after.saturating_sub(hand_before);
    assert_eq!(
        cards_drawn, 3,
        "CR 603.10a: Toothy exiled with 3 counters must draw 3 cards; drew {}. \
         If 0, pre_lba_counters not threaded from ObjectExiled through trigger pipeline.",
        cards_drawn
    );
}

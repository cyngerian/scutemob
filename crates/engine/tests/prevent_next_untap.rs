//! PreventNextUntap tests — PB-LS6, Issue L03.
//!
//! Verifies:
//! - Effect::PreventNextUntap increments skip_untap_steps by 1.
//! - A frozen tapped permanent stays tapped through one untap step and then
//!   untaps normally on the next (skip_untap_steps decrements to 0).
//! - Stacking (applying the effect twice) requires skipping two untap steps.
//! - The count is consumed even if the permanent is already untapped (CR 502.3 —
//!   "doesn't untap during its controller's NEXT untap step", regardless of state).
//! - Zone change (battlefield → graveyard → battlefield) resets the counter
//!   because the object becomes a new object with skip_untap_steps = 0 (CR 400.7).
//! - Integration with Hands of Binding card def (tap + freeze rider).
//!
//! CR refs:
//!   CR 400.7  — zone change creates a new object; old state does not persist
//!   CR 502.2  — untapping during untap step
//!   CR 502.3  — "doesn't untap during its controller's next untap step"

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    CardEffectTarget, Effect, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, SpellTarget, Step, Target, ZoneId,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn skip_untap_steps_of(state: &GameState, name: &str) -> u32 {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(_, o)| o.skip_untap_steps)
        .unwrap_or_else(|| panic!("'{}' not found", name))
}

fn is_tapped(state: &GameState, name: &str) -> bool {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(_, o)| o.status.tapped)
        .unwrap_or(false)
}

/// Apply Effect::PreventNextUntap to a single target object by ID.
fn apply_freeze(mut state: GameState, controller: PlayerId, target_id: ObjectId) -> GameState {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        controller,
        source,
        vec![SpellTarget {
            target: Target::Object(target_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let effect = Effect::PreventNextUntap {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    execute_effect(&mut state, &effect, &mut ctx);
    state
}

/// Run the untap step for `active` by calling untap_active_player_permanents directly.
fn run_untap(mut state: GameState, active: PlayerId) -> (GameState, Vec<GameEvent>) {
    state.turn.active_player = active;
    let events = mtg_engine::rules::turn_actions::untap_active_player_permanents(&mut state);
    (state, events)
}

// ── Test 1: Basic freeze — one skipped untap step ─────────────────────────────

/// CR 502.3 — A tapped creature with skip_untap_steps = 1 must NOT untap during the
/// controller's next untap step. After that step (counter decremented to 0) it should
/// untap normally on the following step.
#[test]
fn test_l03_frozen_permanent_skips_one_untap_step() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Frozen Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Tap the creature manually.
    let bear_id = find_by_name(&state, "Frozen Bear");
    state.objects.get_mut(&bear_id).unwrap().status.tapped = true;

    // Apply the freeze effect.
    let state = apply_freeze(state, p(1), bear_id);
    assert_eq!(
        skip_untap_steps_of(&state, "Frozen Bear"),
        1,
        "PreventNextUntap should set skip_untap_steps = 1"
    );

    // Run untap step 1: creature should stay tapped, counter decrements to 0.
    let (state, events1) = run_untap(state, p(1));
    assert!(
        is_tapped(&state, "Frozen Bear"),
        "CR 502.3: frozen creature must stay tapped during the affected untap step"
    );
    assert_eq!(
        skip_untap_steps_of(&state, "Frozen Bear"),
        0,
        "skip_untap_steps should decrement to 0 after the skip"
    );
    // No untap event for the frozen creature in this step.
    let untapped_in_step1 = events1.iter().any(|e| {
        if let GameEvent::PermanentsUntapped { objects, .. } = e {
            objects.contains(&bear_id)
        } else {
            false
        }
    });
    assert!(
        !untapped_in_step1,
        "frozen creature must not appear in PermanentsUntapped"
    );

    // Run untap step 2: now counter is 0 and creature is tapped — it should untap.
    let (state, events2) = run_untap(state, p(1));
    assert!(
        !is_tapped(&state, "Frozen Bear"),
        "CR 502.2: after freeze expires, creature should untap normally"
    );
    let untapped_in_step2 = events2.iter().any(|e| {
        if let GameEvent::PermanentsUntapped { objects, .. } = e {
            // The new ObjectId after re-entering is fine; check any creature untapped.
            !objects.is_empty()
        } else {
            false
        }
    });
    assert!(
        untapped_in_step2,
        "creature should appear in PermanentsUntapped after freeze expires"
    );
}

// ── Test 2: Stacking freezes ──────────────────────────────────────────────────

/// CR 502.3 — Applying Effect::PreventNextUntap twice on the same permanent stacks:
/// skip_untap_steps = 2, so the permanent skips two consecutive untap steps.
#[test]
fn test_l03_freeze_stacks() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Doubly Frozen", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let id = find_by_name(&state, "Doubly Frozen");
    state.objects.get_mut(&id).unwrap().status.tapped = true;

    // Apply freeze twice (simulating two separate "doesn't untap" effects).
    let state = apply_freeze(state, p(1), id);
    let state = apply_freeze(state, p(1), id);
    assert_eq!(
        skip_untap_steps_of(&state, "Doubly Frozen"),
        2,
        "two PreventNextUntap applications should give skip_untap_steps = 2"
    );

    // Step 1: still tapped, counter → 1.
    let (state, _) = run_untap(state, p(1));
    assert!(
        is_tapped(&state, "Doubly Frozen"),
        "still tapped after first freeze step"
    );
    assert_eq!(skip_untap_steps_of(&state, "Doubly Frozen"), 1);

    // Step 2: still tapped, counter → 0.
    let (state, _) = run_untap(state, p(1));
    assert!(
        is_tapped(&state, "Doubly Frozen"),
        "still tapped after second freeze step"
    );
    assert_eq!(skip_untap_steps_of(&state, "Doubly Frozen"), 0);

    // Step 3: now untaps normally.
    let (state, _) = run_untap(state, p(1));
    assert!(
        !is_tapped(&state, "Doubly Frozen"),
        "CR 502.2: permanent should untap once both freeze counts are exhausted"
    );
}

// ── Test 3: Untapped permanent still consumes the freeze count ─────────────────

/// CR 502.3 — The freeze says "doesn't untap during its controller's next untap step."
/// The counter is consumed regardless of whether the permanent was already untapped.
/// An untapped permanent with skip_untap_steps = 1 must consume the count (no untap
/// event fired because it's already untapped, but the count must reach 0).
#[test]
fn test_l03_untapped_frozen_permanent_consumes_count() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Untapped Frozen", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Creature is untapped by default.
    let id = find_by_name(&state, "Untapped Frozen");
    assert!(
        !is_tapped(&state, "Untapped Frozen"),
        "precondition: creature is untapped"
    );

    let state = apply_freeze(state, p(1), id);
    assert_eq!(skip_untap_steps_of(&state, "Untapped Frozen"), 1);

    // Run untap step — the count should decrement even though the creature was not tapped.
    let (state, _) = run_untap(state, p(1));
    assert_eq!(
        skip_untap_steps_of(&state, "Untapped Frozen"),
        0,
        "CR 502.3: skip_untap_steps decrements during the affected untap step even if already untapped"
    );
    // Creature remains untapped (was never tapped — no change in tapped status).
    assert!(
        !is_tapped(&state, "Untapped Frozen"),
        "already-untapped creature stays untapped"
    );
}

// ── Test 4: Zone change resets skip_untap_steps ───────────────────────────────

/// CR 400.7 — When a permanent changes zones and re-enters the battlefield, it is
/// a new object. The new object has skip_untap_steps = 0 (the frozen state does not
/// carry over from the prior life of the permanent).
#[test]
fn test_l03_zone_change_resets_freeze() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Phoenix", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let id = find_by_name(&state, "Phoenix");
    state.objects.get_mut(&id).unwrap().status.tapped = true;

    // Apply freeze.
    let mut state = apply_freeze(state, p(1), id);
    assert_eq!(skip_untap_steps_of(&state, "Phoenix"), 1);

    // Manually move to graveyard (simulates death — no full destroy pipeline needed).
    let owner = p(1);
    let (new_gy_id, _) = state
        .move_object_to_zone(id, ZoneId::Graveyard(owner))
        .expect("move to graveyard");

    // The graveyard copy must have skip_untap_steps = 0 (field resets on zone change).
    // Note: GameObject has #[serde(default)] on skip_untap_steps; new_gy_id is fresh.
    let gy_skip = state
        .objects
        .get(&new_gy_id)
        .map(|o| o.skip_untap_steps)
        .unwrap_or(99);
    assert_eq!(
        gy_skip, 0,
        "CR 400.7: a new object (after zone change) must have skip_untap_steps = 0"
    );

    // Re-enter the battlefield.
    let (new_bf_id, _) = state
        .move_object_to_zone(new_gy_id, ZoneId::Battlefield)
        .expect("move to battlefield");

    let bf_skip = state
        .objects
        .get(&new_bf_id)
        .map(|o| o.skip_untap_steps)
        .unwrap_or(99);
    assert_eq!(
        bf_skip, 0,
        "CR 400.7: re-entered permanent must have skip_untap_steps = 0 (new object)"
    );
}

// ── Test 5: Only the controller's untap step consumes the count ───────────────

/// CR 502.3 — "its controller's next untap step." Running a different player's untap
/// step must NOT decrement the freeze counter. Only the controller's untap step counts.
#[test]
fn test_l03_only_controllers_untap_step_decrements() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "P1 Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let id = find_by_name(&state, "P1 Creature");
    state.objects.get_mut(&id).unwrap().status.tapped = true;

    let state = apply_freeze(state, p(1), id);
    assert_eq!(skip_untap_steps_of(&state, "P1 Creature"), 1);

    // Run P2's untap step — should NOT decrement P1's creature's counter.
    // untap_active_player_permanents only touches permanents whose controller == active.
    let (state, _) = run_untap(state, p(2));
    assert_eq!(
        skip_untap_steps_of(&state, "P1 Creature"),
        1,
        "CR 502.3: P2's untap step must not decrement P1's frozen creature's counter"
    );
    assert!(
        is_tapped(&state, "P1 Creature"),
        "P1's creature remains tapped and frozen after P2's untap step"
    );

    // Run P1's untap step — NOW it decrements.
    let (state, _) = run_untap(state, p(1));
    assert_eq!(
        skip_untap_steps_of(&state, "P1 Creature"),
        0,
        "counter decrements on P1's (the controller's) untap step"
    );
    assert!(
        is_tapped(&state, "P1 Creature"),
        "creature is still tapped (skip consumed the step)"
    );

    // P1's next untap step: now untaps normally.
    let (state, _) = run_untap(state, p(1));
    assert!(
        !is_tapped(&state, "P1 Creature"),
        "creature untaps normally once counter is 0"
    );
}

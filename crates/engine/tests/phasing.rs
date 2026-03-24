//! Phasing keyword ability tests (CR 702.26).
//!
//! Phasing is a static ability that modifies the rules of the untap step:
//! - Before untapping, all phased-in permanents with Phasing that the active player
//!   controls phase out. (CR 702.26a)
//! - All phased-out permanents that had phased out under that player's control phase in.
//!   (CR 702.26a)
//! - Phasing is NOT a zone change (CR 702.26d) -- no ETB triggers, object identity preserved.
//! - Phased-out permanents are treated as if they do not exist (CR 702.26b).
//!
//! Key rules verified:
//! - CR 702.26a: Phase-out occurs before untap in the untap step.
//! - CR 702.26a: Phase-in occurs before phase-out in the untap step.
//! - CR 702.26d: Phasing does not cause zone-change triggers (no ETB on phase-in).
//! - CR 702.26d: Object identity (ObjectId) is preserved through phasing.
//! - CR 702.26d: Counters persist through phasing.
//! - CR 702.26g: Attached Auras/Equipment phase out indirectly with their host.
//! - CR 702.26g: Indirectly-phased permanents phase in together with their host.
//! - CR 702.26b: Phased-out permanents are excluded from SBAs.
//! - CR 702.26b: Phased-out creatures cannot attack or block.
//! - CR 702.26e: Phased-out permanents are excluded from continuous effects.
//! - CR 702.26d: Tokens with Phasing survive phase-out (not destroyed by CR 704.5d).
//! - CR 702.26a: Multiplayer controller tracking -- phase-in only on correct player's untap.

use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command,
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId, Step,
    ZoneId,
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

fn is_phased_out(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.status.phased_out)
}

fn is_phased_out_indirectly(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.phased_out_indirectly)
}

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
}

/// Advance all priority passes until the step changes.
/// Used to advance from one step to the next (or through auto-advance steps).
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

/// Advance from a step-with-priority all the way to the next step that grants priority.
///
/// Pattern: start at p2's End step with active_player=p2.
/// Passing all → Cleanup (auto) → p1's Untap (TBAs fire) → p1's Upkeep (priority).
/// Returns the state after phasing has occurred (at p1's Upkeep).
fn advance_to_p1_upkeep_via_end_step(state: GameState, players: &[PlayerId]) -> GameState {
    let (state, _) = pass_all(state, players);
    state
}

// ── Test 1: Basic phase-out on untap step ─────────────────────────────────────

#[test]
/// CR 702.26a -- Permanents with Phasing phase out at the start of their controller's untap step.
fn test_phasing_basic_phase_out_on_untap() {
    let p1 = p(1);
    let p2 = p(2);

    // Start at p2's End step. After passing, p1's Untap fires.
    // p1's creature with Phasing should phase out.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Phasing Creature", 2, 2)
                .with_keyword(KeywordAbility::Phasing),
        )
        .build()
        .unwrap();

    // Confirm creature is phased in initially.
    assert!(
        !is_phased_out(&state, "Phasing Creature"),
        "creature should start phased in"
    );

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // After p1's untap step: creature with Phasing should be phased out.
    assert!(
        is_phased_out(&state, "Phasing Creature"),
        "CR 702.26a: creature with Phasing should phase out at the start of controller's untap step"
    );
}

// ── Test 2: Basic phase-in on next untap step ─────────────────────────────────

#[test]
/// CR 702.26a -- A phased-out permanent phases in at the start of its controller's next untap step.
fn test_phasing_basic_phase_in_on_next_untap() {
    let p1 = p(1);
    let p2 = p(2);

    // Start at p2's End step. p1's creature is already phased out.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Phased-Out Creature", 2, 2)
                .with_keyword(KeywordAbility::Phasing),
        )
        .build()
        .unwrap();

    // Manually set phased-out status (simulating previous turn's phase-out).
    let obj_id = find_object(&state, "Phased-Out Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
        // phased_out_indirectly = false (direct phase-out)
    }

    assert!(
        is_phased_out(&state, "Phased-Out Creature"),
        "creature should start phased out"
    );

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // CR 502.1: Phase-in and phase-out happen SIMULTANEOUSLY. The snapshot for
    // phase-out is taken BEFORE phase-in mutations occur, so a creature that is
    // phased-out when the snapshot is taken will NOT appear in the phase-out set.
    // Result: the creature phases IN and does NOT phase out in the same step.
    // On the NEXT untap step, it will phase out again (because it has Phasing).
    assert!(
        !is_phased_out(&state, "Phased-Out Creature"),
        "CR 502.1: simultaneous phasing -- creature phases in and does NOT also phase out in the same step"
    );
}

// ── Test 2b: Phase-in of a creature WITHOUT Phasing (was phased out by external effect) ─────

#[test]
/// CR 702.26a -- A phased-out permanent without the Phasing keyword still phases in
/// on the controlling player's untap step (phase-in doesn't require the Phasing keyword).
fn test_phasing_phase_in_without_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    // Creature without Phasing keyword but manually phased out.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "Plain Creature", 2, 2))
        .build()
        .unwrap();

    // Manually set phased-out status (as if phased out by "Teferi's Protection" or similar).
    let obj_id = find_object(&state, "Plain Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    assert!(
        is_phased_out(&state, "Plain Creature"),
        "creature should start phased out"
    );

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // After p1's untap: creature should have phased in (it has no Phasing keyword,
    // so it doesn't phase out again).
    assert!(
        !is_phased_out(&state, "Plain Creature"),
        "CR 702.26a: phased-out permanent should phase in on controller's untap step"
    );
    assert!(
        is_on_battlefield(&state, "Plain Creature"),
        "creature should be on battlefield after phasing in"
    );
}

// ── Test 3: Object identity preserved through phasing ─────────────────────────

#[test]
/// CR 702.26d -- Phasing does not cause a zone change; ObjectId is preserved.
/// Counters are also preserved while phased out.
fn test_phasing_no_zone_change_preserves_object_id_and_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Counter Creature", 2, 2)
                .with_keyword(KeywordAbility::Phasing),
        )
        .build()
        .unwrap();

    let obj_id_before = find_object(&state, "Counter Creature");

    // Add a +1/+1 counter to the creature before phasing.
    if let Some(obj) = state.objects.get_mut(&obj_id_before) {
        obj.counters = obj
            .counters
            .update(mtg_engine::CounterType::PlusOnePlusOne, 1);
    }

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // After phasing out: object should still exist with same id and counters.
    let obj_id_after = find_object(&state, "Counter Creature");
    assert_eq!(
        obj_id_before, obj_id_after,
        "CR 702.26d: ObjectId must be preserved through phasing (no new object created)"
    );

    // Zone is still Battlefield (phased-out objects remain in their zone).
    let obj = state.objects.get(&obj_id_after).unwrap();
    assert_eq!(
        obj.zone,
        ZoneId::Battlefield,
        "CR 702.26d: phased-out permanent is still in Battlefield zone"
    );

    // Counter is preserved.
    let counter_val = obj
        .counters
        .get(&mtg_engine::CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_val, 1,
        "CR 702.26d: counters are preserved while phased out"
    );
}

// ── Test 4: Indirect phasing -- Aura phases out with host ─────────────────────

#[test]
/// CR 702.26g -- When a permanent phases out, attached Auras/Equipment phase out indirectly.
fn test_phasing_indirect_aura_phases_out_with_host() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Phasing Host", 2, 2).with_keyword(KeywordAbility::Phasing),
        )
        .object(ObjectSpec::card(p1, "Attached Aura"))
        .build()
        .unwrap();

    // Manually attach the Aura to the creature.
    let host_id = find_object(&state, "Phasing Host");
    let aura_id = find_object(&state, "Attached Aura");
    if let Some(host) = state.objects.get_mut(&host_id) {
        host.attachments = vec![aura_id].into_iter().collect();
    }
    if let Some(aura) = state.objects.get_mut(&aura_id) {
        aura.attached_to = Some(host_id);
    }

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // Host should be phased out (directly).
    assert!(
        is_phased_out(&state, "Phasing Host"),
        "CR 702.26g: host with Phasing should phase out"
    );
    // Aura should also be phased out (indirectly).
    assert!(
        is_phased_out(&state, "Attached Aura"),
        "CR 702.26g: Aura attached to phasing host should phase out indirectly"
    );
    // Aura's phased_out_indirectly flag should be set.
    assert!(
        is_phased_out_indirectly(&state, "Attached Aura"),
        "CR 702.26g: Aura's phased_out_indirectly flag should be true"
    );
}

// ── Test 5: Indirect phase-in -- Aura phases in with host ─────────────────────

#[test]
/// CR 702.26g -- An indirectly-phased Aura phases in when its host phases in.
fn test_phasing_indirect_phases_in_together() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "Phased Host", 2, 2))
        .object(ObjectSpec::card(p1, "Phased Aura"))
        .build()
        .unwrap();

    // Setup: host and aura are both phased out. Aura is phased out indirectly.
    let host_id = find_object(&state, "Phased Host");
    let aura_id = find_object(&state, "Phased Aura");

    // Attach aura to host.
    if let Some(host) = state.objects.get_mut(&host_id) {
        host.attachments = vec![aura_id].into_iter().collect();
        host.status.phased_out = true;
        host.phased_out_controller = Some(p1);
    }
    if let Some(aura) = state.objects.get_mut(&aura_id) {
        aura.attached_to = Some(host_id);
        aura.status.phased_out = true;
        aura.phased_out_indirectly = true;
        aura.phased_out_controller = Some(p1);
    }

    assert!(
        is_phased_out(&state, "Phased Host"),
        "host should start phased out"
    );
    assert!(
        is_phased_out(&state, "Phased Aura"),
        "aura should start phased out"
    );

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // After p1's untap: both should phase in (neither has Phasing keyword, so no re-phase-out).
    assert!(
        !is_phased_out(&state, "Phased Host"),
        "CR 702.26g: host should phase in on controller's untap step"
    );
    assert!(
        !is_phased_out(&state, "Phased Aura"),
        "CR 702.26g: indirectly-phased Aura should phase in with its host"
    );

    // Aura should still be attached to the host after phasing in.
    let aura = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Phased Aura")
        .unwrap();
    assert_eq!(
        aura.attached_to,
        Some(host_id),
        "CR 702.26g: Aura should still be attached to host after phasing in"
    );
}

// ── Test 6: No ETB triggers on phase-in ───────────────────────────────────────

#[test]
/// CR 702.26d -- Phasing in does not trigger "when this enters the battlefield" abilities.
fn test_phasing_no_etb_triggers_on_phase_in() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "ETB Creature", 2, 2))
        .build()
        .unwrap();

    // Manually phase out the creature.
    let obj_id = find_object(&state, "ETB Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    let (state, events) = pass_all(state, &[p2, p1]);

    // Check that no ETB-related events were emitted.
    // ETB events would typically show up as PermanentEnteredBattlefield.
    let etb_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .collect();

    assert!(
        etb_events.is_empty(),
        "CR 702.26d: phasing in should NOT trigger PermanentEnteredBattlefield events, found: {:?}",
        etb_events
    );

    // The creature should be phased in (back on the effective battlefield).
    assert!(
        !is_phased_out(&state, "ETB Creature"),
        "CR 702.26d: creature should have phased in"
    );
}

// ── Test 7: PermanentsPhasedOut event emitted ─────────────────────────────────

#[test]
/// CR 702.26a -- The engine emits a PermanentsPhasedOut event when permanents phase out.
fn test_phasing_emits_phased_out_event() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Phasing Creature", 2, 2)
                .with_keyword(KeywordAbility::Phasing),
        )
        .build()
        .unwrap();

    let (_, events) = pass_all(state, &[p2, p1]);

    let phased_out_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentsPhasedOut { player, .. } if *player == p1))
        .collect();

    assert!(
        !phased_out_events.is_empty(),
        "CR 702.26a: PermanentsPhasedOut event should be emitted when a permanent phases out"
    );
}

// ── Test 8: PermanentsPhasedIn event emitted ──────────────────────────────────

#[test]
/// CR 702.26a -- The engine emits a PermanentsPhasedIn event when permanents phase in.
fn test_phasing_emits_phased_in_event() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "Plain Creature", 2, 2))
        .build()
        .unwrap();

    // Manually phase out the creature (no Phasing keyword, so it won't re-phase-out).
    let obj_id = find_object(&state, "Plain Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    let (_, events) = pass_all(state, &[p2, p1]);

    let phased_in_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentsPhasedIn { player, .. } if *player == p1))
        .collect();

    assert!(
        !phased_in_events.is_empty(),
        "CR 702.26a: PermanentsPhasedIn event should be emitted when a permanent phases in"
    );
}

// ── Test 9: Excluded from SBAs ────────────────────────────────────────────────

#[test]
/// CR 702.26b -- Phased-out permanents are excluded from state-based actions.
/// A phased-out creature with 0 toughness is NOT destroyed.
fn test_phasing_excluded_from_sba() {
    let p1 = p(1);
    let p2 = p(2);

    // Use DeclareBlockers step so we can check SBA-relevant fields without advancing.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            // A 0/0 creature would normally die to SBAs, but phasing exempts it.
            ObjectSpec::creature(p1, "Phased-Out Tiny", 0, 0),
        )
        .build()
        .unwrap();

    // Manually phase out the 0/0 creature.
    let obj_id = find_object(&state, "Phased-Out Tiny");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    // Pass priority -- this triggers SBA checks when entering Upkeep-level steps.
    // SBAs should NOT destroy the phased-out 0/0 creature.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .expect("PassPriority should succeed");

    // The creature should still exist (not destroyed by SBA 704.5f/g).
    assert!(
        is_on_battlefield(&state, "Phased-Out Tiny"),
        "CR 702.26b: phased-out permanent should be exempt from SBAs (0/0 not destroyed)"
    );
    assert!(
        is_phased_out(&state, "Phased-Out Tiny"),
        "CR 702.26b: creature should still be phased out"
    );
}

// ── Test 10: Cannot attack while phased out ───────────────────────────────────

#[test]
/// CR 702.26b -- A phased-out creature cannot be declared as an attacker.
fn test_phasing_excluded_from_combat_attack() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p1, "Phased-Out Attacker", 2, 2))
        .build()
        .unwrap();

    // Manually phase out the creature.
    let obj_id = find_object(&state, "Phased-Out Attacker");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    // Attempt to declare the phased-out creature as an attacker.
    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(obj_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.26b: phased-out creature should not be able to attack"
    );
}

// ── Test 11: Cannot block while phased out ────────────────────────────────────

#[test]
/// CR 702.26b -- A phased-out creature cannot be declared as a blocker.
fn test_phasing_excluded_from_combat_block() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Phased-Out Blocker", 2, 2))
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    let blocker_id = find_object(&state, "Phased-Out Blocker");

    // Set up combat with the attacker.
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    // Manually phase out the blocker.
    if let Some(obj) = state.objects.get_mut(&blocker_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p2);
    }

    // Attempt to declare the phased-out creature as a blocker.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.26b: phased-out creature should not be able to block"
    );
}

// ── Test 12: Token with Phasing survives phase-out ────────────────────────────

#[test]
/// CR 702.26d -- Tokens with Phasing survive phase-out. They do not cease to exist
/// while phased out (CR 704.5d only applies to tokens in non-battlefield zones).
fn test_phasing_token_survives_phase_out() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Phasing Token", 1, 1)
                .with_keyword(KeywordAbility::Phasing)
                .token(),
        )
        .build()
        .unwrap();

    // Ensure the token exists and is a token.
    let obj_id = find_object(&state, "Phasing Token");
    assert!(
        state.objects.get(&obj_id).unwrap().is_token,
        "should be a token"
    );

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // Token should still exist after phasing out.
    assert!(
        is_on_battlefield(&state, "Phasing Token"),
        "CR 702.26d: token with Phasing should still exist in Battlefield zone after phasing out"
    );
    assert!(
        is_phased_out(&state, "Phasing Token"),
        "token should be phased out"
    );
}

// ── Test 13: Multiplayer -- phase-in only on correct player's untap ───────────

#[test]
/// CR 702.26a -- A phased-out permanent only phases in on the controlling player's untap step,
/// not during other players' untap steps.
fn test_phasing_multiplayer_controller_tracking() {
    let p1 = p(1);
    let p2 = p(2);

    // p1's creature is phased out. Set up so we advance through p2's untap first.
    // After p2's untap, creature should still be phased out (p2 doesn't control it).
    // We start at p1's End step → p2's Untap fires first.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::End)
        .object(ObjectSpec::creature(p1, "P1 Phased Creature", 2, 2))
        .build()
        .unwrap();

    // Manually phase out p1's creature.
    let obj_id = find_object(&state, "P1 Phased Creature");
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    // After passing at p1's End step → Cleanup → p2's Untap fires → p2's Upkeep.
    // p2's untap should NOT phase in p1's creature.
    let (state, _) = pass_all(state, &[p1, p2]);

    // State should now be at p2's Upkeep. p1's creature should still be phased out.
    assert_eq!(state.turn.active_player, p2, "should be p2's turn now");
    assert!(
        is_phased_out(&state, "P1 Phased Creature"),
        "CR 702.26a: p1's phased-out creature should NOT phase in during p2's untap step"
    );
}

// ── Test 14: Redundant Phasing instances ──────────────────────────────────────

#[test]
/// CR 702.26p -- Multiple instances of Phasing on the same permanent are redundant.
/// Behavior should be identical to a permanent with a single Phasing instance.
fn test_phasing_redundant_instances() {
    let p1 = p(1);
    let p2 = p(2);

    // Two creatures: one with one Phasing, one with two (OrdSet deduplicates).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p2)
        .at_step(Step::End)
        .object(
            ObjectSpec::creature(p1, "Single Phasing", 2, 2).with_keyword(KeywordAbility::Phasing),
        )
        .object(
            // OrdSet deduplicates, so adding Phasing twice is the same as once.
            ObjectSpec::creature(p1, "Double Phasing", 2, 2)
                .with_keyword(KeywordAbility::Phasing)
                .with_keyword(KeywordAbility::Phasing),
        )
        .build()
        .unwrap();

    let state = advance_to_p1_upkeep_via_end_step(state, &[p2, p1]);

    // Both should be phased out (redundant instances behave the same as one).
    assert!(
        is_phased_out(&state, "Single Phasing"),
        "CR 702.26p: single Phasing instance should phase out"
    );
    assert!(
        is_phased_out(&state, "Double Phasing"),
        "CR 702.26p: double Phasing instances are redundant, behavior same as single"
    );
}

// ── Test 16: Phased-out permanents excluded from continuous effects (CR 702.26e) ─────

#[test]
/// CR 702.26e -- A phased-out permanent won't be included in the set of affected
/// objects of a continuous effect. The layer system's phased-out filter (layers.rs)
/// ensures that global P/T effects do not apply to phased-out permanents.
fn test_phasing_excluded_from_continuous_effects() {
    let p1 = PlayerId(1);

    // Two creatures: one phased in, one phased out.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .object(ObjectSpec::creature(p1, "Active Creature", 2, 2))
        .object(ObjectSpec::creature(p1, "Phased-Out Creature", 2, 2))
        .build()
        .unwrap();

    // Manually phase out one creature.
    let phased_out_id = find_object(&state, "Phased-Out Creature");
    if let Some(obj) = state.objects.get_mut(&phased_out_id) {
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p1);
    }

    // Add a global "+1/+1 to all creatures" continuous effect (Layer 7c / PtModify).
    // CR 702.26e: this effect must NOT apply to the phased-out creature.
    state.continuous_effects.push_back(ContinuousEffect {
        id: EffectId(9001),
        source: None,
        timestamp: 100,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    });

    let active_id = find_object(&state, "Active Creature");

    // The active (phased-in) creature should be 3/3 after the effect.
    let active_chars = calculate_characteristics(&state, active_id)
        .expect("active creature must have characteristics");
    assert_eq!(
        active_chars.power,
        Some(3),
        "CR 702.26e: phased-in creature should receive +1/+1 from global effect (power)"
    );
    assert_eq!(
        active_chars.toughness,
        Some(3),
        "CR 702.26e: phased-in creature should receive +1/+1 from global effect (toughness)"
    );

    // The phased-out creature should still be 2/2 -- the effect does NOT apply.
    let phased_out_chars = calculate_characteristics(&state, phased_out_id)
        .expect("phased-out creature must still have characteristics");
    assert_eq!(
        phased_out_chars.power,
        Some(2),
        "CR 702.26e: phased-out creature must NOT receive +1/+1 from global effect (power)"
    );
    assert_eq!(
        phased_out_chars.toughness,
        Some(2),
        "CR 702.26e: phased-out creature must NOT receive +1/+1 from global effect (toughness)"
    );
}

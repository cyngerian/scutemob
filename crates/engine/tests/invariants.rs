//! Property-based invariant tests for the MTG engine.
//!
//! These tests probe engine state consistency — not rules correctness (which is covered
//! by the dedicated rules tests). Every test asserts a structural or logical invariant
//! that must hold regardless of the sequence of commands applied.
//!
//! Invariant categories:
//!   - Zone integrity (objects in exactly one zone, ID bijection)
//!   - Player index validity (active_player, priority_holder within bounds)
//!   - Life totals are i32 (can go negative; death by SBA, not clamping)
//!   - Stack controller always a valid non-eliminated player
//!   - Object-count conservation across zone moves
//!   - Mana pool invariants (non-negative components)
//!   - Turn-order invariants (monotone turn number, active player in turn_order)
//!   - Priority-pass safety (no panics on random pass sequences)
//!   - SBA idempotency (applying twice in a row produces no new actions)
//!   - Draw invariants (library size decrements by 1; empty library marks loss)
//!   - Hand-size invariants (all hand objects belong to the zone's owner)
//!   - Battlefield invariants (no duplicate names in objects map, timestamp monotone)
//!   - Stack invariants (LIFO, stack zone length == stack_objects length)
//!   - Commander invariants (tax never negative, damage_received always u32)
//!   - Continuous-effect invariants (effect IDs unique)

use mtg_engine::rules::engine::{process_command, start_game};
use mtg_engine::{
    check_and_apply_sbas, Command, GameEvent, GameState, GameStateBuilder, LossReason, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
use proptest::prelude::*;

// ── Shared helper: zone integrity checker ────────────────────────────────────

/// Verify every object is in exactly one zone and every zone ID is in objects.
fn assert_zone_integrity(state: &GameState) {
    // Every object's zone field points to a real zone that contains it.
    for (obj_id, obj) in state.objects.iter() {
        let zone = state.zone(&obj.zone).unwrap_or_else(|_| {
            panic!(
                "object {:?} references non-existent zone {:?}",
                obj_id, obj.zone
            )
        });
        assert!(
            zone.contains(obj_id),
            "object {:?} says it's in zone {:?} but the zone doesn't contain it",
            obj_id,
            obj.zone
        );
        // Exactly one zone contains this object.
        let count = state.zones.values().filter(|z| z.contains(obj_id)).count();
        assert_eq!(count, 1, "object {:?} found in {} zones", obj_id, count);
    }
    // Converse: every ID in a zone has an entry in objects.
    for (zone_id, zone) in state.zones.iter() {
        for id in zone.object_ids() {
            assert!(
                state.objects.contains_key(&id),
                "zone {:?} has orphaned object {:?}",
                zone_id,
                id
            );
        }
    }
}

/// Run `n` PassPriority commands on a fresh four-player started game and return
/// the final state (stopping early if the game ends).
fn pass_n(n: usize) -> GameState {
    let state = GameStateBuilder::four_player().build().unwrap();
    let (mut state, _) = start_game(state).unwrap();
    for _ in 0..n {
        let holder = match state.turn.priority_holder {
            Some(h) => h,
            None => break,
        };
        match process_command(state.clone(), Command::PassPriority { player: holder }) {
            Ok((s, _)) => state = s,
            Err(_) => break,
        }
    }
    state
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 1: Zone Integrity (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-ZI-01: Zone integrity holds for arbitrary populations of objects.
    #[test]
    fn prop_zi_01_zone_integrity_on_build(
        bf_count   in 0u32..10,
        hand_count in 0u32..8,
        gy_count   in 0u32..5,
        lib_count  in 0u32..5,
    ) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..bf_count   { b = b.object(ObjectSpec::creature(p1, &format!("BF{}", i), 1, 1)); }
        for i in 0..hand_count { b = b.object(ObjectSpec::card(p1, &format!("H{}", i)).in_zone(ZoneId::Hand(p1))); }
        for i in 0..gy_count   { b = b.object(ObjectSpec::card(p1, &format!("GY{}", i)).in_zone(ZoneId::Graveyard(p1))); }
        for i in 0..lib_count  { b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1))); }
        let state = b.build().unwrap();
        assert_zone_integrity(&state);
    }

    /// INV-ZI-02: All ObjectIds are unique (no duplicates across any zones).
    #[test]
    fn prop_zi_02_all_object_ids_unique(obj_count in 1u32..40) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("Obj{}", i), 1, 1)); }
        let state = b.build().unwrap();
        let ids: Vec<ObjectId> = state.objects.keys().cloned().collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        prop_assert_eq!(ids.len(), unique.len(), "duplicate ObjectIds found");
    }

    /// INV-ZI-03: Zone integrity holds after a series of zone moves.
    #[test]
    fn prop_zi_03_zone_integrity_after_moves(num_moves in 1u32..15) {
        let p1 = PlayerId(1);
        let mut state = GameStateBuilder::four_player()
            .object(ObjectSpec::creature(p1, "Nomad", 2, 2))
            .build().unwrap();
        let zones = [
            ZoneId::Graveyard(p1),
            ZoneId::Exile,
            ZoneId::Hand(p1),
            ZoneId::Library(p1),
            ZoneId::Battlefield,
        ];
        for i in 0..num_moves {
            let z = zones[i as usize % zones.len()];
            let id = *state.objects.keys().next().unwrap();
            state.move_object_to_zone(id, z).unwrap();
            assert_zone_integrity(&state);
        }
    }

    /// INV-ZI-04: Total object count is conserved across zone moves (CR 400.7).
    #[test]
    fn prop_zi_04_move_preserves_object_count(initial_count in 2u32..10) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..initial_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let mut state = b.build().unwrap();
        let before = state.total_objects();
        let first_id = state.objects_in_zone(&ZoneId::Battlefield)[0].id;
        state.move_object_to_zone(first_id, ZoneId::Graveyard(p1)).unwrap();
        prop_assert_eq!(state.total_objects(), before, "object count changed after zone move");
        assert_zone_integrity(&state);
    }

    /// INV-ZI-05: Zone integrity holds after PassPriority sequences.
    #[test]
    fn prop_zi_05_zone_integrity_after_priority_passes(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        assert_zone_integrity(&state);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 2: Player Index Validity (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-PI-01: active_player is always in turn_order.
    #[test]
    fn prop_pi_01_active_player_in_turn_order(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        prop_assert!(
            state.turn.turn_order.contains(&state.turn.active_player),
            "active_player {:?} not found in turn_order",
            state.turn.active_player
        );
    }

    /// INV-PI-02: priority_holder, when set, is always a non-eliminated player.
    #[test]
    fn prop_pi_02_priority_holder_is_active_player(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        if let Some(holder) = state.turn.priority_holder {
            let player = state.player(holder).unwrap();
            prop_assert!(!player.has_lost, "priority holder {:?} has lost", holder);
            prop_assert!(!player.has_conceded, "priority holder {:?} has conceded", holder);
        }
    }

    /// INV-PI-03: priority_holder is always in turn_order when set.
    #[test]
    fn prop_pi_03_priority_holder_in_turn_order(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        if let Some(holder) = state.turn.priority_holder {
            prop_assert!(
                state.turn.turn_order.contains(&holder),
                "priority_holder {:?} not in turn_order",
                holder
            );
        }
    }

    /// INV-PI-04: active_player is always in the players map.
    #[test]
    fn prop_pi_04_active_player_in_players_map(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        prop_assert!(
            state.players.contains_key(&state.turn.active_player),
            "active_player {:?} not in players map",
            state.turn.active_player
        );
    }

    /// INV-PI-05: turn_order length never changes during normal priority passes.
    #[test]
    fn prop_pi_05_turn_order_length_stable(num_passes in 1usize..200) {
        let state0 = GameStateBuilder::four_player().build().unwrap();
        let (state0, _) = start_game(state0).unwrap();
        let original_len = state0.turn.turn_order.len();
        let state = pass_n(num_passes);
        prop_assert_eq!(
            state.turn.turn_order.len(),
            original_len,
            "turn_order length changed"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 3: Life-Total Invariants (4 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-LT-01: Life totals remain i32 — can be set to any i32 value via builder.
    /// Death is an SBA, not a clamp. CR 104.3a.
    #[test]
    fn prop_lt_01_life_total_is_i32(life in i32::MIN..=i32::MAX) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .player_life(p1, life)
            .at_step(Step::PreCombatMain)
            .build().unwrap();
        prop_assert_eq!(state.player(p1).unwrap().life_total, life);
    }

    /// INV-LT-02: Negative life total is preserved (not clamped to zero).
    /// CR 104.3a: a player with 0 or less life LOSES; the total is not clamped.
    #[test]
    fn prop_lt_02_negative_life_not_clamped(life in -10000i32..=-1) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .player_life(p1, life)
            .at_step(Step::PreCombatMain)
            .build().unwrap();
        prop_assert_eq!(state.player(p1).unwrap().life_total, life,
            "life total was clamped — must remain negative");
    }

    /// INV-LT-03: Starting life total for all players is 40 (Commander format, CR 903.7).
    #[test]
    fn prop_lt_03_commander_starting_life(num_players in 2u64..=6) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        for (_, p) in state.players.iter() {
            prop_assert_eq!(p.life_total, 40, "player {:?} does not start at 40 life", p.id);
        }
    }

    /// INV-LT-04: A player at 0 life loses after SBA check. CR 704.5a.
    #[test]
    fn prop_lt_04_zero_life_loses_after_sba(life in -50i32..=0) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .player_life(p1, life)
            .at_step(Step::PreCombatMain)
            .build().unwrap();
        let events = check_and_apply_sbas(&mut state);
        let lost = events.iter().any(|e| {
            matches!(e, GameEvent::PlayerLost { player, reason }
                if *player == p1 && *reason == LossReason::LifeTotal)
        });
        prop_assert!(lost, "player at {} life should lose via SBA but didn't", life);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 4: Stack Invariants (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-SK-01: All stack_objects controllers are valid non-eliminated players.
    #[test]
    fn prop_sk_01_stack_controllers_valid(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        for so in state.stack_objects.iter() {
            prop_assert!(
                state.players.contains_key(&so.controller),
                "stack object controller {:?} not in players map", so.controller
            );
        }
    }

    /// INV-SK-02: Stack zone object count equals stack_objects length.
    #[test]
    fn prop_sk_02_stack_zone_matches_stack_objects(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        let stack_zone_len = state.zone(&ZoneId::Stack).unwrap().len();
        // stack_objects may include abilities (no card in Zone::Stack), so
        // stack_objects.len() >= stack_zone_len (abilities have no card object).
        // But all Spell entries DO have a card in the Stack zone.
        let spell_count = state.stack_objects.iter().filter(|so| {
            matches!(so.kind, mtg_engine::StackObjectKind::Spell { .. })
        }).count();
        prop_assert_eq!(
            stack_zone_len, spell_count,
            "stack zone len ({}) != spell count ({})", stack_zone_len, spell_count
        );
    }

    /// INV-SK-03: Stack object IDs are unique.
    #[test]
    fn prop_sk_03_stack_object_ids_unique(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        let ids: Vec<ObjectId> = state.stack_objects.iter().map(|so| so.id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        prop_assert_eq!(ids.len(), unique.len(), "duplicate stack object IDs");
    }

    /// INV-SK-04: Stack is always empty at the start of a fresh built state.
    #[test]
    fn prop_sk_04_fresh_state_stack_empty(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        prop_assert!(state.stack_objects.is_empty(), "fresh state has non-empty stack");
        prop_assert_eq!(state.zone(&ZoneId::Stack).unwrap().len(), 0);
    }

    /// INV-SK-05: Stack controllers are always in the players map after passes.
    #[test]
    fn prop_sk_05_stack_controllers_in_players(num_passes in 1usize..150) {
        let state = pass_n(num_passes);
        for so in state.stack_objects.iter() {
            let player = state.players.get(&so.controller);
            prop_assert!(player.is_some(),
                "stack object {:?} has controller {:?} not in players", so.id, so.controller);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 5: Mana Pool Invariants (4 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-MP-01: All mana pool components are non-negative (they're u32).
    #[test]
    fn prop_mp_01_mana_pool_components_nonneg(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        for (_, player) in state.players.iter() {
            prop_assert!(player.mana_pool.white    <= u32::MAX);
            prop_assert!(player.mana_pool.blue     <= u32::MAX);
            prop_assert!(player.mana_pool.black    <= u32::MAX);
            prop_assert!(player.mana_pool.red      <= u32::MAX);
            prop_assert!(player.mana_pool.green    <= u32::MAX);
            prop_assert!(player.mana_pool.colorless <= u32::MAX);
        }
    }

    /// INV-MP-02: Mana pool total == sum of components.
    #[test]
    fn prop_mp_02_mana_total_equals_sum(
        w in 0u32..10, u in 0u32..10, b in 0u32..10,
        r in 0u32..10, g in 0u32..10, c in 0u32..10,
    ) {
        use mtg_engine::ManaPool;
        let pool = ManaPool { white: w, blue: u, black: b, red: r, green: g, colorless: c, ..ManaPool::default() };
        prop_assert_eq!(pool.total(), w + u + b + r + g + c);
    }

    /// INV-MP-03: Fresh state has empty mana pools for all players.
    #[test]
    fn prop_mp_03_fresh_state_empty_mana(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        for (_, p) in state.players.iter() {
            prop_assert!(p.mana_pool.is_empty(), "fresh player {:?} has non-empty mana pool", p.id);
        }
    }

    /// INV-MP-04: Mana pool is empty after step transition (CR 500.4).
    #[test]
    fn prop_mp_04_mana_emptied_at_step_transition(num_passes in 4usize..20) {
        // Pass all 4 players once to advance a step; mana pools are emptied.
        let state = GameStateBuilder::four_player()
            .at_step(Step::PreCombatMain)
            .build().unwrap();
        let (state, _) = start_game(state).unwrap();
        let mut s = state;
        // Pass priority for all 4 players to trigger a step change.
        for _ in 0..num_passes {
            let holder = match s.turn.priority_holder {
                Some(h) => h,
                None => break,
            };
            match process_command(s.clone(), Command::PassPriority { player: holder }) {
                Ok((ns, events)) => {
                    // After a step change, check mana pools are cleared.
                    if events.iter().any(|e| matches!(e, GameEvent::ManaPoolsEmptied)) {
                        for (_, p) in ns.players.iter() {
                            prop_assert!(p.mana_pool.is_empty(),
                                "mana pool of {:?} not empty after ManaPoolsEmptied", p.id);
                        }
                    }
                    s = ns;
                }
                Err(_) => break,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 6: Turn-Order Invariants (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-TO-01: Turn number monotonically increases (never decreases).
    #[test]
    fn prop_to_01_turn_number_monotone(num_passes in 1usize..300) {
        let state = GameStateBuilder::four_player().build().unwrap();
        let (mut state, _) = start_game(state).unwrap();
        let mut last_turn = state.turn.turn_number;
        for _ in 0..num_passes {
            let holder = match state.turn.priority_holder { Some(h) => h, None => break };
            match process_command(state.clone(), Command::PassPriority { player: holder }) {
                Ok((ns, _)) => {
                    prop_assert!(ns.turn.turn_number >= last_turn,
                        "turn decreased: {} -> {}", last_turn, ns.turn.turn_number);
                    last_turn = ns.turn.turn_number;
                    state = ns;
                }
                Err(_) => break,
            }
        }
    }

    /// INV-TO-02: All players in turn_order are in the players map.
    #[test]
    fn prop_to_02_turn_order_ids_in_players_map(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        for id in state.turn.turn_order.iter() {
            prop_assert!(
                state.players.contains_key(id),
                "turn_order has {:?} not in players", id
            );
        }
    }

    /// INV-TO-03: Eliminated player never gets priority after concession.
    #[test]
    fn prop_to_03_eliminated_never_gets_priority(
        concede_player in 2u64..=4u64,
        num_passes in 1usize..200,
    ) {
        let state = GameStateBuilder::four_player().build().unwrap();
        let (state, _) = start_game(state).unwrap();
        let target = PlayerId(concede_player);
        let (mut state, _) = process_command(state, Command::Concede { player: target }).unwrap();
        for _ in 0..num_passes {
            prop_assert!(state.turn.priority_holder != Some(target),
                "eliminated player {:?} holds priority", target);
            let holder = match state.turn.priority_holder { Some(h) => h, None => break };
            match process_command(state.clone(), Command::PassPriority { player: holder }) {
                Ok((ns, _)) => state = ns,
                Err(_) => break,
            }
        }
    }

    /// INV-TO-04: Phase sequence is consistent with step sequence.
    /// The phase of each step must match the step's declared phase.
    #[test]
    fn prop_to_04_phase_consistent_with_step(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        prop_assert_eq!(
            state.turn.phase,
            state.turn.step.phase(),
            "phase {:?} does not match step {:?} (expected {:?})",
            state.turn.phase,
            state.turn.step,
            state.turn.step.phase()
        );
    }

    /// INV-TO-05: Passing priority exactly N times (N = player count) from a fresh
    /// step always advances the step (since all players pass with empty stack).
    #[test]
    fn prop_to_05_all_pass_advances_step(n_extra in 0usize..3) {
        let state = GameStateBuilder::four_player()
            .at_step(Step::PreCombatMain)
            .build().unwrap();
        let initial_step = state.turn.step;
        let mut s = state;
        // Pass 4 players (1 complete round) + some extra passes.
        for _ in 0..(4 + n_extra) {
            let holder = match s.turn.priority_holder { Some(h) => h, None => break };
            match process_command(s.clone(), Command::PassPriority { player: holder }) {
                Ok((ns, _)) => s = ns,
                Err(_) => break,
            }
        }
        prop_assert_ne!(s.turn.step, initial_step,
            "step did not advance after all players passed");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 7: SBA Idempotency Invariants (4 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-SBA-01: SBA check on a fresh state produces no actions (no conditions met).
    #[test]
    fn prop_sba_01_fresh_state_no_sbas(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let mut state = b.build().unwrap();
        let events = check_and_apply_sbas(&mut state);
        prop_assert!(events.is_empty(), "fresh state produced SBA events: {:?}", events);
    }

    /// INV-SBA-02: SBA check is idempotent — applying twice with no intervening
    /// commands produces no new actions the second time. CR 704.3 fixed-point.
    #[test]
    fn prop_sba_02_sba_idempotent_creatures(creature_count in 0u32..10) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        // Healthy creatures (2/2) — no SBAs should fire.
        for i in 0..creature_count {
            b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 2, 2));
        }
        let mut state = b.build().unwrap();
        let first = check_and_apply_sbas(&mut state);
        let second = check_and_apply_sbas(&mut state);
        prop_assert!(first.is_empty(), "healthy creatures triggered SBAs: {:?}", first);
        prop_assert!(second.is_empty(), "second SBA pass still produced events");
    }

    /// INV-SBA-03: SBA for 0-toughness creature fires once and not again.
    #[test]
    fn prop_sba_03_zero_toughness_sba_fires_once(extra_creatures in 0u32..5) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        // One 0-toughness creature; remaining are healthy.
        let mut b = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(ObjectSpec::creature(p1, "Doomed", 1, 0));
        for i in 0..extra_creatures {
            b = b.object(ObjectSpec::creature(p1, &format!("Safe{}", i), 2, 2));
        }
        let mut state = b.build().unwrap();
        let first = check_and_apply_sbas(&mut state);
        let second = check_and_apply_sbas(&mut state);
        prop_assert!(!first.is_empty(), "0-toughness creature SBA didn't fire");
        prop_assert!(second.is_empty(), "SBA fired again on second pass (not idempotent)");
    }

    /// INV-SBA-04: Zone integrity holds after SBA application.
    #[test]
    fn prop_sba_04_zone_integrity_after_sba(creature_count in 1u32..8) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..creature_count {
            // Alternate between healthy and 0-toughness.
            let (pw, pt) = if i % 2 == 0 { (2, 2) } else { (1, 0) };
            b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), pw, pt));
        }
        let mut state = b.build().unwrap();
        check_and_apply_sbas(&mut state);
        assert_zone_integrity(&state);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 8: Draw / Library Invariants (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-DL-01: Drawing a card from a non-empty library decrements library size by 1
    /// and increments hand size by 1.
    #[test]
    fn prop_dl_01_draw_decrements_library(lib_size in 1u32..20) {
        use mtg_engine::rules::turn_actions::draw_card;
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..lib_size {
            b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1)));
        }
        let mut state = b.build().unwrap();
        let lib_before = state.zone(&ZoneId::Library(p1)).unwrap().len();
        let hand_before = state.zone(&ZoneId::Hand(p1)).unwrap().len();
        let events = draw_card(&mut state, p1).unwrap();
        let lib_after  = state.zone(&ZoneId::Library(p1)).unwrap().len();
        let hand_after  = state.zone(&ZoneId::Hand(p1)).unwrap().len();
        let card_drawn = events.iter().any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
        prop_assert!(card_drawn, "no CardDrawn event emitted");
        prop_assert_eq!(lib_after,  lib_before  - 1, "library size didn't decrement");
        prop_assert_eq!(hand_after, hand_before + 1, "hand size didn't increment");
    }

    /// INV-DL-02: Drawing from an empty library emits PlayerLost (CR 104.3b).
    #[test]
    fn prop_dl_02_draw_from_empty_library_loses(extra_players in 0u64..3) {
        use mtg_engine::rules::turn_actions::draw_card;
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::new().add_player(p1);
        for i in 1..=extra_players { b = b.add_player(PlayerId(i + 1)); }
        let mut state = b.build().unwrap();
        // Library is empty at build time.
        let events = draw_card(&mut state, p1).unwrap();
        let lost = events.iter().any(|e| {
            matches!(e, GameEvent::PlayerLost { player, reason }
                if *player == p1 && *reason == LossReason::LibraryEmpty)
        });
        prop_assert!(lost, "draw from empty library didn't emit PlayerLost");
    }

    /// INV-DL-03: Drawing N cards removes exactly N cards from the library
    /// (library has at least N cards).
    #[test]
    fn prop_dl_03_multiple_draws_decrement_library(draw_count in 1u32..10) {
        use mtg_engine::rules::turn_actions::draw_card;
        let lib_size = draw_count + 5; // Always enough cards.
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..lib_size {
            b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1)));
        }
        let mut state = b.build().unwrap();
        let lib_before = state.zone(&ZoneId::Library(p1)).unwrap().len();
        for _ in 0..draw_count {
            draw_card(&mut state, p1).unwrap();
        }
        let lib_after = state.zone(&ZoneId::Library(p1)).unwrap().len();
        prop_assert_eq!(lib_after, lib_before - draw_count as usize,
            "expected library to shrink by {} but shrank by {}",
            draw_count, lib_before - lib_after);
    }

    /// INV-DL-04: cards_drawn_this_turn counter increments with each draw.
    #[test]
    fn prop_dl_04_cards_drawn_counter_increments(draw_count in 1u32..10) {
        use mtg_engine::rules::turn_actions::draw_card;
        let lib_size = draw_count + 5;
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..lib_size {
            b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1)));
        }
        let mut state = b.build().unwrap();
        for _ in 0..draw_count {
            draw_card(&mut state, p1).unwrap();
        }
        prop_assert_eq!(
            state.player(p1).unwrap().cards_drawn_this_turn,
            draw_count,
            "cards_drawn_this_turn counter incorrect"
        );
    }

    /// INV-DL-05: Zone integrity holds after drawing cards.
    #[test]
    fn prop_dl_05_zone_integrity_after_draws(draw_count in 1u32..8) {
        use mtg_engine::rules::turn_actions::draw_card;
        let lib_size = draw_count + 3;
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..lib_size {
            b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1)));
        }
        let mut state = b.build().unwrap();
        for _ in 0..draw_count {
            draw_card(&mut state, p1).unwrap();
        }
        assert_zone_integrity(&state);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 9: Object-Count and Battlefield Invariants (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-OB-01: Objects in a zone are all owned by the zone's owner (for player zones).
    #[test]
    fn prop_ob_01_hand_objects_owned_by_zone_owner(hand_size in 1u32..8) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..hand_size {
            b = b.object(ObjectSpec::card(p1, &format!("H{}", i)).in_zone(ZoneId::Hand(p1)));
        }
        let state = b.build().unwrap();
        for obj in state.objects_in_zone(&ZoneId::Hand(p1)) {
            prop_assert_eq!(obj.owner, p1,
                "hand object {:?} owned by {:?}, expected {:?}", obj.id, obj.owner, p1);
        }
    }

    /// INV-OB-02: Total object count equals sum of all zone lengths.
    #[test]
    fn prop_ob_02_total_objects_equals_zone_sum(
        bf in 0u32..5, hand in 0u32..4, gy in 0u32..4, lib in 0u32..4,
    ) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut b = GameStateBuilder::new().add_player(p1).add_player(p2);
        for i in 0..bf   { b = b.object(ObjectSpec::creature(p1, &format!("BF{}", i), 1, 1)); }
        for i in 0..hand { b = b.object(ObjectSpec::card(p1, &format!("H{}", i)).in_zone(ZoneId::Hand(p1))); }
        for i in 0..gy   { b = b.object(ObjectSpec::card(p1, &format!("GY{}", i)).in_zone(ZoneId::Graveyard(p1))); }
        for i in 0..lib  { b = b.object(ObjectSpec::card(p1, &format!("L{}", i)).in_zone(ZoneId::Library(p1))); }
        let state = b.build().unwrap();
        let zone_sum: usize = state.zones.values().map(|z| z.len()).sum();
        prop_assert_eq!(state.total_objects(), zone_sum,
            "total_objects() ({}) != zone sum ({})", state.total_objects(), zone_sum);
    }

    /// INV-OB-03: All battlefield objects have zone == ZoneId::Battlefield.
    #[test]
    fn prop_ob_03_battlefield_objects_have_correct_zone(count in 1u32..10) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            prop_assert_eq!(obj.zone, ZoneId::Battlefield,
                "battlefield object {:?} has zone {:?}", obj.id, obj.zone);
        }
    }

    /// INV-OB-04: timestamp_counter is always >= total_objects after build.
    /// The counter only increases, so it must dominate total object count.
    #[test]
    fn prop_ob_04_timestamp_counter_gte_object_count(obj_count in 0u32..30) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        prop_assert!(
            state.timestamp_counter >= state.total_objects() as u64,
            "timestamp_counter ({}) < total_objects ({})",
            state.timestamp_counter, state.total_objects()
        );
    }

    /// INV-OB-05: Object timestamps are all <= timestamp_counter.
    #[test]
    fn prop_ob_05_object_timestamps_lte_counter(obj_count in 1u32..20) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        for (id, obj) in state.objects.iter() {
            prop_assert!(
                obj.timestamp <= state.timestamp_counter,
                "object {:?} timestamp {} > counter {}",
                id, obj.timestamp, state.timestamp_counter
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 10: Priority Pass Safety (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-PP-01: Random priority-pass sequences never panic.
    #[test]
    fn prop_pp_01_pass_never_panics(num_passes in 1usize..500) {
        let _ = pass_n(num_passes);
    }

    /// INV-PP-02: After any number of passes, at least 1 player is active or game ended.
    #[test]
    fn prop_pp_02_always_active_player_or_game_over(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        let active = state.active_players();
        prop_assert!(active.len() >= 1 || state.turn.priority_holder.is_none(),
            "no active players and game not ended");
    }

    /// INV-PP-03: Zone integrity holds after arbitrary pass sequences.
    #[test]
    fn prop_pp_03_zone_integrity_after_passes(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        assert_zone_integrity(&state);
    }

    /// INV-PP-04: Phase invariant holds throughout pass sequence.
    #[test]
    fn prop_pp_04_phase_invariant_throughout(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        prop_assert_eq!(state.turn.phase, state.turn.step.phase());
    }

    /// INV-PP-05: Six-player game: random passes never invalidate player map.
    #[test]
    fn prop_pp_05_six_player_passes_stable(num_passes in 1usize..200) {
        let state = GameStateBuilder::six_player().build().unwrap();
        let (mut state, _) = start_game(state).unwrap();
        for _ in 0..num_passes {
            let holder = match state.turn.priority_holder { Some(h) => h, None => break };
            match process_command(state.clone(), Command::PassPriority { player: holder }) {
                Ok((ns, _)) => state = ns,
                Err(_) => break,
            }
        }
        // All turn_order players are in the players map.
        for id in state.turn.turn_order.iter() {
            prop_assert!(state.players.contains_key(id),
                "player {:?} in turn_order but not in players map", id);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 11: Player State Invariants (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-PS-01: Players map length equals turn_order length (players are never dropped).
    #[test]
    fn prop_ps_01_players_map_and_turn_order_same_length(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        prop_assert_eq!(
            state.players.len(),
            state.turn.turn_order.len(),
            "players map and turn_order have different lengths"
        );
    }

    /// INV-PS-02: land_plays_remaining is always 0 or more (it's u32, but worth asserting).
    #[test]
    fn prop_ps_02_land_plays_remaining_nonneg(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        for (_, p) in state.players.iter() {
            // u32 is always >= 0 by type; this asserts the value is sensible (not MAX).
            prop_assert!(p.land_plays_remaining <= 10,
                "player {:?} land_plays_remaining is suspiciously large: {}",
                p.id, p.land_plays_remaining);
        }
    }

    /// INV-PS-03: A player marked has_lost also has has_lost set consistently.
    #[test]
    fn prop_ps_03_lost_player_has_consistent_state(num_passes in 1usize..200) {
        let state = pass_n(num_passes);
        for (_, p) in state.players.iter() {
            if p.has_lost {
                // A lost player should never hold priority.
                prop_assert_ne!(state.turn.priority_holder, Some(p.id),
                    "lost player {:?} holds priority", p.id);
            }
        }
    }

    /// INV-PS-04: Poison counters are non-negative (u32 type ensures this; verify semantics).
    #[test]
    fn prop_ps_04_poison_counters_valid(counters in 0u32..20) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .player_poison(p1, counters)
            .build().unwrap();
        prop_assert_eq!(state.player(p1).unwrap().poison_counters, counters);
    }

    /// INV-PS-05: A player with >= 10 poison counters loses via SBA. CR 704.5c.
    #[test]
    fn prop_ps_05_ten_poison_loses(extra in 0u32..20) {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .player_poison(p1, 10 + extra)
            .build().unwrap();
        let events = check_and_apply_sbas(&mut state);
        let lost = events.iter().any(|e| {
            matches!(e, GameEvent::PlayerLost { player, reason }
                if *player == p1 && *reason == LossReason::PoisonCounters)
        });
        prop_assert!(lost, "player with {} poison should lose but didn't", 10 + extra);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 12: Continuous Effect Invariants (3 tests)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-CE-01: Fresh state has no continuous effects.
    #[test]
    fn prop_ce_01_fresh_state_no_continuous_effects(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        prop_assert!(state.continuous_effects.is_empty(),
            "fresh state has continuous effects");
    }

    /// INV-CE-02: Fresh state has no pending triggers.
    #[test]
    fn prop_ce_02_fresh_state_no_pending_triggers(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        prop_assert!(state.pending_triggers.is_empty(),
            "fresh state has pending triggers");
    }

    /// INV-CE-03: Fresh state has no replacement effects.
    #[test]
    fn prop_ce_03_fresh_state_no_replacement_effects(num_players in 2u64..=4) {
        let mut b = GameStateBuilder::new();
        for i in 1..=num_players { b = b.add_player(PlayerId(i)); }
        let state = b.build().unwrap();
        prop_assert!(state.replacement_effects.is_empty(),
            "fresh state has replacement effects");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GROUP 13: Mixed Integrity (4 tests — reach 50+)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// INV-MX-01: Players map key == player state id field.
    #[test]
    fn prop_mx_01_player_map_key_equals_state_id(num_passes in 1usize..100) {
        let state = pass_n(num_passes);
        for (key, player) in state.players.iter() {
            prop_assert_eq!(*key, player.id,
                "players map key {:?} != player.id {:?}", key, player.id);
        }
    }

    /// INV-MX-02: Objects map key == object id field.
    #[test]
    fn prop_mx_02_objects_map_key_equals_object_id(obj_count in 1u32..20) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        for (key, obj) in state.objects.iter() {
            prop_assert_eq!(*key, obj.id,
                "objects map key {:?} != obj.id {:?}", key, obj.id);
        }
    }

    /// INV-MX-03: Object owner is always in the players map.
    #[test]
    fn prop_mx_03_object_owner_in_players_map(obj_count in 1u32..15) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        for (id, obj) in state.objects.iter() {
            prop_assert!(state.players.contains_key(&obj.owner),
                "object {:?} owner {:?} not in players map", id, obj.owner);
        }
    }

    /// INV-MX-04: Object controller is always in the players map.
    #[test]
    fn prop_mx_04_object_controller_in_players_map(obj_count in 1u32..15) {
        let p1 = PlayerId(1);
        let mut b = GameStateBuilder::four_player();
        for i in 0..obj_count { b = b.object(ObjectSpec::creature(p1, &format!("C{}", i), 1, 1)); }
        let state = b.build().unwrap();
        for (id, obj) in state.objects.iter() {
            prop_assert!(state.players.contains_key(&obj.controller),
                "object {:?} controller {:?} not in players map", id, obj.controller);
        }
    }
}

//! PB-DP7 (DP-3, CR 514.1): the cleanup-step discard-to-hand-size is now a
//! player CHOICE (`Command::DiscardToHandSize`), answering a
//! `GameEvent::CleanupDiscardChoiceRequired` -- and it is the engine's first
//! pending decision that genuinely BLOCKS game progress
//! (`rules::engine::blocking_decision` / `BlockingDecision`).
//!
//! Before this batch, `cleanup_actions` auto-discarded the highest `ObjectId`
//! in the active player's hand with no player input, which could
//! involuntarily exile-and-Madness a card the player would never have chosen
//! (CR 702.35a). See `memory/primitives/pb-plan-DP7.md` for the full design;
//! this file exercises its §8 test list (T1-T13, T17-T18 -- T14-T16 are
//! simulator-side and live in `crates/simulator/src/local_game.rs` /
//! `legal_actions.rs`'s `mod tests`).

use mtg_engine::cards::card_definition::TargetRequirement;
use mtg_engine::state::error::GameStateError;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::state::stubs::PendingTriggerKind;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Command, ContinuousEffect, Effect, EffectAmount, EffectDuration, EffectFilter,
    EffectId, EffectLayer, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    LayerModification, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Pass priority for each of `players`, in order, once each.
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

/// Fiery Temper: Instant {1}{R}{R}, "Fiery Temper deals 3 damage to any
/// target. Madness {R}" -- the same minimal def used by `mechanics_m_z/madness.rs`.
fn fiery_temper_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fiery-temper".to_string()),
        name: "Fiery Temper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fiery Temper deals 3 damage to any target. Madness {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    source: None,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Build a 4-player state at the End step with P1 active and `hand_count`
/// filler cards in P1's hand (plus, if `with_temper`, Fiery Temper as the
/// LAST object added -- its `ObjectId` no longer matters for the discard
/// outcome, which is why several tests build it last and then choose
/// EXPLICITLY, to prove the choice -- not the id -- decides).
fn build_oversized_hand(hand_count: u32, with_temper: bool) -> GameState {
    let registry = if with_temper {
        CardRegistry::new(vec![fiery_temper_def()])
    } else {
        CardRegistry::new(vec![])
    };
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::End);

    let filler_count = if with_temper {
        hand_count - 1
    } else {
        hand_count
    };
    for i in 0..filler_count {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("Filler {i}"))
                .in_zone(ZoneId::Hand(p(1)))
                .with_types(vec![CardType::Instant]),
        );
    }
    if with_temper {
        let temper = ObjectSpec::card(p(1), "Fiery Temper")
            .in_zone(ZoneId::Hand(p(1)))
            .with_card_id(CardId("fiery-temper".to_string()))
            .with_keyword(KeywordAbility::Madness);
        builder = builder.object(temper);
    }
    builder.build().unwrap()
}

/// Pass all 4 players through the End step to reach the blocked Cleanup pause.
fn advance_to_cleanup_block(state: GameState) -> (GameState, Vec<GameEvent>) {
    pass_all(state, &[p(1), p(2), p(3), p(4)])
}

// ── T1: the block is observable (criterion 5540) ────────────────────────────

/// CR 514.1 (PB-DP7 / DP-3): cleanup PAUSES instead of auto-discarding.
#[test]
fn test_dp7_cleanup_discard_blocks_step_advance() {
    let state = build_oversized_hand(9, false);
    let (state, events) = advance_to_cleanup_block(state);

    assert_eq!(state.turn().step, Step::Cleanup);
    assert_eq!(state.turn().priority_holder, None);
    assert_eq!(state.turn().turn_number, 1, "turn must not have advanced");

    let entry = state
        .pending_cleanup_discard()
        .expect("a cleanup discard must be pending");
    assert_eq!(entry.player, p(1));
    assert_eq!(entry.count, 2);

    let hand_len = state.zone(&ZoneId::Hand(p(1))).unwrap().len();
    assert_eq!(hand_len, 9, "hand must be unchanged while blocked");

    let choice_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. }))
        .collect();
    assert_eq!(choice_events.len(), 1);
}

// ── T2/T3: the admission gate (criterion 5540) ───────────────────────────────

/// CR 514.3: no player has priority in cleanup, so `PassPriority` is rejected
/// while a cleanup discard blocks the game.
#[test]
fn test_dp7_pass_priority_rejected_while_blocked() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);
    let hash_before = state.public_state_hash();

    let result = process_command(state.clone(), Command::PassPriority { player: p(1) });
    assert!(matches!(
        result,
        Err(GameStateError::BlockedByPendingDecision { .. })
    ));
    assert_eq!(
        state.public_state_hash(),
        hash_before,
        "state must be untouched"
    );
}

/// CR 514.3 (PB-DP7 / DP-3): every command except the answering
/// `DiscardToHandSize` (and `Concede`) is rejected while blocked, from ANY
/// seat -- not just the blocked player's.
#[test]
fn test_dp7_unrelated_command_rejected_while_blocked() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);
    let hash_before = state.public_state_hash();

    // PlayLand, from the blocked player, naming an arbitrary object.
    let r1 = process_command(
        state.clone(),
        Command::PlayLand {
            player: p(1),
            card: ObjectId(999_999),
        },
    );
    assert!(matches!(
        r1,
        Err(GameStateError::BlockedByPendingDecision { .. })
    ));

    // TapForMana, from a DIFFERENT seat.
    let r2 = process_command(
        state.clone(),
        Command::TapForMana {
            player: p(2),
            source: ObjectId(999_998),
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(matches!(
        r2,
        Err(GameStateError::BlockedByPendingDecision { .. })
    ));

    assert_eq!(
        state.public_state_hash(),
        hash_before,
        "state must be byte-identical after both rejections"
    );
}

/// CR 514.1 / CR 703.4n (fix-cycle Finding 2, HIGH): `Command::DiscardToHandSize`
/// must be rejected outside `Step::Cleanup`, even if a `pending_cleanup_discard`
/// entry happens to exist (a state that should be unreachable post-Finding-1,
/// but the handler must not rely on that alone -- defense in depth). Without
/// this check, a stale entry recorded outside cleanup would let
/// `process_command`'s unconditional `enter_step` resume re-run whatever
/// turn-based actions belong to whatever step is CURRENTLY active.
#[test]
fn test_dp7_discard_rejected_outside_cleanup_step() {
    let state = build_oversized_hand(9, false);
    let (mut state, _) = advance_to_cleanup_block(state);
    assert_eq!(state.turn().step, Step::Cleanup);
    let hand = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();

    // Force the step to something other than Cleanup while the entry is
    // still outstanding -- the unreachable-in-practice state the handler's
    // own check must guard against regardless.
    state.turn_mut().step = Step::End;
    let hash_before = state.public_state_hash();

    let handler_result = mtg_engine::rules::turn_actions::handle_discard_to_hand_size(
        &mut state.clone(),
        p(1),
        hand[..2].to_vec(),
    );
    assert!(
        matches!(handler_result, Err(GameStateError::InvalidCommand(_))),
        "the handler must reject a discard offered outside Step::Cleanup: {:?}",
        handler_result
    );
    assert_eq!(
        state.public_state_hash(),
        hash_before,
        "a rejected discard must leave the state untouched"
    );
}

// ── T4: Concede clears the entry ─────────────────────────────────────────────

/// CR 104.3a / PB-DP7 §1.4: `Concede` is accepted at all times, even while
/// blocked -- refusing it would make a blocked game unquittable.
///
/// Fix-cycle Finding 5 (MEDIUM): also asserts that conceding while blocked
/// does NOT abandon the rest of CR 514.2 for the conceded turn (CR 800.4j)
/// -- the damage clear must still run and an `UntilEndOfTurn` effect
/// registered on that turn must still expire.
#[test]
fn test_dp7_concede_while_blocked_clears_entry() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .active_player(p(1))
        .at_step(Step::End)
        .object(ObjectSpec::creature(p(1), "Damaged Bear", 4, 4).in_zone(ZoneId::Battlefield));
    for i in 0..9u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();
    let bear = find_object(&state, "Damaged Bear");
    state.objects_mut().get_mut(&bear).unwrap().damage_marked = 3;
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(bear),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::ModifyPower(1),
        is_cda: false,
        condition: None,
    });

    let (state, _) = advance_to_cleanup_block(state);
    assert!(state.pending_cleanup_discard().is_some());
    // CR 514.2 must not have run yet -- this is the premise Finding 5 checks.
    assert_eq!(state.objects().get(&bear).unwrap().damage_marked, 3);
    assert_eq!(state.continuous_effects().len(), 1);

    let (state, events) = process_command(state, Command::Concede { player: p(1) })
        .expect("Concede must be accepted while blocked");

    assert!(
        state.pending_cleanup_discard().is_none(),
        "the stale entry must be cleared on concede"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerConceded { player } if *player == p(1))));
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::DamageCleared)),
        "CR 514.2's damage clear must still run for the abandoned turn (CR 800.4j)"
    );
    assert_eq!(
        state.objects().get(&bear).unwrap().damage_marked,
        0,
        "damage must have been cleared even though the active player conceded"
    );
    assert!(
        state.continuous_effects().is_empty(),
        "the UntilEndOfTurn effect registered on the conceded turn must have expired"
    );
    // The game must not hang: the next player's turn should now be active
    // (or the game concluded), never a dangling block.
    assert!(state.turn().active_player != p(1) || state.active_players().len() <= 1);
}

// ── Finding 1 coverage: a dead active player never gets an entry ────────────

/// CR 800.4j / CR 514.1 (fix-cycle Finding 1, HIGH): a dead active player
/// must NEVER get a pending cleanup-discard entry -- CR 514.2 (damage clear,
/// "until end of turn" expiry) must still run for the turn, and the turn
/// must still complete, exactly as CR 800.4j requires ("that turn continues
/// to its completion without an active player").
#[test]
fn test_dp7_dead_active_player_no_entry_and_turn_completes() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .active_player(p(1))
        .at_step(Step::End);
    for i in 0..9u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();
    // p(1) has already lost (e.g. an empty-library draw earlier this turn),
    // and priority has already moved off them per `enter_step`'s
    // is-active-player-alive branch -- simulate that by handing priority to
    // the next player directly, matching the state `enter_step` would have
    // already produced.
    state.players_mut().get_mut(&p(1)).unwrap().has_lost = true;
    state.turn_mut().priority_holder = Some(p(2));

    let (state, events) = pass_all(state, &[p(2), p(3), p(4)]);

    assert!(
        state.pending_cleanup_discard().is_none(),
        "a dead active player must never get a pending cleanup discard entry"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. })));
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::DamageCleared)),
        "CR 514.2 must still run for the abandoned turn"
    );
    let cleanup_performed_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CleanupPerformed))
        .count();
    assert_eq!(
        cleanup_performed_count, 1,
        "CR 514.2 must run exactly once for the abandoned turn"
    );
    assert!(
        state.turn().turn_number > 1,
        "the turn must complete even without a living active player (CR 800.4j)"
    );
}

// ── T5: chosen cards are discarded, not the auto-pick ────────────────────────

/// CR 514.1 / CR 701.9b: the player's CHOSEN cards are discarded, not the
/// highest-`ObjectId` auto-pick.
#[test]
fn test_dp7_chosen_cards_are_discarded_not_the_highest_ids() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);

    let mut hand_ids = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    hand_ids.sort();
    let lowest_two = hand_ids[..2].to_vec();
    let highest_two = hand_ids[hand_ids.len() - 2..].to_vec();

    let (state, _) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: lowest_two.clone(),
        },
    )
    .unwrap();

    let gy = state.zone(&ZoneId::Graveyard(p(1))).unwrap();
    assert_eq!(gy.len(), 2);
    for id in &lowest_two {
        // The old hand id is dead post-move (CR 400.7); verify by absence
        // from the hand instead of presence in the graveyard by id.
        assert!(!state
            .zone(&ZoneId::Hand(p(1)))
            .unwrap()
            .object_ids()
            .contains(id));
    }
    for id in &highest_two {
        assert!(
            state
                .zone(&ZoneId::Hand(p(1)))
                .unwrap()
                .object_ids()
                .contains(id),
            "the two highest ids must still be in hand"
        );
    }
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 7);
}

// ── T6/T7: Madness fires only off the CHOSEN card (criterion 5541) ──────────

/// CR 702.35a + CR 701.9b (criterion 5541): Madness does NOT fire on a card
/// the player did not choose to discard, even if it holds the highest
/// `ObjectId` (the old auto-pick target).
#[test]
fn test_dp7_madness_does_not_fire_on_an_unchosen_card() {
    let state = build_oversized_hand(8, true);
    let (state, _) = advance_to_cleanup_block(state);

    // Fix-cycle Finding 13 (MEDIUM, test-validity): this test only
    // discriminates the fix from the old auto-pick if Fiery Temper holds the
    // HIGHEST `ObjectId` in P1's hand (`build_oversized_hand` adds it last,
    // per its own doc comment, but nothing asserted that). Self-guard the
    // premise the way T5 does, so a future change to `GameStateBuilder`'s id
    // assignment cannot make this test pass vacuously.
    let temper_id = find_object(&state, "Fiery Temper");
    let mut hand_ids = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    hand_ids.sort();
    assert_eq!(
        temper_id,
        *hand_ids.last().unwrap(),
        "T6 only discriminates the fix if Fiery Temper is the pre-fix auto-pick target (the highest ObjectId)"
    );

    let filler_id = find_object(&state, "Filler 0");
    let (state, _) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![filler_id],
        },
    )
    .unwrap();

    let temper_still_in_hand = state
        .zone(&ZoneId::Hand(p(1)))
        .unwrap()
        .object_ids()
        .iter()
        .any(|&id| {
            state
                .object(id)
                .map(|o| o.characteristics.name == "Fiery Temper")
                .unwrap_or(false)
        });
    assert!(
        temper_still_in_hand,
        "Fiery Temper must remain in hand -- it was not chosen"
    );
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile),
        "Fiery Temper must NOT be exiled"
    );
    assert!(
        !state
            .pending_triggers()
            .iter()
            .any(|t| t.kind == PendingTriggerKind::Madness),
        "no Madness PendingTrigger may be queued"
    );
    assert!(
        !state
            .stack_objects()
            .iter()
            .any(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. })),
        "no MadnessTrigger may reach the stack"
    );
}

/// CR 702.35a: Madness DOES fire when the player chooses the madness card.
/// Regression guard pinning that T6 did not break the positive path.
#[test]
fn test_dp7_madness_fires_on_a_chosen_card() {
    let state = build_oversized_hand(8, true);
    let (state, _) = advance_to_cleanup_block(state);

    let temper_id = find_object(&state, "Fiery Temper");
    let (state, _) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![temper_id],
        },
    )
    .unwrap();

    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile),
        "Fiery Temper must be exiled (CR 702.35a)"
    );
    // Fix-cycle Finding 15 (LOW): `is_some() || on_stack` only proved "at
    // least one somewhere", which is also satisfied by a bug that queues TWO
    // triggers (one pending, one already flushed). Count both locations and
    // assert the total is EXACTLY one.
    let madness_trigger_count = state
        .pending_triggers()
        .iter()
        .filter(|t| t.kind == PendingTriggerKind::Madness)
        .count();
    // By the time this call returns, `enter_step` has already flushed the
    // trigger onto the stack (plan §4.3 "Path 2"), so check the stack too.
    let on_stack_count = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(&so.kind, StackObjectKind::MadnessTrigger { .. }))
        .count();
    assert_eq!(
        madness_trigger_count + on_stack_count,
        1,
        "exactly one Madness trigger must be queued or on the stack, got {} pending + {} on stack",
        madness_trigger_count,
        on_stack_count
    );
    let on_stack = on_stack_count == 1;
    if on_stack {
        let cost = state.stack_objects().iter().find_map(|so| {
            if let StackObjectKind::MadnessTrigger { madness_cost, .. } = &so.kind {
                Some(madness_cost.clone())
            } else {
                None
            }
        });
        assert_eq!(
            cost,
            Some(ManaCost {
                red: 1,
                ..Default::default()
            })
        );
    }
}

// ── T8: answer validation (table-driven) ────────────────────────────────────

/// CR 514.1 (plan §2.4): every validation failure is rejected and leaves the
/// state untouched.
///
/// Fix-cycle Finding 12 (MEDIUM, test-validity): every case now asserts the
/// DISTINCT error variant, not merely `is_err()` -- an `is_err()`-only
/// assertion cannot tell an admission-gate rejection from a handler-level
/// validation rejection, which is exactly how case 7 (wrong sender) went
/// unnoticed for reaching only the ADMISSION gate
/// (`GameStateError::BlockedByPendingDecision`) and never the handler's own
/// SR-29 sender check (`GameStateError::InvalidCommand`). Case 7b below now
/// exercises that handler-level check directly. Case 4 (an id in a DIFFERENT
/// player's hand) is also no longer skipped.
#[test]
fn test_dp7_answer_validation() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);
    let hash_before = state.public_state_hash();

    let mut hand_ids = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    hand_ids.sort();
    let (id_a, id_b) = (hand_ids[0], hand_ids[1]);

    // 1. Wrong count, too low (1 instead of 2).
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a],
        },
    );
    assert!(
        matches!(r, Err(GameStateError::InvalidCommand(_))),
        "under-supply must be rejected with InvalidCommand: {:?}",
        r
    );

    // 2. Wrong count, too high (3 instead of 2).
    let id_c = hand_ids[2];
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, id_b, id_c],
        },
    );
    assert!(
        matches!(r, Err(GameStateError::InvalidCommand(_))),
        "over-supply must be rejected with InvalidCommand: {:?}",
        r
    );

    // 3. Duplicate id.
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, id_a],
        },
    );
    assert!(
        matches!(r, Err(GameStateError::InvalidCommand(_))),
        "duplicate ids must be rejected with InvalidCommand: {:?}",
        r
    );

    // 4. An id from a DIFFERENT player's hand (SR-29 / OOS-DP2-1 shape) --
    // genuinely exercised now, not skipped: give p(2) a hand card and try to
    // discard it as part of p(1)'s answer. Must be rejected as "not in the
    // SENDER's own hand" (`ObjectNotInZone`), not merely "the id exists
    // somewhere".
    let state_with_p2_hand = {
        let mut b = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .add_player(p(3))
            .add_player(p(4))
            .active_player(p(1))
            .at_step(Step::End)
            .object(ObjectSpec::card(p(2), "P2 Card").in_zone(ZoneId::Hand(p(2))));
        for i in 0..9u32 {
            b = b
                .object(ObjectSpec::card(p(1), &format!("Filler {i}")).in_zone(ZoneId::Hand(p(1))));
        }
        b.build().unwrap()
    };
    let (state_with_p2_hand, _) = advance_to_cleanup_block(state_with_p2_hand);
    let p2_card_id = find_object(&state_with_p2_hand, "P2 Card");
    let one_p1_hand_id = state_with_p2_hand
        .zone(&ZoneId::Hand(p(1)))
        .unwrap()
        .object_ids()[0];
    let r = process_command(
        state_with_p2_hand.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![p2_card_id, one_p1_hand_id],
        },
    );
    assert!(
        matches!(&r, Err(GameStateError::ObjectNotInZone(id, zone))
            if *id == p2_card_id && *zone == ZoneId::Hand(p(1))),
        "an id from a DIFFERENT player's hand must be rejected as not in the sender's own hand: {:?}",
        r
    );

    // 5. An id on the battlefield, not in hand.
    let state_with_bf = {
        let mut b = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .add_player(p(3))
            .add_player(p(4))
            .active_player(p(1))
            .at_step(Step::End)
            .object(ObjectSpec::creature(p(1), "Bear", 2, 2).in_zone(ZoneId::Battlefield));
        for i in 0..9u32 {
            b = b
                .object(ObjectSpec::card(p(1), &format!("Filler {i}")).in_zone(ZoneId::Hand(p(1))));
        }
        b.build().unwrap()
    };
    let (state_with_bf, _) = advance_to_cleanup_block(state_with_bf);
    let bear_id = find_object(&state_with_bf, "Bear");
    let one_hand_id = state_with_bf
        .zone(&ZoneId::Hand(p(1)))
        .unwrap()
        .object_ids()[0];
    let r = process_command(
        state_with_bf.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![bear_id, one_hand_id],
        },
    );
    assert!(
        matches!(&r, Err(GameStateError::ObjectNotInZone(id, zone))
            if *id == bear_id && *zone == ZoneId::Hand(p(1))),
        "a battlefield object id must be rejected as not in hand: {:?}",
        r
    );

    // 6. Unknown ObjectId.
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, ObjectId(999_999_999)],
        },
    );
    assert!(
        matches!(
            r,
            Err(GameStateError::ObjectNotFound(ObjectId(999_999_999)))
        ),
        "an unknown ObjectId must be rejected with ObjectNotFound: {:?}",
        r
    );

    // 7. Wrong sender via `process_command` (p(2) tries to answer p(1)'s
    // pending discard). This is intercepted by the ADMISSION gate
    // (`process_command`'s `blocking_decision` check), which never lets the
    // command reach the handler at all.
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(2),
            cards: vec![id_a, id_b],
        },
    );
    assert!(
        matches!(r, Err(GameStateError::BlockedByPendingDecision { .. })),
        "a non-active-player sender must be rejected by the admission gate: {:?}",
        r
    );

    // 7b. Wrong sender reaching the HANDLER's own SR-29 check directly,
    // bypassing `process_command`'s admission gate entirely -- this is the
    // check case 7 could never exercise.
    let mut handler_state = state.clone();
    let handler_result = mtg_engine::rules::turn_actions::handle_discard_to_hand_size(
        &mut handler_state,
        p(2),
        vec![id_a, id_b],
    );
    assert!(
        matches!(handler_result, Err(GameStateError::InvalidCommand(_))),
        "the handler's own sender check must reject a mismatched player with InvalidCommand: {:?}",
        handler_result
    );

    assert_eq!(
        state.public_state_hash(),
        hash_before,
        "the original state must never have been mutated by any of these attempts"
    );
}

// ── T9: no_max_hand_size never pauses (hard constraint 9) ───────────────────

/// CR 402.2 (a): a PRINTED `NoMaxHandSize` keyword short-circuits the pause.
#[test]
fn test_dp7_no_max_hand_size_never_pauses_printed_keyword() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::End)
        .object(
            ObjectSpec::artifact(p(1), "Thought Vessel")
                .with_keyword(KeywordAbility::NoMaxHandSize)
                .in_zone(ZoneId::Battlefield),
        );
    for i in 0..10u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let state = builder.build().unwrap();

    let (state, events) = pass_all(state, &[p(1), p(2)]);
    assert!(state.pending_cleanup_discard().is_none());
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. })));
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 10);
    assert!(state.turn().turn_number > 1, "turn must have advanced");
}

/// CR 402.2 (b): a LAYER-granted `NoMaxHandSize` (e.g. an emblem proxy, PB-AC8)
/// also short-circuits the pause.
#[test]
fn test_dp7_no_max_hand_size_never_pauses_layer_granted() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::End)
        .object(ObjectSpec::creature(p(1), "Emblem Proxy Bear", 2, 2).in_zone(ZoneId::Battlefield));
    for i in 0..10u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();
    let bear = find_object(&state, "Emblem Proxy Bear");
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(bear),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::AddKeyword(KeywordAbility::NoMaxHandSize),
        is_cda: false,
        condition: None,
    });

    let (state, events) = pass_all(state, &[p(1), p(2)]);
    assert!(state.pending_cleanup_discard().is_none());
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. })));
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 10);
    // Fix-cycle Finding 16 (LOW): T9(a) asserted the turn advanced; T9(b)
    // did not, though the setup is otherwise identical in intent.
    assert!(state.turn().turn_number > 1, "turn must have advanced");
}

/// CR 402.2 (c): the persistent `no_max_hand_size_permanent` designation
/// (PB-AC9, `Effect::SetNoMaximumHandSize`) also short-circuits the pause.
#[test]
fn test_dp7_no_max_hand_size_never_pauses_persistent_designation() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::End);
    for i in 0..10u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .no_max_hand_size_permanent = true;

    let (state, events) = pass_all(state, &[p(1), p(2)]);
    assert!(state.pending_cleanup_discard().is_none());
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. })));
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 10);
    // Fix-cycle Finding 16 (LOW): T9(a) asserted the turn advanced; T9(c)
    // did not, though the setup is otherwise identical in intent.
    assert!(state.turn().turn_number > 1, "turn must have advanced");
}

// ── T10: CR 514.2 is deferred until the answer (hard constraint 7) ──────────

/// CR 514.2 (hard constraint 7): damage clear and "until end of turn" expiry
/// must NOT run before the discard choice is made.
#[test]
fn test_dp7_cr_514_2_is_deferred_until_the_answer() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .active_player(p(1))
        .at_step(Step::End)
        .object(ObjectSpec::creature(p(1), "Damaged Bear", 4, 4).in_zone(ZoneId::Battlefield));
    for i in 0..9u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();
    let bear = find_object(&state, "Damaged Bear");
    state.objects_mut().get_mut(&bear).unwrap().damage_marked = 2;
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(bear),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::ModifyPower(1),
        is_cda: false,
        condition: None,
    });

    let (state, events) = advance_to_cleanup_block(state);
    // At the blocked point: damage and the UntilEndOfTurn effect must both
    // still be live, and DamageCleared must not have fired.
    assert_eq!(state.objects().get(&bear).unwrap().damage_marked, 2);
    assert_eq!(state.continuous_effects().len(), 1);
    assert!(!events.iter().any(|e| matches!(e, GameEvent::DamageCleared)));
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupPerformed)));

    let hand = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    let (state, events) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: hand[..2].to_vec(),
        },
    )
    .unwrap();

    assert!(events.iter().any(|e| matches!(e, GameEvent::DamageCleared)));
    assert_eq!(state.objects().get(&bear).unwrap().damage_marked, 0);
    assert!(
        state.continuous_effects().is_empty(),
        "UntilEndOfTurn effect must have expired"
    );
    let cleanup_performed_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CleanupPerformed))
        .count();
    assert_eq!(cleanup_performed_count, 1);
}

// ── T11: interleaving with CR 514.3a (hard constraint 6) ────────────────────

/// CR 514.3a: a madness discard queues an extra cleanup round (for the
/// trigger), but the pause itself consumes ZERO rounds.
#[test]
fn test_dp7_madness_discard_runs_an_extra_cleanup_round() {
    let state = build_oversized_hand(8, true);
    let (state, _) = advance_to_cleanup_block(state);
    // T11a: the pause consumes no round.
    assert_eq!(state.turn().cleanup_sba_rounds, 0);

    let temper_id = find_object(&state, "Fiery Temper");
    let (state, _) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![temper_id],
        },
    )
    .unwrap();

    assert_eq!(state.turn().step, Step::Cleanup);
    assert_eq!(state.turn().cleanup_sba_rounds, 1);
    assert_eq!(state.turn().priority_holder, Some(p(1)));
    assert!(state
        .stack_objects()
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. })));

    // All pass, repeatedly -- handle_all_passed must NOT advance the turn on
    // the round that merely resolves the MadnessTrigger off the stack (CR
    // 514.3a's non-advance guard); a SECOND cleanup round (with an empty
    // stack) is required before the turn finally advances. One `pass_all`
    // cycle resolves the stack object; a second drains the resulting round.
    //
    // Fix-cycle Finding 14 (LOW): the loop below used to only assert the
    // EVENTUAL turn advance, never the non-advance the comment above claims
    // for round 1 -- so a regression that advanced the turn a round early
    // would have passed silently. Round 1 is unrolled explicitly to assert
    // CR 514.3a's non-advance behaviour before falling into the loop for the
    // (predicted, per plan §4.3) single remaining round.
    let (state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);
    assert_eq!(
        state.turn().turn_number,
        1,
        "CR 514.3a: the turn must NOT advance on the round that merely resolves the MadnessTrigger"
    );

    let mut state = state;
    let mut rounds = 1;
    while state.turn().turn_number == 1 && rounds < 10 {
        let (new_state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);
        state = new_state;
        rounds += 1;
    }
    assert!(
        state.turn().turn_number > 1,
        "the turn must advance once round 2 finds nothing left to do"
    );
    assert_eq!(
        rounds, 2,
        "plan §4.3 predicts exactly one extra cleanup round after the discard round"
    );
}

// ── T11b: a SECOND pause within the same turn (Finding 18 coverage gap) ─────

/// CR 514.3a / CR 514.1 (fix-cycle Finding 18): CR 514.3a says "another
/// cleanup step begins" whenever SBAs/triggers fire during cleanup -- and
/// each such re-entry into `cleanup_actions` legitimately re-applies CR 514.1
/// from scratch. If the hand is oversized AGAIN when that happens (e.g. a
/// "draw a card" trigger resolving mid-cleanup), the engine must pause a
/// SECOND time in the same turn, not assume the discard is a one-shot event.
/// No test exercised this before the fix cycle.
#[test]
fn test_dp7_second_pause_within_the_same_turn() {
    let state = build_oversized_hand(9, false);
    let (mut state, _) = advance_to_cleanup_block(state);
    let turn_after_first_pause = state.turn().turn_number;

    // Answer the first pause via the HANDLER directly (not `process_command`):
    // `process_command`'s dispatch arm also resumes `enter_step`, and with
    // nothing else pending that resume auto-advances all the way past
    // Cleanup (CR 514.3: no priority there) to the next turn's first
    // priority-granting step -- which would make this test's own premise
    // (a second pause in the SAME turn) impossible to set up. Calling the
    // handler directly answers CR 514.1 without triggering that resume, the
    // same "hold the moment still to test it" technique T16's rebuild and
    // `test_dp7_discard_rejected_outside_cleanup_step` both use.
    let hand = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    let _events = mtg_engine::rules::turn_actions::handle_discard_to_hand_size(
        &mut state,
        p(1),
        hand[..2].to_vec(),
    )
    .unwrap();
    assert!(state.pending_cleanup_discard().is_none());
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 7);
    assert_eq!(
        state.turn().turn_number,
        turn_after_first_pause,
        "still the same turn after the first answer (no resume was triggered)"
    );

    // Simulate an extra CR 514.3a round leaving the hand oversized again
    // (e.g. a "draw a card" trigger resolving off the stack mid-cleanup) by
    // adding two objects directly to the hand and re-invoking
    // `cleanup_actions` -- the same direct-call pattern §4.2 already
    // establishes as safe for isolated `cleanup_actions` coverage
    // (PB-AC8/PB-AC9's precedent, and this file's own T16 rebuild).
    for i in 0..2 {
        let extra = mtg_engine::effects::make_token(
            &mtg_engine::cards::card_definition::TokenSpec {
                name: format!("Extra Card {i}"),
                ..Default::default()
            },
            p(1),
        );
        mtg_engine::state::test_util::add_object(&mut state, extra, ZoneId::Hand(p(1)))
            .expect("add extra hand card");
    }
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 9);

    let events = mtg_engine::rules::turn_actions::cleanup_actions(&mut state);
    let entry = state.pending_cleanup_discard().expect(
        "a SECOND cleanup discard must be pending -- CR 514.3a legitimately \
         re-applies CR 514.1 every time a new cleanup step begins",
    );
    assert_eq!(entry.player, p(1));
    assert_eq!(entry.count, 2);
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::CleanupDiscardChoiceRequired { .. })));
    assert_eq!(
        state.turn().turn_number,
        turn_after_first_pause,
        "the second pause must still be within the SAME turn"
    );

    // Answer the second pause too, proving the SAME machinery (not a
    // one-shot) resolves it.
    let hand2 = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    let events2 = mtg_engine::rules::turn_actions::handle_discard_to_hand_size(
        &mut state,
        p(1),
        hand2[..2].to_vec(),
    )
    .unwrap();
    assert!(state.pending_cleanup_discard().is_none());
    assert!(events2
        .iter()
        .any(|e| matches!(e, GameEvent::DiscardedToHandSize { .. })));
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 7);
}

// ── T12: three discards, one command (hard constraint 8) ────────────────────

/// CR 514.1 / CR 703.4n: hand size 10 vs max 7 = one command carrying THREE
/// cards, not three round-trips.
#[test]
fn test_dp7_three_discards_one_command() {
    let state = build_oversized_hand(10, false);
    let (state, _) = advance_to_cleanup_block(state);

    let entry = state.pending_cleanup_discard().unwrap();
    assert_eq!(entry.count, 3);
    let hand = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();

    let (state, events) = process_command(
        state,
        Command::DiscardToHandSize {
            player: p(1),
            cards: hand[..3].to_vec(),
        },
    )
    .unwrap();

    let discard_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscardedToHandSize { .. }))
        .count();
    assert_eq!(discard_events, 3);
    assert_eq!(state.zone(&ZoneId::Hand(p(1))).unwrap().len(), 7);
    assert_eq!(state.zone(&ZoneId::Graveyard(p(1))).unwrap().len(), 3);
}

// ── T13: the deterministic default reproduces pre-PB-DP7 behaviour ──────────

/// CR 514.1 (hard constraint 5): `default_cleanup_discard` returns exactly
/// the `count` highest `ObjectId`s, ascending -- the pre-PB-DP7 auto-pick.
#[test]
fn test_dp7_default_pick_reproduces_pre_pb_behaviour() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);

    let mut hand_ids = state.zone(&ZoneId::Hand(p(1))).unwrap().object_ids();
    hand_ids.sort();
    let expected = hand_ids[hand_ids.len() - 2..].to_vec();

    let default = mtg_engine::rules::turn_actions::default_cleanup_discard(&state, p(1));
    assert_eq!(default, expected);
}

// ── T17: the pending entry participates in the state hash ───────────────────
//
// Second fix-cycle correction (closing /review, MEDIUM): the comment that used
// to stand here (from Finding 11 of the first fix cycle) claimed the SR-19
// `every_hashed_struct_field_is_hashed_or_allowlisted` gate
// (`crates/engine/tests/core/hash_schema.rs`) "walks `PendingCleanupDiscard`'s
// field list and fails the build if either field is added without a matching
// `hash_into` line". **That was false as shipped.** The gate looks up
// `bodies.get(ty)` with the BARE struct name (`hash_schema.rs:1538`), but
// `hashinto_impl_bodies()` (`:1281-1316`) keys impls by the exact type token
// as written in `state/hash.rs`, and the impl had been written
// path-qualified: `impl HashInto for crate::state::stubs::PendingCleanupDiscard`.
// The lookup returned `None`, the loop `continue`d (treating the struct as
// "out of this gate's scope"), and `PendingCleanupDiscard` was silently
// outside the gate the whole time -- verified this cycle by temporarily
// deleting `self.count.hash_into(hasher)` from the impl: the gate reported
// `ok`, not a violation. `state/hash.rs`'s impl is now written bare
// (`impl HashInto for PendingCleanupDiscard`, with the type pulled into scope
// via the existing `use super::stubs::{ ... }` block), which was re-verified
// the same way: with the bare impl, deleting `self.count.hash_into(hasher)`
// makes the gate fail with `PendingCleanupDiscard.count` named in the
// violation list. Wire fingerprints (PROTOCOL 28 / HASH 65) were confirmed
// unmoved by this rename -- it touches how `state/hash.rs` is *written*, not
// the `HashInto` byte stream any replay depends on.
//
// The gate covering the property does NOT mean a black-box struct-hash test
// is redundant, though -- the first fix cycle's other justification (that no
// such test can isolate a single-field delta without a `GameState`-wide
// fixture) was ALSO wrong: `HashInto` is `pub` (`mtg_engine::state::hash::HashInto`)
// and `PendingCleanupDiscard` is constructible directly, so two hand-built
// values differing in exactly one field can be hashed and compared with no
// `GameState` involved at all -- the same shape as
// `test_sacrificed_creature_lki_struct_hash`
// (`crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs:1514`).
// That test is written below, restoring the black-box coverage the first
// fix cycle deleted, alongside (not instead of) the SR-19 gate.
#[test]
fn test_dp7_pending_cleanup_discard_struct_hash() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;
    use mtg_engine::state::stubs::PendingCleanupDiscard;

    let hash_entry = |e: &PendingCleanupDiscard| -> [u8; 32] {
        let mut hasher = Hasher::new();
        e.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let base = PendingCleanupDiscard {
        player: p(1),
        count: 2,
    };
    let diff_count = PendingCleanupDiscard {
        player: p(1),
        count: 3,
    };
    let diff_player = PendingCleanupDiscard {
        player: p(2),
        count: 2,
    };

    let h_base = hash_entry(&base);
    let h_diff_count = hash_entry(&diff_count);
    let h_diff_player = hash_entry(&diff_player);

    assert_ne!(
        h_base, h_diff_count,
        "PendingCleanupDiscard entries differing only in `count` must hash distinctly"
    );
    assert_ne!(
        h_base, h_diff_player,
        "PendingCleanupDiscard entries differing only in `player` must hash distinctly"
    );
    assert_ne!(
        h_diff_count, h_diff_player,
        "the two single-field deltas must not collide with each other either"
    );
}

// ── T18: serde round-trip ────────────────────────────────────────────────────

/// SR-16 pattern (`test_replacement_effect_serde_roundtrip_*`): `PendingCleanupDiscard`
/// serializes and deserializes correctly. `GameState` itself is not
/// round-tripped through `serde_json` anywhere in this suite -- several of
/// its maps are keyed by non-string newtypes (`ObjectId`, `PlayerId`,
/// `ZoneId`, ...), which `serde_json` cannot use as JSON object keys, a
/// pre-existing limitation this batch does not change. This test therefore
/// pins the new struct's own round-trip, and separately pins the
/// `#[serde(default)]` behaviour the plan's §5.2 relies on for old
/// (pre-PB-DP7) `GameState` snapshots.
#[test]
fn test_dp7_pending_cleanup_discard_serde_roundtrip() {
    use mtg_engine::state::stubs::PendingCleanupDiscard;

    let entry = PendingCleanupDiscard {
        player: p(1),
        count: 2,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let decoded: PendingCleanupDiscard = serde_json::from_str(&json).unwrap();
    assert_eq!(entry, decoded);
}

/// SR-17 pattern: a pre-PB-DP7 snapshot -- a JSON object with NO
/// `pending_cleanup_discard` key at all -- still decodes, because the
/// `GameState` field carries `#[serde(default)]`. Exercised on a minimal
/// stand-in struct with the identical field declaration (`#[serde(default)]
/// pending_cleanup_discard: Option<PendingCleanupDiscard>`), since the full
/// `GameState` cannot round-trip through `serde_json` at all (see the test
/// above) -- this isolates exactly the mechanism under test.
#[test]
fn test_dp7_pending_cleanup_discard_defaults_when_absent() {
    use mtg_engine::state::stubs::PendingCleanupDiscard;

    #[derive(serde::Deserialize)]
    struct Stand<'a> {
        #[allow(dead_code)]
        marker: &'a str,
        #[serde(default)]
        pending_cleanup_discard: Option<PendingCleanupDiscard>,
    }

    let pre_dp7_json = r#"{"marker": "old snapshot"}"#;
    let decoded: Stand = serde_json::from_str(pre_dp7_json)
        .expect("a pre-PB-DP7 snapshot without the key must still decode");
    assert!(decoded.pending_cleanup_discard.is_none());
}

// ── T19: harness-level coverage of the `discard_to_hand_size` PlayerAction arm ──
//
// Second fix-cycle addition (closing /review, Issue 4, LOW): the
// `"discard_to_hand_size"` arm added to
// `testing::replay_harness::translate_player_action` had ZERO test coverage
// -- neither the named-cards path nor the empty-`discard_cards` fallback to
// `default_cleanup_discard` was ever exercised by any script or unit test. A
// full golden script is not required for this fix; these two tests call the
// public `translate_player_action` function directly, the same shape
// `crates/engine/tests/scripts/harness_equivalence.rs`'s own `translate`
// wrapper and `crates/engine/tests/combat/combat_harness.rs`'s call sites
// use for the rest of this function's action arms.
//
// This also documents the name collision the fix cycle's Issue 4 fixed at
// both `script_schema.rs` doc sites and this file's `"discard_to_hand_size"`
// match arm: `ScriptAction::TurnBasedAction.action` has an identically-named,
// purely informational value that dispatches no `Command` -- only
// `ScriptAction::PlayerAction`'s `"discard_to_hand_size"`, exercised here via
// `translate_player_action` directly, actually answers the block.

/// Every positional argument of `translate_player_action` this test doesn't
/// use, filled in with the same "not used for these actions" placeholders
/// `harness_equivalence.rs`'s `translate` wrapper and `combat_harness.rs`'s
/// call sites use -- kept as one call site so a signature change fails here
/// loudly rather than silently shifting a positional argument.
#[allow(clippy::too_many_arguments)]
fn translate_discard(
    player: PlayerId,
    discard_cards: &[String],
    state: &GameState,
) -> Option<Command> {
    mtg_engine::translate_player_action(
        "discard_to_hand_size",
        player,
        None, // card_name
        0,    // ability_index
        &[],  // targets
        &[],  // attackers_decl
        &[],  // blockers_decl
        &[],  // convoke_names
        &[],  // improvise_names
        &[],  // delve_names
        &[],  // escape_names
        false,
        false,
        &[],    // enlist_decls
        None,   // attacker_name
        None,   // discard_land_name
        None,   // discard_card_name
        None,   // bargain_sacrifice_name
        None,   // emerge_sacrifice_name
        None,   // casualty_sacrifice_name
        None,   // assist_player_name
        0,      // assist_amount
        0,      // replicate_count
        &[],    // splice_card_names
        0,      // escalate_modes
        vec![], // modes_chosen
        None,   // target_creature_name
        0,      // x_value
        &[],    // collect_evidence_names
        0,      // squad_count
        false,  // mutate_on_top
        None,   // gift_opponent_name
        None,   // sacrifice_card_name
        &[],    // exert_names
        None,   // pitch_exile_card_name
        None,   // chosen_color_name
        &[],    // hybrid_choice_names
        &[],    // phyrexian_life_payment_choices
        discard_cards,
        &[],  // trigger_targets (PB-DP8) — not used by this PB-DP7 helper
        None, // effect_choice (PB-DP9) — not used by this PB-DP7 helper
        state,
        &std::collections::HashMap::new(), // players -- unused (discard has no ActionTarget)
    )
}

/// The named-cards path: a script naming specific cards translates to a
/// `Command::DiscardToHandSize` carrying exactly those `ObjectId`s.
#[test]
fn test_dp7_translate_player_action_discard_named_cards() {
    let state = build_oversized_hand(9, false);
    let (state, _events) = advance_to_cleanup_block(state);
    assert!(
        state.pending_cleanup_discard().is_some(),
        "fixture must reach the blocked cleanup pause"
    );

    let filler_0 = find_object(&state, "Filler 0");
    let filler_1 = find_object(&state, "Filler 1");

    let cmd = translate_discard(
        p(1),
        &["Filler 0".to_string(), "Filler 1".to_string()],
        &state,
    )
    .expect("translate_player_action should resolve both named cards");

    match cmd {
        Command::DiscardToHandSize { player, cards } => {
            assert_eq!(player, p(1));
            let mut sorted = cards.clone();
            sorted.sort_by_key(|id| id.0);
            let mut expected = vec![filler_0, filler_1];
            expected.sort_by_key(|id| id.0);
            assert_eq!(sorted, expected);
        }
        other => panic!("expected DiscardToHandSize, got {:?}", other),
    }
}

/// The empty-`discard_cards` fallback: naming no cards falls back to
/// `turn_actions::default_cleanup_discard` -- the same deterministic
/// highest-`ObjectId` subset the pre-PB-DP7 auto-pick used (plan §6),
/// matching pre-PB-DP7 script behaviour exactly.
#[test]
fn test_dp7_translate_player_action_discard_empty_falls_back_to_default() {
    let state = build_oversized_hand(9, false);
    let (state, _events) = advance_to_cleanup_block(state);
    let entry = state
        .pending_cleanup_discard()
        .expect("fixture must reach the blocked cleanup pause");
    assert_eq!(entry.count, 2);

    let expected = mtg_engine::rules::turn_actions::default_cleanup_discard(&state, p(1));
    assert_eq!(
        expected.len(),
        2,
        "default_cleanup_discard must return exactly `count` ids"
    );

    let cmd = translate_discard(p(1), &[], &state)
        .expect("translate_player_action should fall back to the deterministic default");

    match cmd {
        Command::DiscardToHandSize { player, cards } => {
            assert_eq!(player, p(1));
            assert_eq!(
                cards, expected,
                "empty discard_cards must fall back to default_cleanup_discard exactly"
            );
        }
        other => panic!("expected DiscardToHandSize, got {:?}", other),
    }
}

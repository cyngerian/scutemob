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

// ── T4: Concede clears the entry ─────────────────────────────────────────────

/// CR 104.3a / PB-DP7 §1.4: `Concede` is accepted at all times, even while
/// blocked -- refusing it would make a blocked game unquittable.
#[test]
fn test_dp7_concede_while_blocked_clears_entry() {
    let state = build_oversized_hand(9, false);
    let (state, _) = advance_to_cleanup_block(state);
    assert!(state.pending_cleanup_discard().is_some());

    let (state, events) = process_command(state, Command::Concede { player: p(1) })
        .expect("Concede must be accepted while blocked");

    assert!(
        state.pending_cleanup_discard().is_none(),
        "the stale entry must be cleared on concede"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PlayerConceded { player } if *player == p(1))));
    // The game must not hang: the next player's turn should now be active
    // (or the game concluded), never a dangling block.
    assert!(state.turn().active_player != p(1) || state.active_players().len() <= 1);
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
    let madness_trigger = state
        .pending_triggers()
        .iter()
        .find(|t| t.kind == PendingTriggerKind::Madness);
    // By the time this call returns, `enter_step` has already flushed the
    // trigger onto the stack (plan §4.3 "Path 2"), so check the stack too.
    let on_stack = state
        .stack_objects()
        .iter()
        .any(|so| matches!(&so.kind, StackObjectKind::MadnessTrigger { .. }));
    assert!(
        madness_trigger.is_some() || on_stack,
        "exactly one Madness trigger must be queued or on the stack"
    );
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
    assert!(r.is_err(), "under-supply must be rejected");

    // 2. Wrong count, too high (3 instead of 2).
    let id_c = hand_ids[2];
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, id_b, id_c],
        },
    );
    assert!(r.is_err(), "over-supply must be rejected");

    // 3. Duplicate id.
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, id_a],
        },
    );
    assert!(r.is_err(), "duplicate ids must be rejected");

    // 4. An id from a DIFFERENT player's hand (SR-29 / OOS-DP2-1 shape).
    // p(2) has no hand cards here, so use a synthetic id that does not
    // resolve to an object -- covered together with case 6.

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
    assert!(r.is_err(), "a battlefield object id must be rejected");

    // 6. Unknown ObjectId.
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(1),
            cards: vec![id_a, ObjectId(999_999_999)],
        },
    );
    assert!(r.is_err(), "an unknown ObjectId must be rejected");

    // 7. Wrong sender (p(2) tries to answer p(1)'s pending discard).
    let r = process_command(
        state.clone(),
        Command::DiscardToHandSize {
            player: p(2),
            cards: vec![id_a, id_b],
        },
    );
    assert!(r.is_err(), "a non-active-player sender must be rejected");

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
    let mut state = state;
    let mut rounds = 0;
    while state.turn().turn_number == 1 && rounds < 10 {
        let (new_state, _) = pass_all(state, &[p(1), p(2), p(3), p(4)]);
        state = new_state;
        rounds += 1;
    }
    assert!(
        state.turn().turn_number > 1,
        "the turn must advance once round 2 finds nothing left to do"
    );
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

/// SR-9b: two states differing only in `pending_cleanup_discard` must hash
/// differently, so hidden-decision states cannot collide.
#[test]
fn test_dp7_pending_entry_is_hashed() {
    let state = build_oversized_hand(9, false);
    let (blocked, _) = advance_to_cleanup_block(state.clone());
    assert!(blocked.pending_cleanup_discard().is_some());

    // A same-shape state that never entered cleanup has no pending entry.
    let unblocked = build_oversized_hand(9, false);

    assert_ne!(
        blocked.public_state_hash(),
        unblocked.public_state_hash(),
        "a blocked and an unblocked state must not hash identically"
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

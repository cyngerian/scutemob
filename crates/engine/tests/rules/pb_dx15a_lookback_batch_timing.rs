//! PB-DX15a rider `OOS-DX24-7` — the CR 603.10a look-back set is now scoped to the
//! caller's actual event TIMING, not to whatever slice a caller happened to hand in.
//!
//! # The rule
//!
//! CR 603.10a: a leaves-the-battlefield / zone-scoped ability "looks back in time" — the
//! game asks whether the ability EXISTED immediately prior to the event. A
//! `trigger_zone: Graveyard` ability (Nether Traitor's shape, CR 113.6m) did not exist
//! while its card was still on the battlefield, so a creature dying *at the same time* as
//! that card must not trigger it. Gatherer, Nether Traitor: *"If Nether Traitor and
//! another creature are put into your graveyard at the same time, Nether Traitor's
//! ability won't trigger."*
//!
//! # What "at the same time" means depends on the caller, and that is the fix
//!
//! `check_triggers` builds its suppression set from the `events` slice it is given, and
//! PB-DX24's fix cycle measured that the slice means different things per caller:
//! `sba.rs` hands in ONE CR 704.3 fixpoint pass (genuinely simultaneous), while
//! `resolution.rs` hands in a whole resolution's accumulated events (a SEQUENCE of
//! sub-effects). One set built one way cannot be right for both.
//! `EventBatchTiming` makes it the caller's declaration.
//!
//! # Two corrections to `OOS-DX24-7`'s own fix sketch, both found here
//!
//! The row says: *"rebuild the set per event **prefix** rather than per whole slice, so
//! each event looks back only at deaths strictly earlier in `events`' order."*
//!
//! 1. **Applied to every caller, it makes `sba.rs` wrong** — and `sba.rs` is the caller
//!    the guard was written for. Within one simultaneous batch there IS no earlier or
//!    later, so a prefix makes the answer depend on the slice's incidental ordering.
//!    `t2` pins that: both orderings of a simultaneous batch suppress.
//! 2. **The prefix is what to SUBTRACT, not what to pass.** The set is a *suppression*
//!    set. A source that arrived at an earlier event was already in the graveyard, so it
//!    must be REMOVED. Passing the prefix itself inverts the guard and reproduces the
//!    very defect the row describes — `t1` is the row's own example and would fail under
//!    its own sketch.

use std::collections::HashSet;

use mtg_engine::cards::card_definition::TriggerZone;
use mtg_engine::rules::abilities::{check_triggers_with_timing, EventBatchTiming};
use mtg_engine::rules::events::GameEvent;
use mtg_engine::{
    AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Completeness, Effect,
    EffectAmount, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step,
    TriggerCondition, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Nether Traitor's shape, reduced to the one clause under test: a
/// `trigger_zone: Graveyard` ability that watches creatures dying.
fn graveyard_watcher_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx15a-graveyard-watcher".into()),
        name: "Graveyard Watcher".into(),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever another creature is put into your graveyard from the battlefield, \
                      draw a card."
            .into(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureDies {
                filter: None,
                controller: None,
                owner: None,
                exclude_self: true,
                nontoken_only: false,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: Some(TriggerZone::Graveyard),
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

/// A state with the watcher and a plain creature BOTH already in `p1`'s graveyard —
/// which is the state `check_triggers` sees, because it runs after every event in the
/// slice has been applied.
fn fixture() -> (GameState, ObjectId, ObjectId) {
    let def = graveyard_watcher_def();
    let card_id = def.card_id.clone();
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![def]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::creature(p(1), "Graveyard Watcher", 1, 1)
                .with_card_id(card_id)
                .in_zone(ZoneId::Graveyard(p(1))),
        )
        .object(ObjectSpec::creature(p(1), "Other Creature", 2, 2).in_zone(ZoneId::Graveyard(p(1))))
        .build()
        .unwrap();
    let find = |name: &str| {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(&id, _)| id)
            .unwrap_or_else(|| panic!("{name} not found"))
    };
    let watcher = find("Graveyard Watcher");
    let other = find("Other Creature");
    (state, watcher, other)
}

fn died(new_grave_id: ObjectId, controller: PlayerId) -> GameEvent {
    GameEvent::CreatureDied {
        object_id: ObjectId(9_000 + new_grave_id.0),
        new_grave_id,
        controller,
        pre_death_counters: Default::default(),
        pre_death_power: None,
        pre_death_characteristics: None,
    }
}

fn watcher_triggered(state: &GameState, events: &[GameEvent], timing: EventBatchTiming) -> bool {
    let triggers = check_triggers_with_timing(state, events, timing);
    let sources: HashSet<ObjectId> = triggers.iter().map(|t| t.source).collect();
    let watcher = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Graveyard Watcher")
        .map(|(&id, _)| id)
        .expect("watcher should be in the graveyard");
    sources.contains(&watcher)
}

#[test]
/// **t1 — `OOS-DX24-7`'s own example.** A resolution that SEQUENTIALLY (1) puts the
/// `trigger_zone: Graveyard` source into a graveyard and then (2) kills another creature
/// must fire the source's trigger on (2): by then the ability existed (CR 603.10a).
///
/// This is the case the row calls "over-suppression", and it is the case the row's own
/// prefix sketch does NOT fix — at event (2) the prefix is `{watcher}`, which suppresses.
/// The shipped set is the whole batch MINUS the strictly-earlier arrivals, i.e.
/// `{other}`, which permits.
fn t1_sequential_earlier_arrival_does_not_suppress_a_later_death() {
    let (state, watcher, other) = fixture();
    let events = vec![died(watcher, p(1)), died(other, p(1))];
    assert!(
        watcher_triggered(&state, &events, EventBatchTiming::Sequential),
        "CR 603.10a: the watcher arrived in the graveyard at the FIRST event, so by the \
         SECOND event its graveyard ability existed and must trigger. Suppressing here \
         is OOS-DX24-7's over-suppression -- and is also what the row's own 'pass the \
         prefix' sketch would do."
    );
}

#[test]
/// **t2 — the SBA caller must NOT get t1's answer, in either slice order.** Within one
/// CR 704.3 fixpoint pass the deaths are simultaneous, so CR 603.10a's "immediately
/// prior" means prior to all of them and the Gatherer ruling applies: the watcher's
/// ability did not exist, and does not trigger.
///
/// Both orderings are asserted because that is precisely what a prefix-based set would
/// get wrong — it would make a simultaneous batch's answer depend on an ordering the
/// batch does not have.
fn t2_simultaneous_batch_suppresses_regardless_of_slice_order() {
    let (state, watcher, other) = fixture();
    for (label, events) in [
        (
            "watcher first",
            vec![died(watcher, p(1)), died(other, p(1))],
        ),
        ("other first", vec![died(other, p(1)), died(watcher, p(1))]),
    ] {
        assert!(
            !watcher_triggered(&state, &events, EventBatchTiming::Simultaneous),
            "CR 603.10a / Gatherer (Nether Traitor): with the deaths SIMULTANEOUS \
             ({label}), the watcher was not in the graveyard immediately prior and must \
             NOT trigger"
        );
    }
}

#[test]
/// **t3 — the sequential set still suppresses when the source arrives LATER.**
/// `check_triggers` runs after every event has been applied, so the watcher is sitting in
/// the graveyard by the time the collector enumerates `state.objects` even when its own
/// death is later in the slice. Keeping later-and-current arrivals in the suppression set
/// is what stops it firing off a death that happened before it got there.
///
/// Without this row, "subtract the strictly-earlier arrivals" could be mistaken for
/// "subtract everything", which would fire in both orders.
fn t3_sequential_later_arrival_still_suppresses() {
    let (state, watcher, other) = fixture();
    let events = vec![died(other, p(1)), died(watcher, p(1))];
    assert!(
        !watcher_triggered(&state, &events, EventBatchTiming::Sequential),
        "CR 603.10a: the other creature died BEFORE the watcher reached the graveyard, \
         so the watcher's graveyard ability did not yet exist and must not trigger"
    );
}

#[test]
/// **t4 — non-vacuity.** The watcher fires on a death that has nothing to do with its own
/// arrival, under both timings. Without this, t2 and t3 would be satisfied by a watcher
/// that never triggers at all.
fn t4_watcher_fires_on_an_unrelated_death_under_both_timings() {
    let (state, _watcher, other) = fixture();
    let events = vec![died(other, p(1))];
    for timing in [EventBatchTiming::Simultaneous, EventBatchTiming::Sequential] {
        assert!(
            watcher_triggered(&state, &events, timing),
            "the watcher must trigger on a death it had no part in ({timing:?}) -- \
             otherwise t2/t3 are vacuous"
        );
    }
}

#[test]
/// **t5 — the `/review` HIGH (Issue 1), pinned wrong-way-round.**
///
/// `EventBatchTiming` is a **per-caller** knob, and `resolution.rs` shipped `Sequential`
/// for one implement cycle on the strength of PB-DX24's measurement that the caller is
/// "coarse". **That measurement is right and the granularity is still wrong**, because a
/// resolution's event slice is not uniformly sequential: `Effect::DestroyAll` snapshots
/// the whole battlefield and destroys it in a single loop (`effects/mod.rs:2144-2270`),
/// so a wrath emits N `CreatureDied` events that are **simultaneous** — inside a slice
/// the caller was declaring sequential.
///
/// The corpus reach is not theoretical: **21 defs carry `Effect::DestroyAll`**, and
/// `nether_traitor` — the card this whole guard was written for — is `Complete` and
/// deck-legal. Under `Sequential` it fired its `trigger_zone: Graveyard` ability off a
/// creature that died at the same instant it did, which CR 603.10a and the Gatherer
/// ruling quoted in `check_triggers_with_timing` both forbid.
///
/// This test asserts the SHAPE of a mass destruction — every death in one slice, no
/// ordering between them — gets the simultaneous answer. It is the `t2` case stated in
/// the vocabulary of the caller that got it wrong, and it fails if `resolution.rs` is
/// ever switched back to `Sequential` without first giving the event stream a way to
/// carry per-group boundaries.
///
/// Note what this test deliberately does NOT do: it does not claim `Sequential` is
/// useless. `t1` still holds — a genuinely sequential pair of sub-effects still
/// over-suppresses under `Simultaneous`, which is `OOS-DX24-7`'s live premise. The
/// variant exists, is correct, and has no production caller until the grouping problem
/// is solved. That is stated here rather than left as an unexplained dead variant.
fn t5_a_mass_destruction_inside_one_resolution_is_simultaneous() {
    let (state, watcher, other) = fixture();

    // The shape `Effect::DestroyAll` produces: every death in one slice, and the slice
    // carries no ordering between them because the effect snapshotted first.
    for (label, events) in [
        (
            "watcher first",
            vec![died(watcher, p(1)), died(other, p(1))],
        ),
        ("other first", vec![died(other, p(1)), died(watcher, p(1))]),
    ] {
        assert!(
            !watcher_triggered(&state, &events, EventBatchTiming::Simultaneous),
            "CR 603.10a: a wrath kills the watcher and the other creature AT THE SAME \
             TIME ({label}), so the watcher's graveyard ability did not exist \
             immediately prior and must not trigger. `resolution.rs` must pass \
             Simultaneous until the event stream can carry per-group boundaries."
        );
    }

    // And the wrong answer is genuinely reachable through the other variant, so this
    // test is not asserting something no implementation could get wrong.
    assert!(
        watcher_triggered(
            &state,
            &[died(watcher, p(1)), died(other, p(1))],
            EventBatchTiming::Sequential
        ),
        "non-vacuity: Sequential DOES give the other answer on this exact slice -- which \
         is why passing it at a caller whose slice contains a mass destruction was a \
         live defect and not a stylistic choice"
    );
}

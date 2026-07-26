//! PB-DP8 (DP-6 / OOS-M11-4) — triggered-ability targets become a player choice.
//!
//! CR 603.3d: "The remainder of the process for putting a triggered ability on the
//! stack is identical to the process for casting a spell listed in rules 601.2c-d."
//! CR 601.2c: "The player announces their choice of an appropriate object or player
//! for each target the spell requires."
//! CR 603.3b: the CR 603.3b batch is placed in APNAP order, one ability at a time.

use mtg_engine::rules::abilities::default_trigger_targets;
use mtg_engine::{process_command, Command, GameEvent, GameState, TriggerTargetOption};

/// Answer any outstanding CR 603.3d target choice with the engine's own default,
/// through `process_command`.
///
/// **Panics if nothing is pending** -- so it can never mask a missing block (the
/// `answer_pending_cleanup_discard` precedent from PB-DP7).
pub fn answer_pending_trigger_targets(state: GameState) -> (GameState, Vec<GameEvent>) {
    let entry = state
        .pending_trigger_targets()
        .expect("no CR 603.3d trigger-target choice is pending");
    let player = entry.player;
    let choice_id = entry.choice_id;
    let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
    let targets = default_trigger_targets(&slots);
    process_command(
        state,
        Command::ChooseTriggerTargets {
            player,
            choice_id,
            targets,
        },
    )
    .expect("the engine must accept its own default answer (SR-38)")
}

/// The in-place twin, for the tests that drive `flush_pending_triggers` directly
/// (a `&mut GameState` API) rather than through `process_command`.
///
/// Answers with the engine's own default, which is byte-identical to the
/// pre-PB-DP8 first-match auto-pick, so a test written before this batch keeps
/// pinning exactly what it was written to pin. Returns how many questions were
/// answered; `0` means the flush never suspended.
pub fn answer_pending_trigger_targets_in_place(state: &mut GameState) -> usize {
    let mut n = 0;
    while let Some(entry) = state.pending_trigger_targets() {
        let player = entry.player;
        let choice_id = entry.choice_id;
        let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
        let targets = default_trigger_targets(&slots);
        mtg_engine::rules::abilities::handle_choose_trigger_targets(
            state, player, choice_id, targets,
        )
        .expect("the engine must accept its own default answer (SR-38)");
        n += 1;
        assert!(n < 256, "trigger-target answers did not converge");
    }
    n
}

/// Answer every outstanding CR 603.3d target choice with the engine's default,
/// looping until the CR 603.3b batch completes. Returns the number answered.
pub fn answer_all_pending_trigger_targets(state: GameState) -> (GameState, usize) {
    let mut state = state;
    let mut n = 0;
    while state.pending_trigger_targets().is_some() {
        let (s, _) = answer_pending_trigger_targets(state);
        state = s;
        n += 1;
        assert!(n < 256, "trigger-target answers did not converge");
    }
    (state, n)
}
use mtg_card_defs::all_cards;
use mtg_card_types::cards::card_definition::{AbilityDefinition, Completeness};

/// CR 603.3d / SR-36 — the PB-DP8 roster, derived by enumerating `all_cards()`
/// rather than by grepping source.
///
/// A def is in the roster iff some `AbilityDefinition::Triggered` on **any** of its
/// faces (front, `back_face`, `adventure_face`) declares a non-empty `targets`, and
/// the def is `Completeness::Complete` (i.e. legal in a deck, per SR-2). Those are
/// exactly the defs whose trigger reaches
/// `rules::abilities::flush_pending_triggers`'s CR 603.3d announcement.
///
/// The assertion is `>=` on purpose: the authoring campaign adds cards continuously
/// and an `==` pin would redden on unrelated work.
#[test]
fn test_dp8_roster_enumeration() {
    fn has_targeted_trigger(abilities: &[AbilityDefinition]) -> bool {
        abilities.iter().any(
            |a| matches!(a, AbilityDefinition::Triggered { targets, .. } if !targets.is_empty()),
        )
    }

    let mut roster: Vec<String> = Vec::new();
    let mut incomplete = 0usize;
    for def in all_cards() {
        let mut hit = has_targeted_trigger(&def.abilities);
        if let Some(face) = def.back_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if let Some(face) = def.adventure_face.as_ref() {
            hit |= has_targeted_trigger(&face.abilities);
        }
        if !hit {
            continue;
        }
        if def.completeness == Completeness::Complete {
            roster.push(def.name.clone());
        } else {
            incomplete += 1;
        }
    }
    roster.sort();
    println!(
        "PB-DP8 roster: {} effectively-Complete defs with a targeted triggered ability \
         ({} more carry a non-Complete marker)",
        roster.len(),
        incomplete
    );
    for name in &roster {
        println!("  {name}");
    }
    assert!(
        roster.len() >= 60,
        "PB-DP8 roster collapsed to {} defs (expected >= 60)",
        roster.len()
    );
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

use mtg_engine::cards::card_definition::TargetRequirement;
use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
use mtg_engine::state::error::GameStateError;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::{
    CardEffectTarget, CardRegistry, Effect, EffectAmount, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, Step, Target, TriggerEvent, TriggeredAbilityDef,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

/// An enchantment whose triggered ability fires on any permanent entering and
/// deals 2 damage to `DeclaredTarget { index: 0 }`, with the requirement list
/// supplied by the caller. This is the minimal shape that reaches the CR 603.3d
/// announcement site (`PendingTriggerKind::Normal`, non-empty `targets`).
fn zapper(owner: PlayerId, name: &str, targets: Vec<TargetRequirement>) -> ObjectSpec {
    ObjectSpec::enchantment(owner, name).with_triggered_ability(TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "PB-DP8 fixture: deal 2 damage to the declared target".to_string(),
        effect: Some(Effect::DealDamage {
            source: None,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(2),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets,
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
    })
}

/// Synthesize a `PermanentEnteredBattlefield` event for an already-placed object
/// and drive it through `check_triggers` + `flush_pending_triggers` -- the pair
/// `check_and_flush_triggers` wraps in the real command handlers. Builder-placed
/// objects never go through `resolution.rs`'s ETB pipeline.
///
/// Deliberately does NOT answer any resulting CR 603.3d question: these tests are
/// about the question.
fn fire_etb(state: &mut GameState, entering: ObjectId, controller: PlayerId) -> Vec<GameEvent> {
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: entering,
        player: controller,
    }];
    let triggers = check_triggers(state, &events);
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    flush_pending_triggers(state)
}

/// Two-player board: a zapper controlled by `p1` and `n` creatures controlled by
/// `p1`, all on the battlefield, plus a `Bystander` whose ETB fires the trigger.
fn board_with_creatures(n: usize, players: usize) -> GameState {
    let mut b = GameStateBuilder::new();
    for i in 1..=players {
        b = b.add_player(p(i as u64));
    }
    let mut b = b.with_registry(CardRegistry::new(vec![])).object(zapper(
        p(1),
        "Zapper",
        vec![TargetRequirement::TargetCreature],
    ));
    for i in 0..n {
        b = b.object(ObjectSpec::creature(p(1), &format!("Creature {i}"), 2, 2));
    }
    b.object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap()
}

// ── T1 ────────────────────────────────────────────────────────────────────────

/// CR 603.3d — with two legal creature targets, the flush SUSPENDS: the trigger
/// is not on the stack, no priority is granted, and exactly one
/// `TriggerTargetChoiceRequired` is emitted carrying both candidates.
///
/// Fail-before (expressible on `main`): `state.stack_objects().is_empty()` after
/// the flush. Pre-PB-DP8 the trigger was already on the stack with an auto-picked
/// target, so that assertion failed.
#[test]
fn test_dp8_flush_blocks_on_a_real_target_choice() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let events = fire_etb(&mut state, bystander, p(1));

    assert!(
        state.pending_trigger_targets().is_some(),
        "CR 603.3d: two legal creature targets is a real announcement -- the flush must suspend"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.3: the triggered ability must NOT be on the stack until its controller announces"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 603.3b: priority is granted only after the whole batch is on the stack"
    );
    let asks: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::TriggerTargetChoiceRequired { player, slots, .. } => Some((*player, slots)),
            _ => None,
        })
        .collect();
    assert_eq!(asks.len(), 1, "exactly one question per moment (CR 603.3d)");
    assert_eq!(
        asks[0].0,
        p(1),
        "CR 603.3a: the trigger's controller answers"
    );
    assert_eq!(
        asks[0].1[0].candidates.len(),
        2,
        "both creatures are legal choices (CR 601.2c)"
    );
    assert!(
        !asks[0].1[0].optional,
        "`TargetCreature` is a required slot"
    );
}

// ── T2 ────────────────────────────────────────────────────────────────────────

/// CR 601.2c — the ANNOUNCED target is honoured, not the first match. Answering
/// with the higher-`ObjectId` creature puts that creature on the stack object.
///
/// Fail-before: pre-PB-DP8 `.find()` took the lowest `ObjectId`, so asserting the
/// higher one failed.
#[test]
fn test_dp8_chosen_target_is_honoured_not_first_match() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    let entry = state.pending_trigger_targets().unwrap();
    let choice_id = entry.choice_id;
    let mut ids: Vec<ObjectId> = entry.slots[0]
        .candidates
        .iter()
        .filter_map(|c| match c.target {
            Target::Object(id) => Some(id),
            _ => None,
        })
        .collect();
    ids.sort();
    let highest = *ids.last().unwrap();
    let lowest = ids[0];
    assert_ne!(highest, lowest, "the fixture must offer two distinct ids");
    assert_eq!(
        entry.slots[0].default.as_ref().unwrap().target,
        Target::Object(lowest),
        "the engine's default must still be the pre-PB-DP8 first match (lowest ObjectId)"
    );

    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(highest)]],
        },
    )
    .unwrap();

    assert_eq!(state.stack_objects().len(), 1);
    assert_eq!(
        state.stack_objects()[0].targets[0].target,
        Target::Object(highest),
        "CR 601.2c: the announced target, not the engine's first match"
    );
}

// ── T3 ────────────────────────────────────────────────────────────────────────

/// CR 603.3d / CR 601.2c — a target outside the offered candidate set is
/// rejected, and the caller's state is byte-identical afterwards (ESM criterion
/// 5545). `process_command` takes `GameState` by value, so this holds only
/// because the handler validates before mutating.
#[test]
fn test_dp8_illegal_target_rejected_state_untouched() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let choice_id = state.pending_trigger_targets().unwrap().choice_id;
    let zapper_id = find_object(&state, "Zapper");
    let hash_before = state.public_state_hash();

    // The Zapper is an enchantment: not a legal `TargetCreature`.
    let err = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(zapper_id)]],
        },
    )
    .unwrap_err();
    assert!(
        matches!(&err, GameStateError::InvalidCommand(m) if m.contains("603.3d")),
        "expected a CR 603.3d legality rejection, got {err:?}"
    );

    // A player is not a legal `TargetCreature` either.
    let err2 = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Player(p(2))]],
        },
    )
    .unwrap_err();
    assert!(matches!(err2, GameStateError::InvalidCommand(_)));

    assert_eq!(
        hash_before,
        state.public_state_hash(),
        "a rejected answer must leave the state byte-identical"
    );
}

// ── T4 ────────────────────────────────────────────────────────────────────────

/// CR 603.3d — "if a choice is required ... but no legal choices can be made for
/// it ... the ability is simply removed from the stack." A required slot with
/// zero candidates asks NOTHING and places nothing. Regression guard: this is the
/// compliant fallback PB-DP8 must not break.
#[test]
fn test_dp8_no_legal_candidate_still_removes_the_trigger() {
    let mut state = board_with_creatures(0, 2);
    let bystander = find_object(&state, "Bystander");
    let events = fire_etb(&mut state, bystander, p(1));

    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 603.3d: no legal choice means no question"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.3d: the ability is removed from the stack"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "no AbilityTriggered for a removed ability"
    );
}

// ── T5 ────────────────────────────────────────────────────────────────────────

/// CR 601.2c — an announcement with exactly one legal answer is determined, so
/// the engine places the trigger directly with no round trip. Without this
/// narrowing every single-target board would cost a wire round trip on a question
/// with one answer.
#[test]
fn test_dp8_forced_single_candidate_asks_nothing() {
    let mut state = board_with_creatures(1, 2);
    let bystander = find_object(&state, "Bystander");
    let events = fire_etb(&mut state, bystander, p(1));
    let only = find_object(&state, "Creature 0");

    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 601.2c: one legal answer is not a choice -- no question may be asked"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::TriggerTargetChoiceRequired { .. })),
        "no TriggerTargetChoiceRequired event for a forced choice"
    );
    assert_eq!(state.stack_objects().len(), 1);
    assert_eq!(
        state.stack_objects()[0].targets[0].target,
        Target::Object(only)
    );
}

// ── T6 ────────────────────────────────────────────────────────────────────────

/// CR 603.3b — with two controllers each owing an announcement, the questions are
/// asked as a SEQUENCE in APNAP order: the active player first, then the
/// non-active player, each with its own `choice_id`. Both triggers end up on the
/// stack, with the active player's below (so it resolves last, CR 101.4).
#[test]
fn test_dp8_apnap_sequence_across_two_controllers() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper A",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(zapper(
            p(2),
            "Zapper B",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    // Question 1: the ACTIVE player (CR 101.4 / 603.3b).
    let e1 = state.pending_trigger_targets().unwrap();
    assert_eq!(
        e1.player,
        p(1),
        "CR 603.3b: APNAP -- the active player first"
    );
    let (id1, cid1) = (e1.source, e1.choice_id);
    assert_eq!(id1, find_object(&state, "Zapper A"));
    assert_eq!(
        e1.remaining.len(),
        1,
        "the un-flushed tail of the batch travels inside the entry"
    );
    let (state, _) = answer_pending_trigger_targets(state);

    // Question 2: the non-active player, with a DIFFERENT choice_id.
    let e2 = state
        .pending_trigger_targets()
        .expect("CR 603.3b: the second controller is asked next");
    assert_eq!(e2.player, p(2));
    let cid2 = e2.choice_id;
    assert_ne!(cid1, cid2, "each question gets its own moment guard");
    let (state, _) = answer_pending_trigger_targets(state);

    assert!(state.pending_trigger_targets().is_none());
    assert_eq!(state.stack_objects().len(), 2);
    // CR 101.4 / 603.3b: the active player's ability goes on the stack first, so
    // it is at the BOTTOM and resolves last.
    let bottom = &state.stack_objects()[0];
    assert!(matches!(
        bottom.kind,
        StackObjectKind::TriggeredAbility { source_object, .. } if source_object == id1
    ));
}

// ── T7 ────────────────────────────────────────────────────────────────────────

/// CR 603.3b — a partial flush must not lose the tail. Three triggers, only the
/// second of which needs a choice: at the pause, trigger 1 is on the stack,
/// triggers 2 and 3 are nowhere in `pending_triggers` (the entry owns them), and
/// after the answer all three are on the stack, EACH EXACTLY ONCE.
///
/// This is the assertion that catches every version of the drain/replay bug:
/// `flush_pending_triggers` drains `pending_triggers` up front, so a naive pause
/// would destroy the tail and a naive resume would re-place trigger 1.
#[test]
fn test_dp8_resume_after_partial_flush_places_each_trigger_exactly_once() {
    // Zapper A: one legal target only (forced -- goes straight on the stack).
    // Zapper B: two legal targets (asks).
    // Zapper C: two legal targets (asks, after B).
    // All controlled by p1 so the batch is one controller's, in queue order.
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper A",
            vec![TargetRequirement::TargetEnchantment],
        ))
        .object(zapper(
            p(1),
            "Zapper B",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(zapper(
            p(1),
            "Zapper C",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    // Zapper A's `TargetEnchantment` slot has FOUR candidates (3 zappers +
    // Bystander), so it is not forced either. Answer questions until the batch
    // completes, counting them.
    let mut answered = 0;
    let mut state = state;
    while state.pending_trigger_targets().is_some() {
        assert!(
            state.pending_triggers().is_empty(),
            "the un-flushed tail lives in the entry, never in `pending_triggers`"
        );
        let (s, _) = answer_pending_trigger_targets(state);
        state = s;
        answered += 1;
        assert!(answered <= 3);
    }
    assert_eq!(answered, 3, "three triggers, three announcements");

    assert!(state.pending_trigger_targets().is_none());
    assert_eq!(
        state.stack_objects().len(),
        3,
        "CR 603.3b: every trigger of the batch reaches the stack"
    );
    let mut sources: Vec<ObjectId> = state
        .stack_objects()
        .iter()
        .filter_map(|o| match o.kind {
            StackObjectKind::TriggeredAbility { source_object, .. } => Some(source_object),
            _ => None,
        })
        .collect();
    let before = sources.len();
    sources.sort();
    sources.dedup();
    assert_eq!(
        sources.len(),
        before,
        "each trigger must appear on the stack EXACTLY ONCE (drain/replay guard)"
    );
}

// ── T8 ────────────────────────────────────────────────────────────────────────

/// CR 603.3d — the `choice_id` moment guard. An answer quoting the wrong
/// `choice_id` (or arriving when nothing is pending) is rejected and the state is
/// unchanged. This is what makes an answer to question k inapplicable to question
/// k+1 of the same CR 603.3b batch.
#[test]
fn test_dp8_stale_choice_id_rejected() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let entry = state.pending_trigger_targets().unwrap();
    let choice_id = entry.choice_id;
    let good = entry.slots[0].candidates[0].target.clone();
    let hash_before = state.public_state_hash();

    let err = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id: choice_id + 1,
            targets: vec![vec![good.clone()]],
        },
    )
    .unwrap_err();
    assert!(
        matches!(&err, GameStateError::InvalidCommand(m) if m.contains("stale")),
        "expected a stale-choice_id rejection, got {err:?}"
    );
    assert_eq!(hash_before, state.public_state_hash());

    // Answering when nothing is pending is also rejected.
    let clean = board_with_creatures(2, 2);
    let err2 = process_command(
        clean,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![good]],
        },
    )
    .unwrap_err();
    assert!(matches!(&err2, GameStateError::InvalidCommand(m) if m.contains("no trigger-target")));
}

// ── T9 ────────────────────────────────────────────────────────────────────────

/// CR 603.3a — only the trigger's controller may answer. Two distinct errors, one
/// per path, asserted specifically (PB-DP7 review Finding 12: `is_err()` alone
/// would pass even if the admission gate were the only thing rejecting it).
#[test]
fn test_dp8_sender_validation() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let entry = state.pending_trigger_targets().unwrap();
    let choice_id = entry.choice_id;
    let good = entry.slots[0].candidates[0].target.clone();

    // (a) Through `process_command`: the ADMISSION gate rejects a foreign sender.
    let err = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(2),
            choice_id,
            targets: vec![vec![good.clone()]],
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, GameStateError::BlockedByPendingDecision { .. }),
        "the admission gate must reject a foreign sender, got {err:?}"
    );

    // (b) Direct handler call: the SR-29 in-handler check is the backstop.
    let mut direct = state.clone();
    let err2 = mtg_engine::rules::abilities::handle_choose_trigger_targets(
        &mut direct,
        p(2),
        choice_id,
        vec![vec![good]],
    )
    .unwrap_err();
    assert!(
        matches!(&err2, GameStateError::InvalidCommand(m) if m.contains("603.3a")),
        "the handler's own CR 603.3a check must reject a foreign sender, got {err2:?}"
    );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// CR 800.4d / CR 603.3b / CR 800.4j — the controller concedes while their
/// announcement is outstanding. Their trigger is NOT put on the stack ("If a
/// triggered ability that would be controlled by a player who has left the game
/// would be put onto the stack, it isn't put on the stack"), the rest of the
/// batch IS placed (CR 800.4j: the turn continues), and the game does not hang.
#[test]
fn test_dp8_controller_concedes_mid_choice() {
    // p2 is the ACTIVE player so its trigger is asked first (CR 603.3b APNAP),
    // leaving p1's trigger in the tail. Three players, so p2 conceding does not
    // end the game (CR 104.2a) and the turn genuinely continues (CR 800.4j).
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(2),
            "Zapper P2",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(zapper(
            p(1),
            "Zapper P1",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(2))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let p2_zapper = find_object(&state, "Zapper P2");
    let p1_zapper = find_object(&state, "Zapper P1");
    assert_eq!(state.pending_trigger_targets().unwrap().player, p(2));

    let (state, _) = process_command(state, Command::Concede { player: p(2) }).unwrap();

    assert!(
        state.pending_trigger_targets().is_none()
            || state.pending_trigger_targets().unwrap().player != p(2),
        "the conceding player's entry must be cleared"
    );
    // CR 603.3b / 800.4j: the batch RESUMED on p1's trigger, which has its own
    // real choice. That is correct -- the block moves to p1, it does not vanish.
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(1)),
        "CR 603.3b: the batch continues, so the next controller is now the one asked"
    );
    let (state, _) = answer_pending_trigger_targets(state);
    let sources: Vec<ObjectId> = state
        .stack_objects()
        .iter()
        .filter_map(|o| match o.kind {
            StackObjectKind::TriggeredAbility { source_object, .. } => Some(source_object),
            _ => None,
        })
        .collect();
    assert!(
        !sources.contains(&p2_zapper),
        "CR 800.4d: a departed player's triggered ability is not put on the stack"
    );
    assert!(
        sources.contains(&p1_zapper),
        "CR 800.4j / 603.3b: the rest of the batch is still placed"
    );
}

// ── T10b ──────────────────────────────────────────────────────────────────────

/// CR 800.4d neighbourhood — a controller who has ALREADY left the game is never
/// asked. The engine uses its own default instead (today's behaviour unchanged);
/// asking would hang the game, since nobody could answer.
#[test]
fn test_dp8_dead_controller_is_never_asked() {
    let mut state = board_with_creatures(2, 2);
    state.players_mut().get_mut(&p(1)).unwrap().has_lost = true;
    let bystander = find_object(&state, "Bystander");
    let events = fire_etb(&mut state, bystander, p(1));

    assert!(
        state.pending_trigger_targets().is_none(),
        "a player who has left the game must never be asked to announce"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::TriggerTargetChoiceRequired { .. })));
    assert_eq!(
        state.stack_objects().len(),
        1,
        "OOS-DP8-5: the trigger is still placed with the engine's default -- CR 800.4d's \
         drop is a behaviour flip this batch does not make"
    );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// CR 603.3 — while the batch is suspended, `process_command` rejects everything
/// except the answer and `Concede`. Nobody has priority mid-flush, so
/// `PassPriority` and `TapForMana` (CR 605.3a needs priority) are both illegal,
/// from ANY seat.
#[test]
fn test_dp8_admission_gate_while_suspended() {
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let hash_before = state.public_state_hash();

    for cmd in [
        Command::PassPriority { player: p(1) },
        Command::PassPriority { player: p(2) },
        Command::PlayLand {
            player: p(1),
            card: find_object(&state, "Bystander"),
        },
    ] {
        let err = process_command(state.clone(), cmd.clone()).unwrap_err();
        assert!(
            matches!(err, GameStateError::BlockedByPendingDecision { .. }),
            "{cmd:?} must be rejected while a CR 603.3d announcement is outstanding, got {err:?}"
        );
        assert_eq!(hash_before, state.public_state_hash());
    }
}

// ── T12 ───────────────────────────────────────────────────────────────────────

/// CR 603.3 / CR 117.3a — a flush that suspends inside a priority-granting call
/// site must not grant priority, and MUST grant it once the batch completes.
/// This is the guard set's own test: without the resume-side grant, answering
/// would leave a game in which nobody has priority (a hang).
#[test]
fn test_dp8_no_priority_granted_while_suspended_then_granted_on_resume() {
    // Fire the trigger through the real resolution/priority path: pass priority
    // in a step whose entry flushes triggers.
    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let triggers = check_triggers(
        &state,
        &[GameEvent::PermanentEnteredBattlefield {
            object_id: bystander,
            player: p(1),
        }],
    );
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    state.turn_mut().priority_holder = Some(p(1));

    // `enter_step`'s has-priority branch flushes and would grant priority.
    let (state, events) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    // p1 passing does not itself advance the step here, so drive to the flush:
    let (state, events2) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let all: Vec<GameEvent> = events.into_iter().chain(events2).collect();

    assert!(
        state.pending_trigger_targets().is_some(),
        "the step-entry flush must suspend on the real choice"
    );
    let priority_after_ask = all
        .iter()
        .skip_while(|e| !matches!(e, GameEvent::TriggerTargetChoiceRequired { .. }))
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert_eq!(
        priority_after_ask, 0,
        "CR 603.3b: no priority may be granted after the batch suspends"
    );

    let (state, resume_events) = answer_pending_trigger_targets(state);
    assert!(state.pending_trigger_targets().is_none());
    assert!(
        resume_events
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 117.3a: the suspended call site owed a priority grant; the resume must discharge it"
    );
    assert!(
        state.turn().priority_holder.is_some(),
        "a completed batch must leave someone holding priority, or the game hangs"
    );
}

// ── T13 ───────────────────────────────────────────────────────────────────────

/// CR 603.3d — the determinism pin: `default_trigger_targets` reproduces the
/// pre-PB-DP8 first-match auto-pick for every requirement family, so no bot
/// behaviour changes.
#[test]
fn test_dp8_default_reproduces_pre_pb_behaviour() {
    // Player family: the first live OPPONENT in turn order, not `candidates[0]`
    // (which is the controller, seat 1).
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(p(1), "Zapper", vec![TargetRequirement::TargetAny]))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let slot = &state.pending_trigger_targets().unwrap().slots[0];
    assert_eq!(
        slot.candidates[0].target,
        Target::Player(p(1)),
        "candidates are in seat order, controller first"
    );
    assert_eq!(
        slot.default.as_ref().unwrap().target,
        Target::Player(p(2)),
        "CR 603.3d: the pre-PB-DP8 pick preferred the first live OPPONENT, not candidates[0]"
    );

    // Opponent family: first live opponent, never the controller (PB-EF6).
    let mut state2 = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetOpponent],
        ))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let b2 = find_object(&state2, "Bystander");
    let _ = fire_etb(&mut state2, b2, p(1));
    let slot2 = &state2.pending_trigger_targets().unwrap().slots[0];
    assert_eq!(slot2.candidates.len(), 2, "two live opponents");
    assert!(
        !slot2
            .candidates
            .iter()
            .any(|c| c.target == Target::Player(p(1))),
        "CR 102.3/601.2c: a `TargetOpponent` slot never offers the controller"
    );
    assert_eq!(slot2.default.as_ref().unwrap().target, Target::Player(p(2)));
}

// ── T14 ───────────────────────────────────────────────────────────────────────

/// CR 601.2c — the candidate set is genuinely WIDER than the default. A
/// `TargetAny` slot offers players AND creatures/planeswalkers; before PB-DP8 the
/// player arm returned first and the object arm's matching branches were dead
/// code.
#[test]
fn test_dp8_candidate_set_is_wider_than_the_default() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(p(1), "Zapper", vec![TargetRequirement::TargetAny]))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(2), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let slot = &state.pending_trigger_targets().unwrap().slots[0];

    let players = slot
        .candidates
        .iter()
        .filter(|c| matches!(c.target, Target::Player(_)))
        .count();
    let objects = slot
        .candidates
        .iter()
        .filter(|c| matches!(c.target, Target::Object(_)))
        .count();
    assert_eq!(
        players, 2,
        "CR 601.2c: both live players are legal for `any target`"
    );
    assert_eq!(
        objects, 2,
        "CR 601.2c: both creatures are legal for `any target` -- the half that was \
         unreachable before PB-DP8"
    );

    // And a creature IS accepted as the answer.
    let creature = find_object(&state, "Creature 1");
    let choice_id = state.pending_trigger_targets().unwrap().choice_id;
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(creature)]],
        },
    )
    .unwrap();
    assert_eq!(
        state.stack_objects()[0].targets[0].target,
        Target::Object(creature)
    );
}

// ── T14b ──────────────────────────────────────────────────────────────────────

/// CR 601.2c "up to" — an `UpToN` slot is `optional`, its `default` is `None`, an
/// EMPTY answer is accepted, and the trigger still goes on the stack.
///
/// **This is a corrected plan premise and a genuine behaviour flip.**
/// `pb-plan-DP8.md` §5.2 states the pre-PB-DP8 code "contributed 0 targets" for a
/// permanent-inner `UpToN`. It did not: the arm returned `None`, and the caller
/// treated `None` as "no legal target" and removed the WHOLE TRIGGER from the
/// stack. Sword of Sinew and Steel and Elder Deep-Fiend (both `Complete`) never
/// once put their trigger on the stack. CR 601.2c makes zero targets a legal
/// announcement, so CR 603.3d's removal clause does not apply.
#[test]
fn test_dp8_up_to_n_slot_is_optional_and_zero_targets_is_legal() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::UpToN {
                count: 1,
                inner: Box::new(TargetRequirement::TargetCreature),
            }],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    let entry = state
        .pending_trigger_targets()
        .expect("an optional slot is always a real choice -- 'none' is a second answer");
    let choice_id = entry.choice_id;
    assert!(entry.slots[0].optional, "`UpToN` is CR 601.2c's 'up to'");
    assert!(
        entry.slots[0].default.is_none(),
        "the pre-PB-DP8 pick contributed no target for a permanent-inner `UpToN`"
    );
    assert_eq!(
        entry.slots[0].candidates.len(),
        1,
        "the creature is offered -- a human may pick it"
    );

    // Empty answer accepted; the trigger is placed with zero targets.
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![]],
        },
    )
    .unwrap();
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 601.2c/603.3d: zero targets is a legal announcement, so the ability is NOT \
         removed from the stack (pre-PB-DP8 it always was)"
    );
    assert!(state.stack_objects()[0].targets.is_empty());
}

// ── T15 ───────────────────────────────────────────────────────────────────────

/// CR 603.3d / CR 102.3 — a `TargetOpponent` slot with the only opponent gone has
/// no legal candidate, so the trigger is removed and nothing is asked (the PB-EF6
/// regression guard); in a 4-player game it offers all three opponents and none
/// of them is the controller.
#[test]
fn test_dp8_target_opponent_never_self_and_never_asks_when_alone() {
    // 1v1 with the opponent dead: no candidate at all.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetOpponent],
        ))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p(2)).unwrap().has_conceded = true;
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    assert!(state.pending_trigger_targets().is_none());
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.3d: no legal opponent means the ability is removed -- and it must NEVER \
         fall back to the controller (PB-EF6, CR 102.3)"
    );

    // 4-player: three opponents, none of them the controller.
    let mut state4 = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetOpponent],
        ))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let b4 = find_object(&state4, "Bystander");
    let _ = fire_etb(&mut state4, b4, p(1));
    let slot = &state4.pending_trigger_targets().unwrap().slots[0];
    assert_eq!(slot.candidates.len(), 3);
    assert!(!slot
        .candidates
        .iter()
        .any(|c| c.target == Target::Player(p(1))));
}

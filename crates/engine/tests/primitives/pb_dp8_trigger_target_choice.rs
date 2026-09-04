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
///
/// **PB-DX35 (2026-09, `OOS-DX4-2`): floor lowered 60 -> 59.** `retreat_to_kazandu`
/// (already `Complete`) had its mode-0 target re-shaped OFF the flat `targets` list
/// and into `ModeSelection.mode_targets`, scoped to mode 0 alone -- so this row's
/// predicate (a non-empty FLAT `targets` list) correctly stops counting it, the same
/// shape `decision_gate.rs::canonical_walk_reproduces_pb_dp8_roster` moved for.
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
        roster.len() >= 59,
        "PB-DP8 roster collapsed to {} defs (expected >= 59)",
        roster.len()
    );
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

use mtg_engine::cards::card_definition::TargetRequirement;
use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
use mtg_engine::state::error::GameStateError;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::{
    CardEffectTarget, CardRegistry, Effect, EffectAmount, GameStateBuilder, KeywordAbility,
    ObjectId, ObjectSpec, PlayerId, Step, Target, TriggerEvent, TriggeredAbilityDef, ZoneId,
};

/// The `source_object` of every `TriggeredAbility` currently on the stack, in
/// stack order.
fn trigger_sources(state: &GameState) -> Vec<ObjectId> {
    state
        .stack_objects()
        .iter()
        .filter_map(|o| match o.kind {
            StackObjectKind::TriggeredAbility { source_object, .. } => Some(source_object),
            _ => None,
        })
        .collect()
}

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

/// CR 603.3d / CR 601.2c — a rejected answer must leave the state byte-identical
/// (ESM criterion 5545).
///
/// **Closing-review Finding 7 (LOW): this test used to be vacuous.** It captured
/// `hash_before`, called `process_command(state.clone(), ..)`, and then compared
/// `hash_before` against `state` -- the ORIGINAL, which had been cloned into the
/// call and so could not have changed whatever the handler did. The property does
/// hold at the `process_command` boundary for a structural reason (`GameState` is
/// taken by value and every `?` discards the local copy), but that is not where it
/// is at risk: `handle_choose_trigger_targets` takes `&mut GameState`, and it holds
/// there only because **every** check runs before **any** mutation. So the test now
/// drives the handler itself, once per rejection class, against one `&mut state`.
#[test]
fn test_dp8_illegal_target_rejected_state_untouched() {
    use mtg_engine::rules::abilities::handle_choose_trigger_targets as answer;

    let mut state = board_with_creatures(2, 2);
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let choice_id = state.pending_trigger_targets().unwrap().choice_id;
    let zapper_id = find_object(&state, "Zapper");
    let creature0 = find_object(&state, "Creature 0");
    let hash_before = state.public_state_hash();

    // One case per validation branch that can reject, each against `&mut state`.
    let cases: Vec<(&str, PlayerId, u64, Vec<Vec<Target>>)> = vec![
        // (7) legality: the Zapper is an enchantment, not a legal `TargetCreature`.
        (
            "603.3d",
            p(1),
            choice_id,
            vec![vec![Target::Object(zapper_id)]],
        ),
        // (7) legality: a player is not a legal `TargetCreature` either.
        ("603.3d", p(1), choice_id, vec![vec![Target::Player(p(2))]]),
        // (3) CR 603.3a: the wrong player answers.
        ("", p(2), choice_id, vec![vec![Target::Object(creature0)]]),
        // (4) the moment guard: a stale `choice_id`.
        (
            "",
            p(1),
            choice_id.wrapping_add(7),
            vec![vec![Target::Object(creature0)]],
        ),
        // (5) slot count.
        ("", p(1), choice_id, vec![]),
        // (6) cardinality: a required slot answered with two targets.
        (
            "601.2c",
            p(1),
            choice_id,
            vec![vec![
                Target::Object(creature0),
                Target::Object(find_object(&state, "Creature 1")),
            ]],
        ),
    ];
    for (needle, sender, cid, targets) in cases {
        let err = answer(&mut state, sender, cid, targets.clone())
            .expect_err("this answer must be rejected");
        if !needle.is_empty() {
            assert!(
                format!("{err:?}").contains(needle),
                "expected a CR {needle} rejection, got {err:?}"
            );
        }
        assert_eq!(
            hash_before,
            state.public_state_hash(),
            "a rejected answer must leave the state byte-identical -- \
             `handle_choose_trigger_targets` mutated before it validated \
             (sender {sender:?}, choice {cid}, targets {targets:?})"
        );
    }

    // The block is still outstanding and still answerable from that same state.
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.choice_id),
        Some(choice_id)
    );
    answer(
        &mut state,
        p(1),
        choice_id,
        vec![vec![Target::Object(creature0)]],
    )
    .expect("the legal answer still completes the batch");
    assert_ne!(
        hash_before,
        state.public_state_hash(),
        "and an ACCEPTED answer does change the state -- otherwise the pin above \
         would pass for the wrong reason"
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

// ── T20 (fix cycle, Finding 1) ────────────────────────────────────────────────

/// CR 603.3d / CR 601.2c — an announced answer belongs to exactly ONE trigger.
///
/// Fix-cycle Finding 1 (HIGH). `head_targets` used to be consumed lazily, inside
/// the `else if let Some(pre) = head_targets.take()` arm of the target chain --
/// which sits behind the CR 603.3d "a required slot has no legal candidate"
/// removal. If the head trigger was removed at resume time its answer survived to
/// the NEXT trigger of the CR 603.3b batch: a different ability with different
/// `TargetRequirement`s, whose stack object then carried a target that was never
/// validated against its own requirements.
///
/// Here Zapper A ("target creature") is the head and Zapper B ("target player")
/// is the tail. Both of A's candidates leave the battlefield between the offer and
/// the answer, so A is correctly removed (CR 603.3d) -- and B must then make its
/// OWN announcement rather than inherit A's creature into a player slot.
///
/// **Fail-before**: pre-fix, B was placed immediately with `Target::Object(creature)`
/// and no second question was asked, so `pending_trigger_targets()` was `None`.
#[test]
fn test_dp8_answer_is_bound_to_its_own_trigger_not_the_next_one() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper A",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(zapper(
            p(1),
            "Zapper B",
            vec![TargetRequirement::TargetPlayer],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    let zapper_a = find_object(&state, "Zapper A");
    let zapper_b = find_object(&state, "Zapper B");
    let creature0 = find_object(&state, "Creature 0");
    let entry = state
        .pending_trigger_targets()
        .expect("Zapper A's required slot has two candidates -- a real choice");
    assert_eq!(
        entry.source, zapper_a,
        "CR 603.3b: the head of the batch is asked first"
    );
    let choice_id = entry.choice_id;

    // Both of the head's candidates leave the battlefield before the answer
    // arrives. (`Command::Concede` from a third player is the production route to
    // the same shape; this drives it directly so the probe stays deterministic and
    // independent of the concede gate added by fix-cycle Finding 5.)
    for name in ["Creature 0", "Creature 1"] {
        let id = find_object(&state, name);
        state.objects_mut().get_mut(&id).unwrap().zone = ZoneId::Graveyard(p(1));
    }

    // The answer is still ACCEPTED: CR 603.3d legality is membership in the frozen
    // candidate set the engine itself offered (see the plan's §5.5 item 7).
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(creature0)]],
        },
    )
    .unwrap();

    let placed = trigger_sources(&state);
    assert!(
        !placed.contains(&zapper_a),
        "CR 603.3d: the head's required slot has no legal candidate at resume time, \
         so its ability is removed from the stack"
    );
    assert!(
        !placed.contains(&zapper_b),
        "the answer belonged to Zapper A; Zapper B must not be placed with it"
    );
    let entry2 = state
        .pending_trigger_targets()
        .expect("CR 603.3d: Zapper B owes its OWN announcement -- two live players is a choice");
    assert_eq!(entry2.source, zapper_b);

    let (state, _) = answer_pending_trigger_targets(state);
    let stack_obj = state
        .stack_objects()
        .iter()
        .find(|o| matches!(o.kind, StackObjectKind::TriggeredAbility { source_object, .. } if source_object == zapper_b))
        .expect("Zapper B is on the stack once it has announced");
    assert!(
        matches!(stack_obj.targets[0].target, Target::Player(_)),
        "CR 601.2c: a `TargetPlayer` slot must hold a player, not the previous \
         trigger's creature -- got {:?}",
        stack_obj.targets[0].target
    );
}

// ── T21 (fix cycle, Finding 5 / OOS-DP8-9) ────────────────────────────────────

/// CR 104.3a / CR 603.3b / CR 800.4j — a **foreign** concede must not step over
/// another player's outstanding announcement.
///
/// Fix-cycle Finding 5 (MEDIUM), closing seed OOS-DP8-9. `drop_departed_trigger_flush`
/// only handles the case where the entry's OWN player concedes. When somebody else
/// concedes, `handle_concede` used to run its full priority-advance and turn-advance
/// logic with the CR 603.3b batch still suspended: it could reach `handle_all_passed`
/// -> `resolve_top_of_stack` -> `flush_pending_triggers`, which fires the
/// "re-entered while a CR 603.3d target choice is outstanding" `debug_assert!`, and
/// it could advance a whole turn under the suspended batch.
///
/// **Fail-before**: pre-fix this test panicked inside `process_command` on that
/// `debug_assert!` (`flush_pending_triggers re-entered ...`).
#[test]
fn test_dp8_foreign_concede_does_not_step_over_the_suspended_batch() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    // p2 holds priority, so the concede takes the priority-advance path.
    state.turn_mut().priority_holder = Some(p(2));
    state.turn_mut().players_passed = state.turn().players_passed.update(p(1));
    state.turn_mut().players_passed = state.turn().players_passed.update(p(3));

    let entry = state.pending_trigger_targets().unwrap();
    assert_eq!(entry.player, p(1));
    let choice_id = entry.choice_id;
    let turn_before = state.turn().turn_number;
    let step_before = state.turn().step;

    let (state, events) = process_command(state, Command::Concede { player: p(2) }).unwrap();

    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(1)),
        "a foreign concede must leave the outstanding announcement exactly where it was"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 603.3b: no priority may be granted while the batch is suspended"
    );
    assert_eq!(
        state.turn().turn_number,
        turn_before,
        "CR 603.3b: a turn must not advance under a suspended batch"
    );
    assert_eq!(state.turn().step, step_before);

    // And the block still clears: its player is alive by construction.
    let creature0 = find_object(&state, "Creature 0");
    let (state, resume) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(creature0)]],
        },
    )
    .unwrap();
    assert!(state.pending_trigger_targets().is_none());
    assert_eq!(trigger_sources(&state).len(), 1);
    let _ = resume;

    // CLOSING-REVIEW Finding 1 (HIGH): the gate above skipped `handle_concede`'s
    // priority advance, so the resume is the ONLY remaining chance to get priority
    // off the departed player. This assertion was the one the fix cycle omitted --
    // the test constructed exactly the deadlocking state and never looked at it.
    let holder = state
        .turn()
        .priority_holder
        .expect("CR 603.3b: the resumed batch must leave someone holding priority");
    assert_ne!(
        holder,
        p(2),
        "CR 800.4: priority must not stay pinned on a player who has left the game"
    );
    assert!(
        state
            .players()
            .get(&holder)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false),
        "the priority holder after the resume must be a live player, got {holder:?}"
    );
}

// ── T21b (closing review, Finding 1 — HIGH) ───────────────────────────────────

/// CR 800.4 / CR 603.3b / CR 117.3c — a concede under a suspended batch must not
/// strand priority on the conceded player.
///
/// Closing-review Finding 1 (HIGH), a **regression introduced by the fix cycle's
/// own Finding-5 gate**. `handle_concede` skips its priority-advance block while a
/// foreign `blocking_decision()` is outstanding; the gate's source comment claimed
/// that could not hang because "the resume grants priority itself". That is false
/// for `FlushResumeSite::None`, the resume site of all 30 in-match
/// `check_and_flush_triggers` calls -- `finish_resumed_flush` returns without
/// touching `priority_holder`. The game then has priority pinned on a conceded
/// player: `PassPriority` from them is `PlayerEliminated`, from anyone else
/// `NotPriorityHolder`, and nothing else reassigns the field. Unrecoverable.
///
/// This drives the reviewer's four-step scenario end to end and then proves the
/// game can still be played.
///
/// **Fail-before**: `PassPriority` from every live player was rejected --
/// `NotPriorityHolder { expected: Some(PlayerId(2)), .. }` -- with
/// `priority_holder == Some(p2)`, the conceded seat.
#[test]
fn test_dp8_concede_under_a_suspended_batch_does_not_strand_priority() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    // (1) PB-DP1: the ACTOR holds priority across a `check_and_flush_triggers`
    // suspension, and the suspension is on somebody else's trigger.
    state.turn_mut().priority_holder = Some(p(2));
    let entry = state.pending_trigger_targets().unwrap();
    assert_eq!(entry.player, p(1));
    let choice_id = entry.choice_id;

    // (2) the priority holder concedes; 3 players, so the game is not over.
    let (state, _) = process_command(state, Command::Concede { player: p(2) }).unwrap();
    assert!(
        state.pending_trigger_targets().is_some(),
        "the foreign concede must leave the batch suspended (fix-cycle Finding 5)"
    );

    // (3) the entry's player answers and the batch completes.
    let creature0 = find_object(&state, "Creature 0");
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(creature0)]],
        },
    )
    .unwrap();
    assert!(state.pending_trigger_targets().is_none());
    assert_eq!(trigger_sources(&state).len(), 1, "the batch was placed");

    // (4) the game must still be playable. Pre-fix every one of these failed.
    let holder = state
        .turn()
        .priority_holder
        .expect("CR 603.3b: somebody must hold priority once the batch is complete");
    assert_ne!(holder, p(2), "CR 800.4: not the conceded player");
    let (_, _) = process_command(state.clone(), Command::PassPriority { player: holder })
        .expect("the holder must be able to act -- otherwise the game is deadlocked");

    // And the two rejections that make the deadlock unrecoverable are the ones
    // this test would otherwise have hit.
    let err = process_command(state.clone(), Command::PassPriority { player: p(2) }).unwrap_err();
    assert!(
        matches!(err, GameStateError::PlayerEliminated(_)),
        "a conceded player can never act again: {err:?}"
    );
    for other in [p(1), p(3)] {
        if other == holder {
            continue;
        }
        let err = process_command(state.clone(), Command::PassPriority { player: other });
        assert!(
            matches!(err, Err(GameStateError::NotPriorityHolder { .. })),
            "only the holder may pass"
        );
    }
}

// ── T22 (fix cycle, Finding 2 / card Findings 1+2) ────────────────────────────

/// CR 601.2c — "If the spell has a variable number of targets, the player
/// announces how many targets they will choose before they announce those
/// targets." An `UpToN { count: N }` slot accepts **up to N**, not one.
///
/// Fix-cycle Finding 2 (HIGH). The shipped `TriggerTargetOption` dropped `count`
/// and the cardinality check was a hard `submitted.len() <= 1`, so Elder Deep-Fiend
/// ("tap up to **four** target permanents") and Cloud of Faeries ("untap up to
/// **two** lands") -- both `Complete` -- could still announce at most one target.
///
/// **Fail-before**: `slot.max` did not exist, and the 4-target answer was rejected
/// with "expected 0 or 1".
///
/// **PB-DX28 correction**: Cloud of Faeries was always the WRONG oracle example
/// for this primitive. "untap up to two lands" prints no "target" at all (CR
/// 115.10), so `UpToN` -- a real `TargetRequirement` -- was itself the deviation
/// `OOS-DX4-6` later named: it let hexproof/shroud/protection wrongly restrict the
/// choice. PB-DX28 §1 migrated it onto the new `EffectTarget::ChosenObject`
/// (resolution-time, untargeted) channel instead, so it no longer appears in this
/// census at all. Elder Deep-Fiend's "target permanents" is a REAL target and is
/// untouched.
#[test]
fn test_dp8_up_to_n_accepts_n_targets_not_one() {
    // The one oracle count this fixes, read off the real def (SR-36: enumerated,
    // not grepped) so the fixture is the card's own shape.
    let mut oracle_counts: Vec<(String, u32)> = Vec::new();
    for def in all_cards() {
        if def.name != "Elder Deep-Fiend" {
            continue;
        }
        for ability in &def.abilities {
            if let AbilityDefinition::Triggered { targets, .. } = ability {
                for req in targets {
                    if let TargetRequirement::UpToN { count, .. } = req {
                        oracle_counts.push((def.name.clone(), *count));
                    }
                }
            }
        }
    }
    oracle_counts.sort();
    assert_eq!(
        oracle_counts,
        vec![("Elder Deep-Fiend".to_string(), 4)],
        "oracle: 'tap up to four target permanents'"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::UpToN {
                count: 4,
                inner: Box::new(TargetRequirement::TargetCreature),
            }],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 2", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 3", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    let entry = state.pending_trigger_targets().unwrap();
    let choice_id = entry.choice_id;
    assert_eq!(
        entry.slots[0].max, 4,
        "CR 601.2c: the slot's declared width"
    );
    assert_eq!(entry.slots[0].candidates.len(), 4);
    let ids: Vec<Target> = (0..4)
        .map(|i| Target::Object(find_object(&state, &format!("Creature {i}"))))
        .collect();

    // Five targets in a four-wide slot: rejected.
    let err = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![ids
                .iter()
                .cloned()
                .chain(ids[..1].iter().cloned())
                .collect()],
        },
    )
    .unwrap_err();
    assert!(matches!(&err, GameStateError::InvalidCommand(m) if m.contains("0 to 4")));

    // CR 601.2c: "The same target can't be chosen multiple times for any one
    // instance of the word 'target'." Latent while the cap was 1.
    let err2 = process_command(
        state.clone(),
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![ids[0].clone(), ids[0].clone()]],
        },
    )
    .unwrap_err();
    assert!(
        matches!(&err2, GameStateError::InvalidCommand(m) if m.contains("same target twice")),
        "got {err2:?}"
    );

    // All four: accepted, and all four land on the stack object in order.
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![ids.clone()],
        },
    )
    .unwrap();
    let placed: Vec<Target> = state.stack_objects()[0]
        .targets
        .iter()
        .map(|t| t.target.clone())
        .collect();
    assert_eq!(
        placed, ids,
        "CR 601.2c: 'up to four' means four are announceable"
    );
}

// ── T23 (fix cycle, Finding 6 / card Finding 3) ───────────────────────────────

/// CR 601.2c — an under-filled "up to" slot must not shift the LATER slots'
/// `EffectTarget::DeclaredTarget { index }` down.
///
/// Fix-cycle Finding 6 (MEDIUM). `chosen` was a flat concatenation, so answering
/// `[[], [player]]` produced `[player]` and `DeclaredTarget { index: 0 }` -- the
/// first clause -- resolved to the *second* slot's target. Sword of Sinew and Steel
/// ("destroy up to one target planeswalker **and** up to one target artifact") is
/// correct today only because both of its clauses are `DestroyPermanent`.
///
/// **Fail-before**: `targets[0]` was the player and `targets.len()` was 1.
#[test]
fn test_dp8_under_filled_optional_slot_does_not_shift_later_indices() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![
                TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreature),
                },
                TargetRequirement::TargetPlayer,
            ],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let choice_id = state.pending_trigger_targets().unwrap().choice_id;

    // Slot 0 declined, slot 1 answered.
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![], vec![Target::Player(p(2))]],
        },
    )
    .unwrap();

    let targets = &state.stack_objects()[0].targets;
    assert_eq!(
        targets.len(),
        2,
        "the declined slot keeps its one-wide position"
    );
    assert!(
        targets[0].is_unchosen_slot(),
        "CR 601.2c: slot 0 was declined, so index 0 must name nothing -- got {:?}",
        targets[0].target
    );
    assert_eq!(
        targets[1].target,
        Target::Player(p(2)),
        "slot 1's target must stay at index 1, where its card def's \
         `DeclaredTarget {{ index: 1 }}` reads it"
    );

    // And the all-empty answer still produces an EMPTY list, so CR 608.2b's
    // "all targets are illegal" fizzle cannot fire on a legally-empty announcement.
    let mut state2 = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![
                TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetCreature),
                },
                TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetCreature),
                },
            ],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let b2 = find_object(&state2, "Bystander");
    let _ = fire_etb(&mut state2, b2, p(1));
    let cid2 = state2.pending_trigger_targets().unwrap().choice_id;
    let (state2, _) = process_command(
        state2,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id: cid2,
            targets: vec![vec![], vec![]],
        },
    )
    .unwrap();
    assert!(
        state2.stack_objects()[0].targets.is_empty(),
        "a trailing hole is omitted, never padded"
    );
}

// ── T24 (fix cycle, Finding 8) ────────────────────────────────────────────────

/// CR 601.2c — an `optional` slot with NO candidates has exactly one legal answer
/// ("choose zero"), so it is not a choice and must not be asked.
///
/// Fix-cycle Finding 8 (LOW). The forced-answer check (`forced_trigger_target_answer`
/// since the second closing review; `trigger_target_choice_is_forced` then) excluded every
/// `optional` slot unconditionally, so an "up to one target creature" trigger on an
/// empty board spent a wire round trip on a question with one possible answer.
///
/// **Fail-before**: the flush suspended and `pending_trigger_targets()` was `Some`.
#[test]
fn test_dp8_optional_slot_with_no_candidates_asks_nothing() {
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
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let events = fire_etb(&mut state, bystander, p(1));

    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 601.2c: 'up to one' with nothing to choose from has one legal answer"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::TriggerTargetChoiceRequired { .. })));
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.3d: zero targets is a legal announcement for an 'up to' slot, so the \
         ability is placed rather than removed"
    );
    assert!(state.stack_objects()[0].targets.is_empty());
}

// ── T25 (fix cycle, Finding 3) ────────────────────────────────────────────────

/// CR 603.3b / CR 117.3b — the 31st `check_and_flush_triggers` call site.
///
/// Fix-cycle Finding 3 (MEDIUM). `handle_all_passed`'s
/// `force_resolve_overdue_payments` branch (PB-DP4, CR 118.12a) flushes and then
/// grants priority **unconditionally**. A forced echo decline sacrifices the
/// permanent, which produces a dies-trigger, which can reach the CR 603.3d
/// announcement -- so that flush can suspend, and the grant below it ran anyway.
/// The plan's §16 mechanical check only grepped `flush_pending_triggers\(`, which
/// does not see this site; it now greps `check_and_flush_triggers\(` too.
///
/// **Fail-before**: a `PriorityGiven` was emitted with the CR 603.3b batch
/// half-placed, and answering emitted a second one.
#[test]
fn test_dp8_overdue_payment_branch_grants_no_priority_while_suspended() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::enchantment(p(1), "Reaper").with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureDies,
                intervening_if: None,
                description: "PB-DP8 fixture: on any creature dying, zap a creature".to_string(),
                effect: Some(Effect::DealDamage {
                    source: None,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                }),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: vec![TargetRequirement::TargetCreature],
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
            }),
        )
        .object(ObjectSpec::creature(p(1), "Echoer", 1, 1))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let echoer = find_object(&state, "Echoer");
    // PB-DP4 (CR 702.30a): an unpaid echo whose deadline passes is a decline, and
    // `force_resolve_overdue_payments` sacrifices the permanent.
    state
        .pending_echo_payments_mut()
        .push_back((p(1), echoer, Default::default()));
    state.turn_mut().priority_holder = Some(p(1));

    let (state, e1) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, e2) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let all: Vec<GameEvent> = e1.into_iter().chain(e2).collect();

    assert!(
        state.pending_trigger_targets().is_some(),
        "the forced echo decline's dies-trigger has two legal creature targets, so \
         the flush at `handle_all_passed`'s payment branch must suspend"
    );
    let after_ask = all
        .iter()
        .skip_while(|e| !matches!(e, GameEvent::TriggerTargetChoiceRequired { .. }))
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert_eq!(
        after_ask, 0,
        "CR 603.3b: this site grants priority unconditionally -- it must not do so \
         while the batch is suspended"
    );

    // The obligation is discharged on resume, so the game does not hang.
    let (state, resume) = answer_pending_trigger_targets(state);
    assert!(state.pending_trigger_targets().is_none());
    assert!(
        resume
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 117.3b: the payment branch owed the grant; the resume must discharge it"
    );
    assert!(state.turn().priority_holder.is_some());
}

// ── T26 (fix cycle, Finding 4 / OOS-DP8-10) ───────────────────────────────────

/// CR 514.3a / CR 726 — `enter_step`'s Cleanup guard owed more than a priority
/// grant.
///
/// Fix-cycle Finding 4 (MEDIUM), closing seed OOS-DP8-10 and widening it. The guard
/// returns before `state.turn.cleanup_sba_rounds += 1` **and** before
/// `loop_detection::check_for_mandatory_loop`, so every cleanup round that suspended
/// left the 100-round ratchet where it was (the cleanup step could then never fall
/// through to auto-advance) and CR 726's mandatory-loop draw was never declared for
/// any batch that suspends. `finish_resumed_flush` now reproduces both.
///
/// **Fail-before**: `cleanup_sba_rounds` was unchanged after the answer.
#[test]
fn test_dp8_suspended_cleanup_batch_still_advances_the_ratchet() {
    let mut state = board_with_creatures(2, 2);
    state.turn_mut().step = Step::Cleanup;
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
    let rounds_before = state.turn().cleanup_sba_rounds;

    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    assert!(
        state.pending_trigger_targets().is_some(),
        "the Cleanup-branch flush must suspend on the real choice"
    );
    assert_eq!(
        state.turn().cleanup_sba_rounds,
        rounds_before,
        "the ratchet belongs to the resume, not the suspension"
    );

    let (state, _) = answer_pending_trigger_targets(state);
    assert_eq!(
        state.turn().cleanup_sba_rounds,
        rounds_before + 1,
        "CR 514.3a: the 100-round cleanup ratchet must keep advancing across a \
         suspension, or the cleanup step can never fall through to auto-advance"
    );
    assert!(
        state.turn().priority_holder.is_some(),
        "CR 514.3a: the Cleanup branch also owed the priority grant"
    );
}

// ── T27 (fix cycle, Finding 4 — the CR 726 half) ──────────────────────────────

/// CR 104.4b / CR 726 — the mandatory-loop check is not skipped by a suspension.
///
/// Fix-cycle Finding 4, the half seed OOS-DP8-10 did not name. Both `enter_step`
/// guards return *before* `loop_detection::check_for_mandatory_loop`, so a mandatory
/// infinite loop involving a targeted triggered ability would never produce the
/// `LoopDetected` draw -- the engine would simply cycle forever. `check_for_mandatory_loop`
/// records the position it just examined, so its having run is observable:
/// `Command::ChooseTriggerTargets` resets the table (CR 104.4b, a real choice) and
/// the resume's check re-populates it with exactly the one position.
///
/// **Fail-before**: the table was empty after the resume.
#[test]
fn test_dp8_resume_runs_the_cr726_loop_check() {
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

    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    assert!(state.pending_trigger_targets().is_some());

    let (state, _) = answer_pending_trigger_targets(state);
    assert_eq!(
        state.loop_detection_hashes().len(),
        1,
        "CR 726: `enter_step`'s has-priority guard returned before the mandatory-loop \
         check, so the resume owes it"
    );
}

// ── T28 (fix cycle, Finding 7 — the resolution-tail guard) ────────────────────

/// CR 603.3b / CR 117.3b — the resolution tail's guard, which nothing tested.
///
/// A triggered ability resolves, its damage kills a creature, the resulting
/// dies-trigger has two legal targets, and the flush at `resolution.rs`'s tail
/// suspends. That tail resets `players_passed` and grants priority to the active
/// player immediately after the flush.
#[test]
fn test_dp8_resolution_tail_guard_grants_no_priority_while_suspended() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(1),
            "Zapper",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(
            ObjectSpec::enchantment(p(1), "Reaper").with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureDies,
                intervening_if: None,
                description: "PB-DP8 fixture: on any creature dying, zap a creature".to_string(),
                effect: Some(Effect::DealDamage {
                    source: None,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                }),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: vec![TargetRequirement::TargetCreature],
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
            }),
        )
        .object(ObjectSpec::creature(p(1), "Fragile", 1, 1))
        .object(ObjectSpec::creature(p(1), "Creature 0", 4, 4))
        .object(ObjectSpec::creature(p(1), "Creature 1", 4, 4))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander_creature = find_object(&state, "Fragile");
    let zapper_events = fire_etb(&mut state, bystander_creature, p(1));
    let _ = zapper_events;

    // Announce the Zapper's target: the 1/1, which its 2 damage will kill.
    let choice_id = state.pending_trigger_targets().unwrap().choice_id;
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: p(1),
            choice_id,
            targets: vec![vec![Target::Object(bystander_creature)]],
        },
    )
    .unwrap();
    assert_eq!(state.stack_objects().len(), 1);

    // Resolve it. The 1/1 dies to SBAs, and the resulting dies-trigger has two
    // legal targets (the two 4/4s), so the resolution tail's flush suspends.
    let (state, e1) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, e2) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let all: Vec<GameEvent> = e1.into_iter().chain(e2).collect();

    assert!(
        state.pending_trigger_targets().is_some(),
        "the dies-trigger's flush at the resolution tail must suspend"
    );
    let after_ask = all
        .iter()
        .skip_while(|e| !matches!(e, GameEvent::TriggerTargetChoiceRequired { .. }))
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert_eq!(after_ask, 0, "CR 603.3b: no priority while suspended");

    let (state, resume) = answer_pending_trigger_targets(state);
    assert!(resume
        .iter()
        .any(|e| matches!(e, GameEvent::PriorityGiven { .. })));
    assert!(state.turn().priority_holder.is_some());
}

// ── T29 (fix cycle, Finding 7 — `handle_declare_attackers`' guard) ────────────

/// CR 508.1 / CR 603.3b — `handle_declare_attackers` flushes attack triggers and
/// then unconditionally writes `priority_holder` + `PriorityGiven`. Its guard had
/// no test.
#[test]
fn test_dp8_declare_attackers_guard_grants_no_priority_while_suspended() {
    let attacker =
        ObjectSpec::creature(p(1), "Vanguard", 2, 2).with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfAttacks,
            intervening_if: None,
            description: "PB-DP8 fixture: on attack, zap a creature".to_string(),
            effect: Some(Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
        });
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p(1))
        .at_step(Step::DeclareAttackers)
        .object(attacker)
        .object(ObjectSpec::creature(p(2), "Blocker 0", 3, 3))
        .object(ObjectSpec::creature(p(2), "Blocker 1", 3, 3))
        .build()
        .unwrap();
    let vanguard = find_object(&state, "Vanguard");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p(1),
            attackers: vec![(
                vanguard,
                mtg_engine::state::combat::AttackTarget::Player(p(2)),
            )],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();

    assert!(
        state.pending_trigger_targets().is_some(),
        "CR 508.1: the attack trigger has three legal creature targets -- a real choice"
    );
    let after_ask = events
        .iter()
        .skip_while(|e| !matches!(e, GameEvent::TriggerTargetChoiceRequired { .. }))
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert_eq!(after_ask, 0, "CR 603.3b: no priority while suspended");

    let (state, resume) = answer_pending_trigger_targets(state);
    assert!(
        resume
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 117.3b: `handle_declare_attackers` owed the grant"
    );
    assert!(state.turn().priority_holder.is_some());
}

// ── T30 (fix cycle, Finding 7 — the dead-active-player fallback) ──────────────

/// CR 800.4j — the one resume branch that grants priority to somebody other than
/// the active player: the active player has left the game while the batch was
/// suspended, so `finish_resumed_flush` routes to `next_priority_player`.
#[test]
fn test_dp8_resume_routes_past_a_dead_active_player() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        .object(zapper(
            p(2),
            "Zapper",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(2), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(2), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(2), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let triggers = check_triggers(
        &state,
        &[GameEvent::PermanentEnteredBattlefield {
            object_id: bystander,
            player: p(2),
        }],
    );
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    state.turn_mut().priority_holder = Some(p(1));

    // Drive `enter_step`'s has-priority branch: it suspends, owing a grant.
    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(2))
    );

    // The ACTIVE player leaves while p2's announcement is outstanding.
    let (state, _) = process_command(state, Command::Concede { player: p(1) }).unwrap();
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(2)),
        "fix-cycle Finding 5: the foreign concede leaves the block alone"
    );

    let (state, resume) = answer_pending_trigger_targets(state);
    let granted: Vec<PlayerId> = resume
        .iter()
        .filter_map(|e| match e {
            GameEvent::PriorityGiven { player } => Some(*player),
            _ => None,
        })
        .collect();
    assert!(
        !granted.contains(&p(1)),
        "CR 800.4j: a departed active player must not be handed priority, got {granted:?}"
    );
    assert!(
        state
            .turn()
            .priority_holder
            .is_some_and(|h| h == p(2) || h == p(3)),
        "the resume must hand priority to a live player, or the game hangs"
    );
}

// ── T31 (fix cycle, Finding 7 — `handle_declare_blockers`' guard) ─────────────

/// CR 509.1 / CR 603.3b — the last untested guard site.
/// `handle_declare_blockers` flushes block triggers and then unconditionally
/// resets `players_passed` and grants priority to the active player.
#[test]
fn test_dp8_declare_blockers_guard_grants_no_priority_while_suspended() {
    let wall =
        ObjectSpec::creature(p(2), "Wall", 0, 4).with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfBlocks,
            intervening_if: None,
            description: "PB-DP8 fixture: on block, zap a creature".to_string(),
            effect: Some(Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
        });
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p(1))
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p(1), "Vanguard", 2, 2))
        .object(wall)
        .object(ObjectSpec::creature(p(1), "Bystander Creature", 2, 2))
        .build()
        .unwrap();
    let vanguard = find_object(&state, "Vanguard");
    let wall_id = find_object(&state, "Wall");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p(1),
            attackers: vec![(
                vanguard,
                mtg_engine::state::combat::AttackTarget::Player(p(2)),
            )],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p(2) }).unwrap();
    assert_eq!(state.turn().step, Step::DeclareBlockers);

    let (state, events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p(2),
            blockers: vec![(wall_id, vanguard)],
        },
    )
    .unwrap();

    assert!(
        state.pending_trigger_targets().is_some(),
        "CR 509.1: the block trigger has three legal creature targets -- a real choice"
    );
    let after_ask = events
        .iter()
        .skip_while(|e| !matches!(e, GameEvent::TriggerTargetChoiceRequired { .. }))
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert_eq!(after_ask, 0, "CR 603.3b: no priority while suspended");

    let (state, resume) = answer_pending_trigger_targets(state);
    assert!(
        resume
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 117.3b: `handle_declare_blockers` owed the grant"
    );
    assert!(state.turn().priority_holder.is_some());
}

// ── T32 (fix cycle, Finding 9) ────────────────────────────────────────────────

/// CR 800.4d — the liveness filter and the raw-field guards must not disagree.
///
/// Fix-cycle Finding 9 (LOW). `blocking_decision()` reports `None` for an entry
/// whose player is no longer alive, but `flush_pending_triggers` and the six
/// in-crate guards read the raw field. `handle_concede` clears only its OWN
/// player's entry, so a player eliminated by any other route (the CR 704.5a/b
/// player-loss SBAs, a resolving effect, a replacement) left an entry that was
/// invisible to the admission gate and yet blocked every subsequent flush forever.
///
/// **Fail-before**: the entry survived, `flush_pending_triggers` early-returned on
/// it, and the rest of the CR 603.3b batch was never placed.
#[test]
fn test_dp8_entry_of_a_player_eliminated_outside_concede_is_reaped() {
    let mut state = GameStateBuilder::new()
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
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));
    let p2_zapper = find_object(&state, "Zapper P2");
    let p1_zapper = find_object(&state, "Zapper P1");
    assert_eq!(state.pending_trigger_targets().unwrap().player, p(2));

    // p2 is eliminated by something that is NOT `Command::Concede` -- e.g. the
    // CR 704.5a life-total SBA, which only writes `has_lost`.
    state.players_mut().get_mut(&p(2)).unwrap().has_lost = true;
    assert!(
        state.blocking_decision().is_none(),
        "the liveness filter already hides a dead owner's entry"
    );

    // The next flush must converge rather than early-return on the dead entry.
    let events = flush_pending_triggers(&mut state);
    assert!(
        state.pending_trigger_targets().is_none()
            || state.pending_trigger_targets().unwrap().player != p(2),
        "CR 800.4d: a departed player's entry must not survive the flush"
    );
    let _ = events;
    let (state, _) = answer_all_pending_trigger_targets(state);
    let placed = trigger_sources(&state);
    assert!(
        !placed.contains(&p2_zapper),
        "CR 800.4d: a departed player's triggered ability is not put on the stack"
    );
    assert!(
        placed.contains(&p1_zapper),
        "CR 800.4j / 603.3b: the rest of the batch is still placed"
    );
}

// ── T33 (closing review, Finding 2 — MEDIUM) ──────────────────────────────────

/// CR 603.3b / CR 117.3a — the reap must not discharge the priority debt inside
/// the caller's own flush.
///
/// Closing-review Finding 2 (MEDIUM). `flush_pending_triggers` reaps a departed
/// owner's entry (fix-cycle Finding 9) through `drop_departed_trigger_flush`,
/// which itself calls `finish_resumed_flush(owed, ..)`. When `owed` was set by a
/// guard -- here `FlushResumeSite::EnterStepPriority` -- priority was granted
/// *inside* the flush; the flush then returned, the caller's guard saw
/// `pending_trigger_targets == None` and granted priority **again**. Two
/// `PriorityGiven` for one step entry, two `players_passed` resets, and a
/// `priority_holder` overwrite if the two ever disagreed.
///
/// The debt belongs to a call site whose moment has passed; the CURRENT caller's
/// own obligation is what is owed now, so the reap no longer discharges anything.
///
/// **Fail-before**: two `PriorityGiven` events after the reaping step entry
/// (`left: 2, right: 1`).
#[test]
fn test_dp8_reap_does_not_double_grant_priority_at_a_guarded_site() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        // The entry will belong to p2: it is p2's zapper that triggers.
        .object(zapper(
            p(2),
            "Zapper P2",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
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

    // Suspend at `enter_step`'s has-priority branch, which owes
    // `FlushResumeSite::EnterStepPriority`.
    let mut state = state;
    for seat in [p(1), p(2), p(3)] {
        let (s, _) = process_command(state, Command::PassPriority { player: seat }).unwrap();
        state = s;
        if state.pending_trigger_targets().is_some() {
            break;
        }
    }
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(2)),
        "the step-entry flush must suspend on p2's trigger"
    );

    // p2 is eliminated by a route that is NOT `Command::Concede` (CR 704.5a), so
    // nothing clears the entry -- the next flush reaps it.
    state.players_mut().get_mut(&p(2)).unwrap().has_lost = true;
    state.turn_mut().priority_holder = Some(p(1));
    state.turn_mut().players_passed = imbl::OrdSet::new();

    // Drive a step entry: p1 and p3 pass, `handle_all_passed` advances the step and
    // `enter_step` flushes -- reaping p2's entry on the way in.
    let (state, _ev1) = process_command(state, Command::PassPriority { player: p(1) }).unwrap();
    // p3's pass is the one that empties the priority round: `handle_all_passed` ->
    // `advance_step` -> `enter_step`, whose flush does the reaping.
    let (state, ev2) = process_command(state, Command::PassPriority { player: p(3) }).unwrap();
    let grants = ev2
        .iter()
        .filter(|e| matches!(e, GameEvent::PriorityGiven { .. }))
        .count();
    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 800.4d: the departed owner's entry is reaped"
    );
    assert_eq!(
        grants, 1,
        "CR 603.3b: one step entry grants priority exactly once -- the reaped entry's \
         debt belongs to a call site whose moment has passed"
    );
    assert!(state.turn().priority_holder.is_some());
}

// ── T34 (closing review, Finding 3 — LOW) ─────────────────────────────────────

/// CR 601.2c / SR-38 — the engine must accept its own default answer.
///
/// Closing-review Finding 3 (LOW). For two `TargetPermanentDistinctFrom` slots
/// `trigger_target_candidates` handed both slots `default = candidates.first()`,
/// so `default_trigger_targets` emitted the SAME permanent twice and the handler's
/// own cross-slot distinctness check (8) rejected it. Everything that submits the
/// offered default verbatim -- `StubProvider`, `RandomBot`, `HeuristicBot`, the
/// replay harness's pump, the TUI's announce key -- would take the refusal, and
/// `LocalGame` turns a refused fallback into a `Halted`. Zero corpus exposure
/// today (OOS-DP8-4), but `default_trigger_targets`' doc comment GUARANTEES the
/// engine accepts its output, so the code has to have the property the comment
/// claims.
///
/// **Fail-before**: `InvalidCommand("slots 0 and 1 both require distinct permanents
/// but name the same one (CR 601.2c)")` -- i.e. the engine refusing its own answer.
#[test]
fn test_dp8_default_answer_satisfies_cross_slot_distinctness() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        // Hidden Strings' shape: "tap/untap two target permanents" -- two slots,
        // each of which must name a different permanent than the other.
        .object(zapper(
            p(1),
            "Twister",
            vec![
                TargetRequirement::TargetPermanentDistinctFrom(1),
                TargetRequirement::TargetPermanentDistinctFrom(0),
            ],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    let entry = state
        .pending_trigger_targets()
        .expect("two multi-candidate required slots is a real announcement");
    let slots: Vec<TriggerTargetOption> = entry.slots.iter().cloned().collect();
    assert_eq!(slots.len(), 2);
    let choice_id = entry.choice_id;
    let player = entry.player;
    let answer = default_trigger_targets(&slots);
    assert_ne!(
        answer[0], answer[1],
        "CR 601.2c: the engine's own default must not name the same permanent for \
         two mutually-distinct slots"
    );

    // The real contract: the engine accepts it.
    let (state, _) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player,
            choice_id,
            targets: answer,
        },
    )
    .expect("SR-38: the engine must accept its own default answer");
    assert_eq!(trigger_sources(&state).len(), 1);
}

// ── T35 (second closing review, Finding 1 — MEDIUM) ───────────────────────────

/// CR 800.4 / CR 603.3b — a SECOND concede must not leave priority stranded on a
/// player who left the game earlier.
///
/// Second-closing-review Finding 1 (MEDIUM). The first closing review's HIGH was
/// closed by `abilities::repair_departed_priority_holder` at the end of
/// `resume_trigger_flush` -- but that choke point covers **answers**, not
/// **concessions**. `handle_concede`'s own priority advance is gated on
/// `state.turn.priority_holder == Some(player)`, so it can only repair holdership
/// belonging to the CONCEDER; a holder stranded by an *earlier* departure is
/// invisible to it, and `drop_departed_trigger_flush` (the path a concede takes
/// when the outstanding entry is the conceder's own) never calls the repair.
///
/// The four steps below are the reachable sequence:
///   1. p2 holds priority; the flush suspends on p1's trigger (`FlushResumeSite::None`).
///   2. p2 concedes -- the fix-cycle Finding 5 gate skips the advance because
///      p1's announcement is still outstanding, so `priority_holder` stays `Some(p2)`.
///   3. p1 concedes *instead of answering* -- the entry is dropped, the batch
///      completes, and neither `handle_concede` branch matches (the holder is p2,
///      the active player is p3).
///   4. `priority_holder == Some(p2)`, conceded, with no entry left to repair it.
///
/// **Fail-before** (run, not predicted): `priority must name a player who is still
/// in the game, not PlayerId(2)` -- the identical unrecoverable deadlock the first
/// closing review's HIGH described, one step further out. The assertion is placed
/// on the holder rather than on the `PassPriority` that follows it so the failure
/// names the stranded seat instead of the downstream `PlayerEliminated`.
#[test]
fn test_dp8_second_concede_does_not_strand_priority_from_an_earlier_departure() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(CardRegistry::new(vec![]))
        // The entry belongs to p1: it is p1's zapper that triggers. The active
        // player is p3, so neither `handle_concede` branch can fire for p1.
        .object(zapper(
            p(1),
            "Zapper P1",
            vec![TargetRequirement::TargetPlayer],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(3))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    // (1) the suspension is on p1's trigger, and p2 holds priority (PB-DP1: the
    // actor keeps it across an in-match `check_and_flush_triggers` suspension).
    assert_eq!(
        state.pending_trigger_targets().map(|e| e.player),
        Some(p(1)),
        "multiple live players make `TargetPlayer` a real CR 601.2c choice"
    );
    state.turn_mut().priority_holder = Some(p(2));

    // (2) the priority HOLDER concedes while somebody else's question is open.
    let (state, _) = process_command(state, Command::Concede { player: p(2) }).unwrap();
    assert!(
        state.pending_trigger_targets().is_some(),
        "the foreign concede must leave the batch suspended (fix-cycle Finding 5)"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p(2)),
        "the gate deliberately skips the advance -- this is the state the repair owes"
    );

    // (3) the entry's OWNER concedes instead of answering: CR 800.4d drops their
    // trigger, CR 800.4j places the rest of the batch, and the game continues with
    // p3 and p4 still in it.
    let (state, _) = process_command(state, Command::Concede { player: p(1) }).unwrap();
    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 800.4d: the departed owner's entry is dropped"
    );
    assert_eq!(state.active_players().len(), 2, "the game is not over");

    // (4) the game must still be playable.
    let holder = state
        .turn()
        .priority_holder
        .expect("CR 603.3b: somebody must hold priority once the batch is complete");
    assert!(
        holder == p(3) || holder == p(4),
        "priority must name a player who is still in the game, not {holder:?}"
    );
    let (_, _) = process_command(state.clone(), Command::PassPriority { player: holder })
        .expect("the holder must be able to act -- otherwise the game is deadlocked");
    for gone in [p(1), p(2)] {
        let err =
            process_command(state.clone(), Command::PassPriority { player: gone }).unwrap_err();
        assert!(
            matches!(err, GameStateError::PlayerEliminated(_)),
            "a departed player can never act again: {err:?}"
        );
    }
}

// ── T36 (second closing review, Finding 2 — LOW) ──────────────────────────────

/// CR 603.3d / CR 601.2c — two mutually-distinct slots whose only candidate is the
/// SAME permanent have no legal announcement, so the ability is not put on the
/// stack.
///
/// Second-closing-review Finding 2 (LOW). `default_trigger_targets`' "one
/// exception" doc section named the wrong failure mode for half its cases: with at
/// least one multi-candidate slot the handler's cross-slot check refuses the
/// default (that half is real), but when BOTH slots have exactly one candidate the
/// forced-answer path never consulted that check at all -- it read `candidates[0]`
/// directly and placed the trigger with the same permanent in both slots,
/// **silently violating CR 601.2c** instead of refusing.
///
/// CR 603.3d: "if a choice is required when the triggered ability goes on the stack
/// but no legal choices can be made for it ... the ability is simply removed from
/// the stack." That is the correct disposition here -- the constraint, not the
/// candidate sets, is what has no solution.
///
/// **Fail-before**: `trigger_sources(&state).len()` was `1` (the trigger placed
/// with slot 0 == slot 1), and `state.stack_objects()[0]`'s target list named
/// `Only One` twice.
#[test]
fn test_dp8_forced_answer_that_breaks_distinctness_removes_the_trigger() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        // Shroud (CR 702.18a) keeps the fixture's own permanents out of both
        // candidate sets, so `Only One` is the single legal choice for each slot.
        .object(
            zapper(
                p(1),
                "Twister",
                vec![
                    TargetRequirement::TargetPermanentDistinctFrom(1),
                    TargetRequirement::TargetPermanentDistinctFrom(0),
                ],
            )
            .with_keyword(KeywordAbility::Shroud),
        )
        .object(ObjectSpec::creature(p(1), "Only One", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander").with_keyword(KeywordAbility::Shroud))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bystander = find_object(&state, "Bystander");
    let _ = fire_etb(&mut state, bystander, p(1));

    assert!(
        state.pending_trigger_targets().is_none(),
        "one candidate per slot is not a choice -- nothing may be asked"
    );
    assert!(
        trigger_sources(&state).is_empty(),
        "CR 603.3d: no legal announcement exists, so the ability is not put on the stack \
         -- placing it would name the same permanent for two mutually-distinct slots \
         (CR 601.2c)"
    );
}

// ── T37 (second closing review, Finding 3 — LOW / OOS-DP8-13) ─────────────────

/// CR 514.3a / CR 726 — the reap drops only the reaped site's PRIORITY debt, and
/// keeps its cleanup ratchet.
///
/// Second-closing-review Finding 3 (LOW). The first closing review's Finding 2 fix
/// zeroed the reaped entry's whole `FlushResumeSite`, which is right for the
/// priority half (a double grant inside the current caller's own flush is a real
/// wire anomaly) but threw away the `cleanup_sba_rounds` ratchet and the CR 726
/// mandatory-loop check with it -- and losing a *bound* is a different severity
/// class from emitting a duplicate event (OOS-DP8-13). The two halves are now
/// separated: the obligations run, the grant does not.
///
/// **Fail-before**: `cleanup_sba_rounds` stayed at `0` after the reap
/// (`left: 0, right: 1`).
#[test]
fn test_dp8_reap_keeps_the_cleanup_ratchet_and_drops_only_the_priority_debt() {
    use mtg_engine::state::stubs::FlushResumeSite;

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .with_registry(CardRegistry::new(vec![]))
        // The entry will belong to p2.
        .object(zapper(
            p(2),
            "Zapper P2",
            vec![TargetRequirement::TargetCreature],
        ))
        .object(ObjectSpec::creature(p(1), "Creature 0", 2, 2))
        .object(ObjectSpec::creature(p(1), "Creature 1", 2, 2))
        .object(ObjectSpec::enchantment(p(1), "Bystander"))
        .active_player(p(1))
        .at_step(Step::End)
        .build()
        .unwrap();
    // Queue the trigger WITHOUT flushing, so the first flush that sees it is the
    // one `enter_step` runs in the Cleanup branch.
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
    state.turn_mut().players_passed = imbl::OrdSet::new();

    // Pass the end step out: `handle_all_passed` -> `advance_step` -> Cleanup ->
    // `enter_step`'s cleanup branch, whose flush suspends owing `EnterStepCleanup`.
    let mut state = state;
    for seat in [p(1), p(2), p(3)] {
        let (s, _) = process_command(state, Command::PassPriority { player: seat }).unwrap();
        state = s;
        if state.pending_trigger_targets().is_some() {
            break;
        }
    }
    let entry = state
        .pending_trigger_targets()
        .expect("the cleanup-step flush must suspend on p2's trigger");
    assert_eq!(entry.player, p(2));
    assert_eq!(
        entry.resume_site,
        FlushResumeSite::EnterStepCleanup,
        "the fixture must pin the site whose obligations this test is about"
    );
    let rounds_before = state.turn().cleanup_sba_rounds;

    // p2 leaves by a route that is NOT `Command::Concede` (CR 704.5a), so nothing
    // clears the entry -- the next flush reaps it.
    state.players_mut().get_mut(&p(2)).unwrap().has_lost = true;
    let reaped = flush_pending_triggers(&mut state);

    assert!(
        state.pending_trigger_targets().is_none(),
        "CR 800.4d: the departed owner's entry is reaped"
    );
    assert_eq!(
        state.turn().cleanup_sba_rounds,
        rounds_before + 1,
        "CR 514.3a: the reaped site's ratchet is a BOUND, not a notification -- \
         dropping it turns a bounded cleanup loop into an unbounded one"
    );
    assert!(
        !reaped
            .iter()
            .any(|e| matches!(e, GameEvent::PriorityGiven { .. })),
        "CR 603.3b: the PRIORITY half of the reaped debt still belongs to a call \
         site whose moment has passed (closing-review Finding 2)"
    );
}

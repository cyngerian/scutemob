//! Copy/redirect spell tests — PB-J.
//!
//! Verifies:
//! - Effect::CopySpellOnStack: basic copy (CR 707.10) and multi-copy
//! - Effect::ChangeTargets: must_change true (CR 115.7a) and false (CR 115.7d)
//! - TargetRequirement::TargetSpellOrAbilityWithSingleTarget: behavioral contract
//! - Integration: a single-target-spell redirect pattern (was named after Bolt
//!   Bend; see the note below on why that name moved)
//!
//! **PB-DX25b review Finding T1**: this file's `push_spell_targeting_player`
//! helper returns the `StackObject`'s OWN id -- every test in this file that
//! calls it and announces the returned id directly into `execute_effect` is
//! announcing a STACK-ENTRY id, not a CARD id (CR 601.2c). No real
//! `Command::CastSpell` can ever produce that shape: a real cast announces the
//! CARD's `state.objects` id, a DISJOINT id space from the `StackObject`'s own
//! id (`state::stack_registry::stack_index_for_announced_target`'s whole
//! reason to exist, PB-DX25b/`OOS-DX25-3`). These tests therefore exercise
//! only the DIRECT-id clause of that lookup (the one Ward, CR 702.21a, uses in
//! production) -- never the card-owning-kind clause a real Misdirection/Bolt
//! Bend cast depends on. They stayed green through the entire period both
//! cards could never resolve a legal target in a real game. For coverage of a
//! REAL cast reaching this code, see
//! `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs`
//! (`t1_misdirection_announces_and_resolves`, `t2_bolt_bend_announces_and_
//! resolves`) and `crates/engine/tests/primitives/pb_ef11_spell_single_
//! target.rs`. Not a coverage hole -- V7 of PB-DX25b's revert matrix confirmed
//! this file's own tests DO catch a regression of the direct-id clause -- but
//! a claim problem: this file's tests do not, and never did, prove anything
//! about Bolt Bend or Misdirection specifically.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::stack::{StackObject, StackObjectKind};
use mtg_engine::state::test_util;
use mtg_engine::{
    CardEffectTarget, CardType, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, Target, TargetRequirement, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build a minimal StackObject for a spell.
///
/// PB-DX25c (`OOS-DX25b-3`): `target_requirements` is no longer hardcoded
/// empty -- `rules::retarget::plan_target_change` FAILS CLOSED on an empty
/// list (`crates/card-types/src/state/stack.rs`'s `target_requirements` doc),
/// so a fixture that wants to exercise a real redirect must carry the
/// requirement its pretend-spell would really have been validated against at
/// cast time. Callers that don't care (the CopySpellOnStack tests, which
/// never reach `Effect::ChangeTargets`) may still pass `vec![]`.
fn make_stack_spell(
    id: ObjectId,
    controller: PlayerId,
    source: ObjectId,
    targets: Vec<SpellTarget>,
    target_requirements: Vec<TargetRequirement>,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind: StackObjectKind::Spell {
            source_object: source,
        },
        targets,
        target_requirements,
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        cast_right_half: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        damage_dealt_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}

/// Run an effect as the given controller with declared targets.
fn run_effect(
    mut state: GameState,
    controller: PlayerId,
    targets: Vec<SpellTarget>,
    effect: Effect,
) -> (GameState, Vec<GameEvent>) {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, targets);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

/// Build a base two-player state with no objects.
fn two_player_state() -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap()
}

/// Build a base three-player state with no objects.
fn three_player_state() -> GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap()
}

/// Push a spell onto the stack, returning its assigned ID.
///
/// PB-DX25c: `target_requirements` is the requirement list the pretend-spell
/// was "validated against" at cast time -- pass `vec![TargetRequirement::
/// TargetPlayer]` for a fixture standing in for a real "target player" spell
/// (which is what every caller in this file redirects). An empty list is
/// only correct for a fixture that never reaches `Effect::ChangeTargets`.
fn push_spell_targeting_player(
    state: &mut GameState,
    controller: PlayerId,
    target_player: PlayerId,
    target_requirements: Vec<TargetRequirement>,
) -> ObjectId {
    let id = test_util::next_object_id(state);
    let source = test_util::next_object_id(state); // dummy source
    let spell = make_stack_spell(
        id,
        controller,
        source,
        vec![SpellTarget {
            target: Target::Player(target_player),
            zone_at_cast: None,
        }],
        target_requirements,
    );
    state.stack_objects_mut().push_back(spell);
    id
}

/// Push a targetless spell onto the stack, returning its assigned ID.
fn push_targetless_spell(state: &mut GameState, controller: PlayerId) -> ObjectId {
    let id = test_util::next_object_id(state);
    let source = test_util::next_object_id(state);
    let spell = make_stack_spell(id, controller, source, vec![], vec![]);
    state.stack_objects_mut().push_back(spell);
    id
}

// ── CopySpellOnStack tests ────────────────────────────────────────────────────

#[test]
/// CR 707.10 — CopySpellOnStack creates one copy controlled by the effect controller.
/// The copy has is_copy: true and inherits the original's targets.
fn test_copy_spell_on_stack_basic() {
    let mut state = two_player_state();
    let original_stack_id = push_spell_targeting_player(&mut state, p(1), p(2), vec![]);

    assert_eq!(state.stack_objects().len(), 1);

    // Execute CopySpellOnStack — the controller targets the original stack spell.
    let copy_target = SpellTarget {
        target: Target::Object(original_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![copy_target],
        Effect::CopySpellOnStack {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            count: EffectAmount::Fixed(1),
        },
    );

    // One copy should be on the stack now (total 2).
    assert_eq!(
        state.stack_objects().len(),
        2,
        "expected original + 1 copy on stack"
    );

    // The copy should have is_copy: true.
    let copy = state
        .stack_objects()
        .iter()
        .find(|so| so.is_copy)
        .expect("copy not found");
    assert_eq!(
        copy.controller,
        p(1),
        "copy controlled by effect controller"
    );

    // The copy should inherit the original's player target.
    assert_eq!(copy.targets.len(), 1);
    assert_eq!(copy.targets[0].target, Target::Player(p(2)));

    // A SpellCopied event should be emitted.
    let copied_evt = events
        .iter()
        .find(|e| matches!(e, GameEvent::SpellCopied { .. }));
    assert!(copied_evt.is_some(), "SpellCopied event not emitted");
    if let Some(GameEvent::SpellCopied {
        original_stack_id: orig,
        ..
    }) = copied_evt
    {
        assert_eq!(*orig, original_stack_id);
    }
}

#[test]
/// CR 707.10 — CopySpellOnStack with count=2 creates 2 copies (3 total with original).
/// All copies have is_copy: true.
fn test_copy_spell_on_stack_twice() {
    let mut state = two_player_state();
    let original_stack_id = push_targetless_spell(&mut state, p(1));

    let copy_target = SpellTarget {
        target: Target::Object(original_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![copy_target],
        Effect::CopySpellOnStack {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            count: EffectAmount::Fixed(2),
        },
    );

    // 3 total: original + 2 copies.
    assert_eq!(
        state.stack_objects().len(),
        3,
        "expected original + 2 copies on stack"
    );

    let copies: Vec<_> = state
        .stack_objects()
        .iter()
        .filter(|so| so.is_copy)
        .collect();
    assert_eq!(copies.len(), 2, "expected exactly 2 copies");

    let copy_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellCopied { .. }))
        .collect();
    assert_eq!(copy_events.len(), 2, "expected 2 SpellCopied events");
}

// ── ChangeTargets must_change tests ──────────────────────────────────────────

#[test]
/// CR 115.7a — ChangeTargets must_change: true redirects the target to a different
/// legal player when another player exists. Bolt Bend controller p(1) redirects
/// a spell currently targeting p(2).
fn test_change_targets_must_change_redirects_to_new_player() {
    let mut state = two_player_state();
    // p(2) cast a spell targeting themselves. PB-DX25c: the pretend-spell is
    // a "target player" spell (Bolt Bend's real victim shape), so it carries
    // TargetPlayer -- an empty list fails closed and no redirect happens.
    let bolt_stack_id = push_spell_targeting_player(
        &mut state,
        p(2),
        p(2),
        vec![TargetRequirement::TargetPlayer],
    );

    // p(1) (Bolt Bend controller) redirects the spell.
    let bolt_bend_target = SpellTarget {
        target: Target::Object(bolt_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![bolt_bend_target],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: true,
        },
    );

    let bolt = state
        .stack_objects()
        .iter()
        .find(|s| s.id == bolt_stack_id)
        .expect("bolt not found on stack");

    assert_eq!(bolt.targets.len(), 1);
    // p(2) was the original target; p(1) is the only alternative.
    assert_eq!(
        bolt.targets[0].target,
        Target::Player(p(1)),
        "bolt target should change to p(1)"
    );

    // TargetsChanged event emitted.
    let changed_evt = events
        .iter()
        .find(|e| matches!(e, GameEvent::TargetsChanged { .. }));
    assert!(
        changed_evt.is_some(),
        "TargetsChanged event should be emitted"
    );
}

#[test]
/// CR 115.7a — ChangeTargets must_change: true with no alternative target leaves
/// the target unchanged. "If there are no legal targets to choose from, the target
/// isn't changed."
fn test_change_targets_no_alternative_leaves_unchanged() {
    // Only one player — no alternative player target exists.
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // PB-DX25c: a real requirement list is recorded even though this test's
    // whole point is that no OTHER candidate exists (only p(1) is in the
    // game) -- with it, the "no alternative" fallback fires because the
    // candidate scan finds nothing but the current target; without it, the
    // SAME outcome would fire for the unrelated reason of the fail-closed
    // guard (§3.4). Recording a real list keeps this test discriminating
    // the CR 115.7a "no alternative" case, not the missing-requirement case.
    let spell_stack_id = push_spell_targeting_player(
        &mut state,
        p(1),
        p(1),
        vec![TargetRequirement::TargetPlayer],
    );

    let redirect_target = SpellTarget {
        target: Target::Object(spell_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![redirect_target],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: true,
        },
    );

    let spell = state
        .stack_objects()
        .iter()
        .find(|s| s.id == spell_stack_id)
        .expect("spell not found");
    // Target remains p(1) — no alternative.
    assert_eq!(spell.targets[0].target, Target::Player(p(1)));

    // No TargetsChanged event when no change was made.
    let changed = events
        .iter()
        .any(|e| matches!(e, GameEvent::TargetsChanged { .. }));
    assert!(!changed, "no TargetsChanged event when target is unchanged");
}

#[test]
/// CR 115.7d — ChangeTargets must_change: false (Deflecting Swat) leaves targets
/// unchanged in the deterministic fallback. The player "chose" not to change them.
fn test_change_targets_may_choose_new_leaves_unchanged() {
    let mut state = two_player_state();
    let spell_stack_id = push_spell_targeting_player(
        &mut state,
        p(1),
        p(2),
        vec![TargetRequirement::TargetPlayer],
    );

    let swat_target = SpellTarget {
        target: Target::Object(spell_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![swat_target],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: false,
        },
    );

    let spell = state
        .stack_objects()
        .iter()
        .find(|s| s.id == spell_stack_id)
        .expect("spell not found");
    // Target unchanged — deterministic fallback for CR 115.7d.
    assert_eq!(spell.targets[0].target, Target::Player(p(2)));

    let changed = events
        .iter()
        .any(|e| matches!(e, GameEvent::TargetsChanged { .. }));
    assert!(
        !changed,
        "no TargetsChanged event for may-change with unchanged targets"
    );
}

// ── TargetSpellOrAbilityWithSingleTarget behavioral tests ─────────────────────

#[test]
/// CR 115.7a — TargetSpellOrAbilityWithSingleTarget: a spell with exactly 1 target
/// is a valid target for ChangeTargets. The redirect should succeed and change the
/// single target.
fn test_change_targets_accepts_single_target_spell() {
    let mut state = three_player_state();
    // p(2) cast a spell targeting p(3) — exactly one target.
    let single_target_stack_id = push_spell_targeting_player(
        &mut state,
        p(2),
        p(3),
        vec![TargetRequirement::TargetPlayer],
    );

    // p(1) redirects via ChangeTargets.
    let redirect_target = SpellTarget {
        target: Target::Object(single_target_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![redirect_target],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: true,
        },
    );

    let spell = state
        .stack_objects()
        .iter()
        .find(|s| s.id == single_target_stack_id)
        .expect("spell not found");

    // Target should have changed to p(1) (the effect controller).
    assert_eq!(spell.targets[0].target, Target::Player(p(1)));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "TargetsChanged event should be emitted"
    );
}

#[test]
/// CR 115.7a — ChangeTargets on a spell targeting an object redirects to a
/// different object in the same zone (simplified battlefied redirect).
fn test_change_targets_object_redirect() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(2), "Creature A")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .object(
            ObjectSpec::card(p(1), "Creature B")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Find the two battlefield objects.
    let mut bf_ids: Vec<ObjectId> = state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .collect();
    bf_ids.sort();
    assert_eq!(bf_ids.len(), 2);
    let creature_a_id = bf_ids[0];
    let creature_b_id = bf_ids[1];

    // A spell on the stack targeting Creature A. PB-DX25c: the pretend-spell
    // is a "target creature" spell, so it carries TargetCreature -- both
    // battlefield objects are Creature-typed, so this exercises the type
    // check without deciding the outcome on it. Note (plan §5.5 table): this
    // fixture's `source` id is NOT in `state.objects`, so `source_chars` is
    // `None` here -- it therefore does NOT exercise the CR 702.16b
    // protection-from-colour path (T2 in `pb_dx25c_retarget_legality.rs`
    // does).
    let bolt_stack_id = test_util::next_object_id(&mut state);
    let source = test_util::next_object_id(&mut state);
    let bolt = make_stack_spell(
        bolt_stack_id,
        p(1),
        source,
        vec![SpellTarget {
            target: Target::Object(creature_a_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
        vec![TargetRequirement::TargetCreature],
    );
    state.stack_objects_mut().push_back(bolt);

    let redirect_target = SpellTarget {
        target: Target::Object(bolt_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![redirect_target],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: true,
        },
    );

    let bolt = state
        .stack_objects()
        .iter()
        .find(|s| s.id == bolt_stack_id)
        .expect("bolt not found");

    // Target should change to creature_b (the only other battlefield object).
    assert_eq!(
        bolt.targets[0].target,
        Target::Object(creature_b_id),
        "target should redirect to creature_b"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "TargetsChanged event should be emitted"
    );
    let _ = creature_b_id; // suppress unused warning
}

// ── Single-target-spell redirect (fixture-level, not a Bolt Bend integration test) ──

#[test]
/// CR 115.7a — redirects a single-target spell on the stack from one player to
/// another. Verifies ChangeTargets must_change: true with 3 players.
///
/// **PB-DX25b review Finding T1: renamed** (was
/// `test_bolt_bend_redirects_single_target_spell`) -- this test announces the
/// `StackObject`'s OWN id directly (see this file's module doc), a shape no
/// real Bolt Bend cast can produce, so it proved nothing about Bolt Bend
/// specifically; `pb_dx25b_announced_stack_target_space.rs::
/// t2_bolt_bend_announces_and_resolves` now owns that claim, through a real
/// `Command::CastSpell`.
fn test_change_targets_redirects_single_target_spell_by_stack_entry_id() {
    let mut state = three_player_state();
    // p(2) cast Lightning Bolt targeting p(3).
    let bolt_stack_id = push_spell_targeting_player(
        &mut state,
        p(2),
        p(3),
        vec![TargetRequirement::TargetPlayer],
    );

    // p(1) redirects to p(1) (the controller) -- announcing the STACK-ENTRY
    // id directly (see module doc: no real cast can produce this shape).
    let bolt_bend_declared = SpellTarget {
        target: Target::Object(bolt_stack_id),
        zone_at_cast: Some(ZoneId::Stack),
    };
    let (state, events) = run_effect(
        state,
        p(1),
        vec![bolt_bend_declared],
        Effect::ChangeTargets {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            must_change: true,
        },
    );

    // The bolt's target should now be p(1) (the Bolt Bend controller).
    let bolt = state
        .stack_objects()
        .iter()
        .find(|s| s.id == bolt_stack_id)
        .expect("lightning bolt not found");
    assert_eq!(
        bolt.targets[0].target,
        Target::Player(p(1)),
        "bolt should now target Bolt Bend's controller"
    );

    // TargetsChanged event with correct old/new targets.
    let evt = events
        .iter()
        .find(|e| matches!(e, GameEvent::TargetsChanged { .. }))
        .expect("TargetsChanged event not emitted");
    if let GameEvent::TargetsChanged {
        stack_object_id,
        old_targets,
        new_targets,
    } = evt
    {
        assert_eq!(*stack_object_id, bolt_stack_id);
        assert_eq!(old_targets[0].target, Target::Player(p(3)));
        assert_eq!(new_targets[0].target, Target::Player(p(1)));
    }
}

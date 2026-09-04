//! Tests for PB-EF11 COMMIT 2: `TargetRequirement::TargetSpellWithSingleTarget`
//! (CR 115.7a/115.7b/601.2a/601.2c).
//!
//! **Citation correction (PB-DX25b `OOS-DX25-3`, plan §4.4):** this file
//! previously cited "CR 115.10" for self-targeting prevention -- that rule is
//! the affects-vs-targets rule (CR 115.10/115.10a) and has nothing to do with a
//! spell targeting itself. The correct grounding is CR 601.2a + 601.2c + 115.7a:
//! at the moment targets are announced, the spell being cast has not yet chosen
//! any targets, so it is not an appropriate object for its own "single target".
//! Comment-only correction; behavior is unchanged. See `casting.rs`'s in-source
//! test module for the fuller version of this note.
//!
//! **PB-DX25b (`OOS-DX25-3`) non-vacuity repair (plan §5.2):** every fixture in
//! this file used to build `build_base_state`'s pre-existing stack object with
//! its StackObject's own id EQUAL to its `source_object` (`kind_with_source_object
//! (other_id)` where the StackObject's `id` was also `other_id`) -- collapsing
//! the announced-card-id space and the stack-entry-id space onto one id, the
//! exact defect `casting.rs:6480`/`:6506` (C1/C2) had at HEAD. `build_base_state`
//! now mints a SEPARATE `entry_id` for the StackObject and returns it alongside
//! `other_id` (the announced card id) so callers can tell the two apart; `Test 1`
//! (`test_spell_single_target_accepts_single_target_spell`) is the headline
//! non-vacuity proof -- it is RED against the un-fixed `casting.rs` lookup with
//! this repair in place (see `memory/primitives/pb-DX25b-execution-notes.md` for
//! the executed revert). `Test 6`
//! (`test_misdirection_retargets_single_target_spell`) is similarly rebuilt to
//! place a real victim CARD object in `ZoneId::Stack` and announce THAT id,
//! rather than announcing the StackObject's own id directly into `execute_effect`
//! -- the old version tested a path no real cast can ever produce.
//!
//! Misdirection's oracle ("Change the target of target spell with a single
//! target") needs a single-target restriction that is spell-ONLY — the existing
//! `TargetSpellOrAbilityWithSingleTarget` (Bolt Bend) also legalizes activated
//! and loyalty abilities, which would let Misdirection illegally retarget an
//! ability. This batch adds a sibling `TargetRequirement` variant whose
//! validation (`casting.rs`) additionally requires the target stack object's
//! `kind` to be `StackObjectKind::Spell` or `MutatingCreatureSpell` (both are
//! spells, CR 601/702.140).
//!
//! Precision tests for the self-targeting-prevention and kind-check branches
//! (which require driving `validate_object_satisfies_requirement` directly with
//! an explicit `self_id`, not reachable from the public `Command::CastSpell`
//! pipeline in every case) live alongside the sibling variant's own precision
//! test in `crates/engine/src/rules/casting.rs`'s internal `#[cfg(test)] mod
//! tests` (`test_target_spell_with_single_target_self_and_kind_check`). This
//! file covers the public-API-observable behavior: accepts/rejects through a
//! real cast, the hash discriminant, and the Misdirection card-def integration.
//!
//! `HASH_SCHEMA_VERSION` bumped 54 -> 55 (new `TargetRequirement::
//! TargetSpellWithSingleTarget` discriminant 19). `PROTOCOL_VERSION` bumped
//! 16 -> 17 (`TargetRequirement` is reachable from `AbilityDefinition.targets`,
//! part of the wire closure).

use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::{StackObject, StackObjectKind};
use mtg_engine::state::test_util;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameEvent, GameState, GameStateBuilder, GameStateError, ManaCost, ManaPool, ObjectId,
    ObjectSpec, PlayerId, SpellTarget, Step, Target, TargetRequirement, TypeLine, ZoneId,
    HASH_SCHEMA_VERSION,
};

use mtg_engine::effects::{execute_effect, EffectContext};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Build a minimal StackObject of the given `kind` with `targets`.
///
/// PB-DX25c (`OOS-DX25b-3`): `target_requirements` is a real parameter now,
/// not a hardcoded empty list -- `rules::retarget::plan_target_change` fails
/// closed on an empty list (`crates/card-types/src/state/stack.rs`'s doc), so
/// a fixture that wants a real `Effect::ChangeTargets` redirect must record
/// the requirement its pretend-spell would really have carried.
fn make_stack_object(
    id: ObjectId,
    controller: PlayerId,
    kind: StackObjectKind,
    targets: Vec<SpellTarget>,
    target_requirements: Vec<TargetRequirement>,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind,
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

/// A minimal instant with `targets: vec![TargetRequirement::TargetSpellWithSingleTarget]`
/// — the effect does nothing; only the target validation path is under test.
fn single_target_test_spell() -> CardDefinition {
    CardDefinition {
        name: "EF11 Spell Single Target Test Spell".to_string(),
        card_id: CardId("test-ef11-spell-single-target-spell".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a 3-player state with `single_target_test_spell` in p1's hand, plus a
/// pre-existing stack object of the given `kind`/`target_count` (both a
/// `state.objects` entry in `ZoneId::Stack` and a matching `StackObject` entry).
///
/// PB-DX25b (`OOS-DX25-3`) non-vacuity repair: the `StackObject`'s own id
/// (`entry_id`) is a SEPARATE id from the announced card id (`other_id`) --
/// minted via `test_util::next_object_id`, the same monotone counter a real
/// cast draws from (`state/mod.rs::next_object_id`). Before this repair both
/// were the same value (`other_id` doubled as both the `state.objects` entry's
/// id AND the `StackObject`'s own id), which collapsed the two id spaces this
/// whole batch exists to keep apart and made every caller's cast-time lookup
/// pass regardless of whether `casting.rs` correctly resolved the announced
/// CARD id.
///
/// Returns `(state, test_spell_id, other_id, entry_id)` -- callers announce
/// `other_id` (the card a real player would target); `entry_id` is exposed for
/// callers that need to assert against the `StackObject`'s own id directly
/// (e.g. a `TargetsChanged.stack_object_id` observable).
fn build_base_state(
    other_kind_ability: bool,
    other_target_count: usize,
) -> (GameState, ObjectId, ObjectId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let spell_def = single_target_test_spell();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let test_spell = ObjectSpec::card(p1, "EF11 Spell Single Target Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone())
        .with_types(vec![CardType::Instant]);
    let other_stack_card = ObjectSpec::card(p2, "Other Stack Object").in_zone(ZoneId::Stack);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                ..ManaPool::default()
            },
        )
        .object(test_spell)
        .object(other_stack_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .expect("build_base_state: GameStateBuilder::build must succeed");

    let test_spell_id = find_obj(&state, "EF11 Spell Single Target Test Spell");
    let other_id = find_obj(&state, "Other Stack Object");

    // PB-DX25b non-vacuity: mint a DISTINCT id for the StackObject entry.
    let entry_id = test_util::next_object_id(&mut state);
    assert_ne!(
        entry_id, other_id,
        "PB-DX25b non-vacuity anchor: build_base_state must not collapse the \
         announced-card-id space and the stack-entry-id space onto one id"
    );

    let mut other_targets = Vec::new();
    for _ in 0..other_target_count {
        other_targets.push(SpellTarget {
            target: Target::Player(p3),
            zone_at_cast: None,
        });
    }
    let kind = if other_kind_ability {
        StackObjectKind::ActivatedAbility {
            source_object: other_id,
            ability_index: 0,
            embedded_effect: None,
        }
    } else {
        StackObjectKind::Spell {
            source_object: other_id,
        }
    };
    // Decoy fixtures (never reach `Effect::ChangeTargets`) get an empty list.
    let stack_entry = make_stack_object(entry_id, p2, kind, other_targets, vec![]);
    state.stack_objects_mut().push_back(stack_entry);

    (state, test_spell_id, other_id, entry_id)
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
}

// ── Test 1: accepts a single-target spell ──────────────────────────────────────

/// CR 115.7a/115.7b — a spell on the stack with exactly one declared target is a
/// legal target for `TargetSpellWithSingleTarget`.
#[test]
fn test_spell_single_target_accepts_single_target_spell() {
    let (state, test_spell_id, other_id, entry_id) = build_base_state(false, 1);
    let (state, _events) = cast_spell(state, p(1), test_spell_id, vec![Target::Object(other_id)])
        .unwrap_or_else(|e| {
            panic!(
                "casting at a single-target spell must succeed for TargetSpellWithSingleTarget: {:?}",
                e
            )
        });
    // CR 400.7: casting mints new ObjectIds (the card moves Hand->Stack with a fresh id,
    // and the StackObject itself gets another fresh id), so the original hand-card
    // `test_spell_id` is dead. The successful cast — hence a passing target validation —
    // is proven by the `unwrap_or_else(panic)` above. As a non-vacuous sanity check, a NEW
    // spell entry (one that is not the pre-placed `entry_id` StackObject) is now on
    // the stack: before the cast only `entry_id` was there.
    assert!(
        state
            .stack_objects()
            .iter()
            .any(|s| s.id != entry_id && matches!(s.kind, StackObjectKind::Spell { .. })),
        "a newly cast spell must be on the stack after a successful cast"
    );
}

// ── Test 2: DECOY — rejects a two-target spell (pinned on the count check) ────

/// DECOY, pinned on the `target_count != 1` guard. A spell with TWO declared
/// targets must be REJECTED even though it IS a `StackObjectKind::Spell` — this
/// isolates the count check from the kind check (Test 3 isolates the reverse).
/// Must fail if the count guard is removed.
#[test]
fn test_spell_single_target_rejects_two_target_spell() {
    let (state, test_spell_id, other_id, _entry_id) = build_base_state(false, 2);
    let result = cast_spell(state, p(1), test_spell_id, vec![Target::Object(other_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "DECOY: a spell with 2 targets must be rejected by TargetSpellWithSingleTarget \
         (target_count != 1 guard), got: {:?}",
        result.map(|_| ())
    );
}

// ── Test 3: DECOY — rejects an activated ability (NOT-FOUND path, post-PB-DX25b) ─

/// DECOY. An `ActivatedAbility` on the stack with exactly ONE declared target
/// must be REJECTED — this is the sole difference from
/// `TargetSpellOrAbilityWithSingleTarget`, which would accept it.
///
/// **PB-DX25b (`OOS-DX25-3`) correction — this no longer discriminates the
/// `is_spell` guard, once `build_base_state`'s ids are distinct (see that
/// function's doc).** With `other_id` (the announced card id) and `entry_id`
/// (the ActivatedAbility's own StackObject id) now separate,
/// `stack_index_for_announced_target(&state.stack_objects, other_id)` returns
/// `None` outright: `so.id == other_id` is false (the entry's real id is
/// `entry_id`), and `card_in_stack_zone` returns `None` for every
/// ActivatedAbility kind, so the card-owning-kind clause is also false. The
/// rejection observed here is therefore the LOOKUP returning `None`
/// (NOT-FOUND), not the `is_spell` guard rejecting a FOUND-but-wrong-kind
/// object — deleting the `is_spell` guard would NOT redden this test. The
/// `is_spell` guard's own precision test lives in `casting.rs`'s in-source
/// `#[cfg(test)] mod tests::test_target_spell_with_single_target_self_and_kind_
/// check`, sub-case (ii) (the deliberately-COLLAPSED-id configuration that is
/// the ONLY place in the tree the guard can still be reached with a FOUND
/// object) — see that test's doc for the full account, including sub-case
/// (iii), which is this exact NOT-FOUND shape pinned directly against the
/// private validator.
#[test]
fn test_spell_single_target_rejects_activated_ability() {
    let (state, test_spell_id, other_id, _entry_id) = build_base_state(true, 1);
    let result = cast_spell(state, p(1), test_spell_id, vec![Target::Object(other_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "DECOY: an ActivatedAbility with 1 target must be rejected by \
         TargetSpellWithSingleTarget (spell-only; NOT-FOUND path post-PB-DX25b, \
         see doc comment above), got: {:?}",
        result.map(|_| ())
    );
}

// ── Test 4: self-prevention ─────────────────────────────────────────────────────

/// CR 601.2a/601.2c/115.7a — a spell cannot legally declare itself as its own
/// `TargetSpellWithSingleTarget` target (PB-DX25b `OOS-DX25-3` §4.4 citation
/// correction: previously cited "CR 115.10", the unrelated affects-vs-targets
/// rule — see this file's module doc for the corrected grounding). At
/// cast-time-validation the casting spell is still in `ZoneId::Hand` (it has
/// not yet moved to the stack), so this is rejected by the same early-return
/// block (the zone check fires before the self_id-specific message would) —
/// the observable, user-facing behavior is the same either way: the cast is
/// illegal. The self_id-specific branch itself (message text) is
/// precision-pinned directly in `casting.rs`'s internal test module
/// (`validate_object_satisfies_requirement` is private to the engine crate and
/// not reachable from this external test).
#[test]
fn test_spell_single_target_self_prevention() {
    let (state, test_spell_id, _other_id, _entry_id) = build_base_state(false, 1);
    let result = cast_spell(
        state,
        p(1),
        test_spell_id,
        vec![Target::Object(test_spell_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "a spell cannot legally target itself for TargetSpellWithSingleTarget, got: {:?}",
        result.map(|_| ())
    );
}

// ── Test 5: hash discriminant + live schema sentinel ───────────────────────────

/// HASH_SCHEMA_VERSION live sentinel (54 -> 55) and hash-discriminant pin:
/// `TargetSpellWithSingleTarget` (discriminant 19) must hash distinctly from its
/// sibling `TargetSpellOrAbilityWithSingleTarget` (discriminant 16).
#[test]
fn test_spell_single_target_hash_discriminant() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    assert_eq!(
        HASH_SCHEMA_VERSION, 84u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this \
         assertion and the state/hash.rs history block together; the authoritative check \
         is the SR-17 machine gate in tests/core/hash_schema.rs."
    );

    let hash_req = |req: &TargetRequirement| -> [u8; 32] {
        let mut hasher = Hasher::new();
        req.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let spell_only = TargetRequirement::TargetSpellWithSingleTarget;
    let spell_or_ability = TargetRequirement::TargetSpellOrAbilityWithSingleTarget;

    assert_ne!(
        hash_req(&spell_only),
        hash_req(&spell_or_ability),
        "TargetSpellWithSingleTarget (disc 19) must hash distinctly from \
         TargetSpellOrAbilityWithSingleTarget (disc 16)"
    );
    assert_eq!(
        hash_req(&spell_only),
        hash_req(&spell_only),
        "identical TargetSpellWithSingleTarget requirements must hash identically \
         (sanity, non-vacuity check on the assertion above)"
    );
}

// ── Test 6: Misdirection integration ───────────────────────────────────────────

/// CR 115.7a/115.7b — Misdirection integration: a single-target spell on the
/// stack (targeting p3) is retargeted by Misdirection's `ChangeTargets` effect
/// to the effect controller (p1), mirroring the Bolt Bend integration test
/// pattern (`crates/engine/tests/rules/copy_redirect.rs`).
///
/// **PB-DX25b (`OOS-DX25-3`) rebuild (plan §5.2, `pb_ef11 … :372` row).** The
/// old version of this test announced the STACK-ENTRY id
/// (`bolt_id == bolt.id`, minted directly via `test_util::next_object_id`)
/// straight into `execute_effect`'s `ctx.targets` -- a path NO real cast can
/// ever produce, because the offer layer (`queries::legal_targets_per_slot`)
/// only ever enumerates `state.objects()`, and a stack-entry id is never a
/// member of that set (§1 fact 4/8 of the plan). It was green while testing a
/// fiction. This version places a real victim CARD object directly in
/// `ZoneId::Stack` via `ObjectSpec::card(..).in_zone(ZoneId::Stack)` (the exact
/// shape `casting.rs::handle_cast_spell` produces after `move_object_to_zone`),
/// gives its `StackObject` entry a SEPARATE, distinct id
/// (`victim_entry_id`), and announces the CARD id (`victim_card_id`) --
/// the id space a real Misdirection cast actually receives from
/// `Command::CastSpell.targets`.
#[test]
fn test_misdirection_retargets_single_target_spell() {
    let card = mtg_engine::cards::defs::misdirection::card();
    let AbilityDefinition::Spell {
        effect, targets, ..
    } = &card.abilities[1]
    else {
        panic!("expected Misdirection's second ability to be AbilityDefinition::Spell");
    };
    assert_eq!(
        targets,
        &vec![TargetRequirement::TargetSpellWithSingleTarget],
        "Misdirection's Spell ability must declare TargetSpellWithSingleTarget"
    );

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::card(p2, "PB-DX25b Victim").in_zone(ZoneId::Stack))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // p2 cast a single-target spell targeting p3. `victim_card_id` is the
    // `state.objects` entry -- the CARD id a real cast announces target ids
    // against.
    let victim_card_id = find_obj(&state, "PB-DX25b Victim");
    // PB-DX25b non-vacuity: the StackObject's own id is a SEPARATE mint.
    let victim_entry_id = test_util::next_object_id(&mut state);
    assert_ne!(
        victim_entry_id, victim_card_id,
        "PB-DX25b non-vacuity anchor: the fixture must not collapse the \
         announced-card-id space and the stack-entry-id space onto one id"
    );
    // PB-DX25c: the victim is a "target player" spell (a real single-target
    // burn-style spell), so it carries TargetPlayer -- without a real
    // requirement list, `plan_target_change` fails closed (§3.4) and no
    // redirect happens.
    let victim = make_stack_object(
        victim_entry_id,
        p2,
        StackObjectKind::Spell {
            source_object: victim_card_id,
        },
        vec![SpellTarget {
            target: Target::Player(p3),
            zone_at_cast: None,
        }],
        vec![TargetRequirement::TargetPlayer],
    );
    state.stack_objects_mut().push_back(victim);

    // p1 casts Misdirection targeting the victim's CARD id -- the id a real
    // `Command::CastSpell` would carry, per CR 601.2c.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(victim_card_id),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let events = execute_effect(&mut state, effect, &mut ctx);

    let victim = state
        .stack_objects()
        .iter()
        .find(|s| s.id == victim_entry_id)
        .expect("victim stack entry not found");
    assert_eq!(
        victim.targets[0].target,
        Target::Player(p1),
        "Misdirection should redirect the victim's target to its own controller (p1)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::TargetsChanged { stack_object_id, .. }
                if *stack_object_id == victim_entry_id
        )),
        "TargetsChanged event should be emitted naming the STACK-ENTRY id \
         (victim_entry_id), not the announced card id (victim_card_id) -- \
         GameEvent::TargetsChanged.stack_object_id's OWN doc comment \
         (rules/events.rs:1421-1422, \"The stack object whose targets \
         changed\") says so. No consumer reads this field today \
         (event_view.rs:927 destructures it and discards the field via \
         `..`), so this is a contract correction, not a compatibility fix \
         for an existing reader (PB-DX25b review Finding E5)."
    );
}

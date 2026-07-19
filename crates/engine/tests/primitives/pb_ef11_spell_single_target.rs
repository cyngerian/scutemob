//! Tests for PB-EF11 COMMIT 2: `TargetRequirement::TargetSpellWithSingleTarget`
//! (CR 115.7a/115.7b/115.10).
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
fn make_stack_object(
    id: ObjectId,
    controller: PlayerId,
    kind: StackObjectKind,
    targets: Vec<SpellTarget>,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind,
        targets,
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
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
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
/// Returns (state, test_spell_id, other_stack_object_id).
fn build_base_state(
    other_kind_ability: bool,
    other_target_count: usize,
) -> (GameState, ObjectId, ObjectId) {
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
    let stack_entry = make_stack_object(other_id, p2, kind, other_targets);
    state.stack_objects_mut().push_back(stack_entry);

    (state, test_spell_id, other_id)
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
    let (state, test_spell_id, other_id) = build_base_state(false, 1);
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
    // spell entry (one that is not the pre-placed `other_id` the spell targeted) is now on
    // the stack: before the cast only `other_id` was there.
    assert!(
        state
            .stack_objects()
            .iter()
            .any(|s| s.id != other_id && matches!(s.kind, StackObjectKind::Spell { .. })),
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
    let (state, test_spell_id, other_id) = build_base_state(false, 2);
    let result = cast_spell(state, p(1), test_spell_id, vec![Target::Object(other_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "DECOY: a spell with 2 targets must be rejected by TargetSpellWithSingleTarget \
         (target_count != 1 guard), got: {:?}",
        result.map(|_| ())
    );
}

// ── Test 3: DECOY — rejects an activated ability (pinned on the kind check) ───

/// DECOY, pinned on the `is_spell` guard. An `ActivatedAbility` on the stack
/// with exactly ONE declared target must be REJECTED — this is the sole
/// difference from `TargetSpellOrAbilityWithSingleTarget`, which would accept
/// it. Must fail if the kind guard is removed.
#[test]
fn test_spell_single_target_rejects_activated_ability() {
    let (state, test_spell_id, other_id) = build_base_state(true, 1);
    let result = cast_spell(state, p(1), test_spell_id, vec![Target::Object(other_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "DECOY: an ActivatedAbility with 1 target must be rejected by \
         TargetSpellWithSingleTarget (spell-only, is_spell guard), got: {:?}",
        result.map(|_| ())
    );
}

// ── Test 4: self-prevention ─────────────────────────────────────────────────────

/// CR 115.10 (ruling) — a spell cannot legally declare itself as its own
/// `TargetSpellWithSingleTarget` target. At cast-time-validation the casting
/// spell is still in `ZoneId::Hand` (it has not yet moved to the stack), so
/// this is rejected by the same early-return block (the zone check fires
/// before the self_id-specific message would) — the observable, user-facing
/// behavior is the same either way: the cast is illegal. The self_id-specific
/// branch itself (message text) is precision-pinned directly in
/// `casting.rs`'s internal test module (`validate_object_satisfies_requirement`
/// is private to the engine crate and not reachable from this external test).
#[test]
fn test_spell_single_target_self_prevention() {
    let (state, test_spell_id, _other_id) = build_base_state(false, 1);
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
        HASH_SCHEMA_VERSION, 59u8,
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
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // p2 cast a single-target spell targeting p3.
    let bolt_id = test_util::next_object_id(&mut state);
    let bolt_source = test_util::next_object_id(&mut state);
    let bolt = make_stack_object(
        bolt_id,
        p2,
        StackObjectKind::Spell {
            source_object: bolt_source,
        },
        vec![SpellTarget {
            target: Target::Player(p3),
            zone_at_cast: None,
        }],
    );
    state.stack_objects_mut().push_back(bolt);

    // p1 casts Misdirection targeting the bolt.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(bolt_id),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let events = execute_effect(&mut state, effect, &mut ctx);

    let bolt = state
        .stack_objects()
        .iter()
        .find(|s| s.id == bolt_id)
        .expect("bolt not found");
    assert_eq!(
        bolt.targets[0].target,
        Target::Player(p1),
        "Misdirection should redirect the bolt's target to its own controller (p1)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "TargetsChanged event should be emitted"
    );
}

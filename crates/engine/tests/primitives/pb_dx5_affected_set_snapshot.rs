//! PB-DX5 (OOS-OS7-2): CR **611.2c** — the set of objects a resolution-generated
//! continuous effect affects is determined when that effect begins and never
//! changes afterward, even though the engine keeps re-evaluating the effect's
//! `filter` on every characteristics calculation for LAYER purposes.
//!
//! `ContinuousEffect.affected_set: Option<OrdSet<ObjectId>>` — `Some(set)` for a
//! resolution-generated effect (`Effect::ApplyContinuousEffect`, computed by
//! `rules::layers::snapshot_affected_set` before the effect is pushed);
//! `None` for a static ability's effect (CR 611.3a — genuinely not locked in,
//! `filter` stays live). `rules::layers::effect_applies_to` answers `Some`
//! effects by membership alone.
//!
//! T11 (the candidate-scan-matches-brute-force gate on `candidate_ids_for_filter`)
//! is NOT in this file: `snapshot_affected_set`, `effect_applies_to_object` and
//! `candidate_ids_for_filter` are all `pub(crate)` in `rules::layers`, unreachable
//! from an integration test crate. It lives as an in-source `#[cfg(test)]` unit
//! test in `crates/engine/src/rules/layers.rs`
//! (`pb_dx5_snapshot_tests::snapshot_matches_brute_force_over_every_zone`),
//! immediately after `candidate_ids_for_filter`, mirroring the existing
//! `expect_characteristics_tests` precedent in that same file.
//!
//! Every "fails before the fix" claim below was OBSERVED, not reasoned to: the
//! read-site membership check (`rules/layers.rs::effect_applies_to`'s
//! `if let Some(ref affected) = effect.affected_set { return ... }` block) was
//! reverted, each test below run with its assertion still reading the CR-correct
//! (post-fix) expected value, and the actual reported `left` value recorded in
//! the test's doc comment; the change was then restored. Assertions that would
//! be indistinguishable pre/post fix are labelled non-discriminating rather than
//! given a synthetic pre-fix number.
//!
//! **The fix closes a second, larger pre-existing defect than the one it set
//! out to fix (fix-cycle Finding 1).** Every source-relative filter arm in
//! `effect_applies_to` (`CreaturesYouControl` and its family, `AttachedCreature`,
//! etc.) returns `false` unconditionally once `state.objects.get(&source_id)`
//! is `None`. For an instant or sorcery, `ctx.source` at execution time is the
//! spell's OWN stack card object, and `resolve_top_of_stack_inner`'s
//! `StackObjectKind::Spell` arm moves that object to the graveyard via
//! `state.move_object_to_zone` -- minting a fresh `ObjectId` per CR 400.7 --
//! *after* `execute_effect` has already run every effect in its `Sequence`.
//! Pre-PB-DX5 (no `affected_set` at all, every read live), this meant a mass
//! pump/debuff printed on an instant or sorcery applied to **nobody at all**
//! the instant the spell finished resolving, not merely "a newcomer wrongly
//! got it." **T12 is the only probe in this module that discriminates this
//! mechanism**: it is the only test that drives a mass filter through a real
//! `Command::CastSpell` (every other test calls `execute_effect` directly with
//! a battlefield permanent as `ctx.source`, which never retires). Reverting
//! the membership block and re-running T12 in isolation shows both board
//! creatures' locked P/T collapse from `Some(3)` (the intended +2/+2) to
//! `Some(1)` (their own base power, i.e. no pump applied to EITHER creature) --
//! confirming the "applies to nobody" mechanism, not a newcomer-only leak.
//!
//! **A second, independent divergence (fix-cycle Finding 2 / OOS-DX5-6, see T15
//! below).** `snapshot_affected_set` determines membership from FULLY
//! layer-resolved `chars` (`calculate_characteristics`), while the live
//! per-layer path evaluates the same predicate against `chars` that carry NO
//! Layer-4 modification at all -- not merely none from a *later-timestamped*
//! Layer-4 effect, which is what the original doc block and OOS-DX5-6 claimed
//! (and which was false: at Layer 4 the live gather sees zero Layer-4
//! modifications, earlier- or later-timestamped). T15 reproduces this with
//! Mirror Entity (the corpus's one Layer<=4 mass-filter `Complete` def) and an
//! animated Inkmoth Nexus (a Layer<=4 counterparty that writes the exact
//! characteristic -- `CardType::Creature` -- the filter reads).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::layers::is_effect_active;
use mtg_engine::state::test_util;
use mtg_engine::state::types::SubType;
use mtg_engine::{
    calculate_characteristics, check_and_apply_sbas, process_command, AbilityDefinition,
    CardDefinition, CardId, CardRegistry, CardType, CastSpellData, Command, ContinuousEffect,
    Effect, EffectAmount, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, LayerModification, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, TargetFilter, TypeLine, ZoneId, ZoneTarget,
    HASH_SCHEMA_VERSION,
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

fn power(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.power)
}

fn has_trample(state: &GameState, id: ObjectId) -> bool {
    calculate_characteristics(state, id)
        .map(|c| c.keywords.contains(&KeywordAbility::Trample))
        .unwrap_or(false)
}

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

/// Push a directly-constructed static (`affected_set: None`) `CreaturesYouControl`
/// +N/+N `WhileSourceOnBattlefield` anthem sourced by `source_id`, mirroring
/// `register_static_continuous_effects`'s output for a real card's static
/// ability (CR 611.3a).
fn push_static_anthem(state: &mut GameState, source_id: ObjectId, bonus: i32, effect_id: u64) {
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(effect_id),
        source: Some(source_id),
        timestamp: effect_id,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::ModifyBoth(bonus),
        is_cda: false,
        affected_set: None,
        condition: None,
    });
}

/// Directly move a permanent to a new controller via a Layer-2 `SetController`
/// continuous effect with `affected_set: None` (CR 611.3a — a control-change
/// effect is not itself a mass filter, but constructing it this way, rather
/// than through `Effect::GainControl`, keeps this file independent of that
/// effect's own resolution machinery). Mirrors
/// `pb_ef9_while_you_control_source.rs::borrow_via_source`.
fn set_control_directly(
    state: &mut GameState,
    object_id: ObjectId,
    new_controller: PlayerId,
    effect_id: u64,
) {
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(effect_id),
        source: None,
        timestamp: effect_id,
        layer: EffectLayer::Control,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(object_id),
        modification: LayerModification::SetController(new_controller),
        is_cda: false,
        affected_set: None,
        condition: None,
    });
    state
        .objects_mut()
        .get_mut(&object_id)
        .expect("object exists")
        .controller = new_controller;
}

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

// ── T1 — Golgari Charm mode 0: a mass -1/-1 does not reach a later creature ───

#[test]
/// CR 611.2c. Resolves Golgari Charm mode 0's exact encoding ("All creatures
/// get -1/-1 until end of turn", `EffectFilter::AllCreatures`,
/// `LayerModification::ModifyBoth(-1)`, Layer `PtModify`) against a board with
/// one creature already out, then a second creature enters afterward.
///
/// **Observed pre-fix** (membership check reverted, this exact assertion run):
/// `chars.power` for the late-entering creature was `Some(1)` — the same -1/-1
/// as the creature that was actually present when the effect resolved. Restored
/// after recording.
fn test_611_2c_mass_minus_one_does_not_reach_a_creature_that_enters_later() {
    let state0 = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Original Bear", 2, 2))
        .object(ObjectSpec::creature(p(1), "Late Arrival Bear", 2, 2).in_zone(ZoneId::Hand(p(1))))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let mut state = state0;
    let source = find_object(&state, "Original Bear");
    let late_hand_id = find_object(&state, "Late Arrival Bear");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            filter: EffectFilter::AllCreatures,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let (new_id, _old) =
        test_util::move_object_to_zone(&mut state, late_hand_id, ZoneId::Battlefield).unwrap();

    assert_eq!(
        power(&state, source),
        Some(1),
        "CR 611.2c: the creature present when the effect resolved is in the locked set"
    );
    assert_eq!(
        power(&state, new_id),
        Some(2),
        "CR 611.2c: a creature entering the battlefield after the effect resolved is not \
         in the locked set and keeps its own printed power"
    );
}

// ── T2 — Craterhoof Behemoth: two-part effect, each part's set locked independently ─

#[test]
/// CR 611.2c's last sentence: "If a single continuous effect has parts that
/// modify the characteristics ... and other parts that don't, the set of
/// objects each part applies to is determined independently." Craterhoof's ETB
/// is TWO `ApplyContinuousEffect`s (`ModifyBothDynamic` P/T, `AddKeywords`
/// trample), both `CreaturesYouControl`, both locked at the SAME resolution.
/// Ruling 2025-04-04: "Creatures you begin to control later in the turn won't
/// gain trample and get +X/+X."
///
/// **Observed pre-fix**: the late-entering creature had `power == Some(5)`
/// (its own 2 base + the locked X=3) and `has_trample(..) == true`. Restored
/// after recording.
fn test_611_2c_craterhoof_does_not_pump_a_creature_that_enters_later() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Craterhoof Behemoth", 5, 5))
        .object(ObjectSpec::creature(p(1), "Bear A", 2, 2))
        .object(ObjectSpec::creature(p(1), "Bear B", 2, 2))
        .object(ObjectSpec::creature(p(1), "Late Bear", 2, 2).in_zone(ZoneId::Hand(p(1))))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Craterhoof Behemoth");
    let bear_a = find_object(&state, "Bear A");
    let late_hand_id = find_object(&state, "Late Bear");

    let pump = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBothDynamic {
                amount: Box::new(EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                }),
                negate: false,
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let trample = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::AddKeywords(
                [KeywordAbility::Trample].into_iter().collect(),
            ),
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source, vec![]);
    execute_effect(&mut state, &Effect::Sequence(vec![pump, trample]), &mut ctx);

    // X = 3 creatures controlled at resolution (Craterhoof + Bear A + Bear B).
    assert_eq!(power(&state, source), Some(8), "5 base + X=3");
    assert_eq!(power(&state, bear_a), Some(5), "2 base + X=3");
    assert!(has_trample(&state, source));
    assert!(has_trample(&state, bear_a));

    let (new_id, _old) =
        test_util::move_object_to_zone(&mut state, late_hand_id, ZoneId::Battlefield).unwrap();

    assert_eq!(
        power(&state, new_id),
        Some(2),
        "CR 611.2c + ruling 2025-04-04: a creature entering after Craterhoof's trigger \
         resolved keeps its own printed power, not the locked X pump"
    );
    assert!(
        !has_trample(&state, new_id),
        "CR 611.2c: the late-entering creature does not gain the locked trample grant"
    );
}

// ── T3 — a locked bonus survives a control change ──────────────────────────────

#[test]
/// CR 611.2c: the locked set is membership-only and does not re-derive from
/// `CreaturesYouControl`'s live filter, so a control change after resolution
/// must NOT remove the bonus from the object that has it, and must NOT extend
/// it to an object the new controller already had.
///
/// **Observed pre-fix**: the moved creature's power reverted to `Some(2)` (the
/// live `CreaturesYouControl` filter stopped matching once its controller was
/// no longer p1) instead of staying at the locked `Some(3)`. Restored after
/// recording.
fn test_611_2c_pumped_creature_keeps_the_bonus_after_a_control_change() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // The effect's SOURCE stays a separate, never-moved permanent, so
        // moving the buffed creature does not incidentally also move the
        // source's controller -- if it did, the live `CreaturesYouControl`
        // filter would still match after the move by coincidence (both
        // "source controller" and "object controller" having moved together)
        // and this test would not exercise CR 611.2c at all. Caught by
        // observing exactly that coincidence with the read-site membership
        // check reverted (see the module doc comment).
        .object(ObjectSpec::creature(p(1), "Anthem Source", 1, 1))
        .object(ObjectSpec::creature(p(1), "Original Bear", 2, 2))
        .object(ObjectSpec::creature(p(2), "P2's Own Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Anthem Source");
    let bear = find_object(&state, "Original Bear");
    let p2_bear = find_object(&state, "P2's Own Bear");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(1),
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    assert_eq!(
        power(&state, bear),
        Some(3),
        "baseline: +1/+1 applied at resolution"
    );

    set_control_directly(&mut state, bear, p(2), 9001);

    assert_eq!(
        power(&state, bear),
        Some(3),
        "CR 611.2c: the moved creature is still in the locked set and keeps its +1/+1 \
         even though the effect's source (which stayed with player 1) no longer shares \
         a controller with it"
    );
    assert_eq!(
        power(&state, p2_bear),
        Some(2),
        "the creature player 2 already had is not in the locked set and does not gain \
         the bonus just because the effect's controller-relative filter would now \
         (wrongly) include the moved creature's new teammate"
    );
}

// ── T4a/T4b — the static-ability control group: live re-evaluation is correct ──

#[test]
/// CR 611.3a: a STATIC ability's continuous effect ("isn't 'locked in'; it
/// applies at any given moment to whatever its text indicates") is deliberately
/// unaffected by PB-DX5 — `affected_set: None` at every static-registration
/// site. Regression guard, not a "fails before" probe: this must pass
/// identically before and after the fix.
fn test_611_3a_static_anthem_reaches_a_creature_that_enters_later() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Anthem Source", 1, 1))
        .object(ObjectSpec::creature(p(1), "Original Bear", 2, 2))
        .object(ObjectSpec::creature(p(1), "Late Bear", 2, 2).in_zone(ZoneId::Hand(p(1))))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Anthem Source");
    push_static_anthem(&mut state, source, 1, 9101);

    let original_bear = find_object(&state, "Original Bear");
    assert_eq!(power(&state, original_bear), Some(3), "baseline +1/+1");

    let late_hand_id = find_object(&state, "Late Bear");
    let (new_id, _old) =
        test_util::move_object_to_zone(&mut state, late_hand_id, ZoneId::Battlefield).unwrap();

    assert_eq!(
        power(&state, new_id),
        Some(3),
        "CR 611.3a: a static anthem is not locked in and reaches a creature that enters \
         the battlefield later"
    );
}

#[test]
/// CR 611.3a, the inverse of T3: a STATIC anthem's live filter means a creature
/// that stops being controlled by the anthem source's controller loses the
/// bonus — exactly what T3 proves must NOT happen for a locked resolution
/// effect. Regression guard.
fn test_611_3a_static_anthem_stops_applying_after_a_control_change() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Anthem Source", 1, 1))
        .object(ObjectSpec::creature(p(1), "Original Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Anthem Source");
    push_static_anthem(&mut state, source, 1, 9102);

    let bear = find_object(&state, "Original Bear");
    assert_eq!(power(&state, bear), Some(3), "baseline +1/+1");

    set_control_directly(&mut state, bear, p(2), 9103);

    assert_eq!(
        power(&state, bear),
        Some(2),
        "CR 611.3a: the static anthem's live CreaturesYouControl filter stops matching \
         once the creature is no longer controlled by the anthem source's controller"
    );
}

// ── T5 — Umezawa's Jitte: the bonus stays with the creature equipped at resolution ─

#[test]
/// CR 611.2c + Jitte ruling 2005-02-01 #3: "If the Jitte is moved after the
/// '+2/+2' mode is announced but before it resolves, the bonus is given to the
/// creature that is equipped WHEN THE ABILITY RESOLVES." Reproduces Jitte mode
/// 0's exact encoding (`EffectFilter::AttachedCreature`,
/// `LayerModification::ModifyBoth(2)`, Layer `PtModify`).
///
/// **Observed pre-fix**: after re-equipping to B, `power(A) == Some(1)` (the
/// live `AttachedCreature` filter stopped matching A) and `power(B) == Some(3)`
/// (the filter wrongly started matching B). Restored after recording.
fn test_611_2c_jitte_bonus_stays_with_the_creature_equipped_at_resolution() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Umezawa's Jitte")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p(1), "Creature A", 1, 1))
        .object(ObjectSpec::creature(p(1), "Creature B", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let jitte = find_object(&state, "Umezawa's Jitte");
    let creature_a = find_object(&state, "Creature A");
    let creature_b = find_object(&state, "Creature B");

    state.objects_mut().get_mut(&jitte).unwrap().attached_to = Some(creature_a);

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(2),
            filter: EffectFilter::AttachedCreature,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), jitte, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    assert_eq!(
        power(&state, creature_a),
        Some(3),
        "baseline: +2/+2 on the equipped creature"
    );

    // Re-attach the Jitte to Creature B AFTER the ability already resolved.
    state.objects_mut().get_mut(&jitte).unwrap().attached_to = Some(creature_b);

    assert_eq!(
        power(&state, creature_a),
        Some(3),
        "CR 611.2c / Jitte ruling 2005-02-01 #3: Creature A was equipped when the ability \
         resolved and keeps the +2/+2 even after the Jitte moves"
    );
    assert_eq!(
        power(&state, creature_b),
        Some(1),
        "CR 611.2c: Creature B was not equipped when the ability resolved and does not \
         gain the bonus just by later becoming the equipped creature"
    );
}

// ── T6 — SingleObject-derived filters are behaviourally unchanged ──────────────

#[test]
/// Falsifier for the "79 SingleObject-derived defs are unchanged" prediction
/// (`memory/primitive-wip.md`). `Effect::ApplyContinuousEffect` resolves
/// `EffectFilter::DeclaredTarget { index }` into `EffectFilter::SingleObject(id)`
/// BEFORE `snapshot_affected_set` runs, so the stored `affected_set` is exactly
/// `{id}` -- the fast path in `snapshot_affected_set` returns it without a scan.
fn test_611_2c_single_object_filters_are_behaviourally_unchanged() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Target Creature", 2, 2))
        .object(ObjectSpec::creature(p(1), "Other Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Target Creature");
    let target = source;
    let other = find_object(&state, "Other Creature");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::AddKeyword(KeywordAbility::Trample),
            filter: EffectFilter::DeclaredTarget { index: 0 },
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(
        p(1),
        source,
        vec![mtg_engine::SpellTarget {
            target: mtg_engine::Target::Object(target),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    execute_effect(&mut state, &effect, &mut ctx);

    let stored = state
        .continuous_effects()
        .iter()
        .find(|e| {
            matches!(
                e.modification,
                LayerModification::AddKeyword(KeywordAbility::Trample)
            )
        })
        .expect("the effect was pushed");
    assert_eq!(
        stored.affected_set,
        Some(imbl::OrdSet::unit(target)),
        "a DeclaredTarget-derived resolution effect locks to exactly {{target}}, \
         identical to what live SingleObject id-equality already produced"
    );
    assert!(calculate_characteristics(&state, target)
        .unwrap()
        .keywords
        .contains(&KeywordAbility::Trample));
    assert!(!calculate_characteristics(&state, other)
        .unwrap()
        .keywords
        .contains(&KeywordAbility::Trample));
}

// ── T7/T8 — CR 702.26e/702.26b: phased-out permanents ───────────────────────────

#[test]
/// CR 702.26e: a phased-out permanent is excluded from the affected set at
/// DETERMINATION time, and stays excluded permanently -- phasing it back in
/// does not retroactively add it.
///
/// **Observed pre-fix** (membership check reverted, this exact assertion run):
/// the final `power(&state, phased)` was `Some(1)`, not `Some(2)` -- the live
/// `AllCreatures` filter re-matched the permanent the instant it phased back
/// in and it retroactively took the -1/-1, exactly the CR 702.26e violation
/// this test guards against. Restored after recording.
fn test_702_26e_phased_out_permanent_is_excluded_from_the_set() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Visible Bear", 2, 2))
        .object(ObjectSpec::creature(p(1), "Phased Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Visible Bear");
    let phased = find_object(&state, "Phased Bear");
    {
        let obj = state.objects_mut().get_mut(&phased).unwrap();
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p(1));
    }

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            filter: EffectFilter::AllCreatures,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let stored = state
        .continuous_effects()
        .iter()
        .find(|e| e.filter == EffectFilter::AllCreatures)
        .expect("the effect was pushed");
    assert!(
        !stored.affected_set.as_ref().unwrap().contains(&phased),
        "CR 702.26e: a phased-out permanent is not in the affected set determined while \
         it is phased out"
    );

    // Phase it back in -- CR 611.2c says the set never changes, so it must NOT
    // retroactively gain the -1/-1.
    state
        .objects_mut()
        .get_mut(&phased)
        .unwrap()
        .status
        .phased_out = false;
    assert_eq!(
        power(&state, phased),
        Some(2),
        "CR 702.26e/611.2c: a permanent excluded from the set at determination time \
         does not gain the effect just by phasing back in"
    );
}

#[test]
/// CR 702.26b/702.26f: a permanent that WAS in the locked set is unaffected
/// while phased out, and resumes being affected once it phases back in (the
/// effect has not expired and its membership is unchanged -- only its
/// applicability while phased out is suppressed).
fn test_702_26b_locked_permanent_is_unaffected_while_phased_out_and_resumes_on_phase_in() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let bear = find_object(&state, "Bear");
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            filter: EffectFilter::AllCreatures,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), bear, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    assert_eq!(
        power(&state, bear),
        Some(1),
        "baseline -1/-1, still on the set"
    );

    {
        let obj = state.objects_mut().get_mut(&bear).unwrap();
        obj.status.phased_out = true;
        obj.phased_out_controller = Some(p(1));
    }
    // While phased out, CR 702.26b: "can't be affected by anything else in the
    // game." calculate_characteristics still resolves it (phasing is not a
    // zone change), but the effect must not apply.
    assert_eq!(
        power(&state, bear),
        Some(2),
        "CR 702.26b: a phased-out permanent is unaffected by the locked effect even \
         though it is still in the affected set"
    );

    state
        .objects_mut()
        .get_mut(&bear)
        .unwrap()
        .status
        .phased_out = false;
    assert_eq!(
        power(&state, bear),
        Some(1),
        "CR 702.26f: the effect resumes applying once the permanent phases back in \
         (it never expired -- it was suppressed, not removed)"
    );
}

// ── T9 — CR 400.7: a locked creature that leaves and returns is a new object ────

#[test]
/// CR 400.7: an object that changes zones becomes a NEW object with a new
/// `ObjectId`. A creature in the locked set that dies and is reanimated is a
/// different id and is therefore automatically excluded -- no special-casing
/// needed at the read site.
fn test_400_7_a_locked_creature_that_leaves_and_returns_is_a_new_object() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let bear = find_object(&state, "Bear");
    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(1),
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), bear, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    assert_eq!(power(&state, bear), Some(3), "baseline +1/+1");

    let (dead_id, _old) =
        test_util::move_object_to_zone(&mut state, bear, ZoneId::Graveyard(p(1))).unwrap();
    let (reanimated_id, _old) =
        test_util::move_object_to_zone(&mut state, dead_id, ZoneId::Battlefield).unwrap();

    assert_ne!(
        reanimated_id, bear,
        "CR 400.7: two zone changes, two new ids"
    );
    assert_eq!(
        power(&state, reanimated_id),
        Some(2),
        "the reanimated permanent is a NEW object, absent from the locked affected_set, \
         so it does not carry the +1/+1 forward"
    );
}

// ── T10 — end-to-end SBA consequence, not just a characteristics read ───────────

#[test]
/// CR 611.2c + CR 704.5f: after a mass -1/-1, a 0-toughness creature that WAS
/// present dies to the SBA sweep; a creature that entered afterward is
/// unaffected and survives.
///
/// **Observed pre-fix**: `is_on_battlefield(&state, "Late Arrival")` was
/// `false` -- the late arrival also took -1/-1 (toughness 0) and died to the
/// same SBA sweep. Restored after recording.
fn test_611_2c_sba_after_a_mass_debuff_kills_only_the_locked_creatures() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Frail Bear", 1, 1))
        .object(ObjectSpec::creature(p(1), "Late Arrival", 1, 1).in_zone(ZoneId::Hand(p(1))))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let source = find_object(&state, "Frail Bear");
    let late_hand_id = find_object(&state, "Late Arrival");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            filter: EffectFilter::AllCreatures,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    test_util::move_object_to_zone(&mut state, late_hand_id, ZoneId::Battlefield).unwrap();
    check_and_apply_sbas(&mut state);

    assert!(
        !is_on_battlefield(&state, "Frail Bear"),
        "CR 704.5f: the creature present at resolution took -1/-1 (0 toughness) and died"
    );
    assert!(
        is_on_battlefield(&state, "Late Arrival"),
        "CR 611.2c: the creature that entered after resolution is not in the locked set, \
         keeps its printed toughness, and survives the same SBA sweep"
    );
}

// ── T12 — survives the PB-DP9 abort-and-replay (CR 608.2d) ─────────────────────

fn library_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).in_zone(ZoneId::Library(owner))
}

#[test]
/// CR 608.2d + 611.2c: a resolution whose effect list contains BOTH an
/// `ApplyContinuousEffect` (before) and a search that suspends (after) must
/// produce the same locked set on the successful replay as an otherwise
/// identical resolution that never suspends. PB-DP9's abort-and-replay clones
/// state at entry and restores it WHOLESALE on suspension, so the first
/// (discarded) `ApplyContinuousEffect` execution's mutation never persists --
/// only the replay's does, and it is `snapshot_affected_set` applied to the
/// SAME restored board, so it lands on the same set deterministically.
///
/// **This is also the module's only probe of fix-cycle Finding 1's second,
/// larger defect**, and the only reason it discriminates it: this is the one
/// test that drives the pump through a real `Command::CastSpell`, so
/// `ctx.source` for the `CreaturesYouControl` effect is the SPELL's own stack
/// card object, which `resolve_top_of_stack_inner`'s `StackObjectKind::Spell`
/// arm retires to the graveyard (a fresh `ObjectId`, CR 400.7) once
/// resolution finishes. Every other test in this module calls `execute_effect`
/// directly with a battlefield permanent as `ctx.source`, which never retires
/// and so cannot see this.
///
/// **Observed pre-fix** (membership check reverted, this exact assertion run
/// against the finished resolution): `power(&state, bear_a) == Some(1)` and
/// `power(&state, bear_b) == Some(1)` -- their own base power, i.e. the +2/+2
/// pump applied to NEITHER board creature, not just to a hypothetical
/// newcomer. Pre-PB-DX5 (no `affected_set`, filter re-evaluated live at every
/// characteristics calculation), `CreaturesYouControl`'s source-relative check
/// (`state.objects.get(&source_id)`) returned `None` for every object once the
/// spell card had left the stack, so the whole "creatures you control get
/// +X/+X" spell class silently applied to nobody the moment it resolved.
/// Restored after recording.
fn test_611_2c_snapshot_survives_the_pb_dp9_abort_and_replay() {
    let def = CardDefinition {
        name: "DX5 Pump Then Search".to_string(),
        card_id: CardId("dx5-pump-then-search".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Sorcery],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(2),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(ObjectSpec::creature(p(1), "Board Bear A", 1, 1))
        .object(ObjectSpec::creature(p(1), "Board Bear B", 1, 1))
        // Two library candidates -- CR 701.23d's "exactly one candidate is
        // determined and asks nothing" carve-out must NOT apply here, or the
        // search never suspends and the replay machinery is untested.
        .object(library_creature(p(1), "Library Cat"))
        .object(library_creature(p(1), "Library Dog"));
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "DX5 Pump Then Search");
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p(1),
            card: spell_id,
            targets: vec![],
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
    .expect("cast should succeed");
    let (state, _) = pass_all(state, &[p(1), p(2)]);

    assert!(
        state.pending_effect_choice().is_some(),
        "the search must genuinely suspend (2 library candidates) -- otherwise this test \
         exercises nothing PB-DP9-specific"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-DP9 abort-and-replay: the whole resolution -- including the \
         ApplyContinuousEffect mutation that ran before the suspension -- was restored \
         wholesale, so the spell is still on the stack"
    );

    let entry = state
        .pending_effect_choice()
        .expect("checked above")
        .clone();
    let answer = mtg_engine::effects::default_effect_choice_answer(&entry.question);
    let (state, _) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: entry.player,
            choice_id: entry.choice_id,
            answer,
        },
    )
    .expect("the engine must accept its own default answer");

    let bear_a = find_object(&state, "Board Bear A");
    let bear_b = find_object(&state, "Board Bear B");
    assert_eq!(
        power(&state, bear_a),
        Some(3),
        "post-replay: the ApplyContinuousEffect's locked set still contains both board \
         creatures -- the suspend-and-replay did not lose or duplicate it"
    );
    assert_eq!(power(&state, bear_b), Some(3));

    // Compare against an identical, never-suspending resolution of the SAME
    // effect on the SAME starting board shape.
    let mut baseline = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Board Bear A", 1, 1))
        .object(ObjectSpec::creature(p(1), "Board Bear B", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();
    let baseline_source = find_object(&baseline, "Board Bear A");
    let pump_only = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(2),
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), baseline_source, vec![]);
    execute_effect(&mut baseline, &pump_only, &mut ctx);
    let baseline_a = find_object(&baseline, "Board Bear A");
    let baseline_b = find_object(&baseline, "Board Bear B");
    assert_eq!(
        power(&state, bear_a),
        power(&baseline, baseline_a),
        "the suspend-and-replay path and a non-suspending direct resolution of the same \
         effect on the same board produce the same locked outcome"
    );
    assert_eq!(power(&state, bear_b), power(&baseline, baseline_b));
}

// ── T13 — `is_effect_active` is unaffected by `affected_set` ───────────────────

#[test]
/// CR 611.2a/611.2b: `is_effect_active` answers "is this effect running at
/// all?" (duration + condition) and has no `object_id` parameter, so it cannot
/// and does not consult `affected_set`. Two negative claims: an expired effect
/// with a non-empty `affected_set` stays inactive; a live effect with an EMPTY
/// `affected_set` stays active (CR 611.2b's "does nothing" describes an
/// outcome, not non-existence -- an empty-set shortcut in `is_effect_active`
/// would wrongly skip expiry/control-reversion bookkeeping for it).
fn test_is_effect_active_is_unchanged_by_the_snapshot() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();
    let bear = find_object(&state, "Bear");

    // "Expired": WhileSourceOnBattlefield whose source does not exist.
    let expired = ContinuousEffect {
        id: EffectId(9301),
        source: Some(ObjectId(999_999)),
        timestamp: 9301,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        affected_set: Some(imbl::OrdSet::unit(bear)),
        condition: None,
    };
    assert!(
        !is_effect_active(&state, &expired),
        "a non-empty affected_set does not resurrect an effect whose duration has expired"
    );

    // "Live": UntilEndOfTurn (always active per is_effect_active), empty set.
    let live_but_empty = ContinuousEffect {
        id: EffectId(9302),
        source: None,
        timestamp: 9302,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyBoth(-1),
        is_cda: false,
        affected_set: Some(imbl::OrdSet::new()),
        condition: None,
    };
    assert!(
        is_effect_active(&state, &live_but_empty),
        "an empty affected_set does not deactivate an otherwise-live effect -- CR \
         611.2b's 'does nothing' is a statement about outcome, not existence"
    );

    let _ = state.objects().get(&bear);
}

// ── T15 — review Finding 2 (OOS-DX5-6): a Layer<=4 mass filter reaches a ────────
// ── Layer<=4 counterparty (full-resolution `chars`, not partial)          ───────

#[test]
/// CR 611.2c + OOS-DX5-6. `snapshot_affected_set` calls
/// `calculate_characteristics`, which returns FULLY layer-resolved
/// characteristics -- while the live per-layer path inside
/// `calculate_characteristics` evaluates the very same predicate against
/// `chars` that carry NO Layer-4 modification at all, because that function
/// gathers every Layer-4 effect before applying any of them. The divergence
/// is therefore not scoped to a "later-timestamped" Layer-4 effect (the
/// shipped doc block's and OOS-DX5-6's original, and wrong, qualifier) -- at
/// Layer 4 the live gather sees ZERO Layer-4 modifications, earlier- or
/// later-timestamped. Mirror Entity's `{X}: creatures you control ... gain
/// all creature types` (`AddAllCreatureTypes`, `EffectFilter::CreaturesYouControl`,
/// `EffectLayer::TypeChange`) is the corpus's one Layer<=4 mass-filter
/// `Complete` def, and an animated Inkmoth Nexus is a Layer<=4 counterparty
/// that WRITES the exact characteristic (`CardType::Creature`) the filter
/// reads -- not a "mass-filter def" itself, which is the population
/// OOS-DX5-6 originally (and wrongly) checked.
///
/// **Observed pre-fix** (membership check reverted -- i.e. the live,
/// mid-Layer-4-gather filter re-run, matching pre-PB-DX5 behaviour exactly):
/// the animated Nexus's `chars.subtypes` did NOT contain any creature type
/// granted by `AddAllCreatureTypes` (`contains(&SubType("Human"))` was
/// `false`) -- Nexus was still evaluated as a bare Land against Mirror
/// Entity's `CreaturesYouControl` filter, because at Layer 4 `chars` carries
/// none of Layer 4's own modifications yet, including Nexus's own animate
/// effect. Restored after recording. Post-fix, the snapshot's fully-resolved
/// `chars` sees Nexus as a creature at determination time, locks it into the
/// affected set, and it keeps every creature type for the rest of the
/// duration -- CR 611.2c-correct: "the set of objects it affects is
/// determined when that continuous effect begins," i.e. with all
/// *pre-existing* continuous effects (including Nexus's own animate, which
/// ran first) already applied.
fn test_611_2c_snapshot_uses_full_resolution_a_layer_le4_mass_filter_reaches_a_layer_le4_counterparty(
) {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::land(p(1), "Inkmoth Nexus"))
        .object(ObjectSpec::creature(p(1), "Mirror Entity", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let nexus = find_object(&state, "Inkmoth Nexus");
    let mirror_entity = find_object(&state, "Mirror Entity");

    // Inkmoth Nexus's own animate ability, `inkmoth_nexus.rs`'s exact
    // encoding for the two Layer<=4 parts (`EffectFilter::Source`, so it
    // locks to `{nexus}` unconditionally regardless of this batch -- the
    // point of this probe is what it does to MIRROR ENTITY's filter reading
    // the result, not to its own filter).
    let animate = Effect::Sequence(vec![
        Effect::ApplyContinuousEffect {
            effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
                layer: EffectLayer::TypeChange,
                modification: LayerModification::AddCardTypes(
                    [CardType::Artifact, CardType::Creature]
                        .into_iter()
                        .collect(),
                ),
                filter: EffectFilter::Source,
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            }),
        },
        Effect::ApplyContinuousEffect {
            effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
                layer: EffectLayer::PtSet,
                modification: LayerModification::SetPowerToughness {
                    power: 1,
                    toughness: 1,
                },
                filter: EffectFilter::Source,
                duration: EffectDuration::UntilEndOfTurn,
                condition: None,
            }),
        },
    ]);
    let mut nexus_ctx = EffectContext::new(p(1), nexus, vec![]);
    execute_effect(&mut state, &animate, &mut nexus_ctx);

    // Mirror Entity's `{X}: ... gain all creature types` half only --
    // `mirror_entity.rs`'s exact encoding (`AddAllCreatureTypes`,
    // `CreaturesYouControl`, `EffectLayer::TypeChange`).
    let grant_types = Effect::ApplyContinuousEffect {
        effect_def: Box::new(mtg_engine::CardContinuousEffectDef {
            layer: EffectLayer::TypeChange,
            modification: LayerModification::AddAllCreatureTypes,
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut mirror_ctx = EffectContext::new(p(1), mirror_entity, vec![]);
    execute_effect(&mut state, &grant_types, &mut mirror_ctx);

    let nexus_chars = calculate_characteristics(&state, nexus).expect("nexus exists");
    assert!(
        nexus_chars.card_types.contains(&CardType::Creature),
        "the animate ability's own Layer-4 type change is unconditional (EffectFilter::Source \
         locks to {{nexus}} regardless of the CR 611.2c fix) -- Nexus is a creature"
    );
    assert!(
        nexus_chars.subtypes.contains(&SubType("Human".to_string())),
        "CR 611.2c: snapshot_affected_set determines Mirror Entity's CreaturesYouControl set \
         from FULLY layer-resolved characteristics, so the animated Nexus is seen as a \
         creature at determination time and is locked into the affected set -- it then keeps \
         every creature type AddAllCreatureTypes grants for the rest of the duration"
    );
}

// ── T14 — wire-version sentinel ─────────────────────────────────────────────────

#[test]
/// PB-DX5: `ContinuousEffect.affected_set` (CR 611.2c) bumps HASH_SCHEMA_VERSION
/// 69 -> 70. `PROTOCOL_VERSION` is unmoved (confirmed by
/// `cargo test -p mtg-engine --test core protocol_schema`, not assumed --
/// `ContinuousEffect` is not in the SR-8 wire closure).
fn test_dx5_hash_schema_version_is_70() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 70u8,
        "HASH_SCHEMA_VERSION live sentinel -- PB-DX5, CR 611.2c"
    );
}

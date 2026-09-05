//! PB-DX20 (Aura half) — the offer layer cannot see a keyword-carried target
//! requirement (`OOS-CARDS2-4`, HIGH).
//!
//! `memory/primitives/pb-plan-DX20.md` is authoritative. This file covers the plan's
//! §6 probes T1-T4 for the two new `casting.rs` helpers,
//! `enchant_target_to_requirement` and `aura_spell_target_requirements`, and the
//! `queries::spell_target_requirements` call site that now shares them.
//!
//! ## What was wrong
//!
//! An Aura's target restriction (CR 303.4a) was derived from `KeywordAbility::Enchant`
//! only inside `handle_cast_spell`'s own ad-hoc gate — `queries::spell_target_
//! requirements` (the function every UI/simulator caller uses to populate a target
//! picker) read `card_def_target_requirements` alone, which has no Aura-awareness at
//! all, so every Aura was offered with `(min, max) == (0, 0)`. A browser could never
//! render a picker for a Rancor, and a bot could never announce an Aura target.
//!
//! ## The fix
//!
//! One TOTAL mapping, `enchant_target_to_requirement: &EnchantTarget ->
//! TargetRequirement`, plus `aura_spell_target_requirements` which applies it when the
//! spell is an Aura. Both `handle_cast_spell` and `queries::spell_target_requirements`
//! call the SAME function — they cannot drift by construction, and T1 proves the
//! equivalence by EXECUTION rather than by argument.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, legal_targets_per_slot, process_command,
    spell_target_requirements, target_count_range, AbilityDefinition, AltCostKind, CardDefinition,
    CardId, CardRegistry, CardType, Command, Completeness, EnchantControllerConstraint,
    EnchantFilter, EnchantTarget, GameStateBuilder, GameStateError, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, SuperType, Target, TargetRequirement, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

/// Build an Aura ObjectSpec in hand, with the given EnchantTarget keyword.
/// Mirrors `mechanics_e_l/enchant.rs::aura_in_hand`.
fn aura_in_hand(owner: PlayerId, name: &str, enchant: EnchantTarget) -> ObjectSpec {
    ObjectSpec::enchantment(owner, name)
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(enchant))
        .in_zone(ZoneId::Hand(owner))
}

/// A minimal `CastSpellData` naming exactly `targets` for `card`, cast by `player`.
fn cast(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
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
    }))
}

/// Every `AbilityDefinition::Keyword(KeywordAbility::Enchant(_))` on a def, in
/// oracle-text order. Used by T4's roster gates — enumerated from `all_cards()`
/// (SR-36), never grepped from source.
fn enchant_keywords(def: &CardDefinition) -> Vec<EnchantTarget> {
    def.abilities
        .iter()
        .filter_map(|a| {
            if let AbilityDefinition::Keyword(KeywordAbility::Enchant(et)) = a {
                Some(et.clone())
            } else {
                None
            }
        })
        .collect()
}

fn is_aura_def(def: &CardDefinition) -> bool {
    def.types.subtypes.contains(&SubType("Aura".to_string()))
}

/// Build a fixed 10-candidate board (CR-legal Aura targets and non-targets) plus one
/// Aura in hand carrying `enchant`. Candidate names are fixed so every variant's test
/// can look them up by name. p1 is always the caster.
fn build_board_with_aura(enchant: EnchantTarget) -> (mtg_engine::GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);

    let aura = aura_in_hand(p1, "Test Aura", enchant);

    let own_creature = ObjectSpec::creature(p1, "Own Creature", 2, 2);
    let opp_creature = ObjectSpec::creature(p2, "Opp Creature", 2, 2);
    let own_basic_mountain = ObjectSpec::card(p1, "Own Mountain")
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Mountain".to_string())])
        .with_supertypes(vec![SuperType::Basic])
        .in_zone(ZoneId::Battlefield);
    let own_nonbasic_land = ObjectSpec::card(p1, "Own Karoo")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);
    let opp_mountain = ObjectSpec::card(p2, "Opp Mountain")
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Mountain".to_string())])
        .with_supertypes(vec![SuperType::Basic])
        .in_zone(ZoneId::Battlefield);
    let artifact = ObjectSpec::card(p1, "Own Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);
    let enchantment = ObjectSpec::enchantment(p1, "Own Enchantment");
    let planeswalker = ObjectSpec::planeswalker(p1, "Own Planeswalker", 4);
    let gy_creature = ObjectSpec::creature(p2, "Dead Bear", 2, 2).in_zone(ZoneId::Graveyard(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(own_creature)
        .object(opp_creature)
        .object(own_basic_mountain)
        .object(own_nonbasic_land)
        .object(opp_mountain)
        .object(artifact)
        .object(enchantment)
        .object(planeswalker)
        .object(gy_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    (state, p1, p2)
}

/// Pass priority for all listed players once. Mirrors `mechanics_e_l/enchant.rs::pass_all`.
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
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

// ── T1: the differential-equivalence probe (headline, acceptance criterion 1) ──────

/// CR 303.4a / 702.5a / 702.5d — for every `EnchantTarget` variant, the requirement
/// `spell_target_requirements` synthesizes agrees EXACTLY with what `process_command`
/// accepts, decided by EXECUTION rather than by argument (plan §6 T1). Also asserts
/// non-vacuity: at least one accepted AND one rejected candidate per variant.
///
/// **Revert to watch fail** (plan §6 T1): change the `Filtered` arm's `controller`
/// mapping in `enchant_target_to_requirement` to `TargetController::Any`. The `Filtered`
/// row here ("Enchant Mountain you control") should start accepting "Opp Mountain" via
/// `legal_targets_per_slot` while `process_command` still rejects it (the CR 303.4a gate
/// in `handle_cast_spell` independently re-checks `sba::matches_enchant_target`, which
/// this revert does not touch) — the two sides diverging is exactly what this probe
/// exists to see.
#[test]
fn test_dx20_t1_enchant_target_offer_and_cast_agree_for_every_variant() {
    let variants: Vec<EnchantTarget> = vec![
        EnchantTarget::Creature,
        EnchantTarget::Permanent,
        EnchantTarget::Artifact,
        EnchantTarget::Enchantment,
        EnchantTarget::Land,
        EnchantTarget::Planeswalker,
        EnchantTarget::Player,
        EnchantTarget::CreatureOrPlaneswalker,
        EnchantTarget::Filtered(EnchantFilter {
            has_card_type: Some(CardType::Land),
            has_subtype: Some(SubType("Mountain".to_string())),
            controller: EnchantControllerConstraint::You,
            ..Default::default()
        }),
        // E2 (pb-review-DX20.md): the single `Filtered` row above exercises only 3 of
        // `EnchantFilter`'s 6 fields (`has_card_type`, `has_subtype`, `controller`).
        // The four rows below cover the rest -- `basic` is not hypothetical: it is
        // live in two deck-legal `Complete` defs (`ossification`, `dimensional_exile`)
        // and was never probed before this fix.
        //
        // **Revert executed** (review's exact prediction): swap
        // `enchant_target_to_requirement`'s `basic: f.basic, nonbasic: f.nonbasic`
        // lines. The `basic: true` row below reddens on THIS test (T1) specifically
        // -- offer says "Own Karoo" (nonbasic) is legal, cast still rejects it,
        // because `casting.rs`'s CR 303.4a gate independently re-checks the
        // UNMODIFIED incumbent predicate `sba::matches_enchant_target` -- proving the
        // synth/incumbent disagreement is exactly what T1's offer-vs-cast comparison
        // is built to catch, for a field no earlier probe touched. Verbatim failure
        // in `scratchpad/dx20-reverts.md`.
        EnchantTarget::Filtered(EnchantFilter {
            has_card_type: Some(CardType::Land),
            basic: true,
            controller: EnchantControllerConstraint::You,
            ..Default::default()
        }),
        EnchantTarget::Filtered(EnchantFilter {
            has_card_type: Some(CardType::Land),
            nonbasic: true,
            ..Default::default()
        }),
        EnchantTarget::Filtered(EnchantFilter {
            has_subtypes: vec![
                SubType("Mountain".to_string()),
                SubType("Forest".to_string()),
            ],
            ..Default::default()
        }),
        EnchantTarget::Filtered(EnchantFilter {
            has_card_type: Some(CardType::Land),
            has_subtype: Some(SubType("Mountain".to_string())),
            controller: EnchantControllerConstraint::Opponent,
            ..Default::default()
        }),
    ];

    for enchant in variants {
        let (state, p1, p2) = build_board_with_aura(enchant.clone());
        let aura_id = find_object(&state, "Test Aura");

        let candidates: Vec<(&str, Target)> = vec![
            (
                "Own Creature",
                Target::Object(find_object(&state, "Own Creature")),
            ),
            (
                "Opp Creature",
                Target::Object(find_object(&state, "Opp Creature")),
            ),
            (
                "Own Mountain",
                Target::Object(find_object(&state, "Own Mountain")),
            ),
            (
                "Own Karoo",
                Target::Object(find_object(&state, "Own Karoo")),
            ),
            (
                "Opp Mountain",
                Target::Object(find_object(&state, "Opp Mountain")),
            ),
            (
                "Own Artifact",
                Target::Object(find_object(&state, "Own Artifact")),
            ),
            (
                "Own Enchantment",
                Target::Object(find_object(&state, "Own Enchantment")),
            ),
            (
                "Own Planeswalker",
                Target::Object(find_object(&state, "Own Planeswalker")),
            ),
            (
                "Dead Bear",
                Target::Object(find_object(&state, "Dead Bear")),
            ),
            ("p1", Target::Player(p1)),
            ("p2", Target::Player(p2)),
        ];

        let reqs = spell_target_requirements(&state, aura_id, &[], None, false);
        assert_eq!(
            reqs.len(),
            1,
            "{:?}: exactly one synthesized requirement expected, got {:?}",
            enchant,
            reqs
        );

        let legal = legal_targets_per_slot(&state, p1, aura_id, &reqs);
        assert_eq!(legal.len(), 1, "{:?}: exactly one slot expected", enchant);
        let legal_slot0 = &legal[0];

        let mut accepted = 0usize;
        let mut rejected = 0usize;
        for (label, cand) in &candidates {
            let offer_says_legal = legal_slot0.contains(cand);
            let cast_result = process_command(state.clone(), cast(p1, aura_id, vec![cand.clone()]));
            let cast_says_legal = cast_result.is_ok();
            assert_eq!(
                offer_says_legal,
                cast_says_legal,
                "{:?} / {}: offer says legal={} but cast says legal={} ({:?})",
                enchant,
                label,
                offer_says_legal,
                cast_says_legal,
                cast_result.err()
            );
            if cast_says_legal {
                accepted += 1;
            } else {
                rejected += 1;
            }
        }
        assert!(
            accepted > 0,
            "{:?}: no candidate was accepted (vacuous positive)",
            enchant
        );
        assert!(
            rejected > 0,
            "{:?}: no candidate was rejected (vacuous negative)",
            enchant
        );
    }
}

// ── E1: exact-shape pin + player-side correctness (pb-review-DX20.md finding E1) ───

/// CR 702.5a / 702.5d — E1's fix: T1 above only proves the offer and cast paths agree
/// WITH EACH OTHER, which is blind to a mapping that is stricter (or wider) than the
/// CR-correct answer in BOTH places at once, because both paths now derive the SAME
/// synthesized requirement -- a bug in `enchant_target_to_requirement` itself (e.g.
/// `Permanent -> TargetCreature`, or the exact mistake plan §3.2 warns against in
/// bold, `CreatureOrPlaneswalker -> TargetAny`) would make offer and cast agree with
/// EACH OTHER while both silently disagree with the CR. This test pins the ACTUAL
/// literal `TargetRequirement` for all 9 `EnchantTarget` variants (not merely "offer
/// == cast"), and separately asserts CAST-SIDE player-target correctness directly
/// (not via offer/cast self-consistency): for the 8 non-`Player` variants, BOTH
/// `Target::Player(p1)` and `Target::Player(p2)` must be rejected by `process_command`
/// (CR 702.5d: "can't target permanents" also implies the converse -- a
/// non-player-enchanting Aura can't target a player); for `Player`, both must be
/// accepted and every object candidate rejected.
///
/// **Two reverts executed, proving the two halves discriminate INDEPENDENTLY** (a
/// single combined revert cannot prove this: `assert_eq!` panics and unwinds the test
/// at the first failing assertion, so if the shape check fails it never reaches the
/// player-rejection checks below it in the same loop iteration):
/// 1. Change the `CreatureOrPlaneswalker` arm to `TargetRequirement::TargetAny` (the
///    exact mistake plan §3.2 warns against) -- reddens the EXACT-SHAPE assertion.
/// 2. Leave the mapping untouched; instead add a
///    `TargetRequirement::TargetPermanentWithFilter(_) => Ok(())` arm to
///    `validate_player_satisfies_requirement`'s match (ahead of its catch-all) -- the
///    synthesized requirement's SHAPE is unchanged (the exact-shape assertion stays
///    green), but a player is now wrongly accepted as a `CreatureOrPlaneswalker`
///    target -- reddens the PLAYER-REJECTION assertion, on its own, with the shape
///    check passing right up to that point. Both reverts recorded verbatim in
///    `scratchpad/dx20-reverts.md`.
#[test]
fn test_dx20_e1_exact_shape_pin_and_player_side_correctness() {
    let filtered = EnchantTarget::Filtered(EnchantFilter {
        has_card_type: Some(CardType::Land),
        has_subtype: Some(SubType("Mountain".to_string())),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    });

    let cases: Vec<(EnchantTarget, TargetRequirement)> = vec![
        (EnchantTarget::Creature, TargetRequirement::TargetCreature),
        (EnchantTarget::Permanent, TargetRequirement::TargetPermanent),
        (EnchantTarget::Artifact, TargetRequirement::TargetArtifact),
        (
            EnchantTarget::Enchantment,
            TargetRequirement::TargetEnchantment,
        ),
        (EnchantTarget::Land, TargetRequirement::TargetLand),
        (
            EnchantTarget::Planeswalker,
            TargetRequirement::TargetPlaneswalker,
        ),
        (EnchantTarget::Player, TargetRequirement::TargetPlayer),
        (
            EnchantTarget::CreatureOrPlaneswalker,
            TargetRequirement::TargetPermanentWithFilter(mtg_engine::TargetFilter {
                has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                ..Default::default()
            }),
        ),
        (
            filtered.clone(),
            TargetRequirement::TargetPermanentWithFilter(mtg_engine::TargetFilter {
                has_card_type: Some(CardType::Land),
                has_subtype: Some(SubType("Mountain".to_string())),
                controller: mtg_engine::TargetController::You,
                ..Default::default()
            }),
        ),
    ];

    for (enchant, expected) in &cases {
        let (state, p1, p2) = build_board_with_aura(enchant.clone());
        let aura_id = find_object(&state, "Test Aura");

        // (a) Exact-shape pin: the literal the synthesis returns, not merely "offer
        // agrees with cast".
        let reqs = spell_target_requirements(&state, aura_id, &[], None, false);
        assert_eq!(
            reqs,
            vec![expected.clone()],
            "{:?}: synthesized requirement should be EXACTLY {:?}, got {:?}",
            enchant,
            expected,
            reqs
        );

        // (b) Cast-side player correctness, decided directly against process_command
        // -- NOT via offer/cast self-consistency, which cannot see a mapping bug that
        // moves both sides together.
        let is_player_variant = matches!(enchant, EnchantTarget::Player);
        for (label, target_player) in [("p1", p1), ("p2", p2)] {
            let result = process_command(
                state.clone(),
                cast(p1, aura_id, vec![Target::Player(target_player)]),
            );
            assert_eq!(
                result.is_ok(),
                is_player_variant,
                "{:?} / player target {}: expected cast Ok={} (CR 702.5d), got {:?}",
                enchant,
                label,
                is_player_variant,
                result.err()
            );
        }

        // For the `Player` variant, every OBJECT candidate on the board must be
        // rejected (CR 702.5d: "can't target permanents").
        if is_player_variant {
            for name in [
                "Own Creature",
                "Opp Creature",
                "Own Mountain",
                "Own Karoo",
                "Opp Mountain",
                "Own Artifact",
                "Own Enchantment",
                "Own Planeswalker",
            ] {
                let obj_id = find_object(&state, name);
                let result = process_command(
                    state.clone(),
                    cast(p1, aura_id, vec![Target::Object(obj_id)]),
                );
                assert!(
                    result.is_err(),
                    "EnchantTarget::Player: object candidate {} should be rejected \
                     (CR 702.5d), got Ok",
                    name
                );
            }
        }
    }
}

// ── T2: the cast path and the offer path are the same function ────────────────────

/// CR 303.4a / 601.2c — T2.1: an "Enchant creature" Aura's offer is exactly one
/// requirement, and `target_count_range` reports `(1, 1)` — the value
/// `tools/play-server/src/view.rs:2363` reads to decide whether to render a picker.
///
/// **Revert to watch fail** (plan §6 T2.1/T2.2): make `aura_spell_target_requirements`
/// return `base` unconditionally. This assertion must redden and print the observed
/// `(0, 0)`.
#[test]
fn test_dx20_t2_1_enchant_creature_offer_is_exactly_one_requirement() {
    let (state, p1, _p2) = build_board_with_aura(EnchantTarget::Creature);
    let aura_id = find_object(&state, "Test Aura");

    let reqs = spell_target_requirements(&state, aura_id, &[], None, false);
    // `target_count_range` is asserted FIRST and prints the observed pair verbatim, so a
    // reverted synthesis reddens on the exact `(0, 0)` regression this probe exists to
    // catch (plan §6 T2.1) rather than only on a `Vec` equality whose printed form
    // doesn't spell out the tuple.
    assert_eq!(
        target_count_range(&reqs),
        (1, 1),
        "target_count_range should report (1, 1), not (0, 0) -- observed {:?} for \
         requirements {:?}",
        target_count_range(&reqs),
        reqs
    );
    assert_eq!(
        reqs,
        vec![TargetRequirement::TargetCreature],
        "an Enchant-creature Aura should offer exactly one TargetCreature requirement"
    );
    let _ = p1;
}

/// CR 601.2c / 303.4a — T2.2: a zero-target cast is rejected with `InvalidTarget`
/// (CR 601.2c's count check firing, not the old ad-hoc `InvalidCommand`), and a
/// two-target cast is ALSO rejected ("a target", singular, CR 303.4a) -- both are new
/// behaviour post-PB-DX20 (plan §4.1).
#[test]
fn test_dx20_t2_2_zero_and_two_target_casts_are_both_rejected() {
    let (state, p1, _p2) = build_board_with_aura(EnchantTarget::Creature);
    let aura_id = find_object(&state, "Test Aura");
    let own_creature_id = find_object(&state, "Own Creature");
    let opp_creature_id = find_object(&state, "Opp Creature");

    let zero_target_result = process_command(state.clone(), cast(p1, aura_id, vec![]));
    assert!(
        zero_target_result.is_err(),
        "a zero-target Aura cast should be rejected"
    );
    match zero_target_result.unwrap_err() {
        GameStateError::InvalidTarget(_) => {}
        e => panic!(
            "expected InvalidTarget (CR 601.2c count check), got: {:?}",
            e
        ),
    }

    let two_target_result = process_command(
        state,
        cast(
            p1,
            aura_id,
            vec![
                Target::Object(own_creature_id),
                Target::Object(opp_creature_id),
            ],
        ),
    );
    // E9 (pb-review-DX20.md): match the error VARIANT and a message substring, not
    // merely `is_err()` -- an `is_err()`-only assertion would also pass if the
    // `Creature` mapping broke so badly that NO creature were a legal target at all,
    // which would report this exact shape while the batch's core mapping was
    // silently wrong.
    match two_target_result {
        Err(GameStateError::InvalidTarget(msg)) => {
            assert!(
                msg.contains("expected 1..=1 target(s) but got 2"),
                "expected the CR 303.4a/601.2c count-check message, got: {msg}"
            );
        }
        other => panic!(
            "expected Err(InvalidTarget(\"expected 1..=1 target(s) but got 2\")), got: {:?}",
            other
        ),
    }
}

/// CR 303.4b — T2.3 regression floor: a legal single-target cast still succeeds
/// end-to-end and the Aura attaches on resolution (mirrors
/// `mechanics_e_l/enchant.rs::test_702_5_aura_attaches_to_target_on_resolution`).
#[test]
fn test_dx20_t2_3_legal_single_target_cast_still_succeeds_and_attaches() {
    let (state, p1, p2) = build_board_with_aura(EnchantTarget::Creature);
    let aura_id = find_object(&state, "Test Aura");
    let target_id = find_object(&state, "Own Creature");

    let (state, _) = process_command(state, cast(p1, aura_id, vec![Target::Object(target_id)]))
        .expect("legal single-target Aura cast should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let aura_on_bf = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Test Aura" && o.zone == ZoneId::Battlefield)
        .expect("Aura should be on the battlefield after resolution");
    assert_eq!(
        aura_on_bf.attached_to,
        Some(target_id),
        "CR 303.4b: Aura should be attached to its target on resolution"
    );
}

/// CR 702.11b — T2.4: a hexproof creature is still refused as an Aura target. This
/// pins that PB-DX20 changes NOTHING here: `validate_mapped_targets` applies
/// hexproof/shroud/protection checks unconditionally for any `Target::Object`,
/// regardless of whether `requirements` is empty (plan §4.3).
#[test]
fn test_dx20_t2_4_hexproof_creature_still_rejected_as_aura_target() {
    let p1 = p(1);
    let p2 = p(2);
    let aura = aura_in_hand(p1, "Test Aura", EnchantTarget::Creature);
    let hexproof_creature =
        ObjectSpec::creature(p2, "Hexproof Bear", 2, 2).with_keyword(KeywordAbility::Hexproof);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(aura)
        .object(hexproof_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let aura_id = find_object(&state, "Test Aura");
    let target_id = find_object(&state, "Hexproof Bear");

    let result = process_command(state, cast(p1, aura_id, vec![Target::Object(target_id)]));
    // E9 (pb-review-DX20.md): match the error VARIANT and a hexproof-specific message
    // substring, not merely `is_err()` -- a bare `is_err()` would also pass if the
    // `Creature` mapping broke so no creature were a legal target at all, which would
    // report "hexproof still works" while the batch's core mapping was silently
    // wrong.
    match result {
        Err(GameStateError::InvalidTarget(msg)) => {
            assert!(
                msg.contains("hexproof"),
                "expected the CR 702.11b hexproof-rejection message, got: {msg}"
            );
        }
        other => panic!(
            "expected Err(InvalidTarget(\"object has hexproof and cannot be targeted by \
             opponents\")), got: {:?}",
            other
        ),
    }
}

// ── T3: Bestow (plan §4.5) ──────────────────────────────────────────────────────────

/// CR 702.103b — `spell_target_requirements` with `Some(AltCostKind::Bestow)` returns
/// one `TargetCreature` requirement for a real Bestow card (Boon Satyr, `Complete`),
/// while the SAME call with `alt_cost: None` returns `vec![]` -- a bestow card is a
/// creature spell until the caster says otherwise.
///
/// **Revert** (plan §6 T3): drop the bestow branch in `queries::spell_target_requirements`;
/// the `Some(Bestow)` half should then return `vec![]` too.
#[test]
fn test_dx20_t3_bestow_offers_a_target_only_when_cast_bestowed() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let boon_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Boon Satyr")
            .with_card_id(CardId("boon-satyr".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(boon_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let boon_id = find_object(&state, "Boon Satyr");

    let reqs_no_alt = spell_target_requirements(&state, boon_id, &[], None, false);
    assert_eq!(
        reqs_no_alt,
        Vec::<TargetRequirement>::new(),
        "cast as a plain creature spell, Boon Satyr should offer no targets"
    );

    let reqs_bestow =
        spell_target_requirements(&state, boon_id, &[], Some(AltCostKind::Bestow), false);
    assert_eq!(
        reqs_bestow,
        vec![TargetRequirement::TargetCreature],
        "cast bestowed (CR 702.103b), Boon Satyr should offer exactly one TargetCreature"
    );
}

// ── T4: the second failure mode, and the roster gates (acceptance criterion 3) ─────

/// CR 303.4a / 702.5c / 702.5d — four EXACT roster assertions over `all_cards()`
/// (SR-36 -- never grepped from source), each pinning a shape that would rot silently.
#[test]
fn test_dx20_t4_roster_gates_over_all_cards() {
    let cards = all_cards();
    let aura_defs: Vec<&CardDefinition> = cards.iter().filter(|d| is_aura_def(d)).collect();

    // 1. Aura defs with NO Enchant keyword at all -- exactly {"Animate Dead",
    //    "Curse of Opulence"}, both `inert`. Nothing in `casting.rs`'s Aura gate is
    //    live for these; the assertion says so.
    let mut no_enchant: Vec<&str> = aura_defs
        .iter()
        .filter(|d| enchant_keywords(d).is_empty())
        .map(|d| d.name.as_str())
        .collect();
    no_enchant.sort_unstable();
    assert_eq!(
        no_enchant,
        vec!["Animate Dead", "Curse of Opulence"],
        "the roster of Aura defs with no Enchant keyword moved -- restate the batch's \
         yield claim, don't just re-pin this list (pb-plan-DX20.md §0.9)"
    );
    for d in aura_defs.iter().filter(|d| enchant_keywords(d).is_empty()) {
        assert!(
            matches!(d.completeness, Completeness::Inert(_)),
            "{} has no Enchant keyword and must be Completeness::Inert, got {:?}",
            d.name,
            d.completeness
        );
    }

    // 2. Enchant(Player) defs -- EMPTY. §4.4: the offer would open onto an
    //    unimplemented attachment path (OOS-DX20-2).
    let player_enchants: Vec<&str> = aura_defs
        .iter()
        .filter(|d| {
            enchant_keywords(d)
                .iter()
                .any(|et| matches!(et, EnchantTarget::Player))
        })
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        player_enchants.is_empty(),
        "OOS-DX20-2: {:?} carry Enchant(Player); a target requirement now exists for \
         these but the Aura-to-player attachment path does not -- implement attachment \
         first",
        player_enchants
    );

    // 3. Defs carrying TWO OR MORE Enchant keywords -- EMPTY. §3.1: `get_enchant_target`
    //    keeps only the FIRST via `find_map`, contra CR 702.5c -- OOS-DX20-1.
    let multi_enchant: Vec<&str> = aura_defs
        .iter()
        .filter(|d| enchant_keywords(d).len() >= 2)
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        multi_enchant.is_empty(),
        "OOS-DX20-1: {:?} carry 2+ Enchant keywords; CR 702.5c requires ALL to apply \
         but `get_enchant_target`'s find_map only sees the first -- this corpus \
         exposure was pinned at 0, it is no longer 0",
        multi_enchant
    );

    // 4. Aura defs that ALSO carry an `AbilityDefinition::Spell` -- EMPTY. Guard 3 of
    //    `aura_spell_target_requirements` (§3.1) means such a def would keep its OWN
    //    (non-empty) `base` list and never receive an Enchant-derived requirement.
    let aura_with_spell: Vec<&str> = aura_defs
        .iter()
        .filter(|d| {
            d.abilities
                .iter()
                .any(|a| matches!(a, AbilityDefinition::Spell { .. }))
        })
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        aura_with_spell.is_empty(),
        "{:?} are Auras that ALSO declare AbilityDefinition::Spell.targets -- guard 3 \
         of aura_spell_target_requirements means these get NO Enchant-derived \
         requirement; either that guard needs removing or these defs need review",
        aura_with_spell
    );

    // 5. **INVERTED by PB-DX20b (`scutemob-222`) -- `OOS-DX20-10` is CLOSED.**
    //    `EnchantTarget::Permanent` is almost always a WIDENING of a printed
    //    multi-type restriction ("creature, land, or planeswalker" etc admits
    //    artifacts and enchantments too). PB-DX20 could not express such a line --
    //    `EnchantFilter` had `has_card_type` (ONE type) and `has_subtypes` (an OR
    //    over SUBtypes) and no OR over card TYPES -- so it pinned the one offending
    //    `Complete` def (`imprisoned_in_the_moon`) wrong-way-round at
    //    `{"Imprisoned in the Moon"}` and named THIS assertion as the one to invert
    //    when the expressiveness arrived.
    //
    //    PB-DX20b added `EnchantFilter::has_card_types` (the OR over card types) and
    //    repaired the def to
    //    `Filtered(EnchantFilter { has_card_types: [Creature, Land, Planeswalker] })`,
    //    so the assertion is now the ordinary way round: **no `Complete` Aura def may
    //    declare `EnchantTarget::Permanent`.** A new member appearing here is a fresh
    //    instance of `OOS-DX20-10` -- check the def's printed Enchant line before
    //    accepting it, because a printed line naming any narrower class than "any
    //    permanent" must be declared as a `Filtered` filter, not widened to
    //    `Permanent`.
    //
    //    The bare `EnchantTarget::Permanent` variant is deliberately NOT deleted: a
    //    card really can print "Enchant permanent" -- `Song of the Dryads` does,
    //    verbatim (MCP-verified 2026-09-03; it is NOT in this corpus today, so the
    //    variant has zero `Complete` users, which is exactly what this assertion now
    //    says). The roster is also restricted to `Complete` defs, so a
    //    `partial`/`known_wrong` def may still carry `Permanent` without reddening it.
    let permanent_complete_auras: Vec<&str> = aura_defs
        .iter()
        .filter(|d| matches!(d.completeness, Completeness::Complete))
        .filter(|d| {
            enchant_keywords(d)
                .iter()
                .any(|et| matches!(et, EnchantTarget::Permanent))
        })
        .map(|d| d.name.as_str())
        .collect();
    assert_eq!(
        permanent_complete_auras,
        Vec::<&str>::new(),
        "OOS-DX20-10 (closed by PB-DX20b): no `Complete` Aura def may declare \
         EnchantTarget::Permanent unless its printed Enchant line really is \
         \"Enchant permanent\". `Permanent` is almost always a WIDENING of a printed \
         multi-type restriction -- read the def's oracle text and express the printed \
         line with EnchantFilter (`has_card_types` is the OR over card types) rather \
         than adding a member here, got {:?}",
        permanent_complete_auras
    );

    // Non-vacuity floor: the Enchant-carrying roster and its Complete subset, named so
    // a move here is reported as a finding, not silently re-tuned.
    let enchant_carrying: Vec<&CardDefinition> = cards
        .iter()
        .filter(|d| !enchant_keywords(d).is_empty())
        .collect();
    assert_eq!(
        enchant_carrying.len(),
        23,
        "the Enchant-keyword-carrying roster moved from the pinned 23 -- restate the \
         batch's card-count claim (pb-plan-DX20.md §0.3)"
    );
    let complete_subset = enchant_carrying
        .iter()
        .filter(|d| matches!(d.completeness, Completeness::Complete))
        .count();
    assert_eq!(
        complete_subset, 13,
        "the Complete subset of the Enchant-keyword-carrying roster moved from the \
         pinned 13 -- restate the batch's card-count claim (pb-plan-DX20.md §0.5)"
    );
}

// ── T5: Reconfigure, through the REAL synth path (acceptance criterion 2) ──────────

/// Build Lizard Blades (`Complete`, CR 702.151a) via `enrich_spec_from_def` -- NOT the
/// hand-built `reconfigure_attach_ability` helper in `mechanics_m_z/reconfigure.rs` --
/// so T5 exercises the same `testing/replay_harness.rs` synth site production casting
/// goes through. p1 controls Lizard Blades and "Own Creature"; p2 controls
/// "Opp Creature". p1's mana pool holds 2 colorless (Lizard Blades' Reconfigure cost).
fn build_board_with_lizard_blades() -> (mtg_engine::GameState, PlayerId, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let blades_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lizard Blades")
            .with_card_id(CardId("lizard-blades".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let own_creature = ObjectSpec::creature(p1, "Own Creature", 2, 2);
    let opp_creature = ObjectSpec::creature(p2, "Opp Creature", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(blades_spec)
        .object(own_creature)
        .object(opp_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    (state, p1, p2, blades_id)
}

fn activate_attach(
    state: mtg_engine::GameState,
    player: PlayerId,
    source: ObjectId,
    targets: Vec<Target>,
) -> Result<(mtg_engine::GameState, Vec<mtg_engine::GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index: 0,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// CR 702.151a — T5.1: the Reconfigure attach ability's *real* synth-site requirement
/// (queried the same way `handle_activate_ability` reads it) is EXACTLY
/// `TargetCreatureWithFilter { controller: You, exclude_self: true }`, and
/// `target_count_range` reports `(1, 1)`.
///
/// **Revert to watch fail** (plan §6 T5): set `exclude_self: false` at the
/// `replay_harness.rs` synth site. This assertion must redden on the filter's
/// `exclude_self` field specifically.
#[test]
fn test_dx20_t5_1_reconfigure_attach_requirement_is_the_real_cr_702_151a_shape() {
    let (state, _p1, _p2, blades_id) = build_board_with_lizard_blades();

    let reqs = mtg_engine::ability_target_requirements(&state, blades_id, 0, &[]);
    assert_eq!(
        mtg_engine::target_count_range(&reqs),
        (1, 1),
        "Reconfigure attach should report (1, 1), got {:?} for {:?}",
        mtg_engine::target_count_range(&reqs),
        reqs
    );
    assert_eq!(
        reqs,
        vec![TargetRequirement::TargetCreatureWithFilter(
            mtg_engine::TargetFilter {
                controller: mtg_engine::TargetController::You,
                exclude_self: true,
                ..Default::default()
            }
        )],
        "Reconfigure attach requirement should be exactly CR 702.151a's \
         'another target creature you control' -- got {:?}",
        reqs
    );
}

/// CR 301.5c / 702.151a — T5.2: attach targeting Lizard Blades itself is rejected with
/// `InvalidTarget`. **Assert `Err`, strictly** -- not `Err(_) | Ok(no attachment)` --
/// because the two existing `mechanics_m_z/reconfigure.rs` tests are tolerant by
/// construction and therefore cannot discriminate the `exclude_self` revert (plan §6
/// T5's stated reason this probe exists).
#[test]
fn test_dx20_t5_2_attach_targeting_self_is_strictly_rejected() {
    let (state, p1, _p2, blades_id) = build_board_with_lizard_blades();

    let result = activate_attach(state, p1, blades_id, vec![Target::Object(blades_id)]);
    match result {
        Err(GameStateError::InvalidTarget(_)) => {}
        other => panic!(
            "attaching Lizard Blades to itself should be Err(InvalidTarget), got {:?}",
            other
        ),
    }
}

/// CR 702.151a — T5.3: attach targeting an opponent's creature is rejected with
/// `InvalidTarget` (the `controller: You` clause of the filter).
#[test]
fn test_dx20_t5_3_attach_targeting_opponents_creature_is_strictly_rejected() {
    let (state, p1, _p2, blades_id) = build_board_with_lizard_blades();
    let opp_creature_id = find_object(&state, "Opp Creature");

    let result = activate_attach(state, p1, blades_id, vec![Target::Object(opp_creature_id)]);
    match result {
        Err(GameStateError::InvalidTarget(_)) => {}
        other => panic!(
            "attaching Lizard Blades to an opponent's creature should be \
             Err(InvalidTarget), got {:?}",
            other
        ),
    }
}

/// CR 702.151a / 702.151b — T5.4: attach targeting another creature the caster
/// controls succeeds; after resolution `attached_to == Some(target)` and
/// `Designations::RECONFIGURED` is set.
#[test]
fn test_dx20_t5_4_attach_to_own_other_creature_succeeds_and_attaches() {
    let (state, p1, p2, blades_id) = build_board_with_lizard_blades();
    let own_creature_id = find_object(&state, "Own Creature");

    let (state, _) = activate_attach(state, p1, blades_id, vec![Target::Object(own_creature_id)])
        .expect("attaching Lizard Blades to another creature you control should succeed");
    let (state, _) = pass_all(state, &[p1, p2]);

    let blades_obj = state.objects().get(&blades_id).unwrap();
    assert_eq!(
        blades_obj.attached_to,
        Some(own_creature_id),
        "CR 702.151a: Lizard Blades should be attached to the targeted creature"
    );
    assert!(
        blades_obj
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "CR 702.151b: is_reconfigured should be set after a successful attach"
    );
}

/// CR 702.151a / 601.2c — T5.5: attach with ZERO targets is rejected. This is the
/// discriminating probe for the live defect (plan §6 T5): before PB-DX20 this was
/// `Ok` with the cost paid and a silent fizzle.
///
/// E7 (pb-review-DX20.md): the doc this replaces claimed to test "the mana was NOT
/// spent" (CR 602.2c), but `process_command` takes `GameState` BY VALUE and returns
/// `Err` with no state at all on rejection -- re-reading the caller's own `state`
/// after a rejected call therefore observes an object no code path could possibly
/// have touched. That assertion was structurally guaranteed to pass and is deleted.
/// This is NOT a hole in the engine: `process_command`'s ownership shape makes CR
/// 602.2c's rewind structural here, which is a STRONGER property than any assertion
/// could prove. What this probe actually discriminates is that a zero-target attach
/// is `Err` at all (before PB-DX20 it was `Ok` with a silent fizzle) -- and, as a real
/// observation rather than a decorative one, that the SAME state with mana still in
/// the pool can still perform a LEGAL activation afterward and that activation DOES
/// spend the mana, proving the rejected attempt consumed nothing observable and the
/// state was left genuinely usable.
#[test]
fn test_dx20_t5_5_zero_target_attach_is_rejected_and_the_state_is_still_usable() {
    let (state, p1, _p2, blades_id) = build_board_with_lizard_blades();
    let own_creature_id = find_object(&state, "Own Creature");
    let mana_before = state.players().get(&p1).unwrap().mana_pool.colorless;
    assert_eq!(
        mana_before, 2,
        "sanity: p1 should start with 2 colorless mana"
    );

    let result = activate_attach(state.clone(), p1, blades_id, vec![]);
    assert!(
        result.is_err(),
        "a zero-target Reconfigure attach should be rejected, not silently fizzle"
    );

    // Real observation: the SAME original `state` (untouched by the Err above, by
    // construction of `process_command`'s ownership) can still perform a LEGAL
    // attach, and that legal attach spends the mana -- proving the pool was never
    // silently drained by the rejected zero-target attempt.
    let (legal_state, _) =
        activate_attach(state, p1, blades_id, vec![Target::Object(own_creature_id)])
            .expect("a legal attach on the same original state should still succeed");
    let mana_after_legal = legal_state.players().get(&p1).unwrap().mana_pool.colorless;
    assert_eq!(
        mana_after_legal, 0,
        "the legal attach should have spent the 2 colorless mana Reconfigure costs"
    );
}

/// CR 702.151a — T5.6: ability index 1 (unattach) still has `targets == vec![]`
/// (CR 702.151a's second ability takes no target) and still activates with none.
#[test]
fn test_dx20_t5_6_unattach_ability_still_takes_no_targets() {
    let (state, p1, p2, blades_id) = build_board_with_lizard_blades();
    let own_creature_id = find_object(&state, "Own Creature");

    // Unattach's own requirement list is untouched by this batch.
    let unattach_reqs = mtg_engine::ability_target_requirements(&state, blades_id, 1, &[]);
    assert_eq!(
        unattach_reqs,
        Vec::<TargetRequirement>::new(),
        "Reconfigure unattach should still declare zero target requirements"
    );

    // Attach first, so there is something to unattach.
    let (state, _) = activate_attach(state, p1, blades_id, vec![Target::Object(own_creature_id)])
        .expect("attach should succeed");
    let (mut state, _) = pass_all(state, &[p1, p2]);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 1,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("unattach with zero targets should succeed");
    let (state, _) = pass_all(state, &[p1, p2]);

    let blades_obj = state.objects().get(&blades_id).unwrap();
    assert_eq!(
        blades_obj.attached_to, None,
        "CR 702.151a: unattach should clear attached_to"
    );
    assert!(
        !blades_obj
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "is_reconfigured should be cleared after unattach"
    );
}

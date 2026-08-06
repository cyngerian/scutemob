// Bolt Bend — {3}{R}, Instant
// This spell costs {3} less to cast if you control a creature with power 4 or greater.
// Change the target of target spell or ability with a single target.
//
// CR 115.7a: "Change the target" — must change to a different legal target.
//
// PB-DX25b review Findings C1/E1 (`OOS-DX25b-1`, `OOS-DX25b-3`) -- COMPLETENESS
// DECISION, recorded explicitly rather than left to be inferred. `OOS-DX25b-3`
// CLOSED by PB-DX25c; `OOS-DX25b-1` STAYS OPEN.
//
// This def STAYS `Complete` (no demotion, no note). One known ENGINE-layer
// gap remains (recorded below, filed as a seed), and it applies identically
// to every other card carrying the same requirement -- it is not a fidelity
// problem with THIS def's translation of the printed card:
//
// (1) `OOS-DX25b-1` (OPEN) -- the "or ability" half of "target spell or
//     ability" is unreachable: an activated/triggered ability's stack entry
//     is never added to `state.objects` (`abilities.rs:1381`), so neither the
//     offer layer (`queries::legal_targets_per_slot`) nor the validator
//     (`casting.rs::validate_object_satisfies_requirement`'s opening
//     `state.objects.get(&id)?`) can ever see it. This def correctly declares
//     `TargetSpellOrAbilityWithSingleTarget` -- the requirement variant that
//     names both halves of the printed line -- and the engine's inability to
//     realize the ability half is a `Target::StackObject` id-space gap (a wire
//     change) shared by every card using this requirement, not something a
//     different DSL choice in THIS file could fix. Pinned wrong-way-round by
//     `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs
//     ::t3_ability_half_is_still_unreachable` -- the SUCCESSOR batch that adds
//     a stack-entry target id space must invert that test AND revisit this
//     comment.
// (2) `OOS-DX25b-3` (CLOSED by PB-DX25c) -- CR 115.7a's "another LEGAL
//     target" is now enforced for OBJECT-target redirects:
//     `rules::retarget::plan_target_change` delegates the whole redirect
//     decision to `casting::validate_targets_inner`, the same collective
//     legality arithmetic a real cast is checked against -- the redirect can
//     no longer land on an object that fails the original spell's own
//     `TargetRequirement` (e.g. a "destroy target creature" spell redirected
//     onto a land). Pinned (post-fix) by the same test file's
//     `t9_object_target_redirect_obeys_the_original_requirement` and
//     `t9b_object_target_redirect_fires_with_a_legal_alternative`.
//
// Precedent: `OOS-DX20-10` (a wrong-way-round roster pin filed rather than
// fixed, because the fix belongs to a different subsystem). `completeness`
// describes fidelity to the PRINTED card, not the engine's current behaviour
// under every input -- and this def's translation is faithful.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bolt-bend"),
        name: "Bolt Bend".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell costs {3} less to cast if you control a creature with power 4 or \
                      greater.\nChange the target of target spell or ability with a single target."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 115.7a: Change the target of target spell or ability with a single target.
            // must_change: true — the target MUST be changed to a different legal target.
            // If no other legal target exists, the original target is unchanged.
            effect: Effect::ChangeTargets {
                target: EffectTarget::DeclaredTarget { index: 0 },
                must_change: true,
            },
            targets: vec![TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        self_cost_reduction: Some(SelfCostReduction::ConditionalPowerThreshold {
            threshold: 4,
            reduction: 3,
        }),
        ..Default::default()
    }
}

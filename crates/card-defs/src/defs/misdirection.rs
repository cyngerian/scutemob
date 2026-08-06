// Misdirection — {3}{U}{U}, Instant
// You may exile a blue card from your hand rather than pay this spell's mana cost.
// Change the target of target spell with a single target.
//
// CR 118.9: pitch a blue card instead of the mana cost (no life component — unlike
// Force of Will's pitch, which also pays 1 life).
// CR 115.7a/115.7b: TargetSpellWithSingleTarget is spell-only (unlike Bolt Bend's
// TargetSpellOrAbilityWithSingleTarget, which also legalizes activated/loyalty
// abilities). Misdirection's oracle text says "target spell", not "target spell or
// ability", so the spell-only requirement is correct here.
//
// PB-DX25b review Finding E1 (`OOS-DX25b-3`) CLOSED by PB-DX25c -- COMPLETENESS
// DECISION, recorded explicitly rather than left to be inferred (same
// reasoning as `bolt_bend.rs`'s own note): this def STAYS `Complete`. CR
// 115.7a's "another LEGAL target" is now enforced for OBJECT-target redirects:
// `rules::retarget::plan_target_change` delegates the whole "which object or
// player may become the new target" decision to `casting::validate_targets_
// inner`, the same collective legality arithmetic a real cast is checked
// against -- e.g. a "destroy target creature" spell redirected through
// Misdirection can no longer land on a land. This was a gap in the SHARED
// `Effect::ChangeTargets` resolution logic, reachable from every card that
// uses it, never a fidelity problem with this def's translation of the
// printed card (which correctly declares `TargetSpellWithSingleTarget` and
// `must_change: true`, matching CR 115.7a/115.7b exactly). Pinned
// (post-fix) by
// `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs
// ::t9_object_target_redirect_obeys_the_original_requirement` and
// `::t9b_object_target_redirect_fires_with_a_legal_alternative`.
// `OOS-DX25b-1` (the "or ability" half of "target spell or ability" is
// unreachable, Bolt Bend's shape) and `OOS-DX25b-2` (a copy of a spell is not
// an announceable target, CR 707.10) both STAY OPEN -- neither affects this
// def's completeness: no card was announceable as a copy target before OR
// after PB-DX25c, and the ability half was never Misdirection's own shape
// (it declares the spell-only `TargetSpellWithSingleTarget`, not Bolt Bend's
// `TargetSpellOrAbilityWithSingleTarget`).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("misdirection"),
        name: "Misdirection".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may exile a blue card from your hand rather than pay this spell's mana \
                      cost.\nChange the target of target spell with a single target."
            .to_string(),
        abilities: vec![
            // CR 118.9: pitch a blue card instead of the mana cost (no life component).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Pitch,
                cost: ManaCost::default(),
                details: Some(AltCastDetails::Pitch {
                    costs: vec![Cost::ExileFromHand { color: Color::Blue }],
                    opponents_turn_only: false,
                }),
            },
            // CR 115.7a/115.7b: change the target of target spell with a single target.
            AbilityDefinition::Spell {
                effect: Effect::ChangeTargets {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    must_change: true,
                },
                targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

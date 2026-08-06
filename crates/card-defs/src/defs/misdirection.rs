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
// PB-DX25b review Finding E1 (`OOS-DX25b-3`) -- COMPLETENESS DECISION, recorded
// explicitly rather than left to be inferred (same reasoning as `bolt_bend.rs`'s
// own note, which this batch's coordinator directed be applied here too): this
// def STAYS `Complete`. CR 115.7a's "another LEGAL target" is not enforced for
// OBJECT-target redirects (`effects/mod.rs:7619-7654`, a KNOWN LIMITATION
// self-documented at the call site) -- e.g. a "destroy target creature" spell
// redirected through Misdirection can land on a land. This is a gap in the
// SHARED `Effect::ChangeTargets` resolution logic, reachable from every card
// that uses it, not a fidelity problem with this def's translation of the
// printed card (which correctly declares `TargetSpellWithSingleTarget` and
// `must_change: true`, matching CR 115.7a/115.7b exactly). Pinned
// wrong-way-round by
// `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs
// ::t9_object_target_redirect_ignores_the_original_requirement` -- the
// SUCCESSOR batch that implements object-target legality for
// `Effect::ChangeTargets` (needs the victim spell's `TargetRequirement` list
// stored on `StackObject`, a hashed field, its own batch) must invert that
// test AND revisit this comment. `OOS-DX25b-2` (a copy of a spell is not an
// announceable target, CR 707.10) is a similar, smaller ENGINE-layer gap that
// does not affect this def's completeness: no card was announceable as a
// copy target before OR after this batch, so nothing regressed and nothing
// to decide here.
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

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

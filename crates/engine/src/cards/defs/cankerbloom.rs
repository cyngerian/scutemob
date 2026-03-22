// Cankerbloom — {1}{G} Creature — Phyrexian Fungus 3/2
// {1}, Sacrifice this creature: Choose one —
// • Destroy target artifact.
// • Destroy target enchantment.
// • Proliferate.
//
// Note: AbilityDefinition::Activated lacks a `modes` field (unlike Spell). Targets for all
// modes are declared up-front. Player must declare artifact target (index 0) and enchantment
// target (index 1) even when choosing the Proliferate mode — this is a known DSL limitation
// matching the Abzan Charm pre-declaration pattern. The Choose effect will only execute
// one chosen branch, so unused targets are benign (they have no effect if their mode is not chosen).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cankerbloom"),
        name: "Cankerbloom".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Phyrexian", "Fungus"]),
        oracle_text: "{1}, Sacrifice this creature: Choose one —\n• Destroy target artifact.\n• Destroy target enchantment.\n• Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // {1}, Sacrifice this creature: Choose one —
            // Target index 0: artifact (mode 0)
            // Target index 1: enchantment (mode 1)
            // Mode 2 (proliferate) has no target.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Choose {
                    prompt: "Choose one: destroy artifact, destroy enchantment, or proliferate".to_string(),
                    choices: vec![
                        // Mode 0: Destroy target artifact.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        },
                        // Mode 1: Destroy target enchantment.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                        },
                        // Mode 2: Proliferate.
                        Effect::Proliferate,
                    ],
                },
                timing_restriction: None,
                targets: vec![
                    TargetRequirement::TargetArtifact,
                    TargetRequirement::TargetEnchantment,
                ],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

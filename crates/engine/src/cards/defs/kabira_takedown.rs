// Kabira Takedown // Kabira Plateau — {1}{W} Instant // Land (MDFC)
// Oracle: "Kabira Takedown deals damage equal to the number of creatures you control
// to target creature or planeswalker."
// Note: TargetCreature approximation — no TargetCreatureOrPlaneswalker variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kabira-takedown"),
        name: "Kabira Takedown // Kabira Plateau".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Kabira Takedown deals damage equal to the number of creatures you control to target creature or planeswalker.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 608.2b: Damage amount determined at resolution.
            effect: Effect::DealDamage {
                target: EffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
            // TODO: Should be TargetCreatureOrPlaneswalker — no such variant; using TargetCreature.
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// Mob Justice — {1}{R}, Sorcery
// Mob Justice deals damage equal to the number of creatures you control to target
// player or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mob-justice"),
        name: "Mob Justice".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Mob Justice deals damage equal to the number of creatures you control to target player or planeswalker.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
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
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

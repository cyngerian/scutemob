// Goblin War Strike — {R}, Sorcery
// Deal damage equal to the number of Goblins you control to target player or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-war-strike"),
        name: "Goblin War Strike".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Goblin War Strike deals damage equal to the number of Goblins you control to target player or planeswalker.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Goblin".to_string())),
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

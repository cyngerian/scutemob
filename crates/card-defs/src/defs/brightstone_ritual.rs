// Brightstone Ritual — {R}, Instant
// Add {R} for each Goblin on the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brightstone-ritual"),
        name: "Brightstone Ritual".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Add {R} for each Goblin on the battlefield.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Red,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Goblin".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::EachPlayer,
                    },
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

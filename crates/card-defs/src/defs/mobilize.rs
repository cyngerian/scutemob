// Mobilize — {G}, Sorcery
// Untap all creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mobilize"),
        name: "Mobilize".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Untap all creatures you control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::UntapAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    ..Default::default()
                },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

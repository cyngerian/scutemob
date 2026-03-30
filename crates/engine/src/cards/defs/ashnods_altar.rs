// Ashnod's Altar — {3}, Artifact.
// "Sacrifice a creature: Add {C}{C}."
// CR 602.2: Activated mana ability with sacrifice cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ashnods-altar"),
        name: "Ashnod's Altar".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Add {C}{C}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

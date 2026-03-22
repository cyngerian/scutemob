// In Garruk's Wake — {7}{B}{B} Sorcery
// Destroy all creatures you don't control and all planeswalkers you don't control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("in-garruks-wake"),
        name: "In Garruk's Wake".to_string(),
        mana_cost: Some(ManaCost { generic: 7, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy all creatures you don't control and all planeswalkers you don't control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // Destroy all creatures opponents control.
                Effect::DestroyAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::Opponent,
                        ..Default::default()
                    },
                    cant_be_regenerated: false,
                },
                // Destroy all planeswalkers opponents control.
                Effect::DestroyAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Planeswalker),
                        controller: TargetController::Opponent,
                        ..Default::default()
                    },
                    cant_be_regenerated: false,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

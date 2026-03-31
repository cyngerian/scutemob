// Misty Rainforest — Land; {T}, pay 1 life, sacrifice: search for Forest or Island, put onto battlefield untapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("misty-rainforest"),
        name: "Misty Rainforest".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life, Sacrifice this land: Search your library for a Forest or Island card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Tap,
                Cost::PayLife(1),
                Cost::SacrificeSelf,
            ]),
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        has_subtypes: vec![
                            SubType("Forest".to_string()),
                            SubType("Island".to_string()),
                        ],
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

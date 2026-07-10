// Bountiful Landscape — Land
// {T}: Add {C}.
// {T}, Sacrifice: Search library for a basic Forest, Island, or Mountain card,
// put it onto the battlefield tapped, then shuffle.
// Cycling {G}{U}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bountiful-landscape"),
        name: "Bountiful Landscape".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Search your library for a basic Forest, Island, or Mountain card, put it onto the battlefield tapped, then shuffle.\nCycling {G}{U}{R} ({G}{U}{R}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {T}, Sacrifice: Search for basic Forest, Island, or Mountain.
            // has_subtypes with OR semantics + basic: true matches basic lands with
            // any of the listed subtypes (Forest, Island, Mountain).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::SacrificeSelf]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            basic: true,
                            has_card_type: Some(CardType::Land),
                            has_subtypes: vec![
                                SubType("Forest".to_string()),
                                SubType("Island".to_string()),
                                SubType("Mountain".to_string()),
                            ],
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
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
            },
            // Cycling {G}{U}{R}
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { green: 1, blue: 1, red: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}

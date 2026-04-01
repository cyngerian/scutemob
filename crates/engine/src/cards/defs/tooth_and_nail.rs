// Tooth and Nail — {5}{G}{G}, Sorcery
// Choose one — Search library for up to two creature cards, reveal, put in hand.
// / Put up to two creature cards from hand onto battlefield.
// Entwine {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tooth-and-nail"),
        name: "Tooth and Nail".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n• Search your library for up to two creature cards, reveal them, put them into your hand, then shuffle.\n• Put up to two creature cards from your hand onto the battlefield.\nEntwine {2} (Choose both if you pay the entwine cost.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Search for up to two creatures.
                        // TODO: "up to two" search — SearchLibrary finds one card.
                        Effect::SearchLibrary {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                ..Default::default()
                            },
                            destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                            reveal: true,
                            player: PlayerTarget::Controller,
                            also_search_graveyard: false,
                            shuffle_before_placing: false,
                        },
                        // Mode 1: Put up to two creatures from hand onto battlefield.
                        // TODO: "from hand onto battlefield" — needs MoveZone from hand.
                        Effect::Nothing,
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

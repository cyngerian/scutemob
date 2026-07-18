// Perilous Forays — {3}{G}{G}, Enchantment
// "{1}, Sacrifice a creature: Search your library for a land card with a basic land type,
// put it onto the battlefield tapped, then shuffle."
// CR 305.8: "land card with a basic land type" means any land with a
// Plains/Island/Swamp/Mountain/Forest subtype — includes nonbasic lands with a basic land
// type (e.g. Tropical Island), not just cards with the Basic supertype.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("perilous-forays"),
        name: "Perilous Forays".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{1}, Sacrifice a creature: Search your library for a land card with a basic \
                      land type, put it onto the battlefield tapped, then shuffle."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
                Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
            ]),
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        has_subtypes: vec![
                            SubType("Plains".to_string()),
                            SubType("Island".to_string()),
                            SubType("Swamp".to_string()),
                            SubType("Mountain".to_string()),
                            SubType("Forest".to_string()),
                        ],
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: true },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            timing_restriction: None,
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

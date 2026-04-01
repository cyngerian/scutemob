// Buried Alive — {2}{B}, Sorcery
// Search your library for up to three creature cards, put them into your
// graveyard, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("buried-alive"),
        name: "Buried Alive".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for up to three creature cards, put them into your graveyard, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "up to three" — SearchLibrary finds one card. Using one creature
            // to graveyard as approximation.
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                destination: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                reveal: false,
                player: PlayerTarget::Controller,
                also_search_graveyard: false,
                shuffle_before_placing: false,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

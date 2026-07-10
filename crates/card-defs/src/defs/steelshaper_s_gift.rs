// Steelshaper's Gift — {W}, Sorcery
// Search your library for an Equipment card, reveal it, put it into your hand,
// then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("steelshapers-gift"),
        name: "Steelshaper's Gift".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for an Equipment card, reveal that card, put it into your hand, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_subtype: Some(SubType("Equipment".to_string())),
                    ..Default::default()
                },
                destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                reveal: true,
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

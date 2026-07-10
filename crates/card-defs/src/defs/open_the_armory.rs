// Open the Armory — {1}{W}, Sorcery
// Search your library for an Aura or Equipment card, reveal it, put it into
// your hand, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("open-the-armory"),
        name: "Open the Armory".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_subtypes: vec![SubType("Aura".to_string()), SubType("Equipment".to_string())],
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

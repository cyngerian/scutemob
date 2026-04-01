// Mystical Tutor — {U}, Instant
// Search your library for an instant or sorcery card, reveal it, then shuffle
// and put that card on top.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystical-tutor"),
        name: "Mystical Tutor".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for an instant or sorcery card, reveal it, then shuffle and put that card on top of your library.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_card_types: vec![CardType::Instant, CardType::Sorcery],
                    ..Default::default()
                },
                destination: ZoneTarget::Library {
                    owner: PlayerTarget::Controller,
                    position: LibraryPosition::Top,
                },
                reveal: true,
                player: PlayerTarget::Controller,
                also_search_graveyard: false,
                shuffle_before_placing: true,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

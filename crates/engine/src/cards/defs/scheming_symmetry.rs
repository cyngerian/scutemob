// Scheming Symmetry — {B}, Sorcery
// Choose an opponent. You and that player each search your libraries for a card,
// then shuffle and put that card on top.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scheming-symmetry"),
        name: "Scheming Symmetry".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose an opponent. You and that player each search your libraries for a card, then shuffle and put that card on top.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // "You search" part — the opponent search needs player choice (M10).
            effect: Effect::SearchLibrary {
                filter: TargetFilter::default(),
                destination: ZoneTarget::Library {
                    owner: PlayerTarget::Controller,
                    position: LibraryPosition::Top,
                },
                reveal: false,
                player: PlayerTarget::Controller,
                also_search_graveyard: false,
                shuffle_before_placing: true,
            },
            // TODO: Opponent also searches — needs second SearchLibrary for chosen opponent.
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

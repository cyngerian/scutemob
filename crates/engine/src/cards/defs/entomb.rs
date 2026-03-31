// Entomb — {B}, Instant; search your library for a card, put it into your graveyard, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("entomb"),
        name: "Entomb".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a card, put that card into your graveyard, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    reveal: false,
                    destination: ZoneTarget::Graveyard { owner: PlayerTarget::Controller },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

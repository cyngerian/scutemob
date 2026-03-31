// Thirsting Roots — {G} Sorcery; choose one:
// • Search your library for a basic land, reveal it, put it into your hand, then shuffle.
// • Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thirsting-roots"),
        name: "Thirsting Roots".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one —\n• Search your library for a basic land card, reveal it, put it into your hand, then shuffle.\n• Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Search library for a basic land, reveal it, put into hand, then shuffle.
                    Effect::Sequence(vec![
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: basic_land_filter(),
                            reveal: true,
                            destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                            shuffle_before_placing: false,
                            also_search_graveyard: false,
                        },
                        Effect::Shuffle { player: PlayerTarget::Controller },
                    ]),
                    // Mode 1: Proliferate.
                    Effect::Proliferate,
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

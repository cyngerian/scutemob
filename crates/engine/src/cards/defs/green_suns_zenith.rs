// Green Sun's Zenith — {X}{G}, Sorcery
// Search your library for a green creature card with mana value X or less, put
// it onto the battlefield, then shuffle. Shuffle Green Sun's Zenith into its
// owner's library instead of putting it anywhere else.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("green-suns-zenith"),
        name: "Green Sun's Zenith".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for a green creature card with mana value X or less, put it onto the battlefield, then shuffle. Shuffle Green Sun's Zenith into its owner's library instead of putting it anywhere else.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "mana value X or less" — max_cmc should be XValue, not fixed.
            // TODO: "shuffle into library instead of graveyard" replacement.
            effect: Effect::SearchLibrary {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    colors: Some([Color::Green].into_iter().collect()),
                    ..Default::default()
                },
                destination: ZoneTarget::Battlefield { tapped: false },
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

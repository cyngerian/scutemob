// Chord of Calling — {X}{G}{G}{G}, Instant
// Convoke
// Search your library for a creature card with mana value X or less, put it
// onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chord-of-calling"),
        name: "Chord of Calling".to_string(),
        mana_cost: Some(ManaCost { green: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nSearch your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            AbilityDefinition::Spell {
                // TODO: "mana value X or less" — max_cmc should be XValue.
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
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
            },
        ],
        ..Default::default()
    }
}

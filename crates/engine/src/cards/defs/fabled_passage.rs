// Fabled Passage — Land.
// "{T}, Sacrifice Fabled Passage: Search your library for a basic land card, put it
// onto the battlefield tapped, then shuffle. Then if you control four or more lands,
// untap it."
// Simplification: the "four or more lands" untap condition is deferred (no Conditional
// on destination in the DSL). Modeled as always-tapped basic land fetch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fabled-passage"),
        name: "Fabled Passage".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Sacrifice Fabled Passage: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Then if you control four or more lands, untap it.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}

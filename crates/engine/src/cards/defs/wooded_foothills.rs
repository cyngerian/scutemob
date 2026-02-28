// Wooded Foothills — Land.
// "{T}, Pay 1 life, Sacrifice Wooded Foothills: Search your library for a Mountain
// or Forest card, put it onto the battlefield tapped, then shuffle."
// CR 701.15: Fetchland — activated sacrifice ability that searches for a land.
// Simplification: filter uses basic_land_filter() (basic lands only). The actual
// card can fetch nonbasic dual lands with Mountain/Forest subtypes, but the engine
// TargetFilter cannot express "subtype Mountain OR subtype Forest" with a single filter.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wooded-foothills"),
        name: "Wooded Foothills".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life, Sacrifice Wooded Foothills: Search your library for a Mountain or Forest card, put it onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(1),
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

// Bloodstained Mire — Land.
// "{T}, Pay 1 life, Sacrifice Bloodstained Mire: Search your library for a Swamp
// or Mountain card, put it onto the battlefield tapped, then shuffle."
// Same fetchland pattern as Wooded Foothills; simplified to basic_land_filter().
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodstained-mire"),
        name: "Bloodstained Mire".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life, Sacrifice Bloodstained Mire: Search your library for a Swamp or Mountain card, put it onto the battlefield tapped, then shuffle.".to_string(),
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

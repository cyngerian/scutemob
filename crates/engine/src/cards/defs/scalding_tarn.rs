// Scalding Tarn — Land.
// "{T}, Pay 1 life, Sacrifice Scalding Tarn: Search your library for an Island
// or Mountain card, put it onto the battlefield tapped, then shuffle."
// Same fetchland pattern; simplified to basic_land_filter().
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scalding-tarn"),
        name: "Scalding Tarn".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life, Sacrifice Scalding Tarn: Search your library for an Island or Mountain card, put it onto the battlefield tapped, then shuffle.".to_string(),
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

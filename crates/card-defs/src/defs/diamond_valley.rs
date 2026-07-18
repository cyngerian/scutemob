// Diamond Valley — Land
// {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.
//
// PB-EF10: EffectAmount::ToughnessOfSacrificedCreature (CR 608.2b/608.2i LKI) reads the
// layer-resolved toughness of the sacrificed creature captured at cost-payment time.
// Note: Diamond Valley has no {T}: Add {C} ability — it only produces life via the sacrifice
// outlet. It cannot tap for mana at all.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("diamond-valley"),
        name: "Diamond Valley".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Sacrifice a creature: You gain life equal to the sacrificed creature's \
                      toughness."
            .to_string(),
        abilities: vec![
            // CR 602.2 + CR 608.2b/608.2i: {T}, Sacrifice a creature: gain life equal to
            // the sacrificed creature's LKI toughness.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::ToughnessOfSacrificedCreature,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}

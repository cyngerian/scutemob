// Miren, the Moaning Well — Legendary Land
// {T}: Add {C}.
// {3}, {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.
//
// PB-EF10: EffectAmount::ToughnessOfSacrificedCreature (CR 608.2b/608.2i LKI) reads the
// layer-resolved toughness of the sacrificed creature captured at cost-payment time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miren-the-moaning-well"),
        name: "Miren, the Moaning Well".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}, Sacrifice a creature: You gain life equal to the \
                      sacrificed creature's toughness."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 602.2 + CR 608.2b/608.2i: {3}, {T}, Sacrifice a creature: gain life equal
            // to the sacrificed creature's LKI toughness.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 3,
                        ..Default::default()
                    }),
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

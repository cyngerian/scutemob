// The Great Henge — {7}{G}{G}, Legendary Artifact
// This spell costs {X} less to cast, where X is the greatest power among creatures you control.
// {T}: Add {G}{G}. You gain 2 life.
// Whenever a nontoken creature you control enters, put a +1/+1 counter on it and draw a card.
//
// TODO: SelfCostReduction::GreatestPowerAmongCreatures — DSL has TotalPowerOfCreatures (Ghalta)
//   but not greatest-power. Cost reduction deferred.
// TODO: "nontoken creature" filter — TargetFilter lacks non_token field.
//   Using unfiltered creature trigger (includes tokens — slightly wrong).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-great-henge"),
        name: "The Great Henge".to_string(),
        mana_cost: Some(ManaCost { generic: 7, green: 2, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "This spell costs {X} less to cast, where X is the greatest power among creatures you control.\n{T}: Add {G}{G}. You gain 2 life.\nWhenever a nontoken creature you control enters, put a +1/+1 counter on it and draw a card.".to_string(),
        abilities: vec![
            // {T}: Add {G}{G}. You gain 2 life.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 2, 0),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // Whenever a creature you control enters, +1/+1 counter + draw
            // TODO: should be nontoken only (TargetFilter lacks non_token)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

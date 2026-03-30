// The One Ring — {4}, Legendary Artifact
// Indestructible
// When The One Ring enters, if you cast it, you gain protection from everything
// until your next turn.
// At the beginning of your upkeep, you lose 1 life for each burden counter on
// The One Ring.
// {T}: Put a burden counter on The One Ring, then draw a card for each burden
// counter on The One Ring.
//
// CR 603.4: WasCast intervening-if on the ETB trigger.
// CR 611.2b: UntilYourNextTurn duration on GrantPlayerProtection.
// CR 117.12: CounterCount for burden-based life loss and draw count.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-one-ring"),
        name: "The One Ring".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Indestructible\nWhen The One Ring enters, if you cast it, you gain protection from everything until your next turn.\nAt the beginning of your upkeep, you lose 1 life for each burden counter on The One Ring.\n{T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // CR 702.16j: "When The One Ring enters, if you cast it, you gain protection
            // from everything until your next turn."
            // CR 603.4: WasCast intervening-if — only triggers when cast, not on flicker/reanimate.
            // CR 611.2b: UntilYourNextTurn duration — protection expires at start of your next turn.
            // Note: PlayerId(0) is a placeholder; the engine binds the actual controller
            // when the trigger fires (per the effect's PlayerTarget::Controller resolution).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GrantPlayerProtection {
                    player: PlayerTarget::Controller,
                    qualities: vec![ProtectionQuality::FromAll],
                    duration: Some(crate::state::EffectDuration::UntilYourNextTurn(
                        crate::state::player::PlayerId(0),
                    )),
                },
                intervening_if: Some(Condition::WasCast),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // At the beginning of your upkeep, lose 1 life for each burden counter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::CounterCount {
                        target: EffectTarget::Source,
                        counter: CounterType::Custom("burden".to_string()),
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // {T}: Put a burden counter, then draw cards equal to burden counter count.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::Custom("burden".to_string()),
                        count: 1,
                    },
                    // Draw cards equal to number of burden counters (after adding one).
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::CounterCount {
                            target: EffectTarget::Source,
                            counter: CounterType::Custom("burden".to_string()),
                        },
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

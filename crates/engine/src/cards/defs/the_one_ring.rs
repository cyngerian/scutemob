// The One Ring — {4}, Legendary Artifact
// Indestructible
// When The One Ring enters, if you cast it, you gain protection from everything
// until your next turn.
// At the beginning of your upkeep, you lose 1 life for each burden counter on
// The One Ring.
// {T}: Put a burden counter on The One Ring, then draw a card for each burden
// counter on The One Ring.
//
// TODO: "Protection from everything until your next turn" — ETB-if-cast + temp protection.
// TODO: Upkeep life loss scaling with burden counter count.
// TODO: Draw cards equal to burden counter count — EffectAmount lacks counter-based variant.
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
            // TODO: ETB "if you cast it" protection from everything until next turn.
            // {T}: Put a burden counter, then draw cards equal to burden counters.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::Custom("burden".to_string()),
                        count: 1,
                    },
                    // TODO: Draw cards equal to burden counter count.
                    // Using Fixed(1) as approximation.
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

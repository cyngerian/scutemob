// Nezahal, Primal Tide — {5}{U}{U}, Legendary Creature — Elder Dinosaur 7/7
// This spell can't be countered.
// You have no maximum hand size.
// Whenever an opponent casts a noncreature spell, draw a card.
// Discard three cards: Exile Nezahal. Return it to the battlefield tapped under its
// owner's control at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nezahal-primal-tide"),
        name: "Nezahal, Primal Tide".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dinosaur"],
        ),
        oracle_text: "This spell can't be countered.\nYou have no maximum hand size.\nWhenever an opponent casts a noncreature spell, draw a card.\nDiscard three cards: Exile Nezahal, Primal Tide. Return it to the battlefield tapped under its owner's control at the beginning of the next end step.".to_string(),
        power: Some(7),
        toughness: Some(7),
        cant_be_countered: true,
        abilities: vec![
            // TODO: "No maximum hand size" static not in DSL.
            // Whenever an opponent casts a noncreature spell, draw a card
            // Noncreature filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverOpponentCastsSpell {
                    spell_type_filter: None,
                    noncreature_only: true,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Discard three cards: Exile Nezahal. Return it to the battlefield tapped
            // under its owner's control at the beginning of the next end step.
            AbilityDefinition::Activated {
                // Discard three cards as a cost — DSL has no DiscardCards(N) for N > 1.
                // Using three DiscardCard costs in a Sequence to approximate.
                cost: Cost::Sequence(vec![
                    Cost::DiscardCard,
                    Cost::DiscardCard,
                    Cost::DiscardCard,
                ]),
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::Source,
                    return_timing: crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                    return_tapped: true,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
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

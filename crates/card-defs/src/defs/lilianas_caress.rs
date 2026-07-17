// Liliana's Caress — {1}{B}, Enchantment
// Whenever an opponent discards a card, that player loses 2 life.
//
// Both clauses implemented. `TriggerCondition::WheneverOpponentDiscards` -> builder arm in
// `enrich_spec_from_def` -> `TriggerEvent::OpponentDiscards` dispatch, which tags
// `triggering_player`, so `PlayerTarget::TriggeringPlayer` resolves to the discarding
// opponent. Oracle says "loses 2 life", so `Effect::LoseLife` is exact (contrast megrim.rs,
// which says "deals 2 damage" and cannot use LoseLife -- CR 119.3).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lilianas-caress"),
        name: "Liliana's Caress".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, that player loses 2 life.".to_string(),
        abilities: vec![
            // Whenever an opponent discards a card, that player loses 2 life.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverOpponentDiscards,
                effect: Effect::LoseLife {
                    player: PlayerTarget::TriggeringPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

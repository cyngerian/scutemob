// Glint-Horn Buccaneer — {1}{R}{R}, Creature — Minotaur Pirate 2/4
// Haste
// Whenever you discard a card, this creature deals 1 damage to each opponent.
// {1}{R}, Discard a card: Draw a card. Activate only if this creature is attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glint-horn-buccaneer"),
        name: "Glint-Horn Buccaneer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Minotaur", "Pirate"]),
        oracle_text: "Haste\nWhenever you discard a card, this creature deals 1 damage to each opponent.\n{1}{R}, Discard a card: Draw a card. Activate only if this creature is attacking.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Whenever you discard a card, deal 1 damage to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDiscard,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "{1}{R}, Discard a card: Draw a card. Activate only if attacking."
            // Requires activation condition (is_attacking) + discard as cost.
        ],
        ..Default::default()
    }
}

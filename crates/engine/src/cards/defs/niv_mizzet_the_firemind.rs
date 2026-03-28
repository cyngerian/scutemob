// Niv-Mizzet, the Firemind — {2}{U}{U}{R}{R} Legendary Creature — Dragon Wizard 4/4
// Flying
// Whenever you draw a card, Niv-Mizzet deals 1 damage to any target.
// {T}: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("niv-mizzet-the-firemind"),
        name: "Niv-Mizzet, the Firemind".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Wizard"],
        ),
        oracle_text: "Flying\nWhenever you draw a card, Niv-Mizzet, the Firemind deals 1 damage to any target.\n{T}: Draw a card.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you draw a card, deal 1 damage to any target.
            // TODO: "any target" means player or creature — using each opponent as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
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
            // {T}: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}

// Goblin Chainwhirler — {R}{R}{R}, Creature — Goblin Warrior 3/3
// First strike
// When this creature enters, it deals 1 damage to each opponent and each creature and
// planeswalker they control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-chainwhirler"),
        name: "Goblin Chainwhirler".to_string(),
        mana_cost: Some(ManaCost { red: 3, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "First strike\nWhen this creature enters, it deals 1 damage to each opponent and each creature and planeswalker they control.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // ETB: deal 1 to each opponent and each creature and planeswalker they control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    // Deal 1 damage to each opponent.
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    // Deal 1 damage to each creature opponents control.
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::Opponent,
                            ..Default::default()
                        }),
                        effect: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    // Deal 1 damage to each planeswalker opponents control.
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(TargetFilter {
                            has_card_type: Some(CardType::Planeswalker),
                            controller: TargetController::Opponent,
                            ..Default::default()
                        }),
                        effect: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
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

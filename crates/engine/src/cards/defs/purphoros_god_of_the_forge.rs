// Purphoros, God of the Forge — {3}{R}, Legendary Enchantment Creature — God 6/5
// Indestructible
// As long as your devotion to red is less than five, Purphoros isn't a creature.
// Whenever another creature you control enters, Purphoros deals 2 damage to each opponent.
// {2}{R}: Creatures you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("purphoros-god-of-the-forge"),
        name: "Purphoros, God of the Forge".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to red is less than five, Purphoros isn't a creature.\nWhenever another creature you control enters, Purphoros deals 2 damage to each opponent.\n{2}{R}: Creatures you control get +1/+0 until end of turn.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: "As long as your devotion to red is less than five, Purphoros isn't a creature."
            // Requires devotion count + type-changing continuous effect (Layer 4).
            // Whenever another creature you control enters, deal 2 to each opponent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "{2}{R}: Creatures you control get +1/+0 until end of turn."
            // Requires activated ability that applies a continuous effect to all your creatures.
        ],
        ..Default::default()
    }
}

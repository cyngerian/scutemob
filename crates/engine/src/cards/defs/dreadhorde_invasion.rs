// Dreadhorde Invasion — {1}{B}, Enchantment; upkeep trigger loses 1 life and amasses Zombies 1.
// Second ability (Zombie token with 6+ power attacks → lifelink) is deferred: no DSL support
// for attack triggers filtered by token type + power threshold.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dreadhorde-invasion"),
        name: "Dreadhorde Invasion".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, you lose 1 life and amass Zombies 1. (Put a +1/+1 counter on an Army you control. It's also a Zombie. If you don't control an Army, create a 0/0 black Zombie Army creature token first.)\nWhenever a Zombie token you control with power 6 or greater attacks, it gains lifelink until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::Amass {
                        subtype: "Zombie".to_string(),
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Whenever a Zombie token you control with power 6 or greater attacks,
            // it gains lifelink until end of turn." — needs attack trigger with token-type
            // filter (Zombie) and power threshold (≥6) + grant Lifelink until EOT effect.
            // No DSL support for this pattern yet.
        ],
        ..Default::default()
    }
}

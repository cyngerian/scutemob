// Dreadhorde Invasion — {1}{B}, Enchantment; upkeep trigger loses 1 life and amasses Zombies 1.
// Second ability: whenever a Zombie token you control with power 6+ attacks, it gains
// lifelink until end of turn — EffectFilter::TriggeringCreature (PB-EF4) aims the grant
// at the attacking Zombie.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dreadhorde-invasion"),
        name: "Dreadhorde Invasion".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, you lose 1 life and amass Zombies 1. (Put \
                      a +1/+1 counter on an Army you control. It's also a Zombie. If you don't \
                      control an Army, create a 0/0 black Zombie Army creature token \
                      first.)\nWhenever a Zombie token you control with power 6 or greater \
                      attacks, it gains lifelink until end of turn."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                once_per_turn: false,
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
            // CR 508.1m / CR 611.2a: "Whenever a Zombie token you control with power 6 or
            // greater attacks, it gains lifelink until end of turn." The attack-trigger
            // filter restricts to Zombie tokens with power >= 6; the continuous grant
            // targets the attacking creature via EffectFilter::TriggeringCreature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Zombie".to_string())),
                        min_power: Some(6),
                        is_token: true,
                        ..Default::default()
                    }),
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                        filter: EffectFilter::TriggeringCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
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

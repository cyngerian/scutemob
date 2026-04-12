// Dromoka, the Eternal — {3}{G}{W}, Legendary Creature — Dragon 5/5
// Flying
// Whenever a Dragon you control attacks, bolster 2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dromoka-the-eternal"),
        name: "Dromoka, the Eternal".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever a Dragon you control attacks, bolster 2.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 508.1m / CR 603.2: "Whenever a Dragon you control attacks, bolster 2."
            // PB-N: Dragon subtype filter now available via triggering_creature_filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Bolster {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
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

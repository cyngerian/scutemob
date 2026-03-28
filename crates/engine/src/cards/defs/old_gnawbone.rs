// Old Gnawbone — {5}{G}{G}, Legendary Creature — Dragon 7/7
// Flying
// Whenever a creature you control deals combat damage to a player, create that many
// Treasure tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("old-gnawbone"),
        name: "Old Gnawbone".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever a creature you control deals combat damage to a player, create that many Treasure tokens.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 510.3a: "Whenever a creature you control deals combat damage to a player,
            // create that many Treasure tokens." — per-creature trigger with Repeat for variable count.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None },
                effect: Effect::Repeat {
                    effect: Box::new(Effect::CreateToken {
                        spec: treasure_token_spec(1),
                    }),
                    count: EffectAmount::CombatDamageDealt,
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

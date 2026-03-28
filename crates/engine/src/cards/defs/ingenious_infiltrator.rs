// Ingenious Infiltrator — {2}{U}{B}, Creature — Vedalken Ninja 2/3
// Ninjutsu {U}{B}
// Whenever a Ninja you control deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ingenious-infiltrator"),
        name: "Ingenious Infiltrator".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vedalken", "Ninja"]),
        oracle_text: "Ninjutsu {U}{B}\nWhenever a Ninja you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { blue: 1, black: 1, ..Default::default() },
            },
            // CR 510.3a: "Whenever a Ninja you control deals combat damage to a player,
            // draw a card." — per-creature trigger with Ninja subtype filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Ninja".to_string())),
                        ..Default::default()
                    }),
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
        ],
        ..Default::default()
    }
}

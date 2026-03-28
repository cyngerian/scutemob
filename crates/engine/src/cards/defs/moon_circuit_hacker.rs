// Moon-Circuit Hacker — {1}{U}, Enchantment Creature — Human Ninja 2/1
// Ninjutsu {U}
// Whenever this creature deals combat damage to a player, you may draw a card.
// If you do, discard a card unless this creature entered this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moon-circuit-hacker"),
        name: "Moon-Circuit Hacker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Human", "Ninja"]),
        oracle_text: "Ninjutsu {U}\nWhenever this creature deals combat damage to a player, you may draw a card. If you do, discard a card unless this creature entered this turn.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { blue: 1, ..Default::default() },
            },
            // Combat damage: draw (conditional discard simplified away)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
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

// Megrim — {2}{B}, Enchantment
// Whenever an opponent discards a card, this enchantment deals 2 damage to that player.
//
// TODO: Requires TriggerCondition::WheneverOpponentDiscards which does not exist in the DSL.
// The damage target "that player" also requires the trigger to pass the discarding player
// as a target reference, which is not supported. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("megrim"),
        name: "Megrim".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, this enchantment deals 2 damage to that player.".to_string(),
        abilities: vec![
            // Whenever an opponent discards a card, deal 2 damage to that player.
            // Using LoseLife as approximation (damage vs life-loss semantic difference minor).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverOpponentDiscards,
                effect: Effect::LoseLife {
                    player: PlayerTarget::TriggeringPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong("'deals 2 damage to that player' is modeled as Effect::LoseLife; life loss is not damage (CR 119.3) — unpreventable, unredirectable, triggers no damage triggers, ignores lifelink. STALE: the old claim that TriggerCondition::WheneverOpponentDiscards does not exist — it exists and this def already uses it."),
        ..Default::default()
    }
}

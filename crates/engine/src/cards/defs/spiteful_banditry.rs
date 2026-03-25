// Spiteful Banditry — {X}{R}{R}, Enchantment
// When this enchantment enters, it deals X damage to each creature.
// Whenever one or more creatures your opponents control die, you create a Treasure
// token. This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spiteful-banditry"),
        name: "Spiteful Banditry".to_string(),
        mana_cost: Some(ManaCost { red: 2, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, it deals X damage to each creature.\nWhenever one or more creatures your opponents control die, you create a Treasure token. This ability triggers only once each turn.".to_string(),
        abilities: vec![
            // CR 107.3m: "When this enchantment enters, it deals X damage to each creature."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreature,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::XValue,
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Whenever one or more creatures your opponents control die, create a
            // Treasure token. This ability triggers only once each turn." — The
            // "only once each turn" per-turn throttle is not expressible in the current DSL.
            // Deferred until trigger throttle support is added.
        ],
        ..Default::default()
    }
}

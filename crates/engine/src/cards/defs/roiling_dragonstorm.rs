// Roiling Dragonstorm — {1}{U}, Enchantment
// When this enchantment enters, draw two cards, then discard a card.
// When a Dragon you control enters, return this enchantment to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roiling-dragonstorm"),
        name: "Roiling Dragonstorm".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, draw two cards, then discard a card.\nWhen a Dragon you control enters, return this enchantment to its owner's hand.".to_string(),
        abilities: vec![
            // ETB: draw 2, then discard 1
            // TODO: "then discard a card" — forced discard not easily expressible.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "When a Dragon you control enters, return this to hand" —
            //   Dragon-ETB self-bounce trigger not expressible.
        ],
        ..Default::default()
    }
}

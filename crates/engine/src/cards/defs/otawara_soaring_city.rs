// Otawara, Soaring City — Legendary Land, {T}: Add {U}; Channel ability (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("otawara-soaring-city"),
        name: "Otawara, Soaring City".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {U}.\nChannel — {3}{U}, Discard this card: Return target artifact, creature, enchantment, or planeswalker to its owner's hand. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            // {T}: Add {U}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
            // TODO: Channel ability — discard cost + variable cost reduction + multi-type
            // targeting (artifact/creature/enchantment/planeswalker) not expressible in DSL
        ],
        ..Default::default()
    }
}

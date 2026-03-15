// 59. Scroll Thief — {2U}, Creature — Merfolk Rogue 1/3;
// Whenever this creature deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scroll-thief"),
        name: "Scroll Thief".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Merfolk", "Rogue"]),
        oracle_text: "Whenever this creature deals combat damage to a player, draw a card."
            .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
        }],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

// Mask of Memory — {2}, Artifact — Equipment
// Whenever equipped creature deals combat damage to a player, you may draw two cards.
// If you do, discard a card.
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mask-of-memory"),
        name: "Mask of Memory".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever equipped creature deals combat damage to a player, you may draw two cards. If you do, discard a card.\nEquip {1}".to_string(),
        abilities: vec![
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // draw two cards, then discard a card." (approximation — "may" draw 2 not in DSL)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(2),
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

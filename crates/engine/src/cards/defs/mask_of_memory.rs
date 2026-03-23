// Mask of Memory — {2}, Artifact — Equipment
// Whenever equipped creature deals combat damage to a player, you may draw two cards.
// If you do, discard a card.
// Equip {1}
//
// TODO: "Whenever equipped creature deals combat damage" — equipped-creature trigger
//   not in DSL. Needs WhenEquippedCreatureDealsCombatDamageToPlayer.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mask-of-memory"),
        name: "Mask of Memory".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever equipped creature deals combat damage to a player, you may draw two cards. If you do, discard a card.\nEquip {1}".to_string(),
        abilities: vec![
            // TODO: Equipped-creature combat damage trigger not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

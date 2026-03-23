// Sigil of Sleep — {U}, Enchantment — Aura
// Enchant creature
// Whenever enchanted creature deals damage to a player, return target creature that
// player controls to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sigil-of-sleep"),
        name: "Sigil of Sleep".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage to a player, return target creature that player controls to its owner's hand.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // TODO: "Whenever enchanted creature deals damage to a player" — needs
            // attached-creature damage trigger. "return target creature that player controls"
            // — needs damaged-player-controlled targeting + MoveZone to hand.
        ],
        ..Default::default()
    }
}

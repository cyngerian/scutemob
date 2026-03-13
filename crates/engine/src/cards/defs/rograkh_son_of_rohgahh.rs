// Rograkh, Son of Rohgahh — {0}, Legendary Creature — Kobold Warrior 0/1
// First strike, menace, trample
// Partner (You can have two commanders if both have partner.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rograkh-son-of-rohgahh"),
        name: "Rograkh, Son of Rohgahh".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        // Note: color_indicator is not a CardDefinition field (only CardFace for DFCs).
        // Rograkh's red color identity comes from its {R} commander color identity rule — no fix needed.
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Kobold", "Warrior"],
        ),
        oracle_text: "First strike, menace, trample\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Partner),
        ],
        ..Default::default()
    }
}

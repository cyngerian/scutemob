// Nyxbloom Ancient — {4}{G}{G}{G}, Enchantment Creature — Elemental
// Trample
// If you tap a permanent for mana, it produces three times as much of that mana instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nyxbloom-ancient"),
        name: "Nyxbloom Ancient".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 3, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Elemental"]),
        oracle_text: "Trample\nIf you tap a permanent for mana, it produces three times as much of that mana instead.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 106.12b: "If you tap a permanent for mana, it produces three times as much."
            // Replacement effect: multiplies mana produced by {T}-cost mana abilities by 3.
            // Multiple Nyxbloom Ancients stack multiplicatively (two = 9x, per ruling).
            // Per Nyxbloom ruling: triggered mana abilities (Mirari's Wake, etc.) are NOT affected.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::ManaWouldBeProduced {
                    // PlayerId(0) is a placeholder; bound to controller at ETB registration.
                    controller: PlayerId(0),
                    // No color filter or source filter — applies to all tap-mana (CR 106.12b).
                    color_filter: None,
                    source_filter: None,
                },
                modification: ReplacementModification::MultiplyMana(3),
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

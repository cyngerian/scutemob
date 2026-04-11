// Mana Reflection — {4}{G}{G}, Enchantment
// If you tap a permanent for mana, it produces twice as much of that mana instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-reflection"),
        name: "Mana Reflection".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If you tap a permanent for mana, it produces twice as much of that mana instead.".to_string(),
        abilities: vec![
            // CR 106.12b: "If you tap a permanent for mana, it produces twice as much."
            // Replacement effect: multiplies mana produced by {T}-cost mana abilities by 2.
            // Multiple Mana Reflections stack multiplicatively (two = 4x, per ruling).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::ManaWouldBeProduced {
                    // PlayerId(0) is a placeholder; bound to controller at ETB registration.
                    controller: PlayerId(0),
                    // No color filter or source filter — applies to all tap-mana (CR 106.12b).
                    color_filter: None,
                    source_filter: None,
                },
                modification: ReplacementModification::MultiplyMana(2),
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

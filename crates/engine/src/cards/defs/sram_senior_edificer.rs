// Sram, Senior Edificer — {1}{W}, Legendary Creature — Dwarf Advisor 2/2
// Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card.
//
// ENGINE-BLOCKED: "Aura, Equipment, or Vehicle" are spell subtypes, not CardTypes.
// WheneverYouCastSpell.spell_type_filter accepts Vec<CardType> only. There is no
// spell-subtype filter in the DSL. The unfiltered approximation (draw on every spell)
// produces wrong game state and is omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sram-senior-edificer"),
        name: "Sram, Senior Edificer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dwarf", "Advisor"]),
        oracle_text: "Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // ENGINE-BLOCKED: WheneverYouCastSpell has no spell-subtype filter.
            // Aura, Equipment, and Vehicle are subtypes (not CardTypes), so they cannot
            // be expressed via spell_type_filter: Option<Vec<CardType>>.
        ],
        ..Default::default()
    }
}

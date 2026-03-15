// Birchlore Rangers — {G}, Creature — Elf Ranger 1/1
// Tap two untapped Elves you control: Add one mana of any color.
// Morph {0} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// The tap-two-Elves mana ability is a DSL gap (no multi-tap-creature cost).
// AbilityDefinition::Morph carries the turn-face-up cost {0}.
// KeywordAbility::Morph is the marker for quick presence-checking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birchlore-rangers"),
        name: "Birchlore Rangers".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Elf", "Ranger"]),
        oracle_text:
            "Tap two untapped Elves you control: Add one mana of any color.\n\
             Morph {0}"
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Tap-two-Elves mana ability is a DSL gap — omitted (no multi-tap-creature cost primitive).
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph { cost: ManaCost { ..Default::default() } },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

// Destiny Spinner — {1}{G}, Enchantment Creature — Human 2/3.
// Creature and enchantment spells you control can't be countered.
// {3}{G}: Target land becomes X/X Elemental with trample and haste until EOT, where
// X = number of enchantments you control.
// TODO: DSL gap — "can't be countered" static for specific spell types not expressible
// (AbilityDefinition::Spell has cant_be_countered but only for the card itself, not a
// blanket static grant); land animation with X based on enchantment count not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("destiny-spinner"),
        name: "Destiny Spinner".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Human"]),
        oracle_text: "Creature and enchantment spells you control can't be countered.\n{3}{G}: Target land you control becomes an X/X Elemental creature with trample and haste until end of turn, where X is the number of enchantments you control. It's still a land.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![],
        // TODO: static "can't counter creature/enchantment spells you control"
        // TODO: {3}{G} activated — land animation with enchantment-count X
        ..Default::default()
    }
}

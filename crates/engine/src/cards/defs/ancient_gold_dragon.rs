// Ancient Gold Dragon — {5}{W}{W}, Creature — Elder Dragon 7/10
// Flying
// Whenever this creature deals combat damage to a player, roll a d20. You create a number
// of 1/1 blue Faerie Dragon creature tokens with flying equal to the result.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-gold-dragon"),
        name: "Ancient Gold Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, white: 2, ..Default::default() }),
        types: creature_types(&["Elder", "Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, roll a d20. You create a number of 1/1 blue Faerie Dragon creature tokens with flying equal to the result.".to_string(),
        power: Some(7),
        toughness: Some(10),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Roll a d20. Create a number of tokens equal to the result."
            // DSL gap: d20 results can't produce a variable token count — TokenSpec.count
            // is a fixed i32. A D20 roll producing EffectAmount::D20Result doesn't exist.
            // Wrong to create a fixed count (e.g. 10), so leaving as TODO.
        ],
        ..Default::default()
    }
}

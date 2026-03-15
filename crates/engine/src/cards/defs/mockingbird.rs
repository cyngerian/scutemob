// Mockingbird — {X}{U}, Creature — Bird Bard 1/1
// "Flying
// You may have this creature enter as a copy of any creature on the battlefield with mana value
// less than or equal to the amount of mana spent to cast this creature, except it's a Bird in
// addition to its other types and it has flying."
//
// Flying is implemented.
//
// TODO: DSL gap — the ETB copy effect requires:
// 1. An optional ETB replacement that copies a target creature filtered by mana value <= X.
// 2. Retaining "Bird" subtype and "flying" on the copy.
// No ETB copy variant with X-cost filter or type/keyword overlay exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mockingbird"),
        name: "Mockingbird".to_string(),
        // {X}{U}: x_count = 1 for the {X} symbol.
        mana_cost: Some(ManaCost { blue: 1, x_count: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Bard"]),
        oracle_text: "Flying\nYou may have this creature enter as a copy of any creature on the battlefield with mana value less than or equal to the amount of mana spent to cast this creature, except it's a Bird in addition to its other types and it has flying.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}

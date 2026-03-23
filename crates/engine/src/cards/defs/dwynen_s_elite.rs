// Dwynen's Elite — {1}{G}, Creature — Elf Warrior 2/2
// When this creature enters, if you control another Elf, create a 1/1 green Elf Warrior
// creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dwynen-s-elite"),
        name: "Dwynen's Elite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "When this creature enters, if you control another Elf, create a 1/1 green Elf Warrior creature token.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "If you control another Elf" intervening-if — Condition lacks
            // "you control a permanent with subtype X" variant. Implementing trigger
            // without the condition would create a token even without another Elf,
            // which is wrong behavior. Using TODO per W5 policy.
        ],
        ..Default::default()
    }
}

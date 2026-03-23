// Tyvar the Bellicose — {2}{B}{G}, Legendary Creature — Elf Warrior 5/4
// Whenever one or more Elves you control attack, they gain deathtouch until end of turn.
// Each creature you control has "Whenever a mana ability of this creature resolves, put a
// number of +1/+1 counters on it equal to the amount of mana this creature produced. This
// ability triggers only once each turn."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tyvar-the-bellicose"),
        name: "Tyvar the Bellicose".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "Whenever one or more Elves you control attack, they gain deathtouch until end of turn.\nEach creature you control has \"Whenever a mana ability of this creature resolves, put a number of +1/+1 counters on it equal to the amount of mana this creature produced. This ability triggers only once each turn.\"".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // TODO: DSL gap — "Whenever one or more Elves you control attack" batch trigger
            // with deathtouch grant to the attacking Elves only.
            // TODO: DSL gap — granting triggered abilities to creatures (mana ability trigger).
        ],
        ..Default::default()
    }
}

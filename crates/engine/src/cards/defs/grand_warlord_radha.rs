// Grand Warlord Radha — {2}{R}{G}, Legendary Creature — Elf Warrior 3/4
// Haste
// Whenever one or more creatures you control attack, add that much mana in any combination
// of {R} and/or {G}. Until end of turn, you don't lose this mana as steps and phases end.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grand-warlord-radha"),
        name: "Grand Warlord Radha".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Warrior"]),
        oracle_text: "Haste\nWhenever one or more creatures you control attack, add that much mana in any combination of {R} and/or {G}. Until end of turn, you don't lose this mana as steps and phases end.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: DSL gap — "Whenever one or more creatures you control attack" requires
            // WhenYouDeclareAttackers trigger condition which does not exist in the DSL.
            // TODO: DSL gap — the mana amount equals the number of attacking creatures
            // (dynamic count), and mana persists through phase changes (special mana
            // restriction), neither of which is expressible in the current DSL.
        ],
        ..Default::default()
    }
}

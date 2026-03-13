// Reckless One — {3}{R}, Creature — Goblin Avatar */*
// Haste
// Reckless One's power and toughness are each equal to the number of Goblins on the battlefield.
// TODO: DSL gap — dynamic P/T based on the count of Goblins on the battlefield (all players)
// is not expressible; no CountCreaturesOnBattlefieldWithSubtype EffectAmount exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reckless-one"),
        name: "Reckless One".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Avatar"]),
        oracle_text: "Haste\nReckless One's power and toughness are each equal to the number of Goblins on the battlefield.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: static P/T — power and toughness equal the number of Goblins on the battlefield.
            // DSL gap: no CountCreaturesOnBattlefieldWithSubtype EffectAmount.
        ],
        ..Default::default()
    }
}

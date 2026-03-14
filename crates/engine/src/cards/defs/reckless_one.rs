// Reckless One — {3}{R}, Creature — Goblin Avatar */*
// Haste
// Reckless One's power and toughness are each equal to the number of Goblins on the battlefield.
// TODO: CDA gap — */* requires SetPowerToughness with EffectAmount, not fixed i32.
// PermanentCount({ Creature, Goblin }, EachPlayer) is now available for the count.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reckless-one"),
        name: "Reckless One".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Avatar"]),
        oracle_text: "Haste\nReckless One's power and toughness are each equal to the number of Goblins on the battlefield.".to_string(),
        power: None,   // */*  CDA — engine SBA skips None toughness
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: CDA — P/T = PermanentCount({ Creature + Goblin }, EachPlayer).
            // Needs SetPowerToughness to accept EffectAmount.
        ],
        ..Default::default()
    }
}

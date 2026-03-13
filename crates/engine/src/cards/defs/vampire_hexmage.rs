// Vampire Hexmage — {B}{B}, Creature — Vampire Shaman 2/1
// First strike
// Sacrifice this creature: Remove all counters from target permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-hexmage"),
        name: "Vampire Hexmage".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "First strike\nSacrifice this creature: Remove all counters from target permanent.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // TODO: Activated ability — "Sacrifice this creature: Remove all counters from target permanent."
            // DSL gap: no RemoveAllCounters effect; no self-sacrifice cost.
        ],
        ..Default::default()
    }
}

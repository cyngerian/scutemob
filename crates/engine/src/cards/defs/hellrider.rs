// Hellrider — {2}{R}{R}, Creature — Devil 3/3
// Haste
// Whenever a creature you control attacks, Hellrider deals 1 damage to the player or
// planeswalker it's attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellrider"),
        name: "Hellrider".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Devil"]),
        oracle_text: "Haste\nWhenever a creature you control attacks, Hellrider deals 1 damage to the player or planeswalker it's attacking.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: DSL gap — "Whenever a creature you control attacks" trigger condition does
            // not exist (no WheneverCreatureYouControlAttacks). Additionally, the damage target
            // is "the player or planeswalker IT'S attacking" (the attack target of the
            // triggering creature), which requires referencing the triggering creature's combat
            // assignment — no DSL support for this.
        ],
        ..Default::default()
    }
}

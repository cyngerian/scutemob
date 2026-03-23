// Utvara Hellkite — {6}{R}{R}, Creature — Dragon 6/6
// Flying
// Whenever a Dragon you control attacks, create a 6/6 red Dragon creature token
// with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("utvara-hellkite"),
        name: "Utvara Hellkite".to_string(),
        mana_cost: Some(ManaCost { generic: 6, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever a Dragon you control attacks, create a 6/6 red Dragon creature token with flying.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever a Dragon you control attacks" — DSL lacks a
            //   WheneverCreatureYouControlAttacks trigger (WhenAttacks is self-only).
            //   W5 policy: no approximation.
        ],
        ..Default::default()
    }
}

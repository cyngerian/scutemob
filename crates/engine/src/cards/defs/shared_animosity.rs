// Shared Animosity — {2}{R}, Enchantment
// Whenever a creature you control attacks, it gets +1/+0 until end of turn for each
// other attacking creature that shares a creature type with it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shared-animosity"),
        name: "Shared Animosity".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, it gets +1/+0 until end of turn for each other attacking creature that shares a creature type with it.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature you control attacks" trigger does not exist
            // (no WheneverCreatureYouControlAttacks condition).
            // TODO: DSL gap — the +1/+0 buff amount depends on counting attackers sharing
            // a creature type with the triggering creature (dynamic count, no DSL support).
        ],
        ..Default::default()
    }
}

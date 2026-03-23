// Rakish Heir — {2}{R}, Creature — Vampire 2/2
// Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rakish-heir"),
        name: "Rakish Heir".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: DSL gap — "Whenever a Vampire you control deals combat damage to a player"
            // trigger. WhenDealsCombatDamageToPlayer is self-only, not "a Vampire you control."
        ],
        ..Default::default()
    }
}

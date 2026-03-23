// Grim Hireling — {3}{B}, Creature — Tiefling Rogue 3/2
// Whenever one or more creatures you control deal combat damage to a player, create two
// Treasure tokens.
// {B}, Sacrifice X Treasures: Target creature gets -X/-X until end of turn. Activate only
// as a sorcery.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-hireling"),
        name: "Grim Hireling".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Tiefling", "Rogue"]),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, create two Treasure tokens.\n{B}, Sacrifice X Treasures: Target creature gets -X/-X until end of turn. Activate only as a sorcery.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever one or more creatures you control deal combat damage to a player"
            // — per-creature combat damage trigger with "one or more" grouping is not in DSL.
            // WhenDealsCombatDamage exists only for individual attacker triggers.
            // TODO: "{B}, Sacrifice X Treasures: Target creature gets -X/-X" — X-cost activated
            // ability with variable sacrifice count is not expressible in the DSL.
        ],
        ..Default::default()
    }
}

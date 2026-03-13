// Bloodghast — {B}{B}, Creature — Vampire Spirit 2/1.
// Can't block; has haste if opponent at 10 or less life (conditional static);
// Landfall — whenever a land you control enters, may return from graveyard.
// TODO: DSL gap — "can't block" static ability not expressible; conditional haste
// (opponent life total threshold) not expressible; Landfall return-from-graveyard
// triggered ability from graveyard zone not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodghast"),
        name: "Bloodghast".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Spirit"]),
        oracle_text: "This creature can't block.\nThis creature has haste as long as an opponent has 10 or less life.\nLandfall \u{2014} Whenever a land you control enters, you may return this card from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![],
        // TODO: can't block static; conditional haste (opponent life <= 10); Landfall
        // trigger from graveyard zone
        ..Default::default()
    }
}

// Vampire Gourmand — {1}{B}, Creature — Vampire 2/2
// Whenever this creature attacks, you may sacrifice another creature. If you do, draw a
// card and this creature can't be blocked this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-gourmand"),
        name: "Vampire Gourmand".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever this creature attacks, you may sacrifice another creature. If you do, draw a card and this creature can't be blocked this turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "May sacrifice another creature on attack → draw + can't be blocked"
            //   — optional sacrifice gate + evasion grant not in DSL.
        ],
        ..Default::default()
    }
}

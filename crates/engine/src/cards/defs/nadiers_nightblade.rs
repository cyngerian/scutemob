// Nadier's Nightblade — {2}{B}, Creature — Elf Warrior 1/3
// Whenever a token you control leaves the battlefield, each opponent loses 1 life
// and you gain 1 life.
//
// TODO: "Whenever a token you control leaves the battlefield" — no trigger condition for
//   token-specific zone changes. Using WheneverCreatureDies as approximation (only covers
//   dying tokens, not exile/bounce). Marked as TODO.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nadiers-nightblade"),
        name: "Nadier's Nightblade".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Whenever a token you control leaves the battlefield, each opponent loses 1 life and you gain 1 life.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    }
}

// Blood Seeker — {1}{B}, Creature — Vampire Shaman 1/1
// Whenever a creature an opponent controls enters, you may have that player lose 1 life.
//
// TODO: "that player" — effect should target the entering creature's controller specifically,
//   not all opponents. PlayerTarget lacks "triggering player" reference. Additionally, the
//   effect is optional ("you may") which is not expressible. Wrong multiplayer behavior
//   if implemented with EachOpponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-seeker"),
        name: "Blood Seeker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever a creature an opponent controls enters, you may have that player lose 1 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

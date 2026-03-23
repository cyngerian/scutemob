// Esper Sentinel — {W}, Artifact Creature — Human Soldier 1/1
// Whenever an opponent casts their first noncreature spell each turn, draw a card
// unless that player pays {X}, where X is Esper Sentinel's power.
//
// TODO: Opponent-cast trigger with noncreature filter, once-per-turn,
//   and conditional pay-or-draw. Not expressible in current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("esper-sentinel"),
        name: "Esper Sentinel".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Artifact, CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever an opponent casts their first noncreature spell each turn, draw a card unless that player pays {X}, where X is Esper Sentinel's power.".to_string(),
        power: Some(1),
        toughness: Some(1),
        // TODO: Opponent-cast trigger with noncreature filter, once-per-turn,
        //   and conditional pay-or-draw. Not expressible in current DSL.
        abilities: vec![],
        ..Default::default()
    }
}

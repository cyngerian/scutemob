// Alandra, Sky Dreamer — {2}{U}{U}, Legendary Creature — Merfolk Wizard 2/4
// Whenever you draw your second card each turn, create a 2/2 blue Drake creature
// token with flying.
// Whenever you draw your fifth card each turn, Alandra and Drakes you control each
// get +X/+X until end of turn, where X is the number of cards in your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("alandra-sky-dreamer"),
        name: "Alandra, Sky Dreamer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Merfolk", "Wizard"],
        ),
        oracle_text: "Whenever you draw your second card each turn, create a 2/2 blue Drake creature token with flying.\nWhenever you draw your fifth card each turn, Alandra and Drakes you control each get +X/+X until end of turn, where X is the number of cards in your hand.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: "Whenever you draw your second card each turn" — requires a per-turn
            //   draw-count trigger (fire on Nth draw). WheneverYouDrawACard fires every draw
            //   with no ordinal filter. DSL gap.
            // TODO: "Whenever you draw your fifth card each turn" — same DSL gap.
            //   Also requires X = hand size dynamic buff. W5 policy: no approximation.
        ],
        ..Default::default()
    }
}

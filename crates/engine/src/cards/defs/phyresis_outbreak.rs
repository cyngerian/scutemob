// Phyresis Outbreak — {2}{B}, Sorcery
// Each opponent gets a poison counter. Then each creature your opponents control gets
// -1/-1 until end of turn for each poison counter its controller has.
//
// TODO: The -1/-1 portion requires EffectAmount::CounterCountOf(target_controller, CounterType::Poison)
// — a dynamic amount based on the controlling player's poison counter count. This is not
// in the DSL. Implementing only the "each opponent gets a poison counter" portion would
// produce incomplete game state (missing P/T reduction). W5: abilities empty.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyresis-outbreak"),
        name: "Phyresis Outbreak".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each opponent gets a poison counter. Then each creature your opponents control gets -1/-1 until end of turn for each poison counter its controller has.".to_string(),
        abilities: vec![
            // TODO: "each creature your opponents control gets -1/-1 for each poison counter
            // its controller has" — EffectAmount::CounterCountOf (player's poison count) not
            // in DSL. Cannot implement the -1/-1 portion without the first part also being
            // present (they are linked). W5: omitted entirely.
        ],
        ..Default::default()
    }
}

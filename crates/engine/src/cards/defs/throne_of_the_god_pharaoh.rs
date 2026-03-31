// Throne of the God-Pharaoh — {2}, Legendary Artifact
// At the beginning of your end step, each opponent loses life equal to the number
// of tapped creatures you control.
//
// TODO: "loses life equal to the number of tapped creatures you control" —
// EffectAmount::TappedCreatureCount is not in the DSL. The amount is dynamic
// (count of tapped creatures at trigger resolution) and no equivalent variant exists.
// W5: omitted — Fixed(0) or Fixed(1) would produce wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throne-of-the-god-pharaoh"),
        name: "Throne of the God-Pharaoh".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "At the beginning of your end step, each opponent loses life equal to the number of tapped creatures you control.".to_string(),
        abilities: vec![
            // TODO: EffectAmount::TappedCreatureCount not in DSL.
            // Needed to express "loses life equal to the number of tapped creatures you control".
            // W5: omitted to avoid wrong game state.
        ],
        ..Default::default()
    }
}

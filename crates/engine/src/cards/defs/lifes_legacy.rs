// Life's Legacy — {1}{G}, Sorcery
// "As an additional cost to cast this spell, sacrifice a creature."
// "Draw cards equal to the sacrificed creature's power."
// TODO: DSL gap — two issues:
// 1. Additional cost on a spell (sacrifice a creature as additional cost to cast) is not
//    expressible in the CardDefinition DSL (no spell_additional_cost field for sacrifice).
// 2. EffectAmount::PowerOfSacrificedCreature does not exist. The draw count depends on
//    the power of the sacrificed creature (LKI at cast time).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lifes-legacy"),
        name: "Life's Legacy".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nDraw cards equal to the sacrificed creature's power.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

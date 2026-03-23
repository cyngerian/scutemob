// Greater Good — {2}{G}{G}, Enchantment
// "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards."
// TODO: DSL gap — EffectAmount::PowerOfSacrificedCreature does not exist. The draw count
// depends on the power of the sacrificed creature (LKI). Additionally, "discard three cards"
// requires Effect::DiscardCards which is not in the DSL. Cannot faithfully express either
// part of this ability without new DSL primitives.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("greater-good"),
        name: "Greater Good".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

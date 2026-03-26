// Life's Legacy — {1}{G}, Sorcery
// As an additional cost to cast this spell, sacrifice a creature.
// Draw cards equal to the sacrificed creature's power.
//
// Note: The draw count depends on the sacrificed creature's power (LKI).
// EffectAmount::SacrificedCreaturePower is not yet implemented.
// As a partial fix, sacrifice cost is enforced but draw count is deferred to PB-37.
// For now, this card draws 0 cards (engine does not execute the draw).
// TODO (PB-37): Implement EffectAmount::SacrificedCreaturePower for proper draw count.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lifes-legacy"),
        name: "Life's Legacy".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nDraw cards equal to the sacrificed creature's power.".to_string(),
        // CR 118.8: Mandatory sacrifice of a creature as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        // TODO (PB-37): Draw count equals sacrificed creature's power (LKI).
        // No spell ability until EffectAmount::SacrificedCreaturePower is implemented.
        abilities: vec![],
        ..Default::default()
    }
}

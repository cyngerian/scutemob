// Life's Legacy — {1}{G}, Sorcery
// As an additional cost to cast this spell, sacrifice a creature.
// Draw cards equal to the sacrificed creature's power.
//
// Note: The draw count depends on the sacrificed creature's power (LKI).
// EffectAmount::SacrificedCreaturePower is not yet implemented.
// As a partial fix, sacrifice cost is enforced and a placeholder draw 1 card is used.
// Full draw count (sacrificed creature's power) deferred to PB-37.
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
        // Placeholder: draw 1 card as minimum approximation until
        // EffectAmount::SacrificedCreaturePower is implemented (deferred to PB-37).
        // Drawing 1 is closer to correct than drawing 0 (empty abilities vec).
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

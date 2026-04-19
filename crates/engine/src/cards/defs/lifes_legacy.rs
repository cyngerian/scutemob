// Life's Legacy — {1}{G}, Sorcery
// "As an additional cost to cast this spell, sacrifice a creature.
//  Draw cards equal to the sacrificed creature's power."
//
// CR 118.8: Mandatory additional sacrifice cost at cast time.
// CR 608.2b: Draw count uses the sacrificed creature's LKI power (on-battlefield,
// layer-resolved) captured BEFORE move_object_to_zone at the spell-additional-cost
// sacrifice site (casting.rs). Flows via AdditionalCost::Sacrifice.lki_powers into
// EffectContext.sacrificed_creature_powers at resolution.
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
        abilities: vec![
            // CR 608.2b: Draw count equals the sacrificed creature's LKI power.
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PowerOfSacrificedCreature,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

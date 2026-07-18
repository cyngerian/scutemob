// Momentous Fall — {2}{G}{G}, Instant
// "As an additional cost to cast this spell, sacrifice a creature.
//  You draw cards equal to the sacrificed creature's power, then you gain life
//  equal to its toughness."
//
// CR 118.8: Mandatory additional sacrifice cost at cast time.
// CR 608.2b/608.2i: Both draw count and life gain use the sacrificed creature's LKI
// power/toughness (on-battlefield, layer-resolved) captured BEFORE move_object_to_zone
// at the spell-additional-cost sacrifice site (casting.rs). Flows via
// AdditionalCost::Sacrifice.lki into EffectContext.sacrificed_creature_lki at
// resolution (PB-EF10). Ruling 2010-06-15: the sacrificed creature's last known
// existence on the battlefield is checked for BOTH its power and its toughness.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("momentous-fall"),
        name: "Momentous Fall".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nYou draw \
                      cards equal to the sacrificed creature's power, then you gain life equal \
                      to its toughness."
            .to_string(),
        // CR 118.8: Mandatory sacrifice of a creature as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        abilities: vec![
            // CR 608.2b/608.2i: draw = LKI power, then gain life = LKI toughness.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::PowerOfSacrificedCreature,
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::ToughnessOfSacrificedCreature,
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

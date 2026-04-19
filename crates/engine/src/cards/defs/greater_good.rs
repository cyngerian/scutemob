// Greater Good — {2}{G}{G}, Enchantment
// "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards."
//
// CR 602.2 + CR 701.16 + CR 608.2b: The sacrifice is paid as an activated ability cost.
// The sacrificed creature's power is captured at sacrifice time (LKI) via
// EffectAmount::PowerOfSacrificedCreature. Both draw and discard are non-targeted.
// Note: Effect::DiscardCards exists in the DSL; the prior TODO claiming it was missing was stale.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("greater-good"),
        name: "Greater Good".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards.".to_string(),
        abilities: vec![
            // CR 602.2 + CR 701.16 + CR 608.2b: Sacrifice a creature, draw cards equal to its
            // LKI power, then discard three cards. The Sequence ensures draw happens before discard
            // (so newly drawn cards may be selected for discard per CR 701.7).
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::PowerOfSacrificedCreature,
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

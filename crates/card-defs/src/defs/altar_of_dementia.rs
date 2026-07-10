// Altar of Dementia — {2}, Artifact
// "Sacrifice a creature: Target player mills cards equal to the sacrificed creature's power."
//
// CR 602.2 + CR 701.16 + CR 608.2b: The sacrifice is paid as an activated ability cost.
// The sacrificed creature's power is captured at sacrifice time (LKI) via
// EffectAmount::PowerOfSacrificedCreature and used as the mill count at resolution.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("altar-of-dementia"),
        name: "Altar of Dementia".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Target player mills cards equal to the sacrificed creature's power.".to_string(),
        abilities: vec![
            // CR 602.2 + CR 701.16 + CR 608.2b: Sacrifice a creature, target player mills
            // cards equal to the sacrificed creature's LKI power.
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::MillCards {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::PowerOfSacrificedCreature,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPlayer],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

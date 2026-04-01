// Dispatch — {W}, Instant
// Tap target creature.
// Metalcraft — If you control three or more artifacts, exile that creature.
//
// TODO: Metalcraft conditional — if 3+ artifacts, exile the same target creature instead
//   of just tapping. The Effect::Conditional can check Condition::YouControlNOrMoreOfType
//   for artifacts, but the "exile THAT creature" reference to the declared target within
//   the conditional is the right approach. This is expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dispatch"),
        name: "Dispatch".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Tap target creature.\nMetalcraft — If you control three or more artifacts, exile that creature.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 702.101: Metalcraft — conditional based on controlling 3+ artifacts.
                // Tap the creature first, then conditionally exile it.
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::Conditional {
                        condition: Condition::YouControlNOrMoreWithFilter {
                            count: 3,
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Artifact),
                                ..Default::default()
                            },
                        },
                        if_true: Box::new(Effect::ExileObject {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                        if_false: Box::new(Effect::Nothing),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

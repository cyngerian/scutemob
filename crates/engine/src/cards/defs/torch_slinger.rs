// Torch Slinger {2}{R}
// Creature — Goblin Shaman, 2/2
// Kicker {1}{R}
// When Torch Slinger enters, if it was kicked, it deals 2 damage to
// target creature an opponent controls.
// CR 702.33e: Linked abilities — the ETB trigger is linked to the kicker
// and only fires when the permanent was kicked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("torch-slinger"),
        name: "Torch Slinger".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Goblin".to_string()), SubType("Shaman".to_string())].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Kicker {1}{R}\nWhen Torch Slinger enters, if it was kicked, it deals 2 damage to target creature.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Kicker {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
                is_multikicker: false,
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Conditional {
                    condition: Condition::WasKicked,
                    if_true: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                    if_false: Box::new(Effect::Sequence(vec![])),
                },
                intervening_if: None,
            },
        ],
        back_face: None,
    }
}

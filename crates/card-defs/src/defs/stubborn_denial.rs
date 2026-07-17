// Stubborn Denial — {U}, Instant
// Counter target noncreature spell unless its controller pays {1}.
// Ferocious — If you control a creature with power 4 or greater, counter that
// spell instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stubborn-denial"),
        name: "Stubborn Denial".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell unless its controller pays {1}.\nFerocious \
                      — If you control a creature with power 4 or greater, counter that spell \
                      instead."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-AC2 (CR 118.12a) + Ferocious: "counter unless pays {1}", but if you
            // control a power-4+ creature, counter unconditionally instead.
            effect: Effect::Conditional {
                condition: Condition::YouControlPermanent(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    min_power: Some(4),
                    ..Default::default()
                }),
                if_true: Box::new(Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                }),
                if_false: Box::new(Effect::CounterUnlessPays {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cost: Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                }),
            },
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                non_creature: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

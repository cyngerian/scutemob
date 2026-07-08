// Make Disappear — {1}{U}, Instant; Casualty 1 (As an additional cost, sacrifice a creature
// with power 1 or greater. When you do, copy this spell and you may choose a new target.)
// Counter target spell unless its controller pays {2}.
// CR 702.153: Casualty — additional cost triggers a copy of the spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("make-disappear"),
        name: "Make Disappear".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Casualty 1 (As an additional cost to cast this spell, you may sacrifice a creature with power 1 or greater. When you do, copy this spell, and you may choose a new target for the copy.)\nCounter target spell unless its controller pays {2}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Casualty(1)),
            AbilityDefinition::Spell {
                // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
                effect: Effect::CounterUnlessPays {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter::default())],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

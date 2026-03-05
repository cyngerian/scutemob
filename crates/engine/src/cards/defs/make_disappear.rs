// Make Disappear — {1}{U}, Instant; Casualty 1 (As an additional cost, sacrifice a creature
// with power 1 or greater. When you do, copy this spell and you may choose a new target.)
// Counter target spell unless its controller pays {2}.
// CR 702.153: Casualty — additional cost triggers a copy of the spell.
// NOTE: "unless controller pays {2}" (ward-like effect) is not yet in the DSL.
// The keyword + copy trigger are correctly encoded; the counter-unless effect is stubbed.
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
            // TODO: "Counter target spell unless its controller pays {2}" — requires
            // Effect::CounterUnlessPays or Effect::CounterSpell with cost-branch.
            // Stub until CounterSpell + PayCost effects are in the DSL.
        ],
        ..Default::default()
    }
}

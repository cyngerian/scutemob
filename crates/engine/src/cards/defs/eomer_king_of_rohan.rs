// Éomer, King of Rohan — {3}{R}{W}, Legendary Creature — Human Noble 2/2
// Double strike; ETB: +1/+1 counter per other Human you control; ETB: monarch + deal damage equal to power
// TODO: ETB counter based on creature count and ETB deal-damage-equal-to-power not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eomer-king-of-rohan"),
        name: "Éomer, King of Rohan".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Noble"],
        ),
        oracle_text: "Double strike\nÉomer enters with a +1/+1 counter on it for each other Human you control.\nWhen Éomer enters, target player becomes the monarch. Éomer deals damage equal to its power to any target.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            // TODO: ETB with X +1/+1 counters where X = number of other Humans you control
            // requires count-based counter placement (count_threshold gap).
            // TODO: ETB trigger: make target player monarch + deal power-equals-damage to
            // any target — monarch mechanic and damage-equal-to-power not in DSL.
        ],
        ..Default::default()
    }
}

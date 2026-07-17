// Dawnstrike Vanguard — {5}{W}, Creature — Human Knight 4/5
// Lifelink
// At the beginning of your end step, if you control two or more tapped creatures, put a
// +1/+1 counter on each creature you control other than this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dawnstrike-vanguard"),
        name: "Dawnstrike Vanguard".to_string(),
        mana_cost: Some(ManaCost { generic: 5, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Lifelink\nAt the beginning of your end step, if you control two or more tapped creatures, put a +1/+1 counter on each creature you control other than Dawnstrike Vanguard.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // TODO: DSL gap — end step trigger with tapped creature count condition +
            // mass counter placement excluding self. Multiple DSL gaps.
        ],
        completeness: Completeness::partial("Trigger unimplemented. End step trigger (AtBeginningOfYourEndStep) and 'each creature you control other than this' (ForEachTarget::EachOtherCreatureYouControl, effects/mod.rs:8778) both exist. Blocked ONLY on the intervening-if: Condition::YouControlNOrMoreWithFilter routes through matches_filter, which silently ignores TargetFilter::is_tapped (a GameObject field) — do NOT author it that way. Unblock: check is_tapped in the YouControlNOrMoreWithFilter arm (effects/mod.rs:8508) or add a tapped-count Condition."),
        ..Default::default()
    }
}

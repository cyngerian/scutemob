// Scourge of Valkas — {2}{R}{R}{R}, Creature — Dragon 4/4
// Flying
// Whenever this creature or another Dragon you control enters, it deals X damage to any
// target, where X is the number of Dragons you control.
// {R}: This creature gets +1/+0 until end of turn.
//
// Flying is implemented.
//
// TODO: DSL gap — the first triggered ability requires:
// 1. A trigger that fires when this creature OR another Dragon you control enters — no
//    "trigger on self OR creature-type-filtered ETB" variant exists.
// 2. EffectAmount based on count of Dragons you control — EffectAmount::CountCreaturesYouControl
//    with a subtype filter does not exist in the DSL.
//
// TODO: DSL gap — {R}: This creature gets +1/+0 until end of turn — ApplyContinuousEffect
// with EffectFilter::Source and ModifyPower(1) UntilEndOfTurn is expressible in principle,
// but no "target self" EffectFilter::Source pattern is confirmed for Activated abilities.
// Both non-keyword abilities are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scourge-of-valkas"),
        name: "Scourge of Valkas".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 3, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature or another Dragon you control enters, it deals X damage to any target, where X is the number of Dragons you control.\n{R}: This creature gets +1/+0 until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}

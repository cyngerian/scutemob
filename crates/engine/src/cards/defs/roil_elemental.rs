// Roil Elemental — {3}{U}{U}{U}, Creature — Elemental 3/2
// Flying
// Landfall — Whenever a land you control enters, you may gain control of target creature
// for as long as you control this creature.
//
// Flying is implemented.
//
// TODO: DSL gap — the Landfall triggered ability requires:
// 1. TriggerCondition for "whenever a land you control enters" (Landfall).
// 2. A conditional Control-change effect (SetController) with duration
//    "for as long as you control this creature" — EffectDuration has no
//    "WhileYouControlSource" variant. Both permanent and conditional controller-change
//    effects with a dynamic duration are not expressible in the current DSL.
// The Landfall ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roil-elemental"),
        name: "Roil Elemental".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 3, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Flying\nLandfall \u{2014} Whenever a land you control enters, you may gain control of target creature for as long as you control this creature.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}

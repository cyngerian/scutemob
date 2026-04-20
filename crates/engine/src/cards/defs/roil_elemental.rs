// Roil Elemental — {3}{U}{U}{U}, Creature — Elemental 3/2
// Flying
// Landfall — Whenever a land you control enters, you may gain control of target creature
// for as long as you control this creature.
//
// Flying is implemented.
//
// TODO: Blocker — EffectDuration::WhileYouControlSource variant for "for as long as you
// control this creature" on a control-change effect. Landfall trigger itself is covered by
// TriggerCondition::WheneverPermanentEntersBattlefield { Land + You } + Effect::GainControl
// (CR 207.2c). The dynamic-duration control change is the real gap; a permanent GainControl
// without a restoration condition would produce wrong game state. The Landfall ability is
// omitted until EffectDuration::WhileYouControlSource is added.
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

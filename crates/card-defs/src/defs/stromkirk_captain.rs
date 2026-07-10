// Stromkirk Captain — {1}{B}{R}, Creature — Vampire Soldier 2/2
// First strike
// Other Vampire creatures you control get +1/+1 and have first strike.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stromkirk-captain"),
        name: "Stromkirk Captain".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Soldier"]),
        oracle_text: "First strike\nOther Vampire creatures you control get +1/+1 and have first strike.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // CR 613.1c / Layer 7c: "Other Vampire creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Vampire".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f / Layer 6: "Other Vampire creatures you control have first strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Vampire".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

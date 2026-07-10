// Legion Lieutenant — {W}{B}, Creature — Vampire Knight 2/2
// Other Vampires you control get +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legion-lieutenant"),
        name: "Legion Lieutenant".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "Other Vampires you control get +1/+1.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(
                        SubType("Vampire".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

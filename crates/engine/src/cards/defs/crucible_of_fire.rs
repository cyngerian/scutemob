// Crucible of Fire — {3}{R}, Enchantment
// Dragon creatures you control get +3/+3.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crucible-of-fire"),
        name: "Crucible of Fire".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Dragon creatures you control get +3/+3.".to_string(),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Dragon creatures you control get +3/+3."
            // Uses OtherCreaturesYouControlWithSubtype even though this is an enchantment
            // (not a Dragon), so the "other" exclusion doesn't matter — source won't match.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(3),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Dragon".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

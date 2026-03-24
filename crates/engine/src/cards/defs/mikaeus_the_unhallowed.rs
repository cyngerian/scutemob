// Mikaeus, the Unhallowed — {3}{B}{B}{B}, Legendary Creature — Zombie Cleric 5/5
// Intimidate
// Whenever a Human deals damage to you, destroy it.
// Other non-Human creatures you control get +1/+1 and have undying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mikaeus-the-unhallowed"),
        name: "Mikaeus, the Unhallowed".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Cleric"],
        ),
        oracle_text: "Intimidate\nWhenever a Human deals damage to you, destroy it.\nOther non-Human creatures you control get +1/+1 and have undying.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Intimidate),
            // TODO: "Whenever a Human deals damage to you, destroy it." Blocked: trigger
            // on damage-by-subtype not in DSL.
            // CR 613.4c (Layer 7c): "Other non-Human creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlExcludingSubtype(
                        SubType("Human".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f (Layer 6): "Other non-Human creatures you control have undying."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Undying),
                    filter: EffectFilter::OtherCreaturesYouControlExcludingSubtype(
                        SubType("Human".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

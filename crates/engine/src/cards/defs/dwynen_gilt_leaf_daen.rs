// Dwynen, Gilt-Leaf Daen — {2}{G}{G}, Legendary Creature — Elf Warrior 3/4
// Reach
// Other Elf creatures you control get +1/+1.
// Whenever Dwynen attacks, you gain 1 life for each attacking Elf you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dwynen-gilt-leaf-daen"),
        name: "Dwynen, Gilt-Leaf Daen".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "Reach\nOther Elf creatures you control get +1/+1.\nWhenever Dwynen attacks, you gain 1 life for each attacking Elf you control.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            // CR 613.1c / Layer 7c: "Other Elf creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — "Whenever Dwynen attacks, you gain 1 life for each attacking
            // Elf you control." Needs EffectAmount::AttackingCreatureCountWithSubtype.
        ],
        ..Default::default()
    }
}

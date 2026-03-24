// Silver-Fur Master — {U}{B}, Creature — Rat Ninja 2/2
// Ninjutsu {U}{B}
// Ninjutsu abilities you activate cost {1} less to activate.
// Other Ninja and Rogue creatures you control get +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silver-fur-master"),
        name: "Silver-Fur Master".to_string(),
        mana_cost: Some(ManaCost { blue: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {U}{B}\nNinjutsu abilities you activate cost {1} less to activate.\nOther Ninja and Rogue creatures you control get +1/+1.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { blue: 1, black: 1, ..Default::default() },
            },
            // TODO: DSL gap — "Ninjutsu abilities you activate cost {1} less to activate."
            // Ability cost reduction (not spell cost reduction) not in DSL. Blocked on PB-29.
            // CR 613.4c (Layer 7c): "Other Ninja and Rogue creatures you control get +1/+1."
            // OR semantics: matches creatures with Ninja OR Rogue subtype.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtypes(vec![
                        SubType("Ninja".to_string()),
                        SubType("Rogue".to_string()),
                    ]),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

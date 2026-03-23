// Galadhrim Brigade — {2}{G}, Creature — Elf Soldier 2/2
// Squad {1}{G}
// Other Elves you control get +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("galadhrim-brigade"),
        name: "Galadhrim Brigade".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Soldier"]),
        oracle_text: "Squad {1}{G} (As an additional cost to cast this spell, you may pay {1}{G} any number of times. When this creature enters, create that many tokens that are copies of it.)\nOther Elves you control get +1/+1.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Squad),
            // "Other Elves you control get +1/+1" — lord static ability
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: crate::state::EffectLayer::PtModify,
                    modification: crate::state::LayerModification::ModifyBoth(1),
                    filter: crate::state::EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: crate::state::EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        ..Default::default()
    }
}

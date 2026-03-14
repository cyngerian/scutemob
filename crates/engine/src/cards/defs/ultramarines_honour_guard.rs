// Ultramarines Honour Guard — {3}{W}, Creature — Astartes Warrior 2/2
// Squad {2}; Other creatures you control get +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ultramarines-honour-guard"),
        name: "Ultramarines Honour Guard".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Astartes", "Warrior"]),
        oracle_text: "Squad {2} (As an additional cost to cast this spell, you may pay {2} any number of times. When this creature enters, create that many tokens that are copies of it.)\nOther creatures you control get +1/+1.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.157a: Squad — presence marker; actual cost carried by AbilityDefinition::Squad.
            AbilityDefinition::Keyword(KeywordAbility::Squad),
            // CR 702.157a: Squad {2} — additional cost paid N times; ETB trigger creates N token copies.
            AbilityDefinition::Squad { cost: ManaCost { generic: 2, ..Default::default() } },
            // CR 613.1c / Layer 7c: "Other creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}

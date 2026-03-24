// Serra Ascendant — {W}, Creature — Human Monk 1/1
// Lifelink (Damage dealt by this creature also causes you to gain that much life.)
// As long as you have 30 or more life, this creature gets +5/+5 and has flying.
//
// CR 604.2 / CR 613.1c: The "as long as" clause makes both the P/T boost and the
// flying grant into conditional static abilities. Both are gated on the same condition
// (ControllerLifeAtLeast(30)). Two separate AbilityDefinition::Static entries are used,
// one for the P/T modification (Layer 7c) and one for the ability grant (Layer 6).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serra-ascendant"),
        name: "Serra Ascendant".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Lifelink (Damage dealt by this creature also causes you to gain that much life.)\nAs long as you have 30 or more life, this creature gets +5/+5 and has flying.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // CR 604.2 / CR 613.1c (Layer 7c): "As long as you have 30 or more life,
            // this creature gets +5/+5."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(5),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::ControllerLifeAtLeast(30)),
                },
            },
            // CR 604.2 / CR 613.1f (Layer 6): "As long as you have 30 or more life,
            // this creature has flying."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::ControllerLifeAtLeast(30)),
                },
            },
        ],
        ..Default::default()
    }
}

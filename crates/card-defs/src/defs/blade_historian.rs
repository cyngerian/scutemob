// Blade Historian — {R/W}{R/W}{R/W}{R/W}, Creature — Human Cleric 2/3
// Attacking creatures you control have double strike.
//
// CR 613.1f (Layer 6): Static ability — dynamic grant that applies only to currently-attacking
// creatures. Evaluated at layer-application time using state.combat.attackers (CR 611.3a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blade-historian"),
        name: "Blade Historian".to_string(),
        mana_cost: Some(ManaCost {
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::White),
            ],
            ..Default::default()
        }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "Attacking creatures you control have double strike.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking creatures you control have double strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::DoubleStrike),
                    filter: EffectFilter::AttackingCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

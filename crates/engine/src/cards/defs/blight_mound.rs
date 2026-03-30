// Blight Mound — {2}{B}, Enchantment
// Attacking Pests you control get +1/+0 and have menace.
// Whenever a nontoken creature you control dies, create a 1/1 black and green Pest creature
// token with "When this token dies, you gain 1 life."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blight-mound"),
        name: "Blight Mound".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Pests you control get +1/+0 and have menace.\nWhenever a nontoken creature you control dies, create a 1/1 black and green Pest creature token with \"When this token dies, you gain 1 life.\"".to_string(),
        abilities: vec![
            // CR 613.4c (Layer 7c): "Attacking Pests you control get +1/+0."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Pest".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f (Layer 6): "Attacking Pests you control have menace."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Menace),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Pest".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 603.10a: "Whenever a nontoken creature you control dies, create a 1/1
            // black and green Pest creature token with 'When this creature dies, you gain 1 life.'"
            // Note: TokenSpec cannot carry nested triggered abilities; the death-trigger
            // lifegain on the Pest token is a known DSL gap (token_triggered_abilities).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: true,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Pest".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Pest".to_string())].into_iter().collect(),
                        colors: [Color::Black, Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

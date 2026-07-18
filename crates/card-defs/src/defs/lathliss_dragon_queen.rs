// Lathliss, Dragon Queen — {4}{R}{R}, Legendary Creature — Dragon 6/6
// Flying
// Whenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature
// token with flying.
// {1}{R}: Dragons you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lathliss-dragon-queen"),
        name: "Lathliss, Dragon Queen".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 2,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever another nontoken Dragon you control enters, create a 5/5 \
                      red Dragon creature token with flying.\n{1}{R}: Dragons you control get \
                      +1/+0 until end of turn."
            .to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.2: "Whenever another nontoken Dragon you control enters, create a 5/5
            // red Dragon creature token with flying." exclude_self: true ("another");
            // is_nontoken: true ("nontoken Dragon"). Per Gatherer ruling 2024-11-08, fires
            // once per other nontoken Dragon entering simultaneously. PB-AC0: has_subtype
            // Dragon and is_nontoken are now honored on the creature-ETB path via
            // triggering_creature_filter forwarding.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        controller: TargetController::You,
                        is_nontoken: true,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 5,
                        toughness: 5,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
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
            // CR 613.4c: {1}{R}: Dragons you control get +1/+0 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    red: 1,
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::CreaturesYouControlWithSubtype(SubType(
                            "Dragon".to_string(),
                        )),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}

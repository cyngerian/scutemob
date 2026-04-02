// Camellia, the Seedmiser — {1}{B}{G} Legendary Creature — Squirrel Warlock 3/3
// Menace; other Squirrels have menace (static grant with subtype filter);
// sacrifice-Food trigger creates Squirrel token (TODO: TriggerCondition::WhenSacrificeFood);
// {2}, Forage: implemented with Cost::Forage; targets all Squirrels you control (including
// Camellia due to missing exclude-source in TargetFilter — deferred DSL gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("camellia-the-seedmiser"),
        name: "Camellia, the Seedmiser".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, generic: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Squirrel", "Warlock"],
        ),
        oracle_text: "Menace\nOther Squirrels you control have menace.\nWhenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token.\n{2}, Forage: Put a +1/+1 counter on each other Squirrel you control. (To forage, exile three cards from your graveyard or sacrifice a Food.)".to_string(),
        abilities: vec![
            // CR 702.110: Menace (self).
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // CR 613.1f / Layer 6: "Other Squirrels you control have menace."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Menace),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Squirrel".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },

            // CR 701.21a / CR 603.2: "Whenever you sacrifice one or more Foods, create a 1/1
            // green Squirrel creature token." Per-sacrifice firing (not batched) is the engine-wide
            // approximation; "one or more" is not enforceable with per-event triggers.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Food".to_string())),
                        ..Default::default()
                    }),
                    player_filter: None,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Squirrel".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Squirrel".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
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

            // CR 701.61a: "{2}, Forage: Put a +1/+1 counter on each other Squirrel you control."
            // Note: TargetFilter has no exclude_source field, so this also targets Camellia
            // herself. This is a minor deviation; exclude-source filtering is a deferred DSL gap.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Forage,
                ]),
                effect: Effect::AddCounter {
                    target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Squirrel".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    })),
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        power: Some(3),
        toughness: Some(3),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
    }
}

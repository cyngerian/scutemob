// Edgar, Charmed Groom // Edgar Markov's Coffin — {2}{W}{B}, Legendary Creature —
// Vampire Noble 4/4 (Transform)
// Front: Other Vampires you control get +1/+1.
//        When Edgar, Charmed Groom dies, return it to the battlefield transformed
//        under its owner's control.
// Back:  Edgar Markov's Coffin — Legendary Artifact.
//        At the beginning of your upkeep, create a 1/1 white and black Vampire
//        creature token with lifelink and put a bloodline counter on Edgar
//        Markov's Coffin. Then if there are three or more bloodline counters on
//        it, remove those counters and transform it.
//
// PB-OS4 (OOS-EF5-3): "When Edgar dies, return it to the battlefield
// transformed" is a permanent LEAVING (death) and a NEW permanent entering the
// battlefield already showing its back face (CR 400.7 / 712.18) — the new
// `Effect::ReturnSourceToBattlefieldTransformed` primitive. This is NOT the
// same mechanism as `Effect::TransformSelf` (used below for the back face's own
// "...remove those counters and transform it", which flips the SAME object in
// place — CR 712.18, no new object).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("edgar-charmed-groom"),
        name: "Edgar, Charmed Groom".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Noble"],
        ),
        oracle_text: "Other Vampires you control get +1/+1.\nWhen Edgar, Charmed Groom dies, \
                      return it to the battlefield transformed under its owner's control."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // CR 613.1c: "Other Vampires you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType(
                        "Vampire".to_string(),
                    )),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 400.7 / 712.18 / PB-OS4 (OOS-EF5-3): "When Edgar, Charmed Groom dies,
            // return it to the battlefield transformed under its owner's control."
            // Immediate -- no exile step, no delay (confirmed against cards.sqlite
            // oracle text at authoring time; the plan's brief incorrectly assumed a
            // delayed "at the next end step" return -- see WIP divergence note).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::ReturnSourceToBattlefieldTransformed,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
                once_per_turn: false,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Edgar Markov's Coffin".to_string(),
            mana_cost: None,
            types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &[]),
            oracle_text: "At the beginning of your upkeep, create a 1/1 white and black Vampire \
                          creature token with lifelink and put a bloodline counter on Edgar \
                          Markov's Coffin. Then if there are three or more bloodline counters on \
                          it, remove those counters and transform it."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Vampire".to_string(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
                            colors: [Color::White, Color::Black].into_iter().collect(),
                            power: 1,
                            toughness: 1,
                            count: EffectAmount::Fixed(1),
                            keywords: [KeywordAbility::Lifelink].into_iter().collect(),
                            ..Default::default()
                        },
                    },
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::Custom("bloodline".to_string()),
                        count: 1,
                    },
                    // "Then if there are three or more bloodline counters on it, remove
                    // those counters and transform it." CR 712.18: TransformSelf flips
                    // the SAME object in place (not a new object -- this is the back
                    // face transforming again, distinct from the front face's
                    // return-transformed WhenDies trigger above).
                    Effect::Conditional {
                        condition: Condition::SourceHasCounters {
                            counter: CounterType::Custom("bloodline".to_string()),
                            min: 3,
                        },
                        if_true: Box::new(Effect::Sequence(vec![
                            Effect::RemoveCounter {
                                target: EffectTarget::Source,
                                counter: CounterType::Custom("bloodline".to_string()),
                                count: 3,
                            },
                            Effect::TransformSelf,
                        ])),
                        if_false: Box::new(Effect::Nothing),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
                once_per_turn: false,
            }],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

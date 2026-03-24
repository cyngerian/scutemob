// Goblin War Party — {3}{R}, Sorcery
// Choose one —
// • Create three 1/1 red Goblin creature tokens.
// • Creatures you control get +1/+1 and gain haste until end of turn.
// Entwine {2}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-war-party"),
        name: "Goblin War Party".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one \u{2014}\n\u{2022} Create three 1/1 red Goblin creature tokens.\n\u{2022} Creatures you control get +1/+1 and gain haste until end of turn.\nEntwine {2}{R} (Choose both if you pay the entwine cost.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Entwine),
            AbilityDefinition::Entwine {
                cost: ManaCost { generic: 2, red: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Create three 1/1 red Goblin creature tokens.
                        Effect::CreateToken {
                            spec: TokenSpec {
                                name: "Goblin".to_string(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                                colors: [Color::Red].into_iter().collect(),
                                supertypes: im::OrdSet::new(),
                                power: 1,
                                toughness: 1,
                                count: 3,
                                keywords: im::OrdSet::new(),
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                            },
                        },
                        // Mode 1: CR 613.4c / CR 613.1f: "Creatures you control get +1/+1
                        // and gain haste until end of turn."
                        Effect::Sequence(vec![
                            Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::PtModify,
                                    modification: LayerModification::ModifyBoth(1),
                                    filter: EffectFilter::CreaturesYouControl,
                                    duration: EffectDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            },
                            Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::Ability,
                                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                                    filter: EffectFilter::CreaturesYouControl,
                                    duration: EffectDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            },
                        ]),
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

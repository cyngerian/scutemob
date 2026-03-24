// Goblin Surprise — {2}{R}, Instant
// Choose one —
// • Creatures you control get +2/+0 until end of turn.
// • Create two 1/1 red Goblin creature tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-surprise"),
        name: "Goblin Surprise".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Creatures you control get +2/+0 until end of turn.\n• Create two 1/1 red Goblin creature tokens.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: CR 613.4c: "Creatures you control get +2/+0 until end of turn."
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyPower(2),
                                filter: EffectFilter::CreaturesYouControl,
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 1: Create two 1/1 red Goblin creature tokens.
                        Effect::CreateToken {
                            spec: TokenSpec {
                                name: "Goblin".to_string(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                                colors: [Color::Red].into_iter().collect(),
                                supertypes: im::OrdSet::new(),
                                power: 1,
                                toughness: 1,
                                count: 2,
                                keywords: im::OrdSet::new(),
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                            },
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// Crippling Fear — {2}{B}{B} Sorcery
// Choose a creature type. Creatures that aren't of the chosen type get -3/-3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crippling-fear"),
        name: "Crippling Fear".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Creatures that aren't of the chosen type get -3/-3 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // CR 205.3m: Choose a creature type; stores in ctx.chosen_creature_type.
                    Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                    // CR 613.1c / Layer 7c: -3/-3 to all creatures not of the chosen type.
                    // AllCreaturesExcludingChosenSubtype is substituted at execution time
                    // into AllCreaturesExcludingSubtype(chosen_type) per CR 608.2h.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(-3),
                            filter: EffectFilter::AllCreaturesExcludingChosenSubtype,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

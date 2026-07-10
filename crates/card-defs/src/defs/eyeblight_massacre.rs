// Eyeblight Massacre — {2}{B}{B} Sorcery
// Non-Elf creatures get -2/-2 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eyeblight-massacre"),
        name: "Eyeblight Massacre".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Non-Elf creatures get -2/-2 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 613.1c / Layer 7c: -2/-2 to all creatures (any controller) that are not Elves.
                // AllCreaturesExcludingSubtype applies across all players' battlefields.
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-2),
                        filter: EffectFilter::AllCreaturesExcludingSubtype(SubType("Elf".to_string())),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// Drown in Ichor — {1}{B}, Sorcery
// Target creature gets -4/-4 until end of turn. Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drown-in-ichor"),
        name: "Drown in Ichor".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Target creature gets -4/-4 until end of turn. Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-4),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                Effect::Proliferate,
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

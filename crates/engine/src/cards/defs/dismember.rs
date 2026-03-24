// Dismember — {1}{B/P}{B/P}, Instant
// ({B/P} can be paid with either {B} or 2 life.)
// Target creature gets -5/-5 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dismember"),
        name: "Dismember".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Black), PhyrexianMana::Single(ManaColor::Black)],
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "({B/P} can be paid with either {B} or 2 life.)\nTarget creature gets -5/-5 until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(-5),
                    filter: EffectFilter::DeclaredTarget { index: 0 },
                    duration: EffectDuration::UntilEndOfTurn,
                    condition: None,
                }),
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

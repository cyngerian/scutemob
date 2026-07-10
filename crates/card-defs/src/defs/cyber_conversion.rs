// Cyber Conversion — {2}{U}, Instant
// Target creature becomes an artifact in addition to its other types until end of turn. Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cyber-conversion"),
        name: "Cyber Conversion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature becomes an artifact in addition to its other types until end of turn. Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 613.1b: Layer 4 type change — add Artifact to the target creature's card types.
                Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::TypeChange,
                        modification: LayerModification::AddCardTypes(
                            [CardType::Artifact].into_iter().collect(),
                        ),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                // CR 701.7a: Draw a card.
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

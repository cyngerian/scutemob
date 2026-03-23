// Invigorate — {2}{G}, Instant
// If you control a Forest, rather than pay this spell's mana cost, you may have an
// opponent gain 3 life.
// Target creature gets +4/+4 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("invigorate"),
        name: "Invigorate".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a Forest, rather than pay this spell's mana cost, you may have an opponent gain 3 life.\nTarget creature gets +4/+4 until end of turn.".to_string(),
        abilities: vec![
            // TODO: DSL gap — alternative cost "opponent gains 3 life" not in AltCostKind.
            AbilityDefinition::Spell {
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(4),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                    }),
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

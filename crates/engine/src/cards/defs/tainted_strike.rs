// Tainted Strike — {B}, Instant
// Target creature gets +1/+0 and gains infect until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-strike"),
        name: "Tainted Strike".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature gets +1/+0 and gains infect until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // CR 613.1c / Layer 7c: +1/+0 until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyPower(1),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // CR 613.1f / Layer 6: gains infect until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Infect),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

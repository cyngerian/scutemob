// Unbreakable Formation — {2}{W}, Instant
// Creatures you control gain indestructible until end of turn.
// Addendum — If you cast this spell during your main phase, put a +1/+1 counter on each
// of those creatures and they gain vigilance until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("unbreakable-formation"),
        name: "Unbreakable Formation".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Creatures you control gain indestructible until end of turn.\nAddendum — If you cast this spell during your main phase, put a +1/+1 counter on each of those creatures and they gain vigilance until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // Creatures you control gain indestructible until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Indestructible),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                    // TODO: DSL gap — Addendum: "if during main phase" condition check
                    // for +1/+1 counters + vigilance grant. Condition::CastDuringMainPhase
                    // does not exist.
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

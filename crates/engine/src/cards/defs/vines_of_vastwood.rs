// Vines of Vastwood — {G}, Instant
// Kicker {G}
// Target creature can't be the target of spells or abilities your opponents
// control this turn. If kicked, that creature gets +4/+4 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vines-of-vastwood"),
        name: "Vines of Vastwood".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Kicker {G} (You may pay an additional {G} as you cast this spell.)\nTarget creature can't be the target of spells or abilities your opponents control this turn. If this spell was kicked, that creature gets +4/+4 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Kicker),
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // Grant hexproof-like protection (can't be targeted by opponents).
                    // TODO: "can't be the target of spells or abilities opponents control"
                    // is functionally Hexproof for one turn — no Effect::GrantHexproof
                    // exists. Using ApplyContinuousEffect with AddKeyword(Hexproof).
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Hexproof),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // If kicked, +4/+4 until end of turn.
                    Effect::Conditional {
                        condition: Condition::WasKicked,
                        if_true: Box::new(Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(4),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        }),
                        if_false: Box::new(Effect::Nothing),
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

// Crashing Drawbridge — {2}, Artifact Creature — Wall 0/4
// Defender
// {T}: Creatures you control gain haste until end of turn.
//
// CR 604.2 / CR 613.1f: Activated ability produces a continuous effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crashing-drawbridge"),
        name: "Crashing Drawbridge".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Wall"]),
        oracle_text: "Defender\n{T}: Creatures you control gain haste until end of turn.".to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Defender),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

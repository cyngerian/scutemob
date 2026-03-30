// Steel Hellkite — {6}, Artifact Creature — Dragon 5/5
// Flying
// {2}: This creature gets +1/+0 until end of turn.
// {X}: Destroy each nonland permanent with mana value X whose controller was dealt combat
// damage by this creature this turn. Activate only once each turn.
//
// CR 602.5b: "Activate only once each turn" restriction on second ability via once_per_turn: true.
// TODO: {X} ability main effect — needs "mana value equals X" filter in DestroyAll and
//       "controllers dealt combat damage by this creature this turn" tracking. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("steel-hellkite"),
        name: "Steel Hellkite".to_string(),
        mana_cost: Some(ManaCost { generic: 6, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Artifact, CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\n{2}: This creature gets +1/+0 until end of turn.\n{X}: Destroy each nonland permanent with mana value X whose controller was dealt combat damage by this creature this turn. Activate only once each turn.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // {2}: This creature gets +1/+0 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::PtModify,
                        modification: crate::state::LayerModification::ModifyPower(1),
                        filter: crate::state::EffectFilter::Source,
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // CR 107.3k: {X}: Destroy each nonland permanent with mana value X whose controller
            // was dealt combat damage by this creature this turn. Activate only once each turn.
            // CR 602.5b: once_per_turn: true.
            // TODO: "mana value equals X" filter in DestroyAll — TargetFilter has no
            //        mana-value-equals-X predicate. Deferred.
            // TODO: "controllers dealt combat damage by this creature this turn" tracking. Deferred.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { x_count: 1, ..Default::default() }),
                effect: Effect::Nothing,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: true,
            },
        ],
        ..Default::default()
    }
}

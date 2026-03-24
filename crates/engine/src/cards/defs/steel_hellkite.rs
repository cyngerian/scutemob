// Steel Hellkite — {6}, Artifact Creature — Dragon 5/5
// Flying
// {2}: This creature gets +1/+0 until end of turn.
// {X}: Destroy each nonland permanent with mana value X whose controller was dealt combat
// damage by this creature this turn. Activate only once each turn.
// TODO: DSL gap — {X}: destroy nonland permanents with mana value X among damaged controllers
// requires X-cost activated ability + "mana value equals X" filter + tracking which players were
// dealt combat damage by this creature this turn. None of these are expressible in the current DSL.
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
            },
            // TODO: {X}: Destroy each nonland permanent with mana value X whose controller was
            // dealt combat damage by this creature this turn. Activate only once each turn.
            // DSL gap: X-cost activated ability, mana-value filter on targets, per-turn
            // activation tracking, and "controllers dealt combat damage by this creature"
            // tracking are all unsupported.
        ],
        ..Default::default()
    }
}

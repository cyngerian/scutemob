// Scourge of Valkas — {2}{R}{R}{R}, Creature — Dragon 4/4
// Flying
// Whenever this creature or another Dragon you control enters, it deals X damage to any
// target, where X is the number of Dragons you control.
// {R}: This creature gets +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scourge-of-valkas"),
        name: "Scourge of Valkas".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 3, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature or another Dragon you control enters, it deals X damage to any target, where X is the number of Dragons you control.\n{R}: This creature gets +1/+0 until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever this creature or another Dragon you control enters, it deals X
            // damage to any target, where X is the number of Dragons you control."
            // Blocked on multiple gaps: "this creature OR another Dragon ETB" trigger variant
            // + EffectAmount::CountCreaturesYouControl(filter) not in DSL.
            // CR 613.4c: "{R}: This creature gets +1/+0 until end of turn."
            // EffectFilter::Source resolves to SingleObject(ctx.source) at execution time.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { red: 1, ..Default::default() }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}

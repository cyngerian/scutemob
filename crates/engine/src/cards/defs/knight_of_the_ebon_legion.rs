// Knight of the Ebon Legion — {B}, Creature — Vampire Knight 1/2
// {2}{B}: This creature gets +3/+3 and gains deathtouch until end of turn.
// At the beginning of your end step, if a player lost 4 or more life this turn, put a
// +1/+1 counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("knight-of-the-ebon-legion"),
        name: "Knight of the Ebon Legion".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "{2}{B}: Knight of the Ebon Legion gets +3/+3 and gains deathtouch until end of turn.\nAt the beginning of your end step, if a player lost 4 or more life this turn, put a +1/+1 counter on Knight of the Ebon Legion.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, black: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(3),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // TODO: DSL gap — end step trigger with "if a player lost 4+ life this turn"
            // intervening-if condition.
        ],
        ..Default::default()
    }
}

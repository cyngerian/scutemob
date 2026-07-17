// Massacre Wurm — {3}{B}{B}{B}, Creature — Phyrexian Wurm 6/5
// When this creature enters, creatures your opponents control get -2/-2 until end of turn.
// Whenever a creature an opponent controls dies, that player loses 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("massacre-wurm"),
        name: "Massacre Wurm".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 3, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Wurm"]),
        oracle_text: "When this creature enters, creatures your opponents control get -2/-2 until end of turn.\nWhenever a creature an opponent controls dies, that player loses 2 life.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            // CR 603.1 / CR 613.4c (Layer 7c): ETB trigger — opponents' creatures get -2/-2 until EOT.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(-2),
                        filter: EffectFilter::CreaturesOpponentsControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Whenever a creature an opponent controls dies, that player loses 2 life."
            // Blocked on PB-26: WheneverCreatureDies needs opponent controller filter.
        ],
        completeness: Completeness::partial("Blocked on PlayerTarget::ControllerOf resolution — 'that player loses 2 life' would drain the Wurm's own controller (effects/mod.rs:3609/3661 resolve ControllerOf/OwnerOf to ctx.controller). STALE: the PB-26 'WheneverCreatureDies needs an opponent controller filter' claim — that filter exists and is wired."),
        ..Default::default()
    }
}

// Exuberant Fuseling — {R}, Creature — Phyrexian Goblin Warrior 0/1
// Trample
// This creature gets +1/+0 for each oil counter on it.
// When this creature enters and whenever another creature or artifact you control
// is put into a graveyard from the battlefield, put an oil counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exuberant-fuseling"),
        name: "Exuberant Fuseling".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Goblin", "Warrior"]),
        oracle_text: "Trample\nThis creature gets +1/+0 for each oil counter on it.\nWhen this creature enters and whenever another creature or artifact you control is put into a graveyard from the battlefield, put an oil counter on this creature.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 613.1c / Layer 7c — CDA: "This creature gets +1/+0 for each oil counter on it."
            // ModifyPowerDynamic is a DSL placeholder resolved at Effect::ApplyContinuousEffect
            // execution time (CR 608.2h): the oil counter count on this creature is locked in
            // at the moment the ETB effect fires and applied for WhileSourceOnBattlefield.
            // is_cda: true — this CDA effect applies before non-CDA Layer 7c effects
            // (CR 613.1c ordering within the same layer).
            //
            // Design note: This ETB-triggered ApplyContinuousEffect pattern captures the
            // counter count at entry time. Full dynamic re-evaluation (counter count changes
            // after entry) would require either SetPtDynamic (Layer 7a) or a separate
            // per-counter-add trigger, both of which are out of PB-CC-C scope.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::CounterCount {
                                target: EffectTarget::Source,
                                counter: CounterType::Oil,
                            }),
                            negate: false,
                        },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // ETB: put an oil counter on this creature.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Oil,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: "whenever another creature or artifact you control is put into a graveyard
            // from the battlefield, put an oil counter on this creature" —
            // WheneverCreatureDies covers the creature half (with exclude_self=true, controller=You)
            // but there is no "or artifact" variant in WheneverCreatureDies, and no separate
            // WheneverArtifactDies trigger condition exists. Implementing only the creature
            // half would produce wrong game state (misses artifact deaths).
            // Blocked by: WheneverCreatureOrArtifactDies trigger condition (multi-blocker).
            // Out of PB-CC-C scope per memory/primitives/pb-retriage-CC.md.
        ],
        ..Default::default()
    }
}

// Foundry Street Denizen — {R}, Creature — Goblin Warrior 1/1
// Whenever another red creature enters the battlefield under your control,
// this creature gets +1/+0 until end of turn.
//
// Note: WheneverCreatureEntersBattlefield filter includes the controller constraint,
// color filter, and (PB-XS-E) exclude_self: true to honor the "another" qualifier
// (CR 109.1 / 603.2). The +1/+0 on Source is applied via a Layer-7c continuous effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("foundry-street-denizen"),
        name: "Foundry Street Denizen".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Whenever another red creature enters the battlefield under your control, this creature gets +1/+0 until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.6a: "Whenever another red creature enters under your control, gets +1/+0."
            // WheneverCreatureEntersBattlefield with colors=Red + controller=You filter
            // and exclude_self: true (PB-XS-E) so the Denizen's own ETB does not fire it.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some([Color::Red].iter().copied().collect()),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(1),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

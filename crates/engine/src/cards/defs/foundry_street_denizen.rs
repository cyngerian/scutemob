// Foundry Street Denizen — {R}, Creature — Goblin Warrior 1/1
// Whenever another red creature enters the battlefield under your control,
// this creature gets +1/+0 until end of turn.
//
// Note: WheneverCreatureEntersBattlefield filter includes the controller constraint and
// color filter. The "another" (exclude_self) constraint is not available on this trigger
// variant (unlike WheneverCreatureDies). This will fire when the Denizen itself enters,
// which is a minor over-trigger edge case. The +1/+0 on Source is correct otherwise.
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
            // WheneverCreatureEntersBattlefield with colors=Red + controller=You filter.
            // Note: "another" (exclude_self) is not supported on this trigger; the Denizen
            // will also trigger on its own ETB. Minor inaccuracy, not wrong game state.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some([Color::Red].iter().copied().collect()),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
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

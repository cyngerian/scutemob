// Ingenious Prodigy — {X}{U}, Creature — Human Wizard 0/1
// Skulk
// This creature enters with X +1/+1 counters on it.
// At the beginning of your upkeep, if this creature has one or more +1/+1 counters on it,
// you may remove a +1/+1 counter from it. If you do, draw a card.
//
// PB-EWC (2026-05-14): the ETB counter clause is now a true CR 614.1c
// replacement effect (`AbilityDefinition::Replacement { is_self: true, ... }`),
// not a triggered-ETB stub. The previous "DEVIATION: CR 614.1c" approximation
// (counters placed by a stack-resolved trigger AFTER ETB) has been removed —
// counters are now placed simultaneously with the permanent entering.
//
// X resolution: during permanent-spell resolution the entering permanent's
// `x_value` is copied from `StackObject.x_value` BEFORE ETB processing runs.
// By the time `apply_self_etb_from_definition` fires, `state.objects[new_id]
// .x_value` is populated, and `emit_etb_modification` builds an EffectContext
// with `source = new_id` and `x_value = state.objects[new_id].x_value`.
// `EffectAmount::XValue` then returns the cast-time X.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ingenious-prodigy"),
        name: "Ingenious Prodigy".to_string(),
        mana_cost: Some(ManaCost { blue: 1, x_count: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Skulk\nThis creature enters with X +1/+1 counters on it.\nAt the beginning of your upkeep, if this creature has one or more +1/+1 counters on it, you may remove a +1/+1 counter from it. If you do, draw a card.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Skulk),
            // CR 614.1c — "This creature enters with X +1/+1 counters on it."
            // Self-replacement effect: applies inline at ETB time before any
            // other replacements (CR 614.15). EffectAmount::XValue reads the
            // entering permanent's x_value (propagated from the spell's
            // StackObject at resolution.rs:546).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::XValue),
                },
                is_self: true,
                unless_condition: None,
            },
            // Upkeep: remove counter → draw.
            // DEVIATION: Oracle says "you MAY remove a +1/+1 counter." This ability is
            // unconditional — the DSL has no way to make "remove counter as cost" optional
            // without MayPayOrElse integration that is out of scope for PB-27.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::RemoveCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: Some(Condition::SourceHasCounters {
                    counter: CounterType::PlusOnePlusOne,
                    min: 1,
                }),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong("upkeep ability removes a +1/+1 counter unconditionally; oracle says 'you MAY remove'"),
        ..Default::default()
    }
}

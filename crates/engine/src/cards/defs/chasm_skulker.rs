// Chasm Skulker — {2}{U}, Creature — Squid Horror 1/1
// Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.
// When Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk,
// where X is the number of +1/+1 counters on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chasm-skulker"),
        name: "Chasm Skulker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Squid", "Horror"]),
        oracle_text: "Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.\nWhen Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk, where X is the number of +1/+1 counters on Chasm Skulker.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Whenever you draw a card, put a +1/+1 counter on Chasm Skulker.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouDrawACard,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // When Chasm Skulker dies, create X 1/1 blue Squid creature tokens with islandwalk,
            // where X is the number of +1/+1 counters on it.
            // TODO(OOS-TS-4): WhenDies CounterCount{Source, PlusOnePlusOne} resolves to 0 tokens
            // because move_object_to_zone resets counters to empty (state/mod.rs:420). This card
            // produces wrong game state (always 0 Squid tokens) until a pre-death-counter snapshot
            // mechanism lands in PendingTrigger / EffectContext per CR 603.10a "leaves-battlefield
            // triggers look back in time." Blocked on OOS-TS-4 primitive (memory/primitives/pb-retriage-CC.md).
        ],
        ..Default::default()
    }
}

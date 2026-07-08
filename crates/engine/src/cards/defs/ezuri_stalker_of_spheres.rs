// Ezuri, Stalker of Spheres — {2}{G}{U}, Legendary Creature — Phyrexian Elf Warrior 3/3
// When Ezuri enters, you may pay {3}. If you do, proliferate twice.
// Whenever you proliferate, draw a card.
//
// ENGINE-BLOCKED (2nd ability only): "Whenever you proliferate, draw a card." The
// runtime fires an internal `TriggerEvent::ControllerProliferates` on `GameEvent::
// Proliferated` (rules/abilities.rs), but no `TriggerCondition` DSL variant maps to
// it (verified: only 2 references to `TriggerEvent::ControllerProliferates` in the
// whole engine — the firing site and its hash arm; `replay_harness.rs`'s
// TriggerCondition -> TriggerEvent builder has no arm producing it). Card
// definitions cannot reach this trigger event. The ETB "may pay {3}, if you do
// proliferate twice" ability IS fully expressible (PB-AC2 MayPayThenEffect) and is
// implemented below.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ezuri-stalker-of-spheres"),
        name: "Ezuri, Stalker of Spheres".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Elf", "Warrior"],
        ),
        oracle_text: "When Ezuri enters, you may pay {3}. If you do, proliferate twice.\nWhenever you proliferate, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // PB-AC2 (CR 118.12): "When Ezuri enters, you may pay {3}. If you do,
            // proliferate twice."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Sequence(vec![
                        Effect::Proliferate,
                        Effect::Proliferate,
                    ])),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: "Whenever you proliferate, draw a card." — see module
            // comment. No TriggerCondition variant reaches TriggerEvent::ControllerProliferates.
        ],
        ..Default::default()
    }
}

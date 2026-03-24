// Tireless Tracker — {2}{G}, Creature — Human Scout 3/2
// Landfall — Whenever a land you control enters, investigate.
// Whenever you sacrifice a Clue, put a +1/+1 counter on Tireless Tracker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tireless-tracker"),
        name: "Tireless Tracker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Human", "Scout"]),
        oracle_text: "Landfall — Whenever a land you control enters, investigate.\nWhenever you sacrifice a Clue, put a +1/+1 counter on Tireless Tracker.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Investigate { count: EffectAmount::Fixed(1) },
                intervening_if: None,
                targets: vec![],
            },
            // Whenever you sacrifice a Clue, put a +1/+1 counter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Clue".to_string())),
                        ..Default::default()
                    }),
                    player_filter: None,
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

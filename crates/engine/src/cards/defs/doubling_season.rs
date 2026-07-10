// Doubling Season — {4}{G}, Enchantment
// If an effect would create one or more tokens under your control, it creates twice that
// many of those tokens instead.
// If an effect would put one or more counters on a permanent you control, it puts twice
// that many of those counters on that permanent instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("doubling-season"),
        name: "Doubling Season".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.\nIf an effect would put one or more counters on a permanent you control, it puts twice that many of those counters on that permanent instead.".to_string(),
        abilities: vec![
            // CR 111.1 / CR 614.1: Token-doubling replacement effect.
            // PlayerId(0) placeholder — bound to the controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldCreateTokens {
                    controller_filter: PlayerFilter::Specific(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleTokens,
                is_self: false,
                unless_condition: None,
            },
            // CR 122.6 / CR 614.1: Counter-doubling replacement effect, scoped to
            // permanents the controller controls (unlike Vorinclex, which uses
            // `ObjectFilter::Any` — Doubling Season only doubles counters on "a
            // permanent you control", not counters placed on players).
            // PlayerId(0) placeholder — bound to the controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Any,
                    receiver_filter: ObjectFilter::ControlledBy(PlayerId(0)),
                    counter_filter: None,
                },
                modification: ReplacementModification::DoubleCounters,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

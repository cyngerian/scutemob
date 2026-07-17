// Corpsejack Menace — {2}{B}{G}, Creature — Fungus 4/4
// If one or more +1/+1 counters would be put on a creature you control, twice that many
// +1/+1 counters are put on it instead.
//
// PB-CD (scutemob-18, CR 122.6 / CR 614.1): replacement effect gated on counter type
// (+1/+1 only) and receiver (creature you control). Doubling stacks multiplicatively
// per ruling: two Corpsejacks = 4×, three = 8×.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("corpsejack-menace"),
        name: "Corpsejack Menace".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Fungus"]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, twice \
                      that many +1/+1 counters are put on it instead."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldPlaceCounters {
                placer_filter: PlayerFilter::Any,
                receiver_filter: ObjectFilter::CreatureControlledBy(PlayerId(0)),
                counter_filter: Some(CounterType::PlusOnePlusOne),
            },
            modification: ReplacementModification::DoubleCounters,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

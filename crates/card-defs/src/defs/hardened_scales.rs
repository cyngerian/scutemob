// Hardened Scales — {G}, Enchantment
// If one or more +1/+1 counters would be put on a creature you control, that many plus
// one +1/+1 counters are put on it instead.
//
// PB-CD (scutemob-18, CR 122.6 / CR 614.1): replacement effect gated on counter type
// (+1/+1 only) and receiver (creature you control). counter_filter is the +1/+1 gate;
// CreatureControlledBy is the receiver gate. PlayerId(0) is a placeholder bound to the
// actual controller at registration time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hardened-scales"),
        name: "Hardened Scales".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, that \
                      many plus one +1/+1 counters are put on it instead."
            .to_string(),
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldPlaceCounters {
                placer_filter: PlayerFilter::Any,
                receiver_filter: ObjectFilter::CreatureControlledBy(PlayerId(0)),
                counter_filter: Some(CounterType::PlusOnePlusOne),
            },
            modification: ReplacementModification::AddExtraCounter,
            is_self: false,
            unless_condition: None,
        }],
        ..Default::default()
    }
}

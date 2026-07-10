// Conclave Mentor — {G}{W}, Creature — Centaur Cleric 2/2
// If one or more +1/+1 counters would be put on a creature you control, that many plus
// one +1/+1 counters are put on that creature instead.
// When this creature dies, you gain life equal to its power.
//
// PB-CD (scutemob-18, CR 122.6 / CR 614.1): replacement half implemented as
// AddExtraCounter on +1/+1 counters placed on creature you control. Per ruling,
// the replacement does NOT apply to Conclave Mentor itself if it somehow enters with
// a +1/+1 counter (it's not on the battlefield yet when the ETB-with-counter event
// fires). This matches CR 614.13: replacement effects from a permanent don't apply to
// that permanent's own ETB event unless self-replacement.
//
// Death trigger uses `EffectAmount::SourcePowerAtLastKnownInformation` (PB-LKI-Power,
// CR 603.10a) to honor the 2020-06-23 ruling that gain-life amount equals power as it
// last existed on the battlefield (layer-resolved, including +1/+1 counters added by
// the replacement half), NOT the printed value or graveyard-reset value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("conclave-mentor"),
        name: "Conclave Mentor".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Centaur", "Cleric"]),
        oracle_text: "If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on that creature instead.\nWhen Conclave Mentor dies, you gain life equal to its power.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Any,
                    receiver_filter: ObjectFilter::CreatureControlledBy(PlayerId(0)),
                    counter_filter: Some(CounterType::PlusOnePlusOne),
                },
                modification: ReplacementModification::AddExtraCounter,
                is_self: false,
                unless_condition: None,
            },
            // When Conclave Mentor dies, you gain life equal to its power.
            // CR 603.10a / Ruling 2020-06-23: power read from LKI snapshot
            // (boosted-on-battlefield value, not printed 2/2).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::SourcePowerAtLastKnownInformation,
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

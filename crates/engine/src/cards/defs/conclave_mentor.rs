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
// Death trigger ("you gain life equal to its power") is BLOCKED on LKI power snapshot
// at WhenDies. PB-LKI-CC threaded LKI counters through PendingTrigger; PB-LKI-Power
// (filed as OOS seed in memory/primitives/pb-retriage-CC.md) would extend the same
// pattern to source power. Until that primitive lands, the death trigger is intentionally
// not implemented to avoid wrong game state (it would resolve to 0 from a graveyard'd
// source per CR 122.2 / CR 400.7, just like Toothy did pre-PB-LKI-CC).
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
            // TODO (OOS-LKI-Power, see memory/primitives/pb-retriage-CC.md): WhenDies
            // trigger reading LKI source power. Blocked on LKI power snapshot primitive
            // (extension of PB-LKI-CC's lki_counters pattern to lki_power).
        ],
        ..Default::default()
    }
}

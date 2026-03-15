// Vorinclex, Monstrous Raider — {4}{G}{G}, Legendary Creature — Phyrexian Praetor 6/6
// Trample, haste
// If you would put one or more counters on a permanent or player, put twice that many instead.
// If an opponent would put one or more counters on a permanent or player, they put half that many instead, rounded down.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vorinclex-monstrous-raider"),
        name: "Vorinclex, Monstrous Raider".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Praetor"],
        ),
        oracle_text: "Trample, haste\nIf you would put one or more counters on a permanent or player, put twice that many of each of those kinds of counters on that permanent or player instead.\nIf an opponent would put one or more counters on a permanent or player, they put half that many of each of those kinds of counters on that permanent or player instead, rounded down.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 122.6 / CR 614.1: Double counters placed by controller.
            // PlayerId(0) is a placeholder — bound to the actual controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::Specific(PlayerId(0)),
                    receiver_filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::DoubleCounters,
                is_self: false,
                unless_condition: None,
            },
            // CR 122.6 / CR 614.1: Halve counters placed by opponents.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldPlaceCounters {
                    placer_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
                    receiver_filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::HalveCounters,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

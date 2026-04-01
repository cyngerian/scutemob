// Angrath's Marauders — {5}{R}{R}, Creature — Human Pirate 4/4
// If a source you control would deal damage to a permanent or player, it deals double
// that damage instead.
//
// CR 614.1a: Replacement effect — doubles all damage from sources you control.
// DamageTargetFilter::FromControllerSources(PlayerId(0)) matches damage dealt by sources
// controlled by this card's controller. PlayerId(0) is a placeholder bound at registration.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("angraths-marauders"),
        name: "Angrath's Marauders".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Pirate"]),
        oracle_text: "If a source you control would deal damage to a permanent or player, it deals double that damage instead.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // CR 614.1a: "If a source you control would deal damage, it deals double instead."
            // ReplacementTrigger::DamageWouldBeDealt with FromControllerSources filter.
            // PlayerId(0) is a placeholder that is bound to the actual controller at registration time.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::DamageWouldBeDealt {
                    target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleDamage,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

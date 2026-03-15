// Twinflame Tyrant — {3}{R}{R}, Creature — Dragon 3/5
// Flying
// If a source you control would deal damage to an opponent or a permanent an
// opponent controls, it deals double that damage instead.
//
// Note: The oracle text specifies "to an opponent or a permanent an opponent controls"
// but the current DamageTargetFilter doesn't support target-side opponent filtering.
// This implementation doubles ALL damage from the controller's sources to any target.
// TODO: Add opponent-target filtering to damage doubling (DamageTargetFilter::ToOpponentOrTheirPermanent).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("twinflame-tyrant"),
        name: "Twinflame Tyrant".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nIf a source you control would deal damage to an opponent or a permanent an opponent controls, it deals double that damage instead.".to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 614.1: Damage doubling for sources you control.
            // PlayerId(0) placeholder — bound to controller at registration.
            // NOTE: Missing opponent-target filter — see file header.
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

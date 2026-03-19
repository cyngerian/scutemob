// Twinflame Tyrant — {3}{R}{R}, Creature — Dragon 3/5
// Flying
// If a source you control would deal damage to an opponent or a permanent an
// opponent controls, it deals double that damage instead.
//
// Target-side filtering: DamageTargetFilter::ToOpponentOrTheirPermanent enforces
// "to an opponent or a permanent an opponent controls" per oracle text.
// apply_damage_doubling in replacement.rs checks the target against this filter.
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
            // CR 614.1: Damage doubling for sources you control targeting opponents.
            // PlayerId(0) placeholder — bound to controller at registration.
            // ToOpponentOrTheirPermanent encodes BOTH conditions per oracle text:
            //   1. Source must be controlled by PlayerId(0) (checked in apply_damage_doubling).
            //   2. Target must be an opponent of PlayerId(0) or a permanent they control.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::DamageWouldBeDealt {
                    target_filter: DamageTargetFilter::ToOpponentOrTheirPermanent(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleDamage,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

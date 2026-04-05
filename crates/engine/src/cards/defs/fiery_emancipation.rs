// Fiery Emancipation — {3}{R}{R}{R}, Enchantment
// If a source you control would deal damage to a permanent or player,
// it deals triple that damage to that permanent or player instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fiery-emancipation"),
        name: "Fiery Emancipation".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If a source you control would deal damage to a permanent or player, it deals triple that damage to that permanent or player instead.".to_string(),
        abilities: vec![
            // CR 614.1 / CR 701.10g: "If a source you control would deal damage to a
            // permanent or player, it deals triple that damage instead."
            // Static replacement effect; registered in register_permanent_replacement_abilities.
            // PlayerId(0) is bound to the controller at registration time.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::DamageWouldBeDealt {
                    target_filter: DamageTargetFilter::FromControllerSources(PlayerId(0)),
                },
                modification: ReplacementModification::TripleDamage,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

// City on Fire — {5}{R}{R}{R} Enchantment (Convoke)
// If a source you control would deal damage to a permanent or player, it deals triple
// that damage instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("city-on-fire"),
        name: "City on Fire".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nIf a source you control would deal damage to a permanent or player, it deals triple that damage instead.".to_string(),
        abilities: vec![
            // CR 702.51: Convoke — creatures you control can help pay the mana cost.
            AbilityDefinition::Keyword(KeywordAbility::Convoke),
            // CR 614.1 / CR 701.10g: Static replacement effect — triple damage from sources you control.
            // PlayerId(0) is bound to the actual controller at registration time in
            // rules/replacement.rs register_permanent_replacement_abilities().
            // Pattern mirrors Fiery Emancipation (cards/defs/fiery_emancipation.rs).
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

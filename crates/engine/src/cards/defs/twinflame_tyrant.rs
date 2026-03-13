// Twinflame Tyrant — {3}{R}{R}, Creature — Dragon 3/5; Flying.
// If a source you control would deal damage to an opponent or permanent an opponent
// controls, it deals double that damage instead.
// TODO: DSL gap — damage replacement/doubling effect not expressible in the DSL.
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
        ],
        // TODO: damage doubling replacement effect (ReplacementTrigger/ReplacementModification
        // for doubling damage from sources you control to opponents/their permanents)
        ..Default::default()
    }
}

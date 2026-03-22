// Hydroelectric Specimen // Hydroelectric Laboratory — {2}{U} Creature — Weird 1/4
// Flash
// When this creature enters, you may change the target of target instant or sorcery
// spell with a single target to this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hydroelectric-specimen"),
        name: "Hydroelectric Specimen // Hydroelectric Laboratory".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Weird"]),
        oracle_text: "Flash\nWhen this creature enters, you may change the target of target instant or sorcery spell with a single target to this creature.".to_string(),
        power: Some(1),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: ETB trigger — redirect target of an instant/sorcery spell.
            // DSL gap: target redirection effect not expressible.
        ],
        ..Default::default()
    }
}

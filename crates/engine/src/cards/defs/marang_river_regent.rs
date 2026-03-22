// Marang River Regent // Coil and Catch — {4}{U}{U} Creature — Dragon 6/7
// Flying
// When this creature enters, return up to two other target nonland permanents
// to their owners' hands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marang-river-regent"),
        name: "Marang River Regent // Coil and Catch".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhen this creature enters, return up to two other target nonland permanents to their owners' hands.".to_string(),
        power: Some(6),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ETB trigger — return up to two other target nonland permanents to
            // owners' hands. Bounce effect + nonland filter ARE expressible; DSL gap is
            // "up to N" optional targeting (mandatory 2-target would fizzle with <2 targets).
        ],
        ..Default::default()
    }
}

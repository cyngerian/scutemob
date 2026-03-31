// Brokers Hideout — Land
// When this land enters, sacrifice it. When you do, search your library for a basic
// Forest, Plains, or Island card, put it onto the battlefield tapped, then shuffle
// and you gain 1 life.
//
// TODO: Reflexive trigger pattern ("When you do, ...") is not expressible in the current DSL.
// The ETB causes sacrifice-self, and the "when you do" clause triggers the search+life gain.
// This two-step reflexive pattern (ETB → sacrifice → triggered search) requires a
// ReflexiveTrigger mechanism that does not yet exist. Use abilities: vec![] per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brokers-hideout"),
        name: "Brokers Hideout".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "When this land enters, sacrifice it. When you do, search your library for a basic Forest, Plains, or Island card, put it onto the battlefield tapped, then shuffle and you gain 1 life.".to_string(),
        abilities: vec![
            // TODO: Reflexive trigger ("When you do, ...") not expressible in DSL.
            // ETB → sacrifice self → search for basic Forest/Plains/Island tapped + gain 1 life.
            // Requires a reflexive trigger mechanism. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}

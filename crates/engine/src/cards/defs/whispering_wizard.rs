// Whispering Wizard — {3}{U}, Creature — Human Wizard 3/2
// Whenever you cast a noncreature spell, create a 1/1 white Spirit creature token with flying.
// This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whispering-wizard"),
        name: "Whispering Wizard".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Whenever you cast a noncreature spell, create a 1/1 white Spirit creature token with flying. This ability triggers only once each turn.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever you cast a noncreature spell" — WheneverYouCastSpell lacks a
            // noncreature filter. Additionally "triggers only once each turn" limiter not in DSL.
        ],
        ..Default::default()
    }
}

// Inkshield — {3}{W}{B}, Instant
// Prevent all combat damage that would be dealt to you this turn. For each 1 damage
// prevented this way, create a 2/1 white and black Inkling creature token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inkshield"),
        name: "Inkshield".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Instant], &[]),
        oracle_text: "Prevent all combat damage that would be dealt to you this turn. For each 1 damage prevented this way, create a 2/1 white and black Inkling creature token with flying.".to_string(),
        abilities: vec![
            // TODO: Prevention effect + variable token count based on damage prevented not in DSL
        ],
        ..Default::default()
    }
}

// Living Death — {3}{B}{B} Sorcery
// Each player exiles all creature cards from their graveyard, then sacrifices all
// creatures they control, then puts all cards they exiled this way onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("living-death"),
        name: "Living Death".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player exiles all creature cards from their graveyard, then sacrifices all creatures they control, then puts all cards they exiled this way onto the battlefield.".to_string(),
        abilities: vec![
            // Living Death (2018-03-16 ruling): three-step mass zone change.
            // CR 101.4 (APNAP simultaneous), CR 701.21a (sacrifice semantics).
            // Step 1: exile all creature cards from all graveyards.
            // Step 2: sacrifice all creatures on the battlefield.
            // Step 3: return step-1 exiled cards to the battlefield under their owners' control.
            AbilityDefinition::Spell {
                effect: Effect::LivingDeath,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

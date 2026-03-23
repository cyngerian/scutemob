// Moraug, Fury of Akoum — {4}{R}{R}, Legendary Creature — Minotaur Warrior 6/6
// Each creature you control gets +1/+0 for each time it has attacked this turn.
// Landfall — Whenever a land you control enters, if it's your main phase, there's an
// additional combat phase after this phase. At the beginning of that combat, untap all
// creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moraug-fury-of-akoum"),
        name: "Moraug, Fury of Akoum".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Minotaur", "Warrior"],
        ),
        oracle_text: "Each creature you control gets +1/+0 for each time it has attacked this turn.\nLandfall — Whenever a land you control enters, if it's your main phase, there's an additional combat phase after this phase. At the beginning of that combat, untap all creatures you control.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: DSL gap — dynamic +1/+0 per attack count per creature this turn.
            // Needs per-creature attack tracking + dynamic LayerModification.
            // TODO: DSL gap — landfall trigger with main phase intervening-if that creates
            // additional combat phase. AdditionalCombatPhase effect exists but landfall +
            // main-phase condition combo not easily expressible.
        ],
        ..Default::default()
    }
}

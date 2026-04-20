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
            // Needs per-creature attack tracking + dynamic LayerModification (continuous
            // effect scaled by a per-object counter). Separate from the Landfall ability.
            // TODO: Blocker — main-phase intervening-if + additional combat phase + "untap all
            // creatures at the beginning of that combat" chaining. Landfall trigger itself is
            // covered (TriggerCondition::WheneverPermanentEntersBattlefield + Land + You +
            // intervening_if: Condition::MainPhase, CR 207.2c). The additional-combat-phase-
            // plus-chained-delayed-trigger combo is the gap (no Effect::AdditionalCombatPhase
            // with a nested delayed "untap all" trigger in the DSL).
        ],
        ..Default::default()
    }
}

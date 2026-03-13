// Karlach, Fury of Avernus — {4}{R}, Legendary Creature — Tiefling Barbarian 5/4
// Whenever you attack (first combat phase), untap attackers, grant first strike, add combat phase.
// Choose a Background
// TODO: DSL gap — "untap all attacking creatures + grant first strike + additional combat phase"
// requires a triggered ability on attack with conditional (first combat phase) that applies
// continuous effects to attacking creatures and adds a combat phase — no AddCombatPhase effect
// or GrantKeywordToAttackers effect exists in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("karlach-fury-of-avernus"),
        name: "Karlach, Fury of Avernus".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Tiefling", "Barbarian"],
        ),
        oracle_text: "Whenever you attack, if it's the first combat phase of the turn, untap all attacking creatures. They gain first strike until end of turn. After this phase, there is an additional combat phase.\nChoose a Background (You can have a Background as a second commander.)".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
            // TODO: triggered — whenever you attack (first combat phase only), untap all attacking
            // creatures, grant first strike until end of turn, add an additional combat phase.
            // DSL gaps: no AddCombatPhase effect; no GrantKeywordToAttackers effect;
            // no "first combat phase" intervening-if condition.
        ],
        ..Default::default()
    }
}

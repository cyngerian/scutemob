// Razorkin Needlehead — {R}{R}, Creature — Human Assassin 2/2
// This creature has first strike during your turn.
// Whenever an opponent draws a card, this creature deals 1 damage to them.
// TODO: DSL gap — conditional first strike (only during your turn) requires a layer-6
// conditional keyword grant; KeywordAbility::FirstStrike is always-on only.
// The draw trigger targeting the drawing opponent also lacks a WheneverOpponentDrawsCard
// TriggerCondition variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("razorkin-needlehead"),
        name: "Razorkin Needlehead".to_string(),
        mana_cost: Some(ManaCost { red: 2, ..Default::default() }),
        types: creature_types(&["Human", "Assassin"]),
        oracle_text: "This creature has first strike during your turn.\nWhenever an opponent draws a card, this creature deals 1 damage to them.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        // TODO: first strike only during your turn (conditional keyword)
        // TODO: whenever an opponent draws a card, deal 1 damage to them
        ..Default::default()
    }
}

// Endurance — {1}{G}{G}, Creature — Elemental Incarnation 3/4
// Flash, Reach; ETB: up to one target player puts all graveyard cards on bottom of library
// Evoke — Exile a green card from your hand
// TODO: ETB targeted graveyard-to-library effect requires targeted_trigger gap
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("endurance"),
        name: "Endurance".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elemental", "Incarnation"]),
        oracle_text: "Flash\nReach\nWhen this creature enters, up to one target player puts all the cards from their graveyard on the bottom of their library in a random order.\nEvoke—Exile a green card from your hand.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            // TODO: ETB trigger targeting a player to shuffle their graveyard to the bottom
            // of their library — targeted_trigger with graveyard manipulation effect not in DSL.
        ],
        completeness: Completeness::partial("Two real blockers; the old 'targeted_trigger gap' note was stale (Triggered.targets ships — see eomer_king_of_rohan.rs:51-56). (1) 'up to one target player puts all the cards from their graveyard on the bottom of their library in a random order' — no effect moves one player's whole graveyard to library bottom (EffectTarget has no graveyard-contents variant; ForEachTarget::EachCardInAllGraveyards is all graveyards, not the target's). (2) Evoke—Exile a green card from your hand: AbilityDefinition::Evoke takes ManaCost only (card_definition.rs:453) and cannot carry Cost::ExileFromHand { color }; the bare Keyword(Evoke) marker on line 23 is inert."),
        ..Default::default()
    }
}

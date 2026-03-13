// Voidwing Hybrid — {U}{B}, Creature — Phyrexian Bat 2/1
// Flying, Toxic 1
// When you proliferate, return this card from your graveyard to your hand.
//
// TODO: DSL gap — "When you proliferate, return this card from your graveyard to your hand."
// No TriggerCondition for WhenYouProliferate; no graveyard-return-self effect in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voidwing-hybrid"),
        name: "Voidwing Hybrid".to_string(),
        mana_cost: Some(ManaCost { blue: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Bat"]),
        oracle_text: "Flying\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhen you proliferate, return this card from your graveyard to your hand.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
        ],
        ..Default::default()
    }
}

// Earthquake Dragon — {14}{G}, Creature — Elemental Dragon 10/10
// This spell costs {X} less to cast, where X is the total mana value of Dragons you control.
// Flying, trample
// {2}{G}, Sacrifice a land: Return this card from your graveyard to your hand.
// TODO: "{2}{G}, Sacrifice a land: Return this card from your graveyard to your hand." —
//    activated graveyard ability requiring Cost::SacrificePermanent(land filter) combined with
//    Cost::PayMana, plus a return-from-graveyard effect. DSL gap: compound costs + graveyard return.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("earthquake-dragon"),
        name: "Earthquake Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 14, green: 1, ..Default::default() }),
        types: creature_types(&["Elemental", "Dragon"]),
        oracle_text: "This spell costs {X} less to cast, where X is the total mana value of Dragons you control.\nFlying, trample\n{2}{G}, Sacrifice a land: Return this card from your graveyard to your hand.".to_string(),
        power: Some(10),
        toughness: Some(10),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: {2}{G}, Sacrifice a land: Return from graveyard to hand
        ],
        self_cost_reduction: Some(SelfCostReduction::TotalManaValue {
            filter: TargetFilter {
                has_subtype: Some(SubType("Dragon".to_string())),
                ..Default::default()
            },
        }),
        ..Default::default()
    }
}

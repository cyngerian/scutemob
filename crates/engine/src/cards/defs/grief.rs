// Grief — {2}{B}{B}, Creature — Elemental Incarnation 3/2
// Menace; ETB: target opponent reveals hand, you choose a nonland card, they discard it
// Evoke — Exile a black card from your hand
// TODO: ETB targeted discard (choose nonland from opponent's revealed hand) not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grief"),
        name: "Grief".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elemental", "Incarnation"]),
        oracle_text: "Menace\nWhen this creature enters, target opponent reveals their hand. You choose a nonland card from it. That player discards that card.\nEvoke—Exile a black card from your hand.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            AbilityDefinition::Keyword(KeywordAbility::Evoke),
            // TODO: ETB trigger targeting an opponent to reveal their hand and discard a
            // chosen nonland card — targeted discard with card-type filter (non-land) not
            // in DSL (targeted_trigger gap).
        ],
        ..Default::default()
    }
}

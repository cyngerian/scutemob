// Six — {2}{G}, Legendary Creature — Treefolk 2/4
// Reach
// Whenever Six attacks, mill three cards. You may put a land card from among them into your hand.
// During your turn, nonland permanent cards in your graveyard have retrace.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("six"),
        name: "Six".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Treefolk"]),
        oracle_text: "Reach\nWhenever Six attacks, mill three cards. You may put a land card from among them into your hand.\nDuring your turn, nonland permanent cards in your graveyard have retrace. (You may cast permanent cards from your graveyard by discarding a land card in addition to paying their other costs.)".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            // TODO: DSL gap — the WhenAttacks trigger mills 3 then routes a land card to hand.
            // Effect::RevealAndRoute or Effect::Mill followed by conditional zone routing
            // (put land card from milled cards to hand) is not expressible in the current DSL.
            // TODO: DSL gap — "nonland permanent cards in your graveyard have retrace" is a
            // static ability granting retrace conditionally during your turn; no DSL support.
        ],
        ..Default::default()
    }
}

// Uro, Titan of Nature's Wrath — {1}{G}{U}, Legendary Creature — Elder Giant 6/6
// When Uro enters, sacrifice it unless it escaped.
// Whenever Uro enters or attacks, you gain 3 life and draw a card, then you may put a
// land card from your hand onto the battlefield.
// Escape—{G}{G}{U}{U}, Exile five other cards from your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("uro-titan-of-natures-wrath"),
        name: "Uro, Titan of Nature's Wrath".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Giant"],
        ),
        oracle_text: "When Uro enters, sacrifice it unless it escaped.\nWhenever Uro enters or attacks, you gain 3 life and draw a card, then you may put a land card from your hand onto the battlefield.\nEscape\u{2014}{G}{G}{U}{U}, Exile five other cards from your graveyard.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: "Sacrifice unless it escaped" ETB — needs cast_alt_cost check.
            //   Missing sacrifice means hardcast Uro stays as 6/6 — wrong game state.
            // TODO: "Whenever Uro enters or attacks, gain 3 + draw + optional land drop"
            //   requires sacrifice-unless-escaped to be correct first.
            // Escape {G}{G}{U}{U}, exile 5 cards
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Escape,
                cost: ManaCost { green: 2, blue: 2, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}
